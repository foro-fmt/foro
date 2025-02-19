use crate::cli::GlobalOptions;
use crate::config::get_or_create_default_config;
use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
pub enum ConfigSubCommands {
    Path(ConfigPathArgs),
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    pub subcommand: ConfigSubCommands,
}

#[derive(Parser, Debug)]
pub struct ConfigPathArgs {}

pub fn config_execute_with_args(args: ConfigArgs, global_options: GlobalOptions) -> Result<()> {
    match args.subcommand {
        ConfigSubCommands::Path(s_args) => config_path_execute_with_args(s_args, global_options),
    }
}

pub fn config_path_execute_with_args(
    _args: ConfigPathArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let config_file = global_options
        .config_file
        .or_else(get_or_create_default_config)
        .context("Failed to get config file path")?;

    println!("Config File: {:?}", config_file);

    Ok(())
}
