use crate::app_dir::{cache_dir, cache_dir_res};
use crate::config::Command;
use std::{fs, io};
use url::Url;
use urlencoding;

use anyhow::{Context, Result};
use log::{debug, info};
use reqwest;

pub fn handle_wasm_url(url: Url) -> Result<()> {
    let mut cache = cache_dir_res()?;
    cache.push("cache-url");
    fs::DirBuilder::new().recursive(true).create(&cache)?;

    let encoded = urlencoding::encode(url.as_str());
    cache.push(&encoded.to_string());

    debug!("handling {}, cache path: {}", url, encoded);

    if cache.exists() {
        debug!("cache exists");
    } else {
        debug!("cache does not exist, downloading");

        let response = reqwest::blocking::get(url.to_string())?;

        let mut file = io::BufWriter::new(fs::File::create(cache)?);

        io::copy(&mut response.bytes()?.as_ref(), &mut file)?;
    }

    Ok(())
}

pub fn handle_command(command: Command) {
    match command {
        Command::PluginUrl(url) => {
            handle_wasm_url(url.into_inner());
        }
        Command::SimpleCommand(cmd) => {}
        Command::Finding { .. } => {}
    }
}
