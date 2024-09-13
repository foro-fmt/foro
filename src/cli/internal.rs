use crate::cli::GlobalOptions;
use crate::config::{load_config_and_cache, load_config_and_socket, load_paths};
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use dialoguer::Confirm;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Parser, Debug)]
pub struct InternalInfoArgs {}

#[derive(Deserialize, Debug)]
struct InfoInput {
    pub given_config_file: Option<String>,
    pub given_cache_dir: Option<String>,
    pub given_socket_dir: Option<String>,
}

#[derive(Serialize, Debug)]
struct InfoOutput {
    pub config_file: String,
    pub cache_dir: String,
    pub socket_dir: String,
}

pub fn internal_info_execute_with_args(
    _args: InternalInfoArgs,
    _global_options: GlobalOptions,
) -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let info_input: InfoInput = serde_json::from_str(&input).context("Failed to parse input")?;

    let given_config_file = info_input.given_config_file.map(PathBuf::from);
    let given_cache_dir = info_input.given_cache_dir.map(PathBuf::from);
    let given_socket_dir = info_input.given_socket_dir.map(PathBuf::from);

    let (config_file, cache_dir, socket_dir) =
        load_paths(&given_config_file, &given_cache_dir, &given_socket_dir)?;

    let info_output = InfoOutput {
        config_file: config_file.to_string_lossy().to_string(),
        cache_dir: cache_dir.to_string_lossy().to_string(),
        socket_dir: socket_dir.to_string_lossy().to_string(),
    };

    let output = serde_json::to_string(&info_output).context("Failed to serialize output")?;
    println!("{}", output);

    Ok(())
}

#[derive(Parser, Debug)]
pub enum InternalSubCommands {
    Info(InternalInfoArgs),
}

#[derive(Parser, Debug)]
pub struct InternalArgs {
    #[clap(subcommand)]
    pub subcommand: InternalSubCommands,
}

pub fn internal_execute_with_args(args: InternalArgs, global_options: GlobalOptions) -> Result<()> {
    match args.subcommand {
        InternalSubCommands::Info(s_args) => {
            internal_info_execute_with_args(s_args, global_options)
        }
    }
}
