use crate::cli::format::FormatArgs;
use crate::cli::GlobalOptions;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub global_options: GlobalOptions,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonFormatResponse {
    Success,
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonResponse {
    Format(DaemonFormatResponse),
    Pong,
}
