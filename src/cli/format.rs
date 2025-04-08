use crate::cli::GlobalOptions;
use crate::config::{load_config_and_cache, load_config_and_socket};
use crate::daemon::client::{daemon_is_alive, run_command as daemon_run_command};
use crate::daemon::interface::{DaemonCommands, DaemonFormatArgs, DaemonSocketPath};
use crate::daemon::server::start_daemon;
use crate::debug_long;
use crate::handle_plugin::run::run;
use crate::path_utils::{normalize_path, to_wasm_path};
use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use serde_json::json;
use std::env::current_dir;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Parser, Debug)]
pub struct FormatArgs {
    /// Path to format
    pub path: PathBuf,
    #[clap(long)]
    pub no_daemon: bool,
}

fn format_with_no_daemon(args: FormatArgs, global_options: GlobalOptions) -> Result<()> {
    let (config, cache_dir) =
        load_config_and_cache(&global_options.config_file, &global_options.cache_dir)?;

    let target_path = args.path.canonicalize()?;

    let file = fs::File::open(&args.path)?;
    let mut buf_reader = io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let rule = config
        .find_matched_rule(&target_path, false)
        .context("No rule matched")?;

    debug_long!("run rule: {:?}", rule);

    let res = run(
        &rule.some_cmd,
        json!({
            "wasm-current-dir":  to_wasm_path(&current_dir()?)?,
            "os-current-dir": normalize_path(&current_dir()?)?,
            "wasm-target": to_wasm_path(&target_path)?,
            "os-target":  normalize_path(&target_path)?,
            "raw-target": args.path,
            "target-content": contents,
        }),
        &cache_dir,
        !global_options.no_cache,
    )?;

    debug_long!("{:?}", res);
    info!("Success to format");

    Ok(())
}

pub fn format_execute_with_args(args: FormatArgs, global_options: GlobalOptions) -> Result<()> {
    if args.no_daemon {
        return format_with_no_daemon(args, global_options);
    }

    let (_, socket_dir) =
        load_config_and_socket(&global_options.config_file, &global_options.socket_dir)?;

    let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

    if !daemon_is_alive(&socket)?.0 {
        start_daemon(&socket, false)?;
    }

    daemon_run_command(
        DaemonCommands::Format(DaemonFormatArgs { path: args.path }),
        global_options,
        &socket,
        false,
    )?;

    Ok(())
}
