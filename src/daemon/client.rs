use crate::build_info::get_build_id;
use crate::daemon::interface::{
    BulkFormatSummary, DaemonBulkFormatResponse, DaemonCommandPayload, DaemonCommands,
    DaemonExecutionOptions, DaemonFormatResponse, DaemonResponse, DaemonSocketPath,
};
use crate::daemon::server::start_daemon;
use crate::daemon::startup_lock::StartupLock;
use crate::daemon::uds::UnixStream;
use crate::process_utils::{get_start_time, is_alive};
use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use std::env::current_dir;
use std::io::{ErrorKind, Write};
use std::time::Duration;

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
                DaemonExecutionOptions::default(),
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
    mut execution_options: DaemonExecutionOptions,
    mut stream: UnixStream,
    timeout: Option<Duration>,
) -> Result<DaemonResponse> {
    let cwd = current_dir()?;

    // Convert relative config_file path to absolute path
    if let Some(config_file) = execution_options.config_file {
        execution_options.config_file = Some(cwd.join(config_file).canonicalize()?);
    }

    let buf = serde_json::to_vec(&DaemonCommandPayload {
        command,
        current_dir: cwd,
        execution_options,
    })?;
    stream.write_all(&buf)?;

    debug!("Sent command");

    stream.shutdown(std::net::Shutdown::Write)?;

    stream.set_read_timeout(timeout)?;

    let res = serde_json::from_reader(stream)?;

    debug!("Received response");

    Ok(res)
}

pub fn ensure_daemon_running(
    socket: &DaemonSocketPath,
    execution_options: &DaemonExecutionOptions,
) -> Result<()> {
    let lock = StartupLock::acquire(&socket.socket_dir)?;

    let status = daemon_is_alive(socket)?;

    match status {
        DaemonStatus::NotRunning => {
            start_daemon(socket, lock, false)?;
        }
        DaemonStatus::Running(ref daemon_build_id) => {
            let current_build_id = get_build_id();

            if daemon_build_id != &current_build_id {
                if execution_options.ignore_build_id_mismatch {
                    warn!("Daemon was built with a different build ID (daemon: {}, client: {}). Continuing without restart due to --ignore-build-id-mismatch flag.",
                        daemon_build_id, current_build_id);
                } else {
                    info!("Daemon was built with a different build ID (daemon: {}, client: {}). Restarting daemon.",
                        daemon_build_id, current_build_id);

                    let stop_stream = UnixStream::connect(&socket.socket_path)?;
                    let _ = run_command_inner(
                        DaemonCommands::Stop,
                        execution_options.clone(),
                        stop_stream,
                        None,
                    )?;

                    start_daemon(socket, lock, false)?;
                }
            }
        }
    }

    Ok(())
}

/// Run a command with the daemon.
///
/// This function sends a command to the daemon which is running on the given socket,
/// and outputs the result.
pub fn run_command(
    command: DaemonCommands,
    execution_options: DaemonExecutionOptions,
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

    match run_command_inner(command, execution_options, stream, None)? {
        DaemonResponse::Format(DaemonFormatResponse::Success()) => {
            eprintln!("Formatted successfully.");
        }
        DaemonResponse::Format(DaemonFormatResponse::Ignored(reason)) => {
            eprintln!("File ignored: {}", reason);
        }
        DaemonResponse::Format(DaemonFormatResponse::Error(err)) => {
            return Err(anyhow!(err));
        }
        DaemonResponse::BulkFormat(DaemonBulkFormatResponse::Success(summary)) => {
            eprintln!(
                "Formatted successfully: {}",
                format_bulk_success_message(summary)
            );
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

fn format_bulk_success_message(summary: BulkFormatSummary) -> String {
    if summary.changed_count > 0 {
        format!(
            "{} files processed. {} {} changed.",
            summary.total_count,
            summary.changed_count,
            if summary.changed_count == 1 {
                "file"
            } else {
                "files"
            }
        )
    } else {
        format!("{} files processed. No files changed.", summary.total_count)
    }
}

#[cfg(test)]
mod tests {
    use super::format_bulk_success_message;
    use crate::daemon::interface::BulkFormatSummary;

    #[test]
    fn format_bulk_success_message_reports_no_changes() {
        let summary = BulkFormatSummary {
            total_count: 3,
            changed_count: 0,
            unchanged_count: 3,
        };

        assert_eq!(
            format_bulk_success_message(summary),
            "3 files processed. No files changed."
        );
    }

    #[test]
    fn format_bulk_success_message_reports_changed_files() {
        let summary = BulkFormatSummary {
            total_count: 3,
            changed_count: 2,
            unchanged_count: 1,
        };

        assert_eq!(
            format_bulk_success_message(summary),
            "3 files processed. 2 files changed."
        );
    }

    #[test]
    fn format_bulk_success_message_reports_single_file_changed() {
        let summary = BulkFormatSummary {
            total_count: 5,
            changed_count: 1,
            unchanged_count: 4,
        };

        assert_eq!(
            format_bulk_success_message(summary),
            "5 files processed. 1 file changed."
        );
    }
}
