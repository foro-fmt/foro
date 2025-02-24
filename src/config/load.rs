
use crate::app_dir::{cache_dir_res, config_file, socket_dir_res};
use crate::debug_long;
use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use crate::config::model::Config;

#[allow(unused)]
pub fn load_str(json: &str) -> Result<Config> {
    serde_json::from_str(json).map_err(|e| anyhow!(e))
}

pub fn load_file(path: &PathBuf) -> Result<Config> {
    // memo: in my measurement, this implementation is faster than serde_json::from_reader, etc
    let mut file = fs::File::open(path).context("Failed to open file")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    serde_json::from_slice(&buffer).map_err(|e| anyhow!(e))
}

pub(crate) fn get_or_create_default_config() -> Option<PathBuf> {
    let config_path = config_file()?;

    if !config_path.exists() {
        debug!("try create default config file: {:?}", config_path);

        fs::DirBuilder::new()
            .recursive(true)
            .create(&config_path.parent()?)
            .ok()?;

        let default_config = include_str!("default_config.json");

        fs::write(&config_path, &default_config).ok()?;

        info!("created default config file: {:?}", config_path);
        info!("content: {:?}", default_config);
    }

    Some(config_path)
}

pub(crate) fn load_config_and_cache(
    given_config_file: &Option<PathBuf>,
    given_cache_dir: &Option<PathBuf>,
) -> Result<(Config, PathBuf)> {
    let config_file = given_config_file
        .clone()
        .or_else(get_or_create_default_config)
        .context("Could not get config directory")?;

    let config = load_file(&config_file)
        .with_context(|| format!("Failed to load config file ({:?})", &config_file))?;

    let cache_dir = given_cache_dir
        .clone()
        .or(config.cache_dir.clone())
        .or_else(|| cache_dir_res().ok())
        .context("Failed to get cache directory")?;

    debug!("config file: {:?}", &config_file);
    debug_long!("config: {:?}", &config);
    debug!("cache dir: {:?}", &cache_dir);

    Ok((config, cache_dir))
}

pub(crate) fn load_config_and_socket(
    given_config_file: &Option<PathBuf>,
    given_socket_dir: &Option<PathBuf>,
) -> Result<(Config, PathBuf)> {
    let config_file = given_config_file
        .clone()
        .or_else(get_or_create_default_config)
        .context("Failed to get config directory")?;

    let config = load_file(&config_file)
        .with_context(|| format!("Failed to load config file ({:?})", &config_file))?;

    let socket_dir = given_socket_dir
        .clone()
        .or(config.socket_dir.clone())
        .or_else(|| socket_dir_res().ok())
        .context("Failed to get socket directory")?;

    debug!("config file: {:?}", &config_file);
    debug_long!("config: {:?}", &config);
    debug!("socket dir: {:?}", &socket_dir);

    Ok((config, socket_dir))
}

pub(crate) fn load_paths(
    given_config_file: &Option<PathBuf>,
    given_cache_dir: &Option<PathBuf>,
    given_socket_dir: &Option<PathBuf>,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let config_file = given_config_file
        .clone()
        .or_else(get_or_create_default_config)
        .context("Failed to get config directory")?;

    let config = load_file(&config_file)
        .with_context(|| format!("Failed to load config file ({:?})", &config_file))?;

    let cache_dir = given_cache_dir
        .clone()
        .or(config.cache_dir.clone())
        .or_else(|| cache_dir_res().ok())
        .context("Failed to get cache directory")?;

    let socket_dir = given_socket_dir
        .clone()
        .or(config.socket_dir.clone())
        .or_else(|| socket_dir_res().ok())
        .context("Failed to get socket directory")?;

    Ok((config_file, cache_dir, socket_dir))
}
