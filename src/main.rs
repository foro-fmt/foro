use crate::cli::execute;
use std::process;
use std::str::FromStr;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod app_dir;
mod cli;
mod config;
mod format;
mod handle_plugin;

use handle_plugin::load::load_url_module;

use anyhow::{Context, Result};
use log::LevelFilter;
use serde_json::{json, Value};
use url::Url;
use wasmtime::{Config, Engine, Linker, Store};
use wasmtime_wasi::{preview1, DirPerms, FilePerms, WasiCtxBuilder};

fn main() -> Result<()> {
    let now = SystemTime::now();

    // UNIXエポックからの経過時間を取得
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // 秒とナノ秒をそれぞれ取得
    let seconds = since_the_epoch.as_secs();
    let nanoseconds = since_the_epoch.subsec_nanos();

    // マイクロ秒単位の精度を計算
    let microseconds = nanoseconds / 1_000;
    // println!("{}.{:06}", seconds, microseconds);

    let r = execute();

    let now = SystemTime::now();

    // UNIXエポックからの経過時間を取得
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // 秒とナノ秒をそれぞれ取得
    let seconds = since_the_epoch.as_secs();
    let nanoseconds = since_the_epoch.subsec_nanos();

    // マイクロ秒単位の精度を計算
    let microseconds = nanoseconds / 1_000;
    // println!("{}.{:06}", seconds, microseconds);

    r
}
