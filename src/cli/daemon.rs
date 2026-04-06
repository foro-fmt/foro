use crate::cli::GlobalOptions;
use crate::config::load_config_and_socket;
use crate::daemon::client::{daemon_is_alive, run_command, DaemonStatus};
use crate::daemon::interface::{
    DaemonBulkFormatArgs, DaemonCommands, DaemonExecutionOptions, DaemonFormatArgs,
    DaemonSocketPath,
};
use crate::daemon::server::start_daemon;
use crate::daemon::startup_lock::StartupLock;
use anyhow::Result;
use clap::Parser;
use log::info;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct DaemonStartArgs {
    #[clap(short, long)]
    pub attach: bool,
}

pub fn daemon_start_execute_with_args(
    args: DaemonStartArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, socket_dir) = load_config_and_socket(
        global_options.config_file.as_deref(),
        global_options.socket_dir.as_deref(),
    )?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    #[cfg(windows)]
    if std::env::var("FORO_WINDOWS_IS_DAEMON").is_ok() {
        // Child process spawned by `start_daemon_no_attach` should not re-enter
        // startup-lock/daemon-state checks; parent process already handles them.
        return start_daemon(&socket, StartupLock::noop(), args.attach);
    }

    let lock = StartupLock::acquire(&socket_dir)?;

    if matches!(daemon_is_alive(&socket)?, DaemonStatus::Running(_)) {
        info!("Daemon is already running");
        return Ok(());
    }

    start_daemon(&socket, lock, args.attach)?;

    Ok(())
}

#[derive(Parser, Debug)]
pub struct DaemonRestartArgs {
    #[clap(short, long)]
    pub attach: bool,
}

#[derive(Parser, Debug)]
pub struct DaemonFormatCliArgs {
    /// Path to format
    pub path: PathBuf,
    pub content: String,
}

#[derive(Parser, Debug)]
pub struct DaemonBulkFormatCliArgs {
    /// Paths to format
    pub paths: Vec<PathBuf>,
    /// Number of threads to use
    pub threads: usize,
}

#[derive(Parser, Debug)]
pub enum DaemonServerCommands {
    Format(DaemonFormatCliArgs),
    BulkFormat(DaemonBulkFormatCliArgs),
    Stop,
    Ping,
}

impl From<DaemonServerCommands> for DaemonCommands {
    fn from(value: DaemonServerCommands) -> Self {
        match value {
            DaemonServerCommands::Format(args) => DaemonCommands::Format(DaemonFormatArgs {
                path: args.path,
                content: args.content,
            }),
            DaemonServerCommands::BulkFormat(args) => {
                DaemonCommands::BulkFormat(DaemonBulkFormatArgs {
                    paths: args.paths,
                    threads: args.threads,
                })
            }
            DaemonServerCommands::Stop => DaemonCommands::Stop,
            DaemonServerCommands::Ping => DaemonCommands::Ping,
        }
    }
}

#[derive(Parser, Debug)]
pub enum DaemonSubcommands {
    #[clap(flatten)]
    ServerCommands(DaemonServerCommands),
    Start(DaemonStartArgs),
    Restart(DaemonRestartArgs),
}

#[derive(Parser, Debug)]
pub struct DaemonArgs {
    #[clap(subcommand)]
    pub subcommand: DaemonSubcommands,
}

pub fn daemon_restart_execute_with_args(
    args: DaemonRestartArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, socket_dir) = load_config_and_socket(
        global_options.config_file.as_deref(),
        global_options.socket_dir.as_deref(),
    )?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    #[cfg(windows)]
    if std::env::var("FORO_WINDOWS_IS_DAEMON").is_ok() {
        return start_daemon(&socket, StartupLock::noop(), args.attach);
    }

    let lock = StartupLock::acquire(&socket_dir)?;
    let daemon_options = DaemonExecutionOptions::from(&global_options);

    run_command(DaemonCommands::Stop, daemon_options, &socket, true)?;

    start_daemon(&socket, lock, args.attach)?;

    Ok(())
}

pub fn daemon_execute_with_args(args: DaemonArgs, global_options: GlobalOptions) -> Result<()> {
    match args.subcommand {
        DaemonSubcommands::Start(s_args) => {
            daemon_start_execute_with_args(s_args, global_options)?;
        }
        DaemonSubcommands::Restart(s_args) => {
            daemon_restart_execute_with_args(s_args, global_options)?;
        }
        DaemonSubcommands::ServerCommands(command) => {
            let (_, socket_dir) = load_config_and_socket(
                global_options.config_file.as_deref(),
                global_options.socket_dir.as_deref(),
            )?;

            let socket = DaemonSocketPath::from_socket_dir(&socket_dir);
            let daemon_options = DaemonExecutionOptions::from(&global_options);

            run_command(command.into(), daemon_options, &socket, true)?;
        }
    }

    Ok(())
}
