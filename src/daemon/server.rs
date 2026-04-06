use crate::app_dir::{AppDirResolver, DefaultAppDirResolver};
use crate::bulk_format::{bulk_format, BulkFormatOption};
use crate::config::{load_config_and_cache, read_config_bytes};
use crate::daemon::client::ping;
use crate::daemon::interface::{
    DaemonBulkFormatArgs, DaemonBulkFormatResponse, DaemonCommandPayload, DaemonCommands,
    DaemonExecutionOptions, DaemonFormatArgs, DaemonFormatResponse, DaemonInfo, DaemonResponse,
    DaemonSocketPath, OutputPath,
};
use crate::daemon::startup_lock::StartupLock;
use crate::daemon::uds::{UnixListener, UnixStream};
use crate::debug_long;
use crate::handle_plugin::run::run;
use crate::install_check::check_ready;
use crate::log::IS_DAEMON_MAIN_THREAD;
use crate::log::IS_DAEMON_PROCESS;
use crate::path_utils::{normalize_path, to_wasm_path};
use crate::process_utils::get_start_time;
use anyhow::Result;
use anyhow::{anyhow, Context};
use foro_plugin_utils::data_json_utils::JsonGetter;
use log::{debug, error, info, warn};
use serde_json::json;
use std::fs::{DirBuilder, OpenOptions};
use std::io::prelude::*;
use std::io::{ErrorKind, Read};
use std::net::Shutdown;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, OnceLock};
use std::thread::sleep;
use std::time::Duration;
use std::{fs, process, thread};

static DAEMON_INFO: OnceLock<DaemonInfo> = OnceLock::new();

pub fn daemon_format_execute_with_args(
    args: DaemonFormatArgs,
    current_dir: PathBuf,
    execution_options: DaemonExecutionOptions,
) -> Result<DaemonFormatResponse> {
    let target_path = current_dir.join(&args.path).canonicalize()?;

    let config_bytes = read_config_bytes(execution_options.config_file.as_deref())?;
    let (config, cache_dir) = load_config_and_cache(
        execution_options.config_file.as_deref(),
        execution_options.cache_dir.as_deref(),
    )?;
    check_ready(&config_bytes, &cache_dir)
        .context("Plugins not installed: run `foro install` first")?;

    let rule = match config.find_matched_rule(&target_path) {
        Some(rule) => rule,
        None => {
            return Ok(DaemonFormatResponse::Ignored("No rule matched".to_string()));
        }
    };

    debug_long!("run rule: {:?}", rule);

    let res = run(
        &rule.cmd,
        json!({
            "wasm-current-dir":  to_wasm_path(&current_dir)?,
            "os-current-dir": normalize_path(&current_dir)?,
            "wasm-target": to_wasm_path(&target_path)?,
            "os-target": normalize_path(&target_path)?,
            "raw-target": args.path,
            "target-content": args.content,
        }),
        &cache_dir,
        true,
    )?;

    if let Some(status) = String::get_value_opt(&res, ["format-status"]) {
        match status.as_str() {
            "ignored" => {
                let reason = String::get_value_opt(&res, ["ignored-reason"])
                    .unwrap_or("File ignored".to_string());
                return Ok(DaemonFormatResponse::Ignored(reason));
            }
            "error" => {
                let error = String::get_value_opt(&res, ["format-error"]).context("Failed to get format error. Did you forget to return `format-error` in your plugin?")?;
                return Ok(DaemonFormatResponse::Error(error));
            }
            _ => {}
        }
    }

    Ok(DaemonFormatResponse::Success())
}

pub fn daemon_bulk_format_execute_with_args(
    args: DaemonBulkFormatArgs,
    current_dir: PathBuf,
    execution_options: DaemonExecutionOptions,
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

    let config_bytes = read_config_bytes(execution_options.config_file.as_deref())?;
    let (config, cache_dir) = load_config_and_cache(
        execution_options.config_file.as_deref(),
        execution_options.cache_dir.as_deref(),
    )?;
    check_ready(&config_bytes, &cache_dir)
        .context("Plugins not installed: run `foro install` first")?;

    let opt = BulkFormatOption {
        paths,
        threads: args.threads,
        use_default_ignore: true,
        current_dir,
    };

    let (changed_count, unchanged_count) = bulk_format(&opt, &config, &cache_dir, true)?;
    let total_count = changed_count + unchanged_count;

    let message = if changed_count > 0 {
        format!(
            "{} files processed. {} {} changed.",
            total_count,
            changed_count,
            if changed_count == 1 { "file" } else { "files" }
        )
    } else {
        format!("{total_count} files processed. No files changed.")
    };

    Ok(DaemonBulkFormatResponse::Success(message))
}

pub fn serverside_exec_command(payload: DaemonCommandPayload) -> DaemonResponse {
    match payload.command {
        DaemonCommands::Format(s_args) => {
            let res = daemon_format_execute_with_args(
                s_args,
                payload.current_dir,
                payload.execution_options,
            );

            match res {
                Ok(res) => DaemonResponse::Format(res),
                Err(err) => DaemonResponse::Format(DaemonFormatResponse::Error(format!("{err:#}"))),
            }
        }
        DaemonCommands::BulkFormat(s_args) => {
            let res = daemon_bulk_format_execute_with_args(
                s_args,
                payload.current_dir,
                payload.execution_options,
            );

            match res {
                Ok(res) => DaemonResponse::BulkFormat(res),
                Err(err) => {
                    DaemonResponse::BulkFormat(DaemonBulkFormatResponse::Error(format!("{err:#}")))
                }
            }
        }
        DaemonCommands::Stop => DaemonResponse::Stop,
        DaemonCommands::Ping => DaemonResponse::Pong(DAEMON_INFO.get().unwrap().clone()),
    }
}

fn read_stream_with_retry(stream: &mut UnixStream, buf: &mut Vec<u8>) -> Result<()> {
    // There is a slight time lag between when the UnixStream receives communication and when
    // all the input data is written to the stream, so if it fails, we retry after a short delay.
    // So we need to retry until we get all the data.
    //
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

    log::trace!("read socket input with {} retry", retry_cnt);

    Ok(())
}

fn handle_client(mut stream: UnixStream, stop_sender: Sender<()>) -> Result<()> {
    let mut buf = Vec::new();
    read_stream_with_retry(&mut stream, &mut buf)?;

    log::trace!("{:?}", String::from_utf8_lossy(&buf));

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
        let path = path.as_ref();

        let parent = path.parent().unwrap();
        fs::create_dir_all(parent)?;

        let info_path = parent.join(format!(
            "{}.info",
            path.file_name().unwrap().to_str().unwrap()
        ));

        let listener = match UnixListener::bind(path) {
            Ok(l) => l,
            Err(err) if err.kind() == ErrorKind::AddrInUse => {
                let as_daemon_path = DaemonSocketPath {
                    socket_dir: parent.to_path_buf(),
                    socket_path: path.to_path_buf(),
                    info_path: info_path.clone(),
                };

                if ping(&as_daemon_path)? {
                    return Err(anyhow!("Daemon is already running"));
                } else {
                    info!("Removing dead socket file");
                    let err = fs::remove_file(path);
                    if let Err(err) = err {
                        warn!("Failed to remove dead socket file: {}", err);
                    }
                    let err = fs::remove_file(&info_path);
                    if let Err(err) = err {
                        warn!("Failed to remove dead info file: {}", err);
                    }
                    UnixListener::bind(path)?
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        debug!("writing info...");

        let pid = process::id();
        let start_time = get_start_time(pid)?;
        let build_id = crate::build_info::get_build_id();
        fs::write(&info_path, format!("{pid},{start_time},{build_id}"))?;

        info!("Listening on: {}", path.display());
        info!("info path: {}", info_path.display());

        Ok(Self {
            path: path.to_path_buf(),
            listener,
        })
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
    info!("Daemon process started");

    let (tx, rx) = mpsc::channel();

    socket.listener.set_nonblocking(true).unwrap();

    loop {
        match socket.listener.accept() {
            Ok((stream, _)) => {
                let t_tx = tx.clone();
                thread::spawn(move || {
                    debug!("New client connected");
                    handle_client(stream, t_tx).unwrap();
                    debug!("Client exited");
                });
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => {}
            Err(err) => {
                error!("Failed to accept connection: {}", err);
                break;
            }
        }

        if rx.try_recv().is_ok() {
            break;
        }

        sleep(Duration::from_micros(10));
    }

    info!("Daemon exited");
}

#[cfg(unix)]
fn start_daemon_no_attach(socket: &DaemonSocketPath) -> Result<()> {
    use nix::unistd::{close, fork, setsid, ForkResult};
    use std::os::fd::IntoRawFd;

    let (mut reader, mut writer) = os_pipe::pipe()?;

    match unsafe { fork()? } {
        ForkResult::Parent { child: _child } => {
            info!("Daemon started");

            // If parent holds writer, even if child terminates with an error,
            // reader.read will not return Err and will hang indefinitely.
            // Therefore, it is immediately dropped.
            drop(writer);

            let mut buf = [0];
            match reader.read_exact(buf.as_mut_slice()) {
                Ok(_) => Ok(()),
                Err(err) => {
                    error!("Failed to read from child: {}", err);
                    Err(anyhow!("Failed to start daemon"))
                }
            }
        }
        ForkResult::Child => {
            drop(reader); // same as above

            IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
            IS_DAEMON_MAIN_THREAD.with(|is_main_thread| {
                let _ = is_main_thread.set(true);
            });

            setsid()?;

            // todo: It is not good design to directly obtain log_dir here.

            let resolver = DefaultAppDirResolver {};
            let log_dir = resolver.log_dir_res()?;
            DirBuilder::new().recursive(true).create(&log_dir)?;

            let stdout_fd = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_dir.join("foro-stdout.log"))?
                .into_raw_fd();

            let stderr_fd = OpenOptions::new()
                .create(true)
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
                    build_id: crate::build_info::get_build_id(),
                })
                .unwrap();

            let listener = WrappedUnixSocket::bind(&socket.socket_path)?;

            writer.write_all(&[0])?;

            daemon_main(listener);

            process::exit(0);
        }
    }
}

#[cfg(windows)]
fn wait_for_daemon_ready(
    socket: &DaemonSocketPath,
    child: &mut process::Child,
    timeout: Duration,
) -> Result<()> {
    let started = std::time::Instant::now();
    let mut last_ping_error: Option<anyhow::Error> = None;

    while started.elapsed() < timeout {
        match ping(socket) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(err) => {
                last_ping_error = Some(err);
            }
        }

        if let Some(status) = child.try_wait()? {
            let mut message = format!("Failed to start daemon: child exited early ({status})");
            if let Some(err) = last_ping_error {
                message.push_str(&format!(", last ping error: {err:#}"));
            }
            return Err(anyhow!(message));
        }

        sleep(Duration::from_millis(10));
    }

    let mut message = "Failed to start daemon: timeout waiting for daemon readiness".to_string();
    if let Some(err) = last_ping_error {
        message.push_str(&format!(", last ping error: {err:#}"));
    }
    Err(anyhow!(message))
}

#[cfg(windows)]
fn start_daemon_no_attach(socket: &DaemonSocketPath) -> Result<()> {
    use std::env;
    use std::os::windows::io::{AsHandle, AsRawHandle};
    use winapi::um::handleapi::{SetHandleInformation, INVALID_HANDLE_VALUE};
    use winapi::um::processenv::{GetStdHandle, SetStdHandle};
    use winapi::um::winbase::{
        HANDLE_FLAG_INHERIT, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };
    use winapi::um::winnt::HANDLE;

    if !env::var("FORO_WINDOWS_IS_DAEMON").is_ok() {
        let current_exe = env::current_exe()?;

        // Prevent child process from inheriting caller pipes (used by `Command::output()`).
        // If inherited, test process can block forever waiting for EOF.
        unsafe {
            for std_handle in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
                let handle = GetStdHandle(std_handle);
                if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
                    let _ = SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0);
                }
            }
        }

        let mut child = process::Command::new(current_exe)
            .arg("daemon")
            .arg("start")
            .arg("--socket-dir")
            .arg(&socket.socket_dir)
            .env("FORO_WINDOWS_IS_DAEMON", "1")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .spawn();

        if let Err(err) = child {
            error!("Failed to start daemon: {}", err);
            return Err(anyhow!("Failed to start daemon: {}", err));
        }

        let child = child.as_mut().unwrap();

        info!("Daemon started");

        // Wait until the daemon is truly ready (socket bound and pingable).
        wait_for_daemon_ready(socket, child, Duration::from_secs(15))?;

        return Ok(());
    }

    IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
    IS_DAEMON_MAIN_THREAD.with(|is_main_thread| {
        let _ = is_main_thread.set(true);
    });

    let resolver = DefaultAppDirResolver {};
    let log_dir = resolver.log_dir_res()?;
    DirBuilder::new().recursive(true).create(&log_dir)?;

    let stdout_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_dir.join("foro-stdout.log"))?;

    let stderr_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_dir.join("foro.log"))?;

    let stdout_handle = stdout_file.as_raw_handle();
    let stderr_handle = stderr_file.as_raw_handle();

    unsafe {
        SetStdHandle(STD_INPUT_HANDLE, std::ptr::null_mut());
        SetStdHandle(STD_OUTPUT_HANDLE, stdout_handle as HANDLE);
        SetStdHandle(STD_ERROR_HANDLE, stderr_handle as HANDLE);
    }

    let pid = process::id();
    let start_time = get_start_time(pid)?;

    DAEMON_INFO
        .set(DaemonInfo {
            pid,
            start_time,
            stdout_path: OutputPath::Path(log_dir.join("foro-stdout.log")),
            stderr_path: OutputPath::Path(log_dir.join("foro.log")),
            build_id: crate::build_info::get_build_id(),
        })
        .unwrap();

    let listener = WrappedUnixSocket::bind(&socket.socket_path)?;

    daemon_main(listener);

    process::exit(0);
}

#[cfg(not(any(unix, windows)))]
fn start_daemon_no_attach(socket: &DaemonSocketPath) -> Result<()> {
    panic!(
        "not attached daemon is not supported on this platform! please run `foro daemon start -a`"
    );
}

/// Start the daemon.
///
/// If `attach` is true, the daemon will run in the current process.
/// If `attach` is false, the daemon will run in a separate process.
pub fn start_daemon(socket: &DaemonSocketPath, lock: StartupLock, attach: bool) -> Result<()> {
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
                build_id: crate::build_info::get_build_id(),
            })
            .unwrap();

        let listener = WrappedUnixSocket::bind(&socket.socket_path)?;

        lock.free()?;

        daemon_main(listener);
    } else {
        start_daemon_no_attach(socket)?;
        lock.free()?;
    }

    Ok(())
}
