use crate::app_dir::cache_dir_res;
use crate::cli::GlobalOptions;
use crate::config;
use crate::config::{get_or_create_default_config, load_config_for_cli, load_file, Command};
use crate::handle_plugin::run::run;
use anyhow::{Context, Error, Result};
use clap::builder::{IntoResettable, Resettable, ValueHint};
use clap::Parser;
use log::{debug, info};
use serde_json::json;
use std::env::current_dir;
use std::fmt::Display;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, io};
use url::Url;
use url_serde::{Serde, SerdeUrl};

#[derive(Parser, Debug)]
pub struct FormatArgs {
    /// Path to format
    pub path: PathBuf,
}

pub fn format_execute_with_args(args: FormatArgs, global_options: GlobalOptions) -> Result<()> {
    let (config, cache_dir) =
        load_config_for_cli(&global_options.config_file, &global_options.cache_dir)?;

    let file = fs::File::open(&args.path)?;
    let mut buf_reader = io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let res = run(
        &config.rules.first().unwrap().cmd,
        json!({
            "current-dir": current_dir()?.canonicalize()?.to_str().unwrap(),
            "target": args.path.canonicalize()?.to_str().unwrap(),
            "raw-target": args.path,
            "target-content": contents,
            }
        ),
        &cache_dir,
        !global_options.no_cache,
    )?;

    println!("{:?}", res);

    Ok(())
}
