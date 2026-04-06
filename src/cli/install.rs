use crate::cli::GlobalOptions;
use crate::config::{load_config_and_cache, read_config_bytes};
use crate::install_check::mark_ready;
use anyhow::Result;
use clap::Parser;
use dll_pack::resolve::download;
use dll_pack::resolve::ResolveError;
use dll_pack::THIS_PLATFORM;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};
use url::Url;

#[derive(Parser, Debug)]
pub struct InstallArgs {}

const INSTALL_LOCK_STALE_TIMEOUT: Duration = Duration::from_secs(600);

struct InstallLock {
    path: PathBuf,
}

impl InstallLock {
    fn acquire(cache_dir: &Path) -> Result<Self> {
        fs::create_dir_all(cache_dir)?;

        let path = cache_dir.join(".install.lock");
        let mut taken_lock_started: Option<SystemTime> = None;

        loop {
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    match taken_lock_started {
                        None => taken_lock_started = Some(path.metadata()?.modified()?),
                        Some(t) if t.elapsed()? > INSTALL_LOCK_STALE_TIMEOUT => {
                            let _ = fs::remove_dir_all(&path);
                            continue;
                        }
                        _ => {}
                    }

                    thread::sleep(Duration::from_micros(100));
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

impl Drop for InstallLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn install_execute_with_args(_args: InstallArgs, global_options: GlobalOptions) -> Result<()> {
    let config_bytes = read_config_bytes(global_options.config_file.as_deref())?;
    let (config, cache_dir) = load_config_and_cache(
        global_options.config_file.as_deref(),
        global_options.cache_dir.as_deref(),
    )?;
    let _install_lock = InstallLock::acquire(&cache_dir)?;

    let urls: HashSet<Url> = config.all_plugin_urls().into_iter().collect();

    for url in &urls {
        download_with_wasm_fallback(url, &cache_dir)?;
    }

    mark_ready(&config_bytes, &cache_dir)?;
    Ok(())
}

fn download_with_wasm_fallback(url: &Url, cache_dir: &std::path::PathBuf) -> Result<()> {
    match download(url, cache_dir, THIS_PLATFORM) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.downcast_ref::<ResolveError>().is_some() {
                download(url, cache_dir, "wasm32-wasip1")
            } else {
                Err(e)
            }
        }
    }
}
