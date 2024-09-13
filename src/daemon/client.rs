use crate::cli::GlobalOptions;
use crate::daemon::interface::{
    DaemonCommandPayload, DaemonCommands, DaemonFormatResponse, DaemonPureFormatResponse,
    DaemonResponse, DaemonSocketPath,
};
use crate::process_utils::{get_start_time, is_alive};
use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use std::env::current_dir;
use std::io::{ErrorKind, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::time::Duration;
#[cfg(windows)]
use uds_windows::UnixStream;

fn parse_info(info_str: &str) -> Option<(u32, u64)> {
    let parts: Vec<&str> = info_str.split(',').collect();

    if parts.len() != 2 {
        return None;
    }

    let pid = parts[0].parse().ok()?;
    let start_time = parts[1].parse().ok()?;

    Some((pid, start_time))
}

pub fn daemon_is_alive(socket: &DaemonSocketPath) -> Result<bool> {
    // don't call path.exits()
    // because we can reduce the number of system calls and speed up (a little bit!)
    let content = match std::fs::read_to_string(&socket.info_path) {
        Ok(content) => content,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err.into()),
    };
    let (pid, start_time) = parse_info(&content).context("Failed to parse daemon info")?;

    if !is_alive(pid) {
        return Ok(false);
    }

    if get_start_time(pid)? != start_time {
        return Ok(false);
    }

    Ok(true)
}

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
                        Err(err.into())
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

pub fn run_command(
    command: DaemonCommands,
    global_options: GlobalOptions,
    socket: &DaemonSocketPath,
    check_alive: bool,
) -> Result<()> {
    if check_alive && !daemon_is_alive(&socket)? {
        match command {
            DaemonCommands::Stop => {
                // in rare cases, daemon_is_alive return false, but the process may still be alive
                if !ping(&socket)? {
                    info!("Daemon is not running");
                    return Ok(());
                }
            }
            _ => {
                return Err(anyhow!("Daemon is not running!"));
            }
        }
    }

    let stream = UnixStream::connect(&socket.socket_path)?;

    match run_command_inner(command, global_options, stream, None)? {
        DaemonResponse::Format(DaemonFormatResponse::Success()) => {
            println!("Success to format");
        }
        DaemonResponse::Format(DaemonFormatResponse::Ignored()) => {
            println!("File ignored");
        }
        DaemonResponse::Format(DaemonFormatResponse::Error(err)) => {
            return Err(anyhow!(err));
        }
        DaemonResponse::PureFormat(DaemonPureFormatResponse::Success(formatted)) => {
            println!("Success to format");
            println!("{}", formatted);
        }
        DaemonResponse::PureFormat(DaemonPureFormatResponse::Ignored()) => {
            println!("File ignored");
        }
        DaemonResponse::PureFormat(DaemonPureFormatResponse::Error(err)) => {
            return Err(anyhow!(err));
        }
        DaemonResponse::Stop => {
            println!("Daemon stopped");
        }
        DaemonResponse::Pong(info) => {
            println!("pong!");
            println!("daemon pid: {}", &info.pid);
            println!("daemon start time: {}", &info.start_time);
            println!("daemon log file: {}", &info.stderr_path);
        }
    }

    Ok(())
}
