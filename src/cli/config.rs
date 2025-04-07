use crate::cli::GlobalOptions;
use crate::config::get_or_create_default_config;
use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
pub enum ConfigSubCommands {
    Path(ConfigPathArgs),
    Cat(ConfigCatArgs),
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    pub subcommand: ConfigSubCommands,
}

#[derive(Parser, Debug)]
pub struct ConfigPathArgs {}

#[derive(Parser, Debug)]
pub struct ConfigCatArgs {}

pub fn config_execute_with_args(args: ConfigArgs, global_options: GlobalOptions) -> Result<()> {
    match args.subcommand {
        ConfigSubCommands::Path(s_args) => config_path_execute_with_args(s_args, global_options),
        ConfigSubCommands::Cat(s_args) => config_cat_execute_with_args(s_args, global_options),
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

pub fn config_cat_execute_with_args(
    _args: ConfigCatArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let config_file = global_options
        .config_file
        .or_else(get_or_create_default_config)
        .context("Failed to get config file path")?;

    let content = std::fs::read_to_string(&config_file)
        .with_context(|| format!("Failed to read config file: {:?}", config_file))?;

    println!("{}", content);

    Ok(())
}
