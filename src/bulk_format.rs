use crate::config::Config;
use crate::debug_long;
use crate::handle_plugin::run::run;
use crate::log::DAEMON_THREAD_START;
use crate::path_utils::{normalize_path, to_wasm_path};
use anyhow::{anyhow, Context, Result};
use foro_plugin_utils::data_json_utils::JsonGetter;
use ignore::overrides::OverrideBuilder;
use ignore::{WalkBuilder, WalkState};
use log::{error, info, trace};
use serde_json::json;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{fs, io, thread};

#[derive(Clone)]
pub struct BulkFormatOption {
    pub paths: Vec<PathBuf>,
    pub threads: usize,
    pub use_default_ignore: bool,
    pub current_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BulkFormatSummary {
    pub changed_count: usize,
    pub unchanged_count: usize,
    pub ignored_count: usize,
    pub error_count: usize,
}

impl BulkFormatSummary {
    pub fn processed_count(&self) -> usize {
        self.changed_count + self.unchanged_count + self.ignored_count + self.error_count
    }
}

#[derive(Debug, Clone, Copy)]
enum FormatFileOutcome {
    Changed,
    Unchanged,
    Ignored,
}

fn format_file(
    path: &Path,
    current_dir: &Path,
    config: &Config,
    cache_path: &Path,
    use_cache: bool,
) -> Result<FormatFileOutcome> {
    info!("Formatting: {:?}", path);

    let Some(rule) = config.find_matched_rule(path) else {
        info!("No rule matched, ignored: {:?}", path);
        return Ok(FormatFileOutcome::Ignored);
    };

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

    if let Some(status) = String::get_value_opt(&res, ["format-status"]) {
        match status.as_str() {
            "ignored" => {
                info!("File ignored by formatter: {:?}", path);
                return Ok(FormatFileOutcome::Ignored);
            }
            "error" => {
                let format_error = String::get_value_opt(&res, ["format-error"])
                    .unwrap_or_else(|| "File formatting failed".to_string());
                return Err(anyhow!(format_error));
            }
            _ => {}
        }
    }

    let outcome = if let Some(formatted) = String::get_value_opt(&res, ["formatted-content"]) {
        if formatted != content {
            FormatFileOutcome::Changed
        } else {
            FormatFileOutcome::Unchanged
        }
    } else {
        FormatFileOutcome::Unchanged
    };

    info!("Success to format: {:?} ({:?})", path, outcome);

    Ok(outcome)
}

pub fn bulk_format(
    opt: &BulkFormatOption,
    config: &Config,
    cache_path: &Path,
    use_cache: bool,
) -> Result<BulkFormatSummary> {
    let (fst, rest) = opt.paths.split_first().context("No path given")?;

    let mut walk_builder = WalkBuilder::new(fst);
    for path in rest {
        walk_builder.add(path);
    }

    walk_builder.threads(opt.threads);
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

    let walk = walk_builder.build_parallel();

    let parent_start_time = DAEMON_THREAD_START.with(|start| *start.get_or_init(Instant::now));

    let formatting_threads = Arc::new(Mutex::new(Vec::new()));
    let changed_count = Arc::new(AtomicUsize::new(0));
    let unchanged_count = Arc::new(AtomicUsize::new(0));
    let ignored_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));
    let running_count = Arc::new(AtomicUsize::new(0));

    walk.run(|| {
        Box::new(|entry_res| {
            match entry_res {
                Ok(dir_entry) => {
                    let opt = opt.clone();
                    let config = config.clone();
                    let cache_path = cache_path.to_path_buf();
                    let changed_count = changed_count.clone();
                    let unchanged_count = unchanged_count.clone();
                    let ignored_count = ignored_count.clone();
                    let error_count = error_count.clone();
                    let running_count = running_count.clone();

                    let path = dir_entry.path().to_path_buf();

                    if path.is_dir() {
                        return WalkState::Continue;
                    }

                    let t = thread::spawn(move || {
                        DAEMON_THREAD_START.with(|start| {
                            let _ = start.set(parent_start_time);
                        });

                        while running_count.load(Ordering::SeqCst) >= opt.threads {
                            thread::sleep(std::time::Duration::from_micros(100));
                        }

                        running_count.fetch_add(1, Ordering::SeqCst);

                        let res = format_file(
                            &path,
                            path.parent().unwrap(),
                            &config,
                            &cache_path,
                            use_cache,
                        );

                        running_count.fetch_sub(1, Ordering::SeqCst);

                        match res {
                            Ok(FormatFileOutcome::Changed) => {
                                changed_count.fetch_add(1, Ordering::SeqCst);
                            }
                            Ok(FormatFileOutcome::Unchanged) => {
                                unchanged_count.fetch_add(1, Ordering::SeqCst);
                            }
                            Ok(FormatFileOutcome::Ignored) => {
                                ignored_count.fetch_add(1, Ordering::SeqCst);
                            }
                            Err(err) => {
                                error_count.fetch_add(1, Ordering::SeqCst);
                                error!("Error formatting file: {}", err);
                            }
                        }
                    });

                    formatting_threads.lock().unwrap().push(t);
                }
                Err(err) => {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    error!("Error reading entry: {}", err);
                }
            }

            WalkState::Continue
        })
    });

    for t in Arc::try_unwrap(formatting_threads)
        .unwrap()
        .into_inner()
        .unwrap()
    {
        t.join().unwrap();
    }

    Ok(BulkFormatSummary {
        changed_count: changed_count.load(Ordering::SeqCst),
        unchanged_count: unchanged_count.load(Ordering::SeqCst),
        ignored_count: ignored_count.load(Ordering::SeqCst),
        error_count: error_count.load(Ordering::SeqCst),
    })
}
