use crate::cli::GlobalOptions;
use crate::config::load_config_and_cache;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use dialoguer::Confirm;
use dll_pack::resolve::get_all_cached_dependencies;
use log::{debug, error, info};
use std::fs;
use std::str::FromStr;
use url::Url;

#[derive(Parser, Debug)]
pub struct CacheCleanArgs {
    #[arg(short, long)]
    pub yes: bool,
}

pub fn cache_clean_execute_with_args(
    args: CacheCleanArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, cache_dir) = load_config_and_cache(
        global_options.config_file.as_deref(),
        global_options.cache_dir.as_deref(),
    )?;

    if !cache_dir.exists() {
        debug!("cache directory does not exist, so we do nothing");
        return Ok(());
    }

    if (cache_dir.file_name() != Some("foro".as_ref())) && (!args.yes) {
        debug!("cache directory seems not to be foro cache directory, so we ask the user");

        let confirm = Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete the directory {cache_dir:?}?"
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
pub struct CacheRemoveArgs {
    pub url: String,
}

pub fn cache_remove_execute_with_args(
    args: CacheRemoveArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, cache_dir) = load_config_and_cache(
        global_options.config_file.as_deref(),
        global_options.cache_dir.as_deref(),
    )?;

    // In Windows, it takes a little while for the process to stop,
    // and if you try to connect during that time, we will get an error,
    // so sleep a little

    #[cfg(windows)]
    {
        // todo: Even with this, errors still occur occasionally,
        //   so we must use IPC to ensure that wait.
        sleep(Duration::from_millis(10));
    }

    if !cache_dir.exists() {
        debug!("cache directory does not exist, so we do nothing");
        return Ok(());
    }

    let url = Url::from_str(&args.url).context("Failed to parse URL")?;

    let cached_locations = get_all_cached_dependencies(&url, &cache_dir)?;

    match cached_locations {
        None => {
            info!("No cache found for the URL: {:?}", url);
        }
        Some(deps) => {
            for (dep_url, loc) in deps {
                info!("Removing cache for the URL: {:?}", dep_url);

                if loc.is_dir() {
                    fs::remove_dir_all(&loc)
                        .context(format!("Failed to remove cache directory ({:?})", &loc))?;
                    debug!("Removed cache directory: {:?}", loc);
                } else {
                    fs::remove_file(&loc)
                        .context(format!("Failed to remove cache directory ({:?})", &loc))?;
                    debug!("Removed cache file: {:?}", loc);
                }
            }
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
pub struct CacheDirArgs {}

pub fn cache_dir_execute_with_args(
    _args: CacheDirArgs,
    global_options: GlobalOptions,
) -> Result<()> {
    let (_, cache_dir) = load_config_and_cache(
        global_options.config_file.as_deref(),
        global_options.cache_dir.as_deref(),
    )?;

    println!("Cache Directory: {cache_dir:?}");

    Ok(())
}
#[derive(Parser, Debug)]
pub enum CacheSubCommands {
    Clean(CacheCleanArgs),
    Remove(CacheRemoveArgs),
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
        CacheSubCommands::Remove(s_args) => cache_remove_execute_with_args(s_args, global_options),
        CacheSubCommands::Dir(s_args) => cache_dir_execute_with_args(s_args, global_options),
    }
}
