use crate::cli::GlobalOptions;
use crate::config::load_config_and_cache;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use dialoguer::Confirm;
use log::{debug, error};
use std::fs;

#[derive(Parser, Debug)]
pub struct CacheCleanArgs {
    #[arg(short, long)]
    pub yes: bool,
}

pub fn cache_clean_execute_with_args(
    args: CacheCleanArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, cache_dir) =
        load_config_and_cache(&global_options.config_file, &global_options.cache_dir)?;

    if !cache_dir.exists() {
        debug!("cache directory does not exist, so we do nothing");
        return Ok(());
    }

    if (!(cache_dir.file_name() == Some("foro".as_ref()))) && (!args.yes) {
        debug!("cache directory seems not to be foro cache directory, so we ask the user");

        let confirm = Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete the directory {:?}?",
                cache_dir
            ))
            .default(false)
            .show_default(true);

        let answer = confirm.interact()?;

        debug!("answer: {:?}", answer);

        if !answer {
            error!("User aborted the operation");
            return Ok(());
        }
    }

    fs::remove_dir_all(&cache_dir).context(format!(
        "Failed to remove cache directory ({:?})",
        &cache_dir
    ))?;

    Ok(())
}

#[derive(Parser, Debug)]
pub struct CacheDirArgs {}

pub fn cache_dir_execute_with_args(
    _args: CacheDirArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, cache_dir) =
        load_config_and_cache(&global_options.config_file, &global_options.cache_dir)?;

    println!("Cache Directory: {:?}", cache_dir);

    Ok(())
}
#[derive(Parser, Debug)]
pub enum CacheSubCommands {
    Clean(CacheCleanArgs),
    Dir(CacheDirArgs),
}

#[derive(Parser, Debug)]
pub struct CacheArgs {
    #[clap(subcommand)]
    pub subcommand: CacheSubCommands,
}

pub fn cache_execute_with_args(args: CacheArgs, global_options: GlobalOptions) -> Result<()> {
    match args.subcommand {
        CacheSubCommands::Clean(s_args) => cache_clean_execute_with_args(s_args, global_options),
        CacheSubCommands::Dir(s_args) => cache_dir_execute_with_args(s_args, global_options),
    }
}
