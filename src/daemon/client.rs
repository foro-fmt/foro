use crate::cli::GlobalOptions;
use crate::daemon::interface::{
    DaemonCommandPayload, DaemonCommands, DaemonFormatResponse, DaemonResponse,
};
use crate::daemon::server::start_daemon;
use anyhow::__private::kind::TraitKind;
use anyhow::{anyhow, Result};
use log::{error, info};
use std::env::current_dir;
use std::io::ErrorKind;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub fn ping(socket_path: &PathBuf) -> Result<bool> {
    match UnixStream::connect(socket_path) {
        Ok(stream) => {
            match run_command_inner(
                DaemonCommands::Ping,
                GlobalOptions {
                    config_file: None,
                    cache_dir: None,
                    no_cache: false,
                },
                stream,
                Some(Duration::from_secs(1)),
            ) {
                Ok(DaemonResponse::Pong) => Ok(true),
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

pub fn run_command_inner(
    command: DaemonCommands,
    global_options: GlobalOptions,
    stream: UnixStream,
    timeout: Option<Duration>,
) -> Result<DaemonResponse> {
    serde_json::to_writer(
        &stream,
        &DaemonCommandPayload {
            command,
            current_dir: current_dir()?,
            global_options,
        },
    )?;

    info!("Sent command");

    stream.shutdown(std::net::Shutdown::Write)?;

    stream.set_read_timeout(timeout)?;

    let res = serde_json::from_reader(stream)?;

    info!("Received response");

    Ok(res)
}

pub fn run_command(
    command: DaemonCommands,
    global_options: GlobalOptions,
    socket: &PathBuf,
    no_auto_start: bool,
) -> Result<()> {
    if !ping(&socket)? {
        if no_auto_start {
            return Err(anyhow!("Daemon is not running"));
        } else {
            start_daemon(&socket, false)?;
        }
    }

    let stream = UnixStream::connect(&socket)?;

    match run_command_inner(command, global_options, stream, None)? {
        DaemonResponse::Format(DaemonFormatResponse::Success) => {
            println!("Success to format");
        }
        DaemonResponse::Format(DaemonFormatResponse::Ignored) => {
            println!("File ignored");
        }
        DaemonResponse::Format(DaemonFormatResponse::Error(err)) => {
            return Err(anyhow!(err));
        }
        DaemonResponse::Pong => {
            println!("pong");
        }
    }

    Ok(())
}
