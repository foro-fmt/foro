use crate::config::load_str;
use anyhow::{bail, Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use xxhash_rust::xxh3::xxh3_128;

pub fn config_hash(config_bytes: &[u8]) -> Result<String> {
    let config_text =
        std::str::from_utf8(config_bytes).context("Failed to read config as UTF-8")?;
    let config = load_str(config_text)?;

    let urls = config
        .all_plugin_urls()
        .into_iter()
        .map(|url| url.to_string())
        .collect::<BTreeSet<_>>();

    let normalized = urls.into_iter().collect::<Vec<_>>().join("\0");

    Ok(format!("{:032x}", xxh3_128(normalized.as_bytes())))
}

fn marker_path(cache_dir: &Path, hash: &str) -> PathBuf {
    cache_dir.join("ready").join(hash)
}

pub fn check_ready(config_bytes: &[u8], cache_dir: &Path) -> Result<()> {
    let hash = config_hash(config_bytes)?;
    let marker = marker_path(cache_dir, &hash);
    if !marker.exists() {
        bail!(
            "plugins not downloaded for current config.\n\
             Run `foro install` to download them."
        );
    }
    Ok(())
}

pub fn mark_ready(config_bytes: &[u8], cache_dir: &Path) -> Result<()> {
    let hash = config_hash(config_bytes)?;
    let ready_dir = cache_dir.join("ready");
    fs::create_dir_all(&ready_dir)?;
    fs::write(ready_dir.join(hash), "")?;
    Ok(())
}
