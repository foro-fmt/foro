use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct DaemonFormatArgs {
    /// Path to format
    pub path: PathBuf,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DaemonBulkFormatArgs {
    /// Paths to format
    pub paths: Vec<PathBuf>,
    /// Number of threads to use
    pub threads: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonCommands {
    Format(DaemonFormatArgs),
    BulkFormat(DaemonBulkFormatArgs),
    Stop,
    Ping,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DaemonExecutionOptions {
    pub config_file: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub socket_dir: Option<PathBuf>,
    pub ignore_build_id_mismatch: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DaemonCommandPayload {
    pub command: DaemonCommands,
    pub current_dir: PathBuf,
    pub execution_options: DaemonExecutionOptions,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonFormatResponse {
    Success(),
    Ignored(String), // Ignored with reason
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonBulkFormatResponse {
    Success(BulkFormatSummary),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BulkFormatSummary {
    pub total_count: usize,
    pub changed_count: usize,
    pub unchanged_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonResponse {
    Format(DaemonFormatResponse),
    BulkFormat(DaemonBulkFormatResponse),
    Stop,
    Pong(DaemonInfo),
}

pub struct DaemonSocketPath {
    pub socket_dir: PathBuf,
    pub socket_path: PathBuf,
    pub info_path: PathBuf,
}

/// The path to the socket and info file for the daemon.
///
/// The socket file is used to communicate with the daemon.
/// The info file is used to store the daemon's pid, start time, and log file paths.
impl DaemonSocketPath {
    pub fn from_socket_dir(socket_dir: &Path) -> Self {
        Self {
            socket_dir: socket_dir.to_path_buf(),
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

/// Information about the daemon process.
///
/// It includes the daemon's pid, start time, and log file paths.
///
/// It's used to display the daemon's status on `foro daemon ping`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DaemonInfo {
    pub pid: u32,
    pub start_time: u64,
    pub stdout_path: OutputPath,
    pub stderr_path: OutputPath,
    pub build_id: String,
}
