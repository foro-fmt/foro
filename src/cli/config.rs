use crate::cli::GlobalOptions;
use crate::config::get_or_create_default_config;
use anyhow::{anyhow, Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
pub enum ConfigSubCommands {
    Path(ConfigPathArgs),
    Show(ConfigShowArgs),
    Default(ConfigDefaultArgs),
    Edit(ConfigEditArgs),
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    pub subcommand: ConfigSubCommands,
}

#[derive(Parser, Debug)]
pub struct ConfigPathArgs {}

#[derive(Parser, Debug)]
pub struct ConfigShowArgs {}

#[derive(Parser, Debug)]
pub struct ConfigDefaultArgs {}

#[derive(Parser, Debug)]
pub struct ConfigEditArgs {
    /// Editor to use for editing the config file
    pub editor: Option<String>,
}

pub fn config_execute_with_args(args: ConfigArgs, global_options: GlobalOptions) -> Result<()> {
    match args.subcommand {
        ConfigSubCommands::Path(s_args) => config_path_execute_with_args(s_args, global_options),
        ConfigSubCommands::Show(s_args) => config_show_execute_with_args(s_args, global_options),
        ConfigSubCommands::Default(s_args) => {
            config_default_execute_with_args(s_args, global_options)
        }
        ConfigSubCommands::Edit(s_args) => config_edit_execute_with_args(s_args, global_options),
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

    println!("Config File: {config_file:?}");

    Ok(())
}

pub fn config_show_execute_with_args(
    _args: ConfigShowArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let config_file = global_options
        .config_file
        .or_else(get_or_create_default_config)
        .context("Failed to get config file path")?;

    let content = std::fs::read_to_string(&config_file)
        .with_context(|| format!("Failed to read config file: {config_file:?}"))?;

    println!("{content}");

    Ok(())
}

pub fn config_default_execute_with_args(
    _args: ConfigDefaultArgs,
    _global_options: GlobalOptions,
) -> Result<()> {
    let default_config = include_str!("../config/default_config.json");
    println!("{default_config}");
    Ok(())
}

pub fn config_edit_execute_with_args(
    args: ConfigEditArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let config_file = global_options
        .config_file
        .or_else(get_or_create_default_config)
        .context("Failed to get config file path")?;

    let editor = args.editor.unwrap_or_else(|| "vi".to_string());

    let status = std::process::Command::new(&editor)
        .arg(&config_file)
        .status()
        .with_context(|| format!("Failed to execute editor: {editor}"))?;

    if !status.success() {
        return Err(anyhow!("Editor exited with non-zero status: {}", status));
    }

    Ok(())
}
