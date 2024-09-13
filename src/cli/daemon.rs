// platform-dependent: foro communicates with a daemon via unix domain socket.
//   this is platform-dependent.
//   the part about passing the socket path as a command argument also needs
// to be considered as an alternative.

use crate::cli::cache::CacheSubCommands;
use crate::cli::format::FormatArgs;
use crate::cli::GlobalOptions;
use crate::config::{load_config_and_cache, load_config_and_socket};
use crate::daemon::client::{ping, run_command};
use crate::daemon::interface::{DaemonCommands, DaemonSocketPath};
use crate::handle_plugin::run::run;
use crate::main;
use anyhow::{anyhow, Result};
use clap::Parser;
use log::{debug, error, info};
use os_pipe::PipeWriter;
use serde_json::json;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io, process, time};
use crate::daemon::server::start_daemon;

#[derive(Parser, Debug)]
pub struct DaemonStartArgs {
    #[clap(short, long)]
    pub attach: bool,
}

pub fn daemon_start_execute_with_args(
    args: DaemonStartArgs,
    global_options: GlobalOptions,
    daemon_global: DaemonGlobalOptions,
) -> Result<()> {
    let (_, socket_dir) =
        load_config_and_socket(&global_options.config_file, &daemon_global.socket_path)?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    start_daemon(&socket, args.attach)
}

#[derive(Parser, Debug)]
pub enum DaemonSubcommands {
    #[clap(flatten)]
    ServerCommands(DaemonCommands),
    Start(DaemonStartArgs),
}

#[derive(Parser, Debug)]
pub struct DaemonGlobalOptions {
    #[arg(long, value_name = "PATH")]
    pub socket_path: Option<PathBuf>,
    #[arg(long)]
    pub no_auto_start: bool,
}

#[derive(Parser, Debug)]
pub struct DaemonArgs {
    #[clap(subcommand)]
    pub subcommand: DaemonSubcommands,

    #[command(flatten)]
    pub global_options: DaemonGlobalOptions,
}

pub fn daemon_execute_with_args(args: DaemonArgs, global_options: GlobalOptions) -> Result<()> {
    match args.subcommand {
        DaemonSubcommands::Start(s_args) => {
            daemon_start_execute_with_args(s_args, global_options, args.global_options)?;
        }
        DaemonSubcommands::ServerCommands(command) => {
            let (_, socket_dir) = load_config_and_socket(
                &global_options.config_file,
                &args.global_options.socket_path,
            )?;

            let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

            run_command(
                command,
                global_options,
                &socket,
                args.global_options.no_auto_start,
            )?;
        }
    }

    Ok(())
}
