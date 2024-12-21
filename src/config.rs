use crate::app_dir::{cache_dir_res, config_file, socket_dir_res};
use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PureCommand {
    PluginUrl(url_serde::SerdeUrl),
    CommandIO { io: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum WriteCommand {
    Pure(PureCommand),
    SimpleCommand(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CommandWithControlFlow<T> {
    If {
        run: Box<CommandWithControlFlow<T>>,
        cond: String,
        on_true: Box<CommandWithControlFlow<T>>,
        on_false: Box<CommandWithControlFlow<T>>,
    },
    Sequential(Vec<CommandWithControlFlow<T>>),
    Set {
        set: HashMap<String, String>,
    },
    Command(T),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SomeCommand {
    Pure {
        cmd: CommandWithControlFlow<PureCommand>,
    },
    Write {
        write_cmd: CommandWithControlFlow<WriteCommand>,
    },
}

impl SomeCommand {
    pub fn is_pure(&self) -> bool {
        match self {
            SomeCommand::Pure { .. } => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rule {
    pub on: OnRule,
    #[serde(flatten)]
    pub some_cmd: SomeCommand,
}

impl Rule {
    pub fn on_match(&self, target_path: &PathBuf, force_pure: bool) -> bool {
        if force_pure && !self.some_cmd.is_pure() {
            return false;
        }

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

fn none<T>() -> Option<T> {
    None
}

impl Config {
    pub fn find_matched_rule(&self, target_path: &PathBuf, force_pure: bool) -> Option<Rule> {
        for rule in &self.rules {
            if rule.on_match(target_path, force_pure) {
                return Some(rule.clone());
            }
        }

        None
    }
}

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

        let default_config = r#"{
    "rules": [
        {
            "on": [".ts", ".tsx", ".json"],
            "cmd": "https://github.com/nahco314/foro-biome/releases/latest/download/foro-biome.dllpack"
        },
        {
            "on": ".rs",
            "write_cmd": "rustfmt +nightly --unstable-features --skip-children {{ target }}"
        },
        {
            "on": ".py",
            "cmd": "https://github.com/nahco314/foro-ruff/releases/latest/download/foro-ruff.dllpack"
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
        .with_context(|| format!("Failed to load config file ({:?})", &config_file))?;

    let cache_dir = given_cache_dir
        .clone()
        .or(config.cache_dir.clone())
        .or_else(|| cache_dir_res().ok())
        .context("Failed to get cache directory")?;

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
        .with_context(|| format!("Failed to load config file ({:?})", &config_file))?;

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
