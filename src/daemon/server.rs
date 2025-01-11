use crate::app_dir::log_dir_res;
use crate::bulk_format::{bulk_format, BulkFormatOption};
use crate::cli::GlobalOptions;
use crate::config::{load_config_and_cache, Rule, SomeCommand};
use crate::daemon::client::ping;
use crate::daemon::interface::{
    DaemonBulkFormatArgs, DaemonBulkFormatResponse, DaemonCommandPayload, DaemonCommands,
    DaemonFormatArgs, DaemonFormatResponse, DaemonInfo, DaemonPureFormatArgs,
    DaemonPureFormatResponse, DaemonResponse, DaemonSocketPath, OutputPath,
};
use crate::debug_long;
use crate::handle_plugin::run::{run, run_pure};
use crate::log::IS_DAEMON_PROCESS;
use crate::log::{DAEMON_THREAD_START, IS_DAEMON_MAIN_THREAD};
use crate::process_utils::get_start_time;
use anyhow::Result;
use anyhow::{anyhow, Context};
use foro_plugin_utils::data_json_utils::JsonGetter;
use log::{debug, error, info, trace, warn};
use nix::unistd::{close, fork, setsid, ForkResult};
use notify::Watcher;
use serde_json::json;
use std::fmt::format;
use std::fs::{DirBuilder, OpenOptions};
use std::io::prelude::*;
use std::io::{ErrorKind, Read};
use std::net::Shutdown;
use std::os::fd::IntoRawFd;
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, OnceLock};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{fs, io, process, thread};
#[cfg(windows)]
use uds_windows::{UnixListener, UnixStream};

static DAEMON_INFO: OnceLock<DaemonInfo> = OnceLock::new();

pub fn daemon_format_execute_with_args(
    args: DaemonFormatArgs,
    current_dir: PathBuf,
    global_options: GlobalOptions,
) -> Result<DaemonFormatResponse> {
    let target_path = current_dir.join(&args.path).canonicalize()?;
    let target_path_outer = target_path.clone();

    let (tx, rx) = mpsc::channel();

    let parent_start_time = DAEMON_THREAD_START.with(|start| *start.get_or_init(|| Instant::now()));
    let t_target_path = target_path.clone();

    // Even if formatting command has finished (i.e., writing to the file has completed),
    // there is still some delay before the function call returns due to memory cleanup and
    // other processes. Therefore, we run it in a separate thread, monitor the file for changes,
    // and have the main thread return the result immediately once a change is detected (or once
    // the thread execution is complete).
    //
    // We refer to this here as a “quick trick”.

    // todo: maybe to implement `?` operator to DaemonFormatResponse is better
    let t = thread::spawn(move || -> Result<Option<DaemonFormatResponse>> {
        DAEMON_THREAD_START.with(|start| {
            let _ = start.set(parent_start_time);
        });

        let (config, cache_dir) =
            load_config_and_cache(&global_options.config_file, &global_options.cache_dir)?;

        // todo: why not fs::read ?
        let file = fs::File::open(&t_target_path)?;
        let mut buf_reader = io::BufReader::new(file);
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;
        drop(buf_reader);

        let rule = match config.find_matched_rule(&t_target_path, false) {
            Some(rule) => rule,
            None => {
                let reason = "No rule matched".to_string();
                return Ok(Some(DaemonFormatResponse::Ignored(reason)));
            }
        };

        debug_long!("run rule: {:?}", rule);

        let res = run(
            &rule.some_cmd,
            json!({
                "current-dir": current_dir.canonicalize()?.to_str().unwrap(),
                "target": &t_target_path.to_str().unwrap(),
                "raw-target": args.path,
                "target-content": content,
                }
            ),
            &cache_dir,
            !global_options.no_cache,
        )?;

        println!("{:?}", res);

        if let Some(status) = String::get_value_opt(&res, ["format-status"]) {
            match status.as_str() {
                "ignored" => {
                    let reason = String::get_value_opt(&res, ["ignored-reason"])
                        .unwrap_or("File ignored".to_string());
                    return Ok(Some(DaemonFormatResponse::Ignored(reason)));
                }
                "error" => {
                    let error = String::get_value_opt(&res, ["format-error"]).context("Failed to get format error. Did you forget to return `format-error` in your plugin?")?;
                    return Ok(Some(DaemonFormatResponse::Error(error)));
                }
                _ => {}
            }
        }

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

    Ok(DaemonFormatResponse::Success())
}

pub fn daemon_pure_format_execute_with_args(
    args: DaemonPureFormatArgs,
    current_dir: PathBuf,
    global_options: GlobalOptions,
) -> Result<DaemonPureFormatResponse> {
    let target_path = current_dir.join(&args.path).canonicalize()?;

    let (config, cache_dir) =
        load_config_and_cache(&global_options.config_file, &global_options.cache_dir)?;

    let rule = match config.find_matched_rule(&target_path, true) {
        Some(rule) => rule,
        None => {
            let found_rule_non_pure = config.find_matched_rule(&target_path, false).is_none();

            let reason = if !found_rule_non_pure {
                "No rule matched"
            } else {
                "No rule matched (but found non-pure rule)"
            }
            .to_string();

            return Ok(DaemonPureFormatResponse::Ignored(reason));
        }
    };

    debug_long!("run rule: {:?}", rule);

    let pure_cmd = match rule {
        Rule {
            some_cmd: SomeCommand::Pure { cmd },
            ..
        } => cmd,
        _ => {
            unreachable!()
        }
    };

    let res = run_pure(
        &pure_cmd,
        json!({
            "current-dir": target_path.parent().unwrap().to_str().unwrap(),
            "target": &target_path.to_str().unwrap(),
            "raw-target": args.path,
            "target-content": args.content,
            }
        ),
        &cache_dir,
        !global_options.no_cache,
    )?;

    if let Some(status) = String::get_value_opt(&res, ["format-status"]) {
        match status.as_str() {
            "success" => {
                let formatted = String::get_value_opt(&res, ["formatted-content"]).context("Failed to get formatted content. Did you forget to return `formatted-content` in your plugin?")?;
                return Ok(DaemonPureFormatResponse::Success(formatted));
            }
            "ignored" => {
                let reason = String::get_value_opt(&res, ["ignored-reason"])
                    .unwrap_or("File ignored".to_string());
                return Ok(DaemonPureFormatResponse::Ignored(reason));
            }
            "error" => {
                let error = String::get_value_opt(&res, ["format-error"]).context("Failed to get format error. Did you forget to return `format-error` in your plugin?")?;
                return Ok(DaemonPureFormatResponse::Error(error));
            }
            _ => {}
        }
    }

    Ok(DaemonPureFormatResponse::Error(
        "Plugin does not return valid value".to_string(),
    ))
}

pub fn daemon_bulk_format_execute_with_args(
    args: DaemonBulkFormatArgs,
    current_dir: PathBuf,
    global_options: GlobalOptions,
) -> Result<DaemonBulkFormatResponse> {
    let paths = args
        .paths
        .iter()
        .map(|p| {
            current_dir
                .join(p)
                .canonicalize()
                .map_err(anyhow::Error::from)
        })
        .collect::<Result<Vec<PathBuf>>>()?;

    let (config, cache_dir) =
        load_config_and_cache(&global_options.config_file, &global_options.cache_dir)?;

    let opt = BulkFormatOption {
        paths,
        threads: 3,
        use_default_ignore: true,
        current_dir,
    };

    let success_count = bulk_format(&opt, &config, &cache_dir, !global_options.no_cache)?;

    Ok(DaemonBulkFormatResponse::Success(format!(
        "Formated {success_count} files"
    )))
}

pub fn serverside_exec_command(payload: DaemonCommandPayload) -> DaemonResponse {
    match payload.command {
        DaemonCommands::Format(s_args) => {
            let res = daemon_format_execute_with_args(
                s_args,
                payload.current_dir,
                payload.global_options,
            );

            match res {
                Ok(res) => DaemonResponse::Format(res),
                Err(err) => {
                    DaemonResponse::Format(DaemonFormatResponse::Error(format!("{:#}", err)))
                }
            }
        }
        DaemonCommands::PureFormat(s_args) => {
            let res = daemon_pure_format_execute_with_args(
                s_args,
                payload.current_dir,
                payload.global_options,
            );

            match res {
                Ok(res) => DaemonResponse::PureFormat(res),
                Err(err) => DaemonResponse::PureFormat(DaemonPureFormatResponse::Error(format!(
                    "{:#}",
                    err
                ))),
            }
        }
        DaemonCommands::BulkFormat(s_args) => {
            let res = daemon_bulk_format_execute_with_args(
                s_args,
                payload.current_dir,
                payload.global_options,
            );

            match res {
                Ok(res) => DaemonResponse::BulkFormat(res),
                Err(err) => DaemonResponse::BulkFormat(DaemonBulkFormatResponse::Error(format!(
                    "{:#}",
                    err
                ))),
            }
        }
        DaemonCommands::Stop => DaemonResponse::Stop,
        DaemonCommands::Ping => DaemonResponse::Pong(DAEMON_INFO.get().unwrap().clone()),
    }
}

fn read_stream_with_retry(stream: &mut UnixStream, buf: &mut Vec<u8>) -> Result<()> {
    // There is a slight time lag between when the UnixStream receives communication and when
    // all the input data is written to the stream, so if it fails, we retry after a short delay.
    // retry_cnt is recorded and logged for debugging purposes only.

    let mut retry_cnt = 0;

    loop {
        let res = stream.read_to_end(buf);

        match res {
            Ok(_) => {
                break;
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                retry_cnt += 1;
                sleep(Duration::from_micros(10));
                continue;
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    trace!("read socket input with {} retry", retry_cnt);

    Ok(())
}

fn handle_client(mut stream: UnixStream, stop_sender: Sender<()>) -> Result<()> {
    let mut buf = Vec::new();
    read_stream_with_retry(&mut stream, &mut buf)?;

    trace!("{:?}", String::from_utf8_lossy(&buf));

    #[cfg(target_os = "linux")]
    stream.shutdown(Shutdown::Read)?;

    let payload: DaemonCommandPayload = serde_json::from_slice(&buf)?;

    debug_long!("Received: {:?}", &payload);

    let response = serverside_exec_command(payload);

    debug_long!("Response: {:?}", &response);

    let response_string = serde_json::to_string(&response)?;

    stream.write_all(response_string.as_bytes())?;

    if let DaemonResponse::Stop = response {
        stop_sender.send(())?;
    }

    Ok(())
}

pub struct WrappedUnixSocket {
    path: PathBuf,
    listener: UnixListener,
}

impl WrappedUnixSocket {
    pub(crate) fn bind(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let parent = path.parent().unwrap();
        fs::create_dir_all(parent)?;

        let info_path = parent.join(format!(
            "{}.info",
            path.file_name().unwrap().to_str().unwrap()
        ));

        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(err) if err.kind() == ErrorKind::AddrInUse => {
                let as_daemon_path = DaemonSocketPath {
                    socket_path: path.clone(),
                    info_path: info_path.clone(),
                };

                if ping(&as_daemon_path)? {
                    return Err(anyhow!("Daemon is already running"));
                } else {
                    info!("Removing dead socket file");
                    let err = fs::remove_file(&path);
                    if let Err(err) = err {
                        warn!("Failed to remove dead socket file: {}", err);
                    }
                    let err = fs::remove_file(&info_path);
                    if let Err(err) = err {
                        warn!("Failed to remove dead info file: {}", err);
                    }
                    UnixListener::bind(&path)?
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        debug!("writing info...");

        let pid = process::id();
        let start_time = get_start_time(pid)?;
        fs::write(&info_path, format!("{},{}", pid, start_time))?;

        info!("Listening on: {}", path.display());
        info!("info path: {}", info_path.display());

        Ok(Self { path, listener })
    }
}

impl Drop for WrappedUnixSocket {
    fn drop(&mut self) {
        || -> Option<()> {
            let _ = fs::remove_file(&self.path);
            let parent = self.path.parent()?;
            let info_path = parent.join(format!("{}.info", self.path.file_name()?.to_str()?));
            let _ = fs::remove_file(info_path);
            None
        }();
    }
}

/// Core function of the daemon.
pub fn daemon_main(socket: WrappedUnixSocket) {
    info!("Daemon started");

    let (tx, rx) = mpsc::channel();

    socket.listener.set_nonblocking(true).unwrap();

    loop {
        match socket.listener.accept() {
            Ok((stream, _)) => {
                let t_tx = tx.clone();
                thread::spawn(move || {
                    info!("New client connected");
                    handle_client(stream, t_tx).unwrap();
                    info!("Client exited");
                });
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => {}
            Err(err) => {
                error!("Failed to accept connection: {}", err);
                break;
            }
        }

        if let Ok(_) = rx.try_recv() {
            break;
        }

        sleep(Duration::from_micros(10));
    }

    info!("Daemon exited");
}

/// Start the daemon.
///
/// If `attach` is true, the daemon will run in the current process.
/// If `attach` is false, the daemon will run in a separate process.
pub fn start_daemon(socket: &DaemonSocketPath, attach: bool) -> Result<()> {
    info!("Starting daemon (attach: {})", attach);

    if attach {
        IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
        IS_DAEMON_MAIN_THREAD.with(|is_main_thread| {
            let _ = is_main_thread.set(true);
        });

        let pid = process::id();
        let start_time = get_start_time(pid)?;

        DAEMON_INFO
            .set(DaemonInfo {
                pid,
                start_time,
                stdout_path: OutputPath::Attached,
                stderr_path: OutputPath::Attached,
            })
            .unwrap();

        let listener = WrappedUnixSocket::bind(&socket.socket_path)?;
        daemon_main(listener);
    } else {
        #[cfg(unix)]
        {
            let (mut reader, mut writer) = os_pipe::pipe()?;

            match unsafe { fork()? } {
                ForkResult::Parent { child: _child } => {
                    info!("Daemon started");

                    let mut buf = [0];
                    match reader.read(buf.as_mut_slice()) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to read from child: {}", err);
                            return Err(anyhow!("Failed to start daemon"));
                        }
                    }
                }
                ForkResult::Child => {
                    IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
                    IS_DAEMON_MAIN_THREAD.with(|is_main_thread| {
                        let _ = is_main_thread.set(true);
                    });

                    setsid()?;

                    let log_dir = log_dir_res()?;
                    DirBuilder::new().recursive(true).create(&log_dir)?;

                    let stdout_fd = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(log_dir.join("foro-stdout.log"))?
                        .into_raw_fd();

                    let stderr_fd = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(log_dir.join("foro.log"))?
                        .into_raw_fd();

                    close(0)?;
                    nix::unistd::dup2(stdout_fd, 1)?;
                    nix::unistd::dup2(stderr_fd, 2)?;

                    let pid = process::id();
                    let start_time = get_start_time(pid)?;

                    DAEMON_INFO
                        .set(DaemonInfo {
                            pid,
                            start_time,
                            stdout_path: OutputPath::Path(log_dir.join("foro-stdout.log")),
                            stderr_path: OutputPath::Path(log_dir.join("foro.log")),
                        })
                        .unwrap();

                    let listener = WrappedUnixSocket::bind(&socket.socket_path)?;

                    writer.write_all(&[0])?;

                    daemon_main(listener);

                    process::exit(0);
                }
            }
        }
        #[cfg(not(unix))]
        {
            panic!(
                "not attached daemon is not supported on this platform! please run `foro daemon start -a`"
            );
        }
    }

    Ok(())
}
