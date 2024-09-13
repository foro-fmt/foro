use crate::cli::format::FormatArgs;
use crate::cli::GlobalOptions;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Parser, Serialize, Deserialize, Debug)]
pub struct DaemonFormatArgs {
    /// Path to format
    pub path: PathBuf,
}

#[derive(Parser, Serialize, Deserialize, Debug)]
pub enum DaemonCommands {
    Format(DaemonFormatArgs),
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DaemonCommandPayload {
    pub command: DaemonCommands,
    pub current_dir: PathBuf,
    pub global_options: GlobalOptions,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonFormatResponse {
    Success,
    Ignored,
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonResponse {
    Format(DaemonFormatResponse),
    Pong,
}

pub struct DaemonSocketPath {
    pub socket_path: PathBuf,
}

impl DaemonSocketPath {
    pub fn from_socket_dir(socket_dir: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_dir.as_ref().join("daemon-cmd.sock"),
        }
    }
}
