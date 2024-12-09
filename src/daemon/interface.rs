use crate::cli::GlobalOptions;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Parser, Serialize, Deserialize, Debug)]
pub struct DaemonFormatArgs {
    /// Path to format
    pub path: PathBuf,
}

#[derive(Parser, Serialize, Deserialize, Debug)]
pub struct DaemonPureFormatArgs {
    /// Path to format
    pub path: PathBuf,
    pub content: String,
}

#[derive(Parser, Serialize, Deserialize, Debug)]
pub enum DaemonCommands {
    Format(DaemonFormatArgs),
    PureFormat(DaemonPureFormatArgs),
    Stop,
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
    Success(),
    Ignored(String), // Ignored with reason
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonPureFormatResponse {
    Success(String),
    Ignored(String), // Ignored with reason
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonResponse {
    Format(DaemonFormatResponse),
    PureFormat(DaemonPureFormatResponse),
    Stop,
    Pong(DaemonInfo),
}

pub struct DaemonSocketPath {
    pub socket_path: PathBuf,
    pub info_path: PathBuf,
}

impl DaemonSocketPath {
    pub fn from_socket_dir(socket_dir: &PathBuf) -> Self {
        Self {
            socket_path: socket_dir.join("daemon-cmd.sock"),
            info_path: socket_dir.join("daemon-cmd.sock.info"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OutputPath {
    Path(PathBuf),
    Attached,
}

impl fmt::Display for OutputPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutputPath::Path(path) => write!(f, "{}", path.display()),
            OutputPath::Attached => write!(f, "<attached>"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DaemonInfo {
    pub pid: u32,
    pub start_time: u64,
    pub stdout_path: OutputPath,
    pub stderr_path: OutputPath,
}
