use crate::cli::format::FormatArgs;
use crate::cli::{GlobalOptions, DAEMON_THREAD_START, IS_DAEMON_MAIN_THREAD, IS_DAEMON_PROCESS};
use crate::config::{load_config_and_cache, load_config_and_socket};
use crate::daemon::client::ping;
use crate::daemon::interface::{
    DaemonCommandPayload, DaemonCommands, DaemonFormatArgs, DaemonFormatResponse, DaemonResponse,
};
use crate::handle_plugin::run::run;
use anyhow::{anyhow, Context, Result};
use log::{debug, error, info};
use nix::libc::statx;
use nix::unistd::{fork, ForkResult};
use notify::Watcher;
use os_pipe::PipeWriter;
use serde_json::json;
use std::env::current_dir;
use std::ffi::CString;
use std::io::{ErrorKind, Read};
use std::net::Shutdown;
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{fs, io, mem, process, thread, time};
use wasmtime::component::__internal::wasmtime_environ::wasmparser::Payload;

pub fn daemon_format_execute_with_args(
    args: DaemonFormatArgs,
    current_dir: PathBuf,
    global_options: GlobalOptions,
) -> Result<DaemonFormatResponse> {
    let no_quick_magic =
        std::env::var_os("FORO_NO_QUICK_MAGIC").is_some_and(|s| s != "0" && s != "");

    debug!("no_quick_magic: {}", no_quick_magic);

    let target_path = current_dir.join(&args.path).canonicalize()?;
    let target_path_outer = target_path.clone();

    let (tx, rx) = mpsc::channel();

    let parent_start_time = DAEMON_THREAD_START.with(|start| *start.get_or_init(|| Instant::now()));

    // todo: maybe to implement `?` operator to DaemonFormatResponse is better
    let t = thread::spawn(move || -> Result<Option<DaemonFormatResponse>> {
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

        info!("run rule: {:?}", rule);

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
            info!("quick magic detected file changed");
            break;
        }

        if rx.try_recv().is_ok() {
            info!("quick magic detected child finished");
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

    info!("main process exit");

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

fn serverside_exec_command(payload: DaemonCommandPayload) -> DaemonResponse {
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

fn handle_client(mut stream: UnixStream) -> Result<()> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let payload: DaemonCommandPayload = serde_json::from_slice(&buf)?;

    info!("Received: {:?}", &payload);

    let response = serverside_exec_command(payload);

    info!("Response: {:?}", &response);

    stream.shutdown(Shutdown::Read)?;

    serde_json::to_writer(stream, &response)?;

    Ok(())
}

pub struct WrappedUnixSocket {
    path: PathBuf,
    listener: UnixListener,
}

impl WrappedUnixSocket {
    pub(crate) fn bind(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let parent = path.parent().context("Failed to get parent")?;
        fs::create_dir_all(parent)?;

        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(err) if err.kind() == ErrorKind::AddrInUse => {
                if ping(&path)? {
                    return Err(anyhow!("Daemon is already running"));
                } else {
                    info!("Removing dead socket file");
                    fs::remove_file(&path)?;
                    UnixListener::bind(&path)?
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        info!("Listening on: {}", path.display());

        Ok(Self { path, listener })
    }
}

impl Drop for WrappedUnixSocket {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn daemon_main(socket: WrappedUnixSocket) -> Result<()> {
    for stream in socket.listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    info!("New client connected");
                    handle_client(stream).unwrap();
                });
            }
            Err(err) => {
                error!("Error: {}", err);
            }
        }
    }

    Ok(())
}

pub fn start_daemon(socket: &PathBuf, attach: bool) -> Result<()> {
    info!("Starting daemon (attach: {})", attach);

    let listener = WrappedUnixSocket::bind(socket)?;

    if attach {
        IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
        IS_DAEMON_MAIN_THREAD.with(|is_main_thread| {
            let _ = is_main_thread.set(true);
        });

        info!("Daemon started");

        daemon_main(listener)?;
    } else {
    }

    Ok(())
}
