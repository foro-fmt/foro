use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod cache;
pub mod format;
use format::*;

use crate::cli::cache::{cache_execute_with_args, CacheArgs};
use log::{debug, info, logger, warn};

#[derive(Parser, Debug)]
pub enum SubCommands {
    Cache(CacheArgs),
    Format(FormatArgs),
}

#[derive(Parser, Debug)]
pub struct GlobalOptions {
    /// The path to an onefmt.json file to use for configuration
    #[arg(long, value_name = "PATH")]
    pub config_file: Option<PathBuf>,

    /// The path to directory to use for caching
    #[arg(long, value_name = "PATH")]
    pub cache_dir: Option<PathBuf>,

    /// Avoid reading from or writing to the cache
    #[arg(long, default_value = "false")]
    pub no_cache: bool,
}

#[derive(Parser, Debug)]
pub struct Command {
    #[clap(subcommand)]
    pub subcommand: SubCommands,

    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    #[command(flatten)]
    pub global_options: GlobalOptions,
}

pub fn execute_with_args(args: Command) -> Result<()> {
    let start_time = Instant::now();

    env_logger::Builder::new()
        .filter_module("onefmt", args.verbose.log_level_filter())
        .format(move |buf, record| {
            // 経過時間を取得
            let elapsed = start_time.elapsed();
            let elapsed_micros = elapsed.as_micros();

            let level = record.level();
            let level_style = buf.default_level_style(level);

            let path = record.module_path().unwrap_or("");

            write!(buf, "[{elapsed_micros:>5} μs ")?;
            write!(buf, "{level_style}{level:<5}{level_style:#} ")?;
            write!(buf, "{path}] ")?;
            write!(buf, "{body}\n", body = record.args())
        })
        .init();

    debug!("start onefmt: {:?}", &args);

    let global_options = args.global_options;

    match args.subcommand {
        SubCommands::Cache(args) => cache_execute_with_args(args, global_options),
        SubCommands::Format(args) => format_execute_with_args(args, global_options),
    }?;

    debug!("end onefmt");

    Ok(())
}

pub fn execute() -> Result<()> {
    let args = Command::parse();

    execute_with_args(args)
}
