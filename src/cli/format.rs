use crate::app_dir::cache_dir_res;
use crate::cli::GlobalOptions;
use crate::config;
use crate::config::{get_or_create_default_config, load_config_for_cli, load_file, Command};
use crate::handle_plugin::run::run;
use anyhow::{Context, Error, Result};
use clap::builder::{IntoResettable, Resettable, ValueHint};
use clap::Parser;
use log::{debug, info};
use serde_json::json;
use std::env::current_dir;
use std::fmt::Display;
use std::io::{stdout, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{fs, io, process, thread};
use url::Url;
use url_serde::{Serde, SerdeUrl};

#[derive(Parser, Debug)]
pub struct FormatArgs {
    /// Path to format
    pub path: PathBuf,
}

pub fn format_execute_with_args(
    args: FormatArgs,
    global_options: GlobalOptions,
    start_time_system: SystemTime,
) -> Result<()> {
    let no_quick_magic =
        std::env::var_os("ONEFMT_NO_QUICK_MAGIC").is_some_and(|s| s != "0" && s != "");
    let inner_quick_magic = std::env::var_os("ONEFMT_NO_QUICK_MAGIC").is_some_and(|s| s == "inner");

    debug!("no_quick_magic: {}", no_quick_magic);
    debug!("inner_quick_magic: {}", inner_quick_magic);

    if !no_quick_magic {
        info!("running quick magic");

        let target_path = args.path.canonicalize()?;

        let raw_args = std::env::args().collect::<Vec<_>>();
        let (exec, args) = raw_args.split_first().unwrap();

        let mut proc = process::Command::new(exec)
            .args(args)
            .env("ONEFMT_NO_QUICK_MAGIC", "inner")
            .env(
                "ONEFMT_START_MICROS",
                start_time_system
                    .duration_since(UNIX_EPOCH)?
                    .as_micros()
                    .to_string(),
            )
            .stdout(process::Stdio::piped())
            .spawn()
            .context("Failed to execute command")?;

        let (tx, rx) = mpsc::channel();
        let tx0 = tx.clone();
        let tx1 = tx.clone();

        let thread_waiting_proc = thread::spawn(move || {
            let buf: &mut [u8] = &mut [0];
            let _ = proc.stdout.unwrap().read_exact(buf);
            info!("proc stdout read done");
            tx0.send(1).unwrap();
        });

        let thread_checking_file_modify = thread::spawn(move || {
            let metadata = fs::metadata(&target_path).unwrap();
            let modified_time = metadata.modified().unwrap();

            loop {
                let metadata = fs::metadata(&target_path).unwrap();
                let new_modified_time = metadata.modified().unwrap();

                if new_modified_time != modified_time {
                    break;
                }

                thread::sleep(std::time::Duration::from_micros(500));
            }

            info!("file modified");

            tx1.send(1).unwrap();
        });

        let _ = rx.recv()?;

        thread_waiting_proc.join().unwrap();
        thread_checking_file_modify.join().unwrap();

        info!("main process exit");

        process::exit(0);
    }

    let (config, cache_dir) =
        load_config_for_cli(&global_options.config_file, &global_options.cache_dir)?;

    let file = fs::File::open(&args.path)?;
    let mut buf_reader = io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let res = run(
        &config.rules.first().unwrap().cmd,
        json!({
            "current-dir": current_dir()?.canonicalize()?.to_str().unwrap(),
            "target": args.path.canonicalize()?.to_str().unwrap(),
            "raw-target": args.path,
            "target-content": contents,
            }
        ),
        &cache_dir,
        !global_options.no_cache,
    )?;

    if inner_quick_magic {
        match writeln!(&mut stdout(), "y") {
            Ok(_) => {}
            Err(err) if err.kind() == io::ErrorKind::BrokenPipe => process::exit(141),
            Err(err) => {
                return Err(err.into());
            }
        }
    } else {
        println!("{:?}", res);
    }

    Ok(())
}
