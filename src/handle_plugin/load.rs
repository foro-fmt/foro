use crate::app_dir::cache_dir_res;
use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use serde_json::{json, Value};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{fs, io, slice};
use tempfile::tempfile;
use url::Url;
use wasmtime::*;
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

fn _display_file_size(path: PathBuf) -> String {
    match fs::metadata(path) {
        Ok(metadata) => {
            let file_size = metadata.len();

            const KILOBYTE: u64 = 1024;
            const MEGABYTE: u64 = KILOBYTE * 1024;
            const GIGABYTE: u64 = MEGABYTE * 1024;
            const TERABYTE: u64 = GIGABYTE * 1024;

            if file_size >= TERABYTE {
                format!("{:.2} TB", file_size as f64 / TERABYTE as f64)
            } else if file_size >= GIGABYTE {
                format!("{:.2} GB", file_size as f64 / GIGABYTE as f64)
            } else if file_size >= MEGABYTE {
                format!("{:.2} MB", file_size as f64 / MEGABYTE as f64)
            } else if file_size >= KILOBYTE {
                format!("{:.2} KB", file_size as f64 / KILOBYTE as f64)
            } else {
                format!("{} bytes", file_size)
            }
        }
        Err(_) => "unknown size".to_string(),
    }
}

pub(crate) fn _load_module_base_with_cache(
    engine: &Engine,
    lazy_module_bin: impl Fn() -> Result<Vec<u8>>,
    mut cache_path: impl AsRef<Path>,
) -> Result<Module> {
    let mut cache_path = cache_path.as_ref().to_path_buf();

    if cache_path.exists() {
        debug!("loading from cache: {}", cache_path.display());
        debug!(
            "cached module size: {}",
            _display_file_size(cache_path.clone())
        );

        let module;
        unsafe {
            module = Module::deserialize_file(&engine, &cache_path)?;
        }

        Ok(module)
    } else {
        info!(
            "cache does not exist, download/create module: {}",
            cache_path.display()
        );

        let bin_module = lazy_module_bin()?;
        let module = Module::from_binary(&engine, bin_module.as_slice())?;
        let bin = module.serialize()?;

        fs::create_dir_all(cache_path.parent().context("Failed to get parent")?)?;
        fs::write(&cache_path, bin)?;

        Ok(module)
    }
}

pub(crate) fn _load_module_base(
    engine: &Engine,
    module_name_for_log: &str,
    lazy_module_bin: impl Fn() -> Result<Vec<u8>>,
    cache_path: impl AsRef<Path>,
    use_cache: bool,
) -> Result<Module> {
    if use_cache {
        _load_module_base_with_cache(engine, lazy_module_bin, cache_path)
    } else {
        info!("loading module {} (no cache!)", module_name_for_log);

        let bin_module = lazy_module_bin()?;
        let module = Module::from_binary(&engine, bin_module.as_slice())?;

        Ok(module)
    }
}

pub(crate) fn load_local_module(
    engine: &Engine,
    module_path: impl AsRef<Path> + Clone,
    cache_dir: &PathBuf,
    use_cache: bool,
) -> Result<Module> {
    let mut cache_path = cache_dir.clone();
    cache_path.push("cache-local");

    for component in fs::canonicalize(&module_path)?.components().skip(1) {
        cache_path.push(component);
    }

    let module_path_name = module_path
        .as_ref()
        .to_str()
        .context("Failed to get file name")?;

    _load_module_base(
        engine,
        module_path_name,
        || Ok(fs::read(module_path.clone())?),
        cache_path,
        use_cache,
    )
}

pub(crate) fn load_url_module(
    engine: &Engine,
    url: &Url,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<Module> {
    let mut cache_path = cache_path.clone();
    cache_path.push("cache-url");

    let encoded = urlencoding::encode(url.as_str());
    cache_path.push(&encoded.to_string());

    let url_as_str = url.as_str();

    let download = || {
        info!("fetching {}", url);

        let response = reqwest::blocking::get(url.to_string())?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to download: {}", response.status()));
        }

        Ok(response.bytes()?.to_vec())
    };

    _load_module_base(engine, url_as_str, download, cache_path, use_cache)
}
