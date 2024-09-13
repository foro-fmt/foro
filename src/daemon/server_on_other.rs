use crate::cli::format::FormatArgs;
use crate::cli::{GlobalOptions, DAEMON_THREAD_START, IS_DAEMON_MAIN_THREAD, IS_DAEMON_PROCESS};
use crate::config::{load_config_and_cache, load_config_and_socket};
use crate::daemon::client::ping;
use crate::daemon::interface::{
    DaemonCommandPayload, DaemonCommands, DaemonFormatArgs, DaemonFormatResponse, DaemonResponse,
    DaemonSocketPath,
};
use crate::daemon::server_exec::serverside_exec_command;
use crate::handle_plugin::run::run;
use anyhow::{anyhow, Context, Result};
use clap::builder::Str;
use log::{debug, error, info, trace};
use notify::Watcher;
use os_pipe::PipeWriter;
use serde_json::json;
use std::env::current_dir;
use std::ffi::CString;
use std::io::{ErrorKind, Read};
use std::net::Shutdown;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{fs, io, mem, process, thread, time};
use wasmtime::component::__internal::wasmtime_environ::wasmparser::Payload;

fn handle_client_other_os(
    space: &PathBuf,
    name: String,
    payload: DaemonCommandPayload,
) -> Result<()> {
    debug!("Received: {:?}", &payload);

    let response = serverside_exec_command(payload);

    debug!("Response: {:?}", &response);

    filemq::req_res::write_message_res(space, &name, response)?;

    Ok(())
}

pub fn daemon_main_other_os(space: &PathBuf) -> Result<()> {
    loop {
        if let Some((name, msg)) =
            filemq::req_res::read_earliest_req::<DaemonCommandPayload>(space)?
        {
            let own_space = space.clone();
            thread::spawn(move || {
                info!("New client connected");
                handle_client_other_os(&own_space, name, msg).unwrap();
                info!("Client exited");
            });
        }
    }
}

pub fn start_daemon(socket: &DaemonSocketPath, attach: bool) -> Result<()> {
    info!("Starting daemon (attach: {})", attach);

    if attach {
        IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
        IS_DAEMON_MAIN_THREAD.with(|is_main_thread| {
            let _ = is_main_thread.set(true);
        });

        info!("Daemon started");

        daemon_main_other_os(&socket.filemq_space_dir)?;
    } else {
    }

    Ok(())
}
