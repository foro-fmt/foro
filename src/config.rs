use crate::app_dir::{cache_dir, cache_dir_res, config_file, socket_dir_res};
use crate::cli::GlobalOptions;
use anyhow::{anyhow, Context, Result};
use clap::builder::Str;
use log::{debug, info, trace};
use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};
use url::Url;
use url_serde;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OnRule {
    Extension(String),
    Or(Vec<OnRule>),
}

impl OnRule {
    pub fn on_match(&self, target_path: &PathBuf) -> bool {
        match self {
            OnRule::Extension(ext) => target_path
                .extension()
                .map_or(false, |e| &format!(".{}", e.to_string_lossy()) == ext),
            OnRule::Or(rules) => rules.iter().any(|rule| rule.on_match(target_path)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SomePath {
    SinglePath(PathBuf),
    Or(Vec<PathBuf>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Command {
    PluginUrl(url_serde::SerdeUrl),
    SimpleCommand(String),
    CommandIO {
        io: String,
    },
    Finding {
        finding: Box<Command>,
        if_found: Box<Command>,
        #[serde(rename = "else")]
        else_: Box<Command>,
    },
    Sequential(Vec<Command>),
    NativeDll {
        #[serde(rename = "__deprecation_native_dll")]
        native_dll: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rule {
    pub on: OnRule,
    pub cmd: Command,
}

impl Rule {
    pub fn on_match(&self, target_path: &PathBuf) -> bool {
        self.on.on_match(target_path)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
    #[serde(default = "none")]
    pub cache_dir: Option<PathBuf>,
    #[serde(default = "none")]
    pub socket_dir: Option<PathBuf>,
}

fn true_() -> bool {
    true
}

fn none<T>() -> Option<T> {
    None
}

impl Config {
    pub fn find_matched_rule(&self, target_path: &PathBuf) -> Option<Rule> {
        for rule in &self.rules {
            if rule.on_match(target_path) {
                return Some(rule.clone());
            }
        }

        None
    }
}

pub fn load_str(json: &str) -> Result<Config> {
    serde_json::from_str(json).map_err(|e| anyhow!(e))
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
            "on": [".ts", ".tsx", ".json"],
            "cmd": "https://github.com/nahco314/foro-biome/releases/latest/download/foro_biome.wasm"
        },
        {
            "on": ".rs",
            "cmd": "rustfmt +nightly --unstable-features --skip-children {{ target }}"
        },
        {
            "on": ".py",
            "cmd": "https://github.com/nahco314/foro-ruff/releases/latest/download/foro_ruff.wasm"
        }
    ]
}
"#;

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
        .context(format!("Failed to load config file ({:?})", &config_file).to_string())?;

    let cache_dir = given_cache_dir
        .clone()
        .or(config.cache_dir.clone())
        .unwrap_or(cache_dir_res()?);

    debug!("config file: {:?}", &config_file);
    debug!("config: {:?}", &config);
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
        .context(format!("Failed to load config file ({:?})", &config_file).to_string())?;

    let socket_dir = given_socket_dir
        .clone()
        .or(config.socket_dir.clone())
        .or_else(|| socket_dir_res().ok())
        .context("Failed to get socket directory")?;

    debug!("config file: {:?}", &config_file);
    debug!("config: {:?}", &config);
    debug!("socket dir: {:?}", &socket_dir);

    Ok((config, socket_dir))
}
