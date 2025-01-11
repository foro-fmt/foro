use crate::cli::GlobalOptions;
use crate::config::load_config_and_socket;
use crate::daemon::client::{daemon_is_alive, run_command as daemon_run_command};
use crate::daemon::interface::{DaemonBulkFormatArgs, DaemonCommands, DaemonSocketPath};
use crate::daemon::server::start_daemon;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct BulkFormatArgs {
    /// Paths to format
    #[clap(default_value = ".")]
    pub paths: Vec<PathBuf>,
}

pub fn bulk_format_execute_with_args(
    args: BulkFormatArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, socket_dir) =
        load_config_and_socket(&global_options.config_file, &global_options.socket_dir)?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    if !daemon_is_alive(&socket)? {
        start_daemon(&socket, false)?;
    }

    daemon_run_command(
        DaemonCommands::BulkFormat(DaemonBulkFormatArgs { paths: args.paths }),
        global_options,
        &socket,
        false,
    )?;

    Ok(())
}
