use crate::cli::{GlobalOptions, DAEMON_THREAD_START};
use crate::config::load_config_and_cache;
use crate::daemon::interface::{
    DaemonCommandPayload, DaemonCommands, DaemonFormatArgs, DaemonFormatResponse, DaemonResponse,
};
use crate::handle_plugin::run::run;
use log::{debug, trace};
use notify::Watcher;
use serde_json::json;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{fs, io, thread};

pub fn daemon_format_execute_with_args(
    args: DaemonFormatArgs,
    current_dir: PathBuf,
    global_options: GlobalOptions,
) -> anyhow::Result<DaemonFormatResponse> {
    let no_quick_trick =
        std::env::var_os("FORO_NO_QUICK_TRICK").is_some_and(|s| s != "0" && s != "");

    trace!("no_quick_trick: {}", no_quick_trick);

    let target_path = current_dir.join(&args.path).canonicalize()?;
    let target_path_outer = target_path.clone();

    let (tx, rx) = mpsc::channel();

    let parent_start_time = DAEMON_THREAD_START.with(|start| *start.get_or_init(|| Instant::now()));

    // todo: maybe to implement `?` operator to DaemonFormatResponse is better
    let t = thread::spawn(move || -> anyhow::Result<Option<DaemonFormatResponse>> {
        DAEMON_THREAD_START.with(|start| {
            let _ = start.set(parent_start_time);
        });

        let (config, cache_dir) =
            load_config_and_cache(&global_options.config_file, &global_options.cache_dir)?;

        let file = fs::File::open(&target_path)?;
        let mut buf_reader = io::BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        let rule = match config.find_matched_rule(&target_path) {
            Some(rule) => rule,
            None => {
                return Ok(Some(DaemonFormatResponse::Ignored));
            }
        };

        debug!("run rule: {:?}", rule);

        let res = run(
            &rule.cmd,
            json!({
                "current-dir": current_dir.canonicalize()?.to_str().unwrap(),
                "target": target_path.to_str().unwrap(),
                "raw-target": args.path,
                "target-content": contents,
                }
            ),
            &cache_dir,
            !global_options.no_cache,
        )?;

        println!("{:?}", res);

        tx.send(0)?;

        Ok(None)
    });

    let (w_tx, w_rx) = mpsc::channel();

    let mut watcher = notify::RecommendedWatcher::new(
        w_tx,
        notify::Config::default().with_poll_interval(Duration::from_micros(100)),
    )?;

    watcher.watch(&target_path_outer, notify::RecursiveMode::NonRecursive)?;

    loop {
        if w_rx.try_recv().is_ok() {
            debug!("quick trick detected file changed");
            break;
        }

        if rx.try_recv().is_ok() {
            debug!("quick trick detected child finished");
            break;
        }

        if t.is_finished() {
            break;
        }

        sleep(Duration::from_micros(10));
    }

    if t.is_finished() {
        let res = t.join().unwrap();
        match res {
            Ok(Some(res)) => {
                return Ok(res);
            }
            Err(err) => {
                return Err(err);
            }
            _ => {}
        }
    }

    debug!("main process exit");

    let now = SystemTime::now();

    // UNIXエポックからの経過時間を取得
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // 秒とナノ秒をそれぞれ取得
    let seconds = since_the_epoch.as_secs();
    let nanoseconds = since_the_epoch.subsec_nanos();

    // マイクロ秒単位の精度を計算
    let microseconds = nanoseconds / 1_000;
    println!("{}.{:06}", seconds, microseconds);

    Ok(DaemonFormatResponse::Success)
}

pub fn serverside_exec_command(payload: DaemonCommandPayload) -> DaemonResponse {
    match payload.command {
        DaemonCommands::Format(s_args) => {
            match daemon_format_execute_with_args(
                s_args,
                payload.current_dir,
                payload.global_options,
            ) {
                Ok(res) => DaemonResponse::Format(res),
                Err(err) => DaemonResponse::Format(DaemonFormatResponse::Error(err.to_string())),
            }
        }
        DaemonCommands::Ping => DaemonResponse::Pong,
    }
}
