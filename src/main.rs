#![feature(test)]
#![feature(thread_id_value)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::cli::execute;

mod app_dir;
mod build_info;
mod bulk_format;
mod cli;
mod config;
mod daemon;
mod handle_plugin;
mod log;
mod path_utils;
mod process_utils;

use anyhow::Result;

fn main() -> Result<()> {
    execute()
}
