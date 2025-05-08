use crate::cli::GlobalOptions;
use crate::config::load_config_and_socket;
use crate::daemon::client::run_command as daemon_run_command;
use crate::daemon::interface::{DaemonBulkFormatArgs, DaemonCommands, DaemonSocketPath};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct BulkFormatArgs {
    /// Paths to format
    #[clap(default_value = ".")]
    pub paths: Vec<PathBuf>,
    /// Number of threads to use
    #[clap(short, long, default_value = "0")]
    pub threads: usize,
}

pub fn bulk_format_execute_with_args(
    args: BulkFormatArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, socket_dir) = load_config_and_socket(
        global_options.config_file.as_deref(),
        global_options.socket_dir.as_deref(),
    )?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    crate::daemon::client::ensure_daemon_running(&socket, &global_options)?;

    let threads = if args.threads == 0 {
        num_cpus::get()
    } else {
        args.threads
    };

    daemon_run_command(
        DaemonCommands::BulkFormat(DaemonBulkFormatArgs {
            paths: args.paths,
            threads,
        }),
        global_options,
        &socket,
        true,
    )?;

    Ok(())
}
