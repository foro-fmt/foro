#![feature(test)]

use crate::cli::{execute, GlobalOptions};
use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod app_dir;
mod cli;
mod config;
mod daemon;
mod handle_plugin;
mod process_utils;

use crate::daemon::interface::{DaemonCommandPayload, DaemonCommands, DaemonFormatArgs};
use anyhow::Result;

fn main() -> Result<()> {
    // env::set_var("LD_LIBRARY_PATH", "/home/nahco314/.rustup/toolchains/nightly-2024-08-17-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib:");

    eprintln!(
        "{:?}",
        serde_json::to_string(&DaemonCommandPayload {
            command: DaemonCommands::Format(DaemonFormatArgs {
                path: PathBuf::from("./asd")
            }),
            current_dir: env::current_dir().unwrap(),
            global_options: GlobalOptions {
                config_file: None,
                cache_dir: None,
                socket_dir: None,
                no_cache: false,
            },
        })
    );

    let now = SystemTime::now();

    // UNIXエポックからの経過時間を取得
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // 秒とナノ秒をそれぞれ取得
    let seconds = since_the_epoch.as_secs();
    let nanoseconds = since_the_epoch.subsec_nanos();

    // マイクロ秒単位の精度を計算
    let microseconds = nanoseconds / 1_000;
    eprintln!("{}.{:06}", seconds, microseconds);

    let r = execute();

    let now = SystemTime::now();

    // UNIXエポックからの経過時間を取得
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // 秒とナノ秒をそれぞれ取得
    let seconds = since_the_epoch.as_secs();
    let nanoseconds = since_the_epoch.subsec_nanos();

    // マイクロ秒単位の精度を計算
    let microseconds = nanoseconds / 1_000;
    eprintln!("{}.{:06}", seconds, microseconds);

    r
}
