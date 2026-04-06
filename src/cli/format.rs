use crate::cli::GlobalOptions;
use crate::config::load_paths;
use crate::daemon::client::{ensure_daemon_running, run_command as daemon_run_command};
use crate::daemon::interface::{DaemonBulkFormatArgs, DaemonCommands, DaemonFormatArgs, DaemonSocketPath};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct FormatArgs {
    /// Paths to format
    #[clap(default_value = ".")]
    pub paths: Vec<PathBuf>,
    /// Number of threads to use
    #[clap(short, long, default_value = "0")]
    pub threads: usize,
}

pub fn format_execute_with_args(args: FormatArgs, global_options: GlobalOptions) -> Result<()> {
    let (_, _, socket_dir) = load_paths(
        global_options.config_file.as_deref(),
        global_options.cache_dir.as_deref(),
        global_options.socket_dir.as_deref(),
    )?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    ensure_daemon_running(&socket, &global_options)?;

    // If only one path is given and it's a file, use Format command
    if args.paths.len() == 1 && args.paths[0].is_file() {
        let content = std::fs::read_to_string(&args.paths[0])?;
        daemon_run_command(
            DaemonCommands::Format(DaemonFormatArgs {
                path: args.paths[0].clone(),
                content,
            }),
            global_options,
            &socket,
            false,
        )?;
    } else {
        // Otherwise, use BulkFormat command
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
    }

    Ok(())
}
