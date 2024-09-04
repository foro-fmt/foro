use crate::app_dir::{cache_dir, cache_dir_res, config_file};
use crate::cli::GlobalOptions;
use anyhow::{anyhow, Context, Result};
use clap::builder::Str;
use log::{debug, info};
use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt::Write;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};
use url::Url;
use url_serde;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum OnRule {
    Extension(String),
    Or(Vec<OnRule>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SomePath {
    SinglePath(PathBuf),
    Or(Vec<PathBuf>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Command {
    PluginUrl(url_serde::SerdeUrl),
    SimpleCommand(String),
    Finding {
        finding: Box<Command>,
        if_found: Box<Command>,
        #[serde(rename = "else")]
        else_: Box<Command>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    pub on: OnRule,
    pub cmd: Command,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
    #[serde(default = "none")]
    pub cache_path: Option<PathBuf>,
}

fn true_() -> bool {
    true
}

fn none<T>() -> Option<T> {
    None
}

pub fn load_str(json: &str) -> Result<Config> {
    serde_json::from_str(json).context("Failed to parse JSON")
}

pub fn load_file(path: &PathBuf) -> Result<Config> {
    let mut file = fs::File::open(path).context("Failed to open file")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    serde_json::from_slice(&buffer).map_err(|e| anyhow!(e))
}

pub(crate) fn get_or_create_default_config() -> Option<PathBuf> {
    let mut config_path = config_file()?;

    if !config_path.exists() {
        debug!("try create default config file: {:?}", config_path);

        fs::DirBuilder::new()
            .recursive(true)
            .create(&config_path.parent()?)
            .ok()?;

        let default_config = r#"{
    "rules": [
        {
            "on": [".ts", "tsx"],
            "cmd": {
                "finding": "https://github.com/nahco314/onefmt-find-biome/releases/download/v0.0.1/onefmt_find_biome.wasm",
                "if_found": "{{ biome }} format --write {{ target }}",
                "else": "http://0.0.0.0:8000/target/wasm32-wasi/super-release/onefmt_biome_fallback.wasm"
            }
        }
    ]
}
"#;

        fs::write(&config_path, default_config).ok()?;

        info!("created default config file: {:?}", config_path);
    }

    Some(config_path)
}

pub(crate) fn load_config_for_cli(
    given_config_file: &Option<PathBuf>,
    given_cache_dir: &Option<PathBuf>,
) -> Result<(Config, PathBuf)> {
    let config_file = given_config_file
        .clone()
        .or_else(get_or_create_default_config)
        .context("Could not get config directory")?;

    let config = load_file(&config_file)
        .context(format!("Failed to load config file ({:?})", &config_file).to_string())?;

    let cache_dir = given_cache_dir
        .clone()
        .or(config.cache_path.clone())
        .unwrap_or(cache_dir_res()?);

    debug!("config file: {:?}", &config_file);
    debug!("config: {:?}", &config);
    debug!("cache dir: {:?}", &cache_dir);

    Ok((config, cache_dir))
}
