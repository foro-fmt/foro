use crate::cli::GlobalOptions;
use crate::daemon::interface::{
    DaemonCommandPayload, DaemonCommands, DaemonFormatResponse, DaemonResponse, DaemonSocketPath,
};
use crate::daemon::server::start_daemon;
use anyhow::__private::kind::TraitKind;
use anyhow::{anyhow, Result};
use log::{debug, error, info};
use std::env::current_dir;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub fn ping(socket: &DaemonSocketPath) -> Result<bool> {
    match run_command_inner_other_os(
        DaemonCommands::Ping,
        GlobalOptions {
            config_file: None,
            cache_dir: None,
            no_cache: false,
        },
        &socket.filemq_space_dir,
        Some(Duration::from_secs(1)),
    ) {
        Ok(Some(DaemonResponse::Pong)) => Ok(true),
        Ok(Some(_)) => Ok(false),
        Ok(None) => Ok(false),
        Err(err) => Err(err.into()),
    }
}

fn run_command_inner_other_os(
    command: DaemonCommands,
    global_options: GlobalOptions,
    space: &PathBuf,
    timeout: Option<Duration>,
) -> Result<Option<DaemonResponse>> {
    let res_handler = filemq::req_res::write_message_req::<_, DaemonResponse>(
        space,
        DaemonCommandPayload {
            command,
            current_dir: current_dir()?,
            global_options,
        },
    )?;

    debug!("Sent command");

    let start_time = std::time::Instant::now();

    loop {
        if let Some(res) = res_handler.try_read()? {
            debug!("Received response");

            return Ok(Some(res));
        }

        if let Some(timeout) = timeout {
            if start_time.elapsed() > timeout {
                return Ok(None);
            }
        }

        sleep(Duration::from_micros(100));
    }
}

pub fn run_command(
    command: DaemonCommands,
    global_options: GlobalOptions,
    socket: &DaemonSocketPath,
    no_auto_start: bool,
) -> Result<()> {
    if !ping(&socket)? {
        if no_auto_start {
            return Err(anyhow!("Daemon is not running"));
        } else {
            start_daemon(&socket, false)?;
        }
    }

    // Ok(None) means timeout, so we don't need to handle it
    let res = run_command_inner_other_os(command, global_options, &socket.filemq_space_dir, None)?
        .unwrap();

    match res {
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
