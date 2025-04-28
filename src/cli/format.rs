use crate::cli::GlobalOptions;
use crate::config::{load_config_and_cache, load_config_and_socket};
use crate::daemon::client::{ensure_daemon_running, run_command as daemon_run_command};
use crate::daemon::interface::{DaemonCommands, DaemonFormatArgs, DaemonSocketPath};
use crate::debug_long;
use crate::handle_plugin::run::run;
use crate::path_utils::{normalize_path, to_wasm_path};
use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use serde_json::json;
use std::env::current_dir;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Parser, Debug)]
pub struct FormatArgs {
    /// Path to format
    pub path: PathBuf,
}

pub fn format_execute_with_args(args: FormatArgs, global_options: GlobalOptions) -> Result<()> {
    let (_, socket_dir) =
        load_config_and_socket(&global_options.config_file, &global_options.socket_dir)?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    ensure_daemon_running(&socket, &global_options)?;

    daemon_run_command(
        DaemonCommands::Format(DaemonFormatArgs { path: args.path }),
        global_options,
        &socket,
        false,
    )?;

    Ok(())
}
