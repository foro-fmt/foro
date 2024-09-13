use crate::cli::GlobalOptions;
use crate::config::load_config_and_socket;
use crate::daemon::client::run_command;
use crate::daemon::interface::{DaemonCommands, DaemonSocketPath};
use crate::daemon::server::start_daemon;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct DaemonStartArgs {
    #[clap(short, long)]
    pub attach: bool,
}

pub fn daemon_start_execute_with_args(
    args: DaemonStartArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, socket_dir) =
        load_config_and_socket(&global_options.config_file, &global_options.socket_dir)?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    start_daemon(&socket, args.attach)
}

#[derive(Parser, Debug)]
pub struct DaemonRestartArgs {
    #[clap(short, long)]
    pub attach: bool,
}

#[derive(Parser, Debug)]
pub enum DaemonSubcommands {
    #[clap(flatten)]
    ServerCommands(DaemonCommands),
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
    let (_, socket_dir) =
        load_config_and_socket(&global_options.config_file, &global_options.socket_dir)?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    run_command(DaemonCommands::Stop, global_options, &socket, true)?;

    start_daemon(&socket, args.attach)
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
            let (_, socket_dir) =
                load_config_and_socket(&global_options.config_file, &global_options.socket_dir)?;

            let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

            run_command(command, global_options, &socket, true)?;
        }
    }

    Ok(())
}
