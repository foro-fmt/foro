use crate::config::Config;
use crate::daemon::interface::DaemonFormatResponse;
use crate::debug_long;
use crate::handle_plugin::run::run;
use crate::log::DAEMON_THREAD_START;
use anyhow::{Context, Result};
use ignore::overrides::OverrideBuilder;
use ignore::{WalkBuilder, WalkParallel, WalkState};
use log::{debug, error, info};
use serde_json::json;
use std::io::Read;
use std::path::PathBuf;
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

fn format_file(
    path: &PathBuf,
    current_dir: &PathBuf,
    config: &Config,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<()> {
    info!("Formatting: {:?}", path);

    let file = fs::File::open(&path)?;
    let mut buf_reader = io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let rule = config
        .find_matched_rule(&path, false)
        .context("No rule matched")?;

    debug_long!("run rule: {:?}", rule);

    let res = run(
        &rule.some_cmd,
        json!({
            "current-dir": current_dir.canonicalize()?.to_str().unwrap(),
            "target": path,
            "raw-target": path,
            "target-content": contents,
            }
        ),
        cache_path,
        use_cache,
    )?;

    debug_long!("{:?}", res);
    info!("Success to format: \"{:?}\"", path);

    Ok(())
}

pub fn bulk_format(
    opt: &BulkFormatOption,
    config: &Config,
    cache_path: &PathBuf,
    use_cache: bool,
) -> Result<usize> {
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

            overrides_builder.add(&format!("!{}", line)).unwrap();
        }

        let overrides = overrides_builder.build().unwrap();
        walk_builder.overrides(overrides);
    }

    let walk = walk_builder.build_parallel();

    let parent_start_time = DAEMON_THREAD_START.with(|start| *start.get_or_init(|| Instant::now()));

    let formatting_threads = Arc::new(Mutex::new(Vec::new()));
    let success_count = Arc::new(AtomicUsize::new(0));

    walk.run(|| {
        Box::new(|entry_res| {
            match entry_res {
                Ok(dir_entry) => {
                    let opt = opt.clone();
                    let config = config.clone();
                    let cache_path = cache_path.clone();
                    let success_count = success_count.clone();

                    let t = thread::spawn(move || {
                        DAEMON_THREAD_START.with(|start| {
                            let _ = start.set(parent_start_time);
                        });

                        let path = dir_entry.path();

                        if path.is_file() {
                            let res = format_file(
                                &path.to_path_buf(),
                                &opt.current_dir,
                                &config,
                                &cache_path,
                                use_cache,
                            );

                            match res {
                                Ok(_) => {
                                    success_count.fetch_add(1, Ordering::SeqCst);
                                }
                                Err(err) => {
                                    error!("Error formatting file: {}", err);
                                }
                            }
                        }
                    });

                    formatting_threads.lock().unwrap().push(t);
                }
                Err(err) => {
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

    Ok(success_count.load(Ordering::SeqCst))
}
