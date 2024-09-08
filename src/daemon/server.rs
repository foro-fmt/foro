use crate::cli::format::FormatArgs;
use crate::cli::{GlobalOptions, IS_DAEMON_PROCESS};
use crate::config::{load_config_and_cache, load_config_and_socket};
use crate::daemon::client::ping;
use crate::daemon::interface::{
    DaemonCommandPayload, DaemonCommands, DaemonFormatArgs, DaemonFormatResponse, DaemonResponse,
};
use crate::handle_plugin::run::run;
use anyhow::{anyhow, Context, Result};
use log::{debug, error, info};
use nix::unistd::{fork, ForkResult};
use os_pipe::PipeWriter;
use serde_json::json;
use std::env::current_dir;
use std::io::{ErrorKind, Read};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io, process, thread, time};

pub fn daemon_format_execute_with_args(
    args: DaemonFormatArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (config, cache_dir) =
        load_config_and_cache(&global_options.config_file, &global_options.cache_dir)?;

    let file = fs::File::open(&args.path)?;
    let mut buf_reader = io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let res = run(
        &config.rules.first().unwrap().cmd,
        json!({
            "current-dir": current_dir()?.canonicalize()?.to_str().unwrap(),
            "target": args.path.canonicalize()?.to_str().unwrap(),
            "raw-target": args.path,
            "target-content": contents,
            }
        ),
        &cache_dir,
        !global_options.no_cache,
    )?;

    println!("{:?}", res);

    Ok(())
}

fn serverside_exec_command(
    cmd: DaemonCommands,
    global_options: GlobalOptions,
) -> Result<DaemonResponse> {
    match cmd {
        DaemonCommands::Format(s_args) => {
            match daemon_format_execute_with_args(s_args, global_options) {
                Ok(_) => Ok(DaemonResponse::Format(DaemonFormatResponse::Success)),
                Err(err) => Ok(DaemonResponse::Format(DaemonFormatResponse::Error(
                    err.to_string(),
                ))),
            }
        }
        DaemonCommands::Ping => Ok(DaemonResponse::Pong),
    }
}

fn handle_client(mut stream: UnixStream) -> Result<()> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let payload: DaemonCommandPayload = serde_json::from_slice(&buf)?;

    info!("Received: {:?}", &payload);

    let response = serverside_exec_command(payload.command, payload.global_options)?;

    info!("Response: {:?}", &response);

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
                if ping(&path)? {
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

pub fn daemon_main(socket: WrappedUnixSocket) -> Result<()> {
    for stream in socket.listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                error!("Error: {}", err);
            }
        }
    }

    Ok(())
}

pub fn start_daemon(socket: &PathBuf, attach: bool) -> Result<()> {
    let listener = WrappedUnixSocket::bind(socket)?;

    if attach {
        IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
        daemon_main(listener)?;
    } else {
    }

    Ok(())
}
