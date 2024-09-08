use anyhow::{Context, Result};
use std::cell::LazyCell;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};

pub(crate) fn config_file() -> Option<PathBuf> {
    let mut a = dirs::config_dir()?;
    a.push("onefmt.json");
    Some(a)
}

pub(crate) fn cache_dir() -> Option<PathBuf> {
    let mut a = dirs::cache_dir()?;
    a.push("onefmt");
    Some(a)
}

pub(crate) fn socket_dir() -> Option<PathBuf> {
    // TODO: runtime_dir is None in Mac/Windows so we need to handle it
    let mut a = dirs::runtime_dir()?;
    a.push("onefmt");
    Some(a)
}

pub(crate) fn config_file_res() -> Result<PathBuf> {
    config_file().context("Failed to get default config file")
}

pub(crate) fn cache_dir_res() -> Result<PathBuf> {
    cache_dir().context("Failed to get default cache dir")
}

pub(crate) fn socket_dir_res() -> Result<PathBuf> {
    socket_dir().context("Failed to get default socket dir")
}
