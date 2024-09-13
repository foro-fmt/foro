use crate::cli::GlobalOptions;
use crate::config::{load_config_and_cache, load_config_and_socket};
use crate::daemon::client::{daemon_is_alive, run_command as daemon_run_command};
use crate::daemon::interface::{DaemonCommands, DaemonFormatArgs, DaemonSocketPath};
use crate::daemon::server::start_daemon;
use crate::handle_plugin::run::run;
use anyhow::{Context, Result};
use clap::Parser;
use log::debug;
use serde_json::json;
use std::env::current_dir;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Parser, Debug)]
pub struct FormatArgs {
    /// Path to format
    pub path: PathBuf,
    pub no_daemon: bool,
}

pub fn format_execute_with_args(args: FormatArgs, global_options: GlobalOptions) -> Result<()> {
    if !args.no_daemon {
        let (_, socket_dir) =
            load_config_and_socket(&global_options.config_file, &global_options.socket_dir)?;

        let socket = DaemonSocketPath::from_socket_dir(&socket_dir);

        if !daemon_is_alive(&socket)? {
            start_daemon(&socket, false)?;
        }

        daemon_run_command(
            DaemonCommands::Format(DaemonFormatArgs { path: args.path }),
            global_options,
            &socket,
            false,
        )?;
    } else {
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

        debug!("run rule: {:?}", rule);

        let res = run(
            &rule.some_cmd,
            json!({
                "current-dir": current_dir()?.canonicalize()?.to_str().unwrap(),
                "target": target_path,
                "raw-target": args.path,
                "target-content": contents,
                }
            ),
            &cache_dir,
            !global_options.no_cache,
        )?;

        println!("{:?}", res);
    }

    Ok(())
}
