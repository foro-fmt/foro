use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::InfoLevel;
use std::cell::OnceCell;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

mod bulk_format;
mod cache;
mod config;
mod daemon;
pub mod format;
mod internal;

use format::*;

use crate::cli::bulk_format::{bulk_format_execute_with_args, BulkFormatArgs};
use crate::cli::cache::{cache_execute_with_args, CacheArgs};
use crate::cli::config::{config_execute_with_args, ConfigArgs};
use crate::cli::daemon::{daemon_execute_with_args, DaemonArgs};
use crate::cli::internal::{internal_execute_with_args, InternalArgs};
use crate::log::{init_env_logger, DAEMON_THREAD_START, IS_DAEMON_MAIN_THREAD, IS_DAEMON_PROCESS};
use log::trace;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
pub enum SubCommands {
    Cache(CacheArgs),
    Config(ConfigArgs),
    Daemon(DaemonArgs),
    Format(FormatArgs),
    BulkFormat(BulkFormatArgs),
    #[clap(hide = true)]
    Internal(InternalArgs),
}

#[derive(Parser, Serialize, Deserialize, Debug, Clone)]
pub struct GlobalOptions {
    /// The path to an foro.json file to use for configuration
    #[arg(long, value_name = "PATH")]
    pub config_file: Option<PathBuf>,

    /// The path to directory to use for caching
    #[arg(long, value_name = "PATH")]
    pub cache_dir: Option<PathBuf>,

    /// The path to directory to use for the daemon socket
    #[arg(long, value_name = "PATH")]
    pub socket_dir: Option<PathBuf>,

    /// Avoid reading from or writing to the cache
    #[arg(long, default_value = "false")]
    pub no_cache: bool,

    /// Avoid logging log content
    #[arg(long, default_value = "false")]
    pub no_long_log: bool,
    
    #[arg(long, default_value = "false")]
    pub ignore_build_id_mismatch: bool,
}

#[derive(Parser, Debug)]
pub struct Command {
    #[clap(subcommand)]
    pub subcommand: SubCommands,

    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity<InfoLevel>,

    #[command(flatten)]
    pub global_options: GlobalOptions,
}

pub fn execute_with_args(args: Command) -> Result<()> {
    init_env_logger(
        args.verbose.log_level_filter(),
        args.global_options.no_long_log,
    );

    trace!("start foro: {:?}", &args);

    let global_options = args.global_options;

    match args.subcommand {
        SubCommands::Cache(s_args) => cache_execute_with_args(s_args, global_options),
        SubCommands::Config(s_args) => config_execute_with_args(s_args, global_options),
        SubCommands::Daemon(s_args) => daemon_execute_with_args(s_args, global_options),
        SubCommands::Format(s_args) => format_execute_with_args(s_args, global_options),
        SubCommands::BulkFormat(s_args) => bulk_format_execute_with_args(s_args, global_options),
        SubCommands::Internal(s_args) => internal_execute_with_args(s_args, global_options),
    }?;

    trace!("end foro");

    Ok(())
}

pub fn execute() -> Result<()> {
    let args = Command::parse();

    execute_with_args(args)
}
