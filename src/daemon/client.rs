use crate::build_info::get_build_id;
use crate::cli::GlobalOptions;
use crate::daemon::interface::{
    DaemonBulkFormatResponse, DaemonCommandPayload, DaemonCommands, DaemonFormatResponse,
    DaemonPureFormatResponse, DaemonResponse, DaemonSocketPath,
};
use crate::daemon::server::start_daemon;
use crate::daemon::uds::UnixStream;
use crate::process_utils::{get_start_time, is_alive};
use anyhow::{anyhow, Context, Result};
use file_lock::{FileLock, FileOptions};
use log::{debug, info, warn};
use std::env::current_dir;
use std::io::{ErrorKind, Read, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn parse_info(info_str: &str) -> Option<(u32, u64, String)> {
    let parts: Vec<&str> = info_str.split(',').collect();

    if parts.len() != 3 {
        return None;
    }

    let pid = parts[0].parse().ok()?;
    let start_time = parts[1].parse().ok()?;
    let build_id = parts[2].to_string();

    Some((pid, start_time, build_id))
}

#[derive(Debug, Clone)]
pub enum DaemonStatus {
    NotRunning,
    Running(String),
}

/// Check if the daemon is alive.
///
/// It works as follows:
/// 1. Reads the daemon's pid and start time from the daemon's information file
///    (which is located in the same position as the socket, etc.)
/// 2. Asks the OS for information about the process with that pid
/// 3. Checks that the process is currently alive and that the start time matches
///    (because after a process has ended, another process with the same pid may start)
///
/// This is similar to [ping], but this function only determines whether the process is alive,
/// whereas ping actually communicates and makes a determination.
/// In other words, ping is more accurate but also slower.
pub fn daemon_is_alive(socket: &DaemonSocketPath) -> Result<DaemonStatus> {
    // note: don't call path.exits()
    //       because we can reduce the number of system calls and speed up (a little bit!)
    let content = match std::fs::read_to_string(&socket.info_path) {
        Ok(content) => content,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(DaemonStatus::NotRunning),
        Err(err) => return Err(err.into()),
    };
    let (pid, start_time, build_id) =
        parse_info(&content).context("Failed to parse daemon info")?;

    if !is_alive(pid) {
        return Ok(DaemonStatus::NotRunning);
    }

    if get_start_time(pid)? != start_time {
        return Ok(DaemonStatus::NotRunning);
    }

    Ok(DaemonStatus::Running(build_id))
}

/// Send a ping to the daemon.
///
/// See also [daemon_is_alive].
pub fn ping(socket: &DaemonSocketPath) -> Result<bool> {
    match UnixStream::connect(&socket.socket_path) {
        Ok(stream) => {
            match run_command_inner(
                DaemonCommands::Ping,
                GlobalOptions {
                    config_file: None,
                    cache_dir: None,
                    socket_dir: None,
                    no_cache: false,
                    long_log: false,
                    ignore_build_id_mismatch: false,
                },
                stream,
                Some(Duration::from_secs(1)),
            ) {
                Ok(DaemonResponse::Pong(_)) => Ok(true),
                Ok(_) => Ok(false),
                Err(err) => {
                    let was_timed_out = err
                        .downcast_ref::<serde_json::Error>()
                        .and_then(|e| e.io_error_kind())
                        .is_some_and(|k| k == ErrorKind::WouldBlock);

                    if was_timed_out {
                        info!("ping timed out");
                        Ok(false)
                    } else {
                        Err(err)
                    }
                }
            }
        }
        Err(err) if err.kind() == ErrorKind::ConnectionRefused => Ok(false),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err.into()),
    }
}

fn run_command_inner(
    command: DaemonCommands,
    global_options: GlobalOptions,
    mut stream: UnixStream,
    timeout: Option<Duration>,
) -> Result<DaemonResponse> {
    let buf = serde_json::to_vec(&DaemonCommandPayload {
        command,
        current_dir: current_dir()?,
        global_options,
    })?;
    stream.write_all(&buf)?;

    debug!("Sent command");

    stream.shutdown(std::net::Shutdown::Write)?;

    stream.set_read_timeout(timeout)?;

    let res = serde_json::from_reader(stream)?;

    debug!("Received response");

    Ok(res)
}

/// ロックファイルにメタデータを書き込む
fn write_lock_metadata(lock: &mut FileLock) -> Result<()> {
    let now = SystemTime::now();
    let since_epoch = now
        .duration_since(UNIX_EPOCH)
        .context("Time went backwards")?;

    let pid = std::process::id();
    let timestamp = since_epoch.as_secs_f64();

    // ロックファイルにタイムスタンプとプロセスIDを書き込む
    let metadata = format!("{},{}", timestamp, pid);
    lock.file.set_len(0)?;
    lock.file.write_all(metadata.as_bytes())?;
    lock.file.flush()?;

    Ok(())
}

/// ロックファイルからメタデータを読み取る
fn read_lock_metadata(lock: &mut FileLock) -> Result<(f64, u32)> {
    let mut content = String::new();
    lock.file.read_to_string(&mut content)?;

    let parts: Vec<&str> = content.split(',').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid lock file format"));
    }

    let timestamp = parts[0]
        .parse::<f64>()
        .context("Failed to parse timestamp")?;
    let pid = parts[1].parse::<u32>().context("Failed to parse PID")?;

    Ok((timestamp, pid))
}

pub fn ensure_daemon_running(
    socket: &DaemonSocketPath,
    global_options: &GlobalOptions,
) -> Result<DaemonStatus> {
    let status = daemon_is_alive(socket)?;

    match status {
        DaemonStatus::NotRunning => {
            if ping(socket)? {
                info!("Daemon is already running (detected by ping)");
                return daemon_is_alive(socket);
            }

            // ロックファイルを作成または開く（書き込みモードで排他的ロックを取得）
            let file_options = FileOptions::new().read(true).write(true).create(true);

            let lock_result = FileLock::lock(&socket.lock_path, true, file_options);

            match lock_result {
                Ok(mut lock) => {
                    info!("Acquired lock for daemon startup");

                    if ping(socket)? {
                        info!("Daemon was started by another process while waiting for lock");
                        drop(lock);
                        return daemon_is_alive(socket);
                    }

                    // ロックファイルにメタデータを書き込む
                    write_lock_metadata(&mut lock)?;

                    let start_result = start_daemon(socket, false);

                    drop(lock);

                    start_result?;

                    daemon_is_alive(socket)
                }
                Err(_) => {
                    let read_options = FileOptions::new().read(true);
                    let read_lock_result = FileLock::lock(&socket.lock_path, false, read_options);

                    match read_lock_result {
                        Ok(mut read_lock) => {
                            if ping(socket)? {
                                info!(
                                    "Daemon was started by another process while waiting for lock"
                                );
                                drop(read_lock);
                                return daemon_is_alive(socket);
                            }

                            // ロックファイルからタイムスタンプを読み取る
                            let metadata_result = read_lock_metadata(&mut read_lock);
                            drop(read_lock); // 読み取り専用ロックを解放

                            if let Ok((timestamp, _)) = metadata_result {
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .context("Time went backwards")?
                                    .as_secs_f64();

                                if now - timestamp > 1.0 {
                                    info!("Breaking stale lock (older than 1 second)");

                                    if ping(socket)? {
                                        info!("Daemon was started by another process after stale lock detection");
                                        return daemon_is_alive(socket);
                                    }

                                    let force_lock_result = FileLock::lock(
                                        &socket.lock_path,
                                        true,
                                        FileOptions::new().read(true).write(true).create(true),
                                    );
                                    if let Ok(mut force_lock) = force_lock_result {
                                        if ping(socket)? {
                                            info!("Daemon was started by another process after breaking lock");
                                            drop(force_lock);
                                            return daemon_is_alive(socket);
                                        }

                                        // ロックファイルにメタデータを更新
                                        write_lock_metadata(&mut force_lock)?;

                                        let start_result = start_daemon(socket, false);
                                        drop(force_lock);
                                        start_result?;

                                        return daemon_is_alive(socket);
                                    }
                                } else {
                                    info!("Waiting for another process to start the daemon");
                                    std::thread::sleep(Duration::from_millis(100));

                                    if ping(socket)? {
                                        info!("Daemon was successfully started by another process");
                                        return daemon_is_alive(socket);
                                    }

                                    std::thread::sleep(Duration::from_millis(900));
                                    return daemon_is_alive(socket);
                                }
                            }
                        }
                        Err(_) => {
                            info!("Cannot read lock file, retrying daemon startup");

                            if ping(socket)? {
                                info!("Daemon was started by another process while attempting to read lock");
                                return daemon_is_alive(socket);
                            }

                            std::thread::sleep(Duration::from_millis(50));
                            let retry_lock = FileLock::lock(
                                &socket.lock_path,
                                true,
                                FileOptions::new().read(true).write(true).create(true),
                            );

                            if let Ok(mut lock) = retry_lock {
                                if ping(socket)? {
                                    info!("Daemon was started by another process after retry");
                                    drop(lock);
                                    return daemon_is_alive(socket);
                                }

                                write_lock_metadata(&mut lock)?;
                                let start_result = start_daemon(socket, false);
                                drop(lock);
                                start_result?;

                                return daemon_is_alive(socket);
                            }
                        }
                    }

                    daemon_is_alive(socket)
                }
            }
        }
        DaemonStatus::Running(ref daemon_build_id) => {
            let current_build_id = get_build_id();

            if daemon_build_id != &current_build_id {
                if global_options.ignore_build_id_mismatch {
                    warn!("Daemon was built with a different build ID (daemon: {}, client: {}). Continuing without restart due to --ignore-build-id-mismatch flag.", 
                        daemon_build_id, current_build_id);
                } else {
                    info!("Daemon was built with a different build ID (daemon: {}, client: {}). Restarting daemon.", 
                        daemon_build_id, current_build_id);

                    let file_options_restart =
                        FileOptions::new().read(true).write(true).create(true);

                    let lock_result = FileLock::lock(&socket.lock_path, true, file_options_restart);

                    if let Ok(mut lock) = lock_result {
                        info!("Acquired lock for daemon restart");
                        write_lock_metadata(&mut lock)?;

                        let stop_stream = UnixStream::connect(&socket.socket_path)?;
                        let _ = run_command_inner(
                            DaemonCommands::Stop,
                            global_options.clone(),
                            stop_stream,
                            None,
                        )?;

                        let start_result = start_daemon(socket, false);
                        drop(lock);
                        start_result?;

                        return daemon_is_alive(socket);
                    } else {
                        info!("Could not acquire lock for daemon restart, another process may be handling it");
                        return Ok(status);
                    }
                }
            }

            Ok(status)
        }
    }
}

/// Run a command with the daemon.
///
/// This function sends a command to the daemon which is running on the given socket,
/// and outputs the result.
pub fn run_command(
    command: DaemonCommands,
    global_options: GlobalOptions,
    socket: &DaemonSocketPath,
    check_alive: bool,
) -> Result<()> {
    if check_alive {
        match daemon_is_alive(socket)? {
            DaemonStatus::NotRunning => {
                match command {
                    DaemonCommands::Stop => {
                        // in rare cases, daemon_is_alive return false, but the process may still be alive
                        if !ping(socket)? {
                            info!("Daemon is not running");
                            return Ok(());
                        }
                    }
                    _ => {
                        return Err(anyhow!("Daemon is not running!"));
                    }
                }
            }
            DaemonStatus::Running(_) => {}
        }
    }

    let stream = UnixStream::connect(&socket.socket_path)?;

    match run_command_inner(command, global_options, stream, None)? {
        DaemonResponse::Format(DaemonFormatResponse::Success()) => {
            info!("Success to format");
        }
        DaemonResponse::Format(DaemonFormatResponse::Ignored(reason)) => {
            info!("File ignored. reason: {}", reason);
        }
        DaemonResponse::Format(DaemonFormatResponse::Error(err)) => {
            return Err(anyhow!(err));
        }
        DaemonResponse::PureFormat(DaemonPureFormatResponse::Success(formatted)) => {
            info!("Success to format");
            println!("{formatted}");
        }
        DaemonResponse::PureFormat(DaemonPureFormatResponse::Ignored(reason)) => {
            info!("File ignored. reason: {}", reason);
        }
        DaemonResponse::PureFormat(DaemonPureFormatResponse::Error(err)) => {
            return Err(anyhow!(err));
        }
        DaemonResponse::BulkFormat(DaemonBulkFormatResponse::Success(message)) => {
            info!("Success to format: {}", message);
        }
        DaemonResponse::BulkFormat(DaemonBulkFormatResponse::Error(err)) => {
            return Err(anyhow!(err));
        }
        DaemonResponse::Stop => {
            info!("Daemon stopped");
        }
        DaemonResponse::Pong(info) => {
            println!("pong!");
            println!("daemon pid: {}", &info.pid);
            println!("daemon start time: {}", &info.start_time);
            println!("daemon log file: {}", &info.stderr_path);
            println!("daemon build id: {}", &info.build_id);
        }
    }

    Ok(())
}
