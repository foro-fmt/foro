use crate::cli::GlobalOptions;
use crate::config::{load_config_and_cache, read_config_bytes};
use crate::install_check::mark_ready;
use anyhow::Result;
use clap::Parser;
use dll_pack::resolve::ResolveError;
use dll_pack::{prefetch, THIS_PLATFORM};
use std::collections::HashSet;
use url::Url;

#[derive(Parser, Debug)]
pub struct InstallArgs {}

pub fn install_execute_with_args(_args: InstallArgs, global_options: GlobalOptions) -> Result<()> {
    let config_bytes = read_config_bytes(global_options.config_file.as_deref())?;
    let (config, cache_dir) = load_config_and_cache(
        global_options.config_file.as_deref(),
        global_options.cache_dir.as_deref(),
    )?;

    let urls: HashSet<Url> = config.all_plugin_urls().into_iter().collect();

    for url in &urls {
        prefetch_with_wasm_fallback(url, &cache_dir)?;
    }

    mark_ready(&config_bytes, &cache_dir)?;
    Ok(())
}

fn prefetch_with_wasm_fallback(url: &Url, cache_dir: &std::path::PathBuf) -> Result<()> {
    match prefetch(url, cache_dir, THIS_PLATFORM) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.downcast_ref::<ResolveError>().is_some() {
                prefetch(url, cache_dir, "wasm32-wasip1")
            } else {
                Err(e)
            }
        }
    }
}
