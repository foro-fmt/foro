use crate::config::Config;
use crate::debug_long;
use crate::handle_plugin::run::run;
use crate::log::DAEMON_THREAD_START;
use crate::path_utils::{normalize_path, to_wasm_path};
use anyhow::{Context, Result};
use foro_plugin_utils::data_json_utils::JsonGetter;
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use log::{error, info, trace};
use serde_json::json;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::{fs, io, thread};

#[derive(Clone)]
pub struct BulkFormatOption {
    pub paths: Vec<PathBuf>,
    pub threads: usize,
    pub use_default_ignore: bool,
    pub current_dir: PathBuf,
}

fn format_file(
    path: &Path,
    current_dir: &Path,
    config: &Config,
    cache_path: &Path,
    use_cache: bool,
) -> Result<bool> {
    // Return type changed to bool: true indicates the file was changed
    info!("Formatting: {:?}", path);

    let rule = config.find_matched_rule(path).context("No rule matched")?;

    debug_long!("run rule: {:?}", rule);

    let file = fs::File::open(path)?;
    let mut buf_reader = io::BufReader::new(file);
    let mut content = String::new();
    buf_reader.read_to_string(&mut content)?;

    trace!("opened file: {:?}", path);

    let res = run(
        &rule.cmd,
        json!({
            "wasm-current-dir":  to_wasm_path(current_dir)?,
            "os-current-dir": normalize_path(current_dir)?,
            "wasm-target": to_wasm_path(path)?,
            "os-target": normalize_path(path)?,
            "raw-target": path,
            "target-content": content,
        }),
        cache_path,
        use_cache,
    )?;

    debug_long!("{:?}", res);

    let was_changed = if let Some(formatted) = String::get_value_opt(&res, ["formatted-content"]) {
        formatted != content // Check if content has changed by comparing with original
    } else {
        false // Consider unchanged if no formatted content is available
    };

    info!(
        "Success to format: {:?} ({})",
        path,
        if was_changed { "changed" } else { "unchanged" }
    );

    Ok(was_changed) // Return whether the file was changed
}

pub fn bulk_format(
    opt: &BulkFormatOption,
    config: &Config,
    cache_path: &Path,
    use_cache: bool,
) -> Result<(usize, usize)> {
    // Returns (count of changed files, count of unchanged files)
    let (fst, rest) = opt.paths.split_first().context("No path given")?;

    let mut walk_builder = WalkBuilder::new(fst);
    for path in rest {
        walk_builder.add(path);
    }

    walk_builder.add_custom_ignore_filename(".foro-ignore");

    if opt.use_default_ignore {
        let default_ignore_content = include_str!("./default_ignore.txt");
        let mut overrides_builder = OverrideBuilder::new(&opt.current_dir);

        for line in default_ignore_content.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            overrides_builder.add(&format!("!{line}")).unwrap();
        }

        let overrides = overrides_builder.build().unwrap();
        walk_builder.overrides(overrides);
    }

    let parent_start_time = DAEMON_THREAD_START.with(|start| *start.get_or_init(Instant::now));
    let worker_count = opt.threads.max(1);

    let changed_count = Arc::new(AtomicUsize::new(0)); // Count of files that were changed
    let unchanged_count = Arc::new(AtomicUsize::new(0)); // Count of files that were not changed

    let walk = walk_builder.build();
    let mut files = Vec::new();
    for entry_res in walk {
        match entry_res {
            Ok(dir_entry) => {
                let path = dir_entry.path().to_path_buf();

                if path.is_dir() {
                    continue;
                }

                files.push(path);
            }
            Err(err) => {
                error!("Error reading entry: {}", err);
            }
        }
    }

    let files = Arc::new(files);
    let next_index = Arc::new(AtomicUsize::new(0));

    let mut worker_threads = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let files = files.clone();
        let next_index = next_index.clone();
        let config = config.clone();
        let cache_path = cache_path.to_path_buf();
        let changed_count = changed_count.clone();
        let unchanged_count = unchanged_count.clone();

        let worker = thread::spawn(move || {
            DAEMON_THREAD_START.with(|start| {
                let _ = start.set(parent_start_time);
            });

            loop {
                let index = next_index.fetch_add(1, Ordering::SeqCst);
                if index >= files.len() {
                    break;
                }

                let path = &files[index];
                let res = format_file(
                    path,
                    path.parent().unwrap(),
                    &config,
                    &cache_path,
                    use_cache,
                );

                match res {
                    Ok(was_changed) => {
                        if was_changed {
                            changed_count.fetch_add(1, Ordering::SeqCst);
                        } else {
                            unchanged_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    Err(err) => {
                        error!("Error formatting file: {}", err);
                    }
                }
            }
        });

        worker_threads.push(worker);
    }

    for t in worker_threads {
        t.join().unwrap();
    }

    Ok((
        changed_count.load(Ordering::SeqCst),
        unchanged_count.load(Ordering::SeqCst),
    ))
}
