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

fn handle_client_unix(mut stream: UnixStream) -> Result<()> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let payload: DaemonCommandPayload = serde_json::from_slice(&buf)?;

    debug!("Received: {:?}", &payload);

    let response = serverside_exec_command(payload);

    debug!("Response: {:?}", &response);

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
                let as_daemon_path = DaemonSocketPath {
                    socket_path: path.clone(),
                };

                if ping(&as_daemon_path)? {
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

pub fn daemon_main_unix(socket: WrappedUnixSocket) -> Result<()> {
    for stream in socket.listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    info!("New client connected");
                    handle_client_unix(stream).unwrap();
                    info!("Client exited");
                });
            }
            Err(err) => {
                error!("Error: {}", err);
            }
        }
    }

    Ok(())
}

pub fn start_daemon(socket: &DaemonSocketPath, attach: bool) -> Result<()> {
    info!("Starting daemon (attach: {})", attach);

    let listener = WrappedUnixSocket::bind(&socket.socket_path)?;

    if attach {
        IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
        IS_DAEMON_MAIN_THREAD.with(|is_main_thread| {
            let _ = is_main_thread.set(true);
        });

        info!("Daemon started");

        daemon_main_unix(listener)?;
    } else {
    }

    Ok(())
}
