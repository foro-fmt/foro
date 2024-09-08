use crate::cli::GlobalOptions;
use crate::daemon::interface::{
    DaemonCommandPayload, DaemonCommands, DaemonFormatResponse, DaemonResponse,
};
use crate::daemon::server::start_daemon;
use anyhow::{anyhow, Result};
use log::error;
use std::io::ErrorKind;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread;
use std::thread::sleep;

pub fn ping(socket_path: &PathBuf) -> Result<bool> {
    match UnixStream::connect(socket_path) {
        Ok(stream) => {
            if let DaemonResponse::Pong = run_command_inner(
                DaemonCommands::Ping,
                GlobalOptions {
                    config_file: None,
                    cache_dir: None,
                    no_cache: false,
                },
                stream,
            )? {
                Ok(true)
            } else {
                Ok(false)
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
) -> Result<DaemonResponse> {
    serde_json::to_writer(
        &stream,
        &DaemonCommandPayload {
            command,
            global_options,
        },
    )?;

    stream.shutdown(std::net::Shutdown::Write)?;

    let res = serde_json::from_reader(stream)?;

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

    match run_command_inner(command, global_options, stream)? {
        DaemonResponse::Format(DaemonFormatResponse::Success) => {
            println!("Success to format");
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
