use anstyle::Style;
use clap::builder::styling::Color;
use env_logger::fmt::style::RgbColor;
use env_logger::Target;
use log::LevelFilter;
use std::cell::OnceCell;
use std::io::{Result as IOResult, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::time::Instant;

pub static IS_DAEMON_PROCESS: AtomicBool = AtomicBool::new(false);
thread_local!(pub static DAEMON_THREAD_START: OnceCell<Instant> = const { OnceCell::new() });
thread_local!(pub static IS_DAEMON_MAIN_THREAD: OnceCell<bool> = const { OnceCell::new() });
pub static NO_LONG_LOG: AtomicBool = AtomicBool::new(false);

// created by iwanthue (https://medialab.github.io/iwanthue/)
const COLOR_LIST: [Color; 32] = [
    Color::Rgb(RgbColor(156, 70, 103)),
    Color::Rgb(RgbColor(65, 198, 93)),
    Color::Rgb(RgbColor(176, 88, 208)),
    Color::Rgb(RgbColor(107, 184, 56)),
    Color::Rgb(RgbColor(92, 96, 206)),
    Color::Rgb(RgbColor(200, 187, 57)),
    Color::Rgb(RgbColor(158, 131, 232)),
    Color::Rgb(RgbColor(148, 189, 74)),
    Color::Rgb(RgbColor(205, 72, 162)),
    Color::Rgb(RgbColor(58, 141, 51)),
    Color::Rgb(RgbColor(213, 65, 107)),
    Color::Rgb(RgbColor(77, 187, 120)),
    Color::Rgb(RgbColor(208, 63, 55)),
    Color::Rgb(RgbColor(68, 199, 174)),
    Color::Rgb(RgbColor(216, 104, 47)),
    Color::Rgb(RgbColor(78, 127, 195)),
    Color::Rgb(RgbColor(222, 153, 54)),
    Color::Rgb(RgbColor(132, 84, 156)),
    Color::Rgb(RgbColor(168, 155, 51)),
    Color::Rgb(RgbColor(165, 153, 221)),
    Color::Rgb(RgbColor(97, 119, 25)),
    Color::Rgb(RgbColor(219, 133, 188)),
    Color::Rgb(RgbColor(100, 137, 66)),
    Color::Rgb(RgbColor(219, 121, 119)),
    Color::Rgb(RgbColor(75, 182, 211)),
    Color::Rgb(RgbColor(160, 79, 49)),
    Color::Rgb(RgbColor(124, 190, 138)),
    Color::Rgb(RgbColor(218, 152, 106)),
    Color::Rgb(RgbColor(52, 130, 92)),
    Color::Rgb(RgbColor(154, 110, 45)),
    Color::Rgb(RgbColor(176, 177, 107)),
    Color::Rgb(RgbColor(109, 106, 44)),
];

static TEST_LOG_BUF: LazyLock<RwLock<Arc<Mutex<Vec<u8>>>>> =
    LazyLock::new(|| RwLock::new(Arc::new(Mutex::new(Vec::new()))));

#[derive(Clone)]
struct SharedBuf(Arc<Mutex<Vec<u8>>>);

impl Write for SharedBuf {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> IOResult<()> {
        Ok(())
    }
}

pub(crate) struct LogTestConfig {}

pub(crate) fn init_env_logger(
    level_filter: LevelFilter,
    no_long_log: bool,
    test: Option<LogTestConfig>,
) {
    NO_LONG_LOG.store(no_long_log, Ordering::SeqCst);

    let start_time = Instant::now();

    let mut logger = env_logger::Builder::new();

    logger
        .filter_module("foro", level_filter)
        .filter_module("dll_pack", level_filter)
        .format(move |buf, record| {
            if IS_DAEMON_PROCESS.load(Ordering::SeqCst) {
                let now = buf.timestamp_micros();

                let elapsed =
                    DAEMON_THREAD_START.with(|start| start.get_or_init(Instant::now).elapsed());
                let elapsed_micros = elapsed.as_micros();

                let is_main_thread = IS_DAEMON_MAIN_THREAD
                    .with(|is_main_thread| *is_main_thread.get_or_init(|| false));

                let level = record.level();
                let level_style = buf.default_level_style(level);

                let path = record.module_path().unwrap_or("");

                write!(buf, "[{now} ")?;
                if !is_main_thread {
                    let thread_id = std::thread::current().id().as_u64().get() as usize;
                    let thread_color = COLOR_LIST[thread_id % COLOR_LIST.len()];
                    let style = Style::new().fg_color(Some(thread_color));

                    write!(buf, "{style}{thread_id:>3}{style:#} ")?;
                    write!(buf, "{elapsed_micros:>5} μs ")?;
                }
                write!(buf, "{level_style}{level:<5}{level_style:#} ")?;
                write!(buf, "{path}] ")?;
                write!(buf, "{body}", body = record.args())?;
                writeln!(buf)?;
            } else {
                let elapsed = start_time.elapsed();
                let elapsed_micros = elapsed.as_micros();

                let level = record.level();
                let level_style = buf.default_level_style(level);

                let path = record.module_path().unwrap_or("");

                write!(buf, "[{elapsed_micros:>5} μs ")?;
                write!(buf, "{level_style}{level:<5}{level_style:#} ")?;
                write!(buf, "{path}] ")?;
                write!(buf, "{body}", body = record.args())?;
                writeln!(buf)?;
            }

            Ok(())
        });

    if let Some(LogTestConfig {}) = test {
        logger.is_test(true).target(Target::Pipe(Box::new(SharedBuf(
            TEST_LOG_BUF.read().unwrap().clone(),
        ))));
    }

    logger.init();
}

#[macro_export]
macro_rules! debug_long {
    ($($arg:tt)*) => {
        if !$crate::log::NO_LONG_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            log::debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! trace_long {
    ($($arg:tt)*) => {
        if !$crate::log::NO_LONG_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            log::trace!($($arg)*);
        }
    };
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use ctor::ctor;

    use log::{debug, LevelFilter};
    use regex::Regex;
    use serial_test::serial;

    use std::thread;

    // If you do not initialize the logger at the start of the process, the log macro will be
    // executed first by other unit tests, making it impossible to set the logger.
    #[ctor]
    fn foo() {
        init_env_logger(LevelFilter::Debug, false, Some(LogTestConfig {}));
    }

    fn clear_log() {
        TEST_LOG_BUF.read().unwrap().lock().unwrap().clear();
    }

    fn take_log() -> String {
        let binding = TEST_LOG_BUF.read().unwrap();
        let mut guard = binding.lock().unwrap();
        let out = String::from_utf8_lossy(&guard[..]).into_owned();
        guard.clear();
        out
    }

    fn init_logger() {
        clear_log()
    }

    #[test]
    #[serial]
    fn debug_shows_elapsed() {
        init_logger();

        IS_DAEMON_PROCESS.store(false, Ordering::SeqCst);

        debug!("hello {}", 42);

        let out = take_log();
        let re = Regex::new(r"\[ *\d+ μs DEBUG foro::log::tests] hello 42").unwrap();

        assert!(re.is_match(&out));
    }

    #[test]
    #[serial]
    fn long_logs_respect_flag() {
        init_logger();

        NO_LONG_LOG.store(true, Ordering::SeqCst);
        debug_long!("0_should_not_be_printed");
        trace_long!("0_should_not_be_printed");

        NO_LONG_LOG.store(false, Ordering::SeqCst);
        debug_long!("1_should_be_printed");

        let out = take_log();

        assert!(!out.contains("0_should_not_be_printed"));
        assert!(out.contains("1_should_be_printed"));
    }

    #[test]
    #[serial]
    fn daemon_thread_coloring() {
        init_logger();

        IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
        IS_DAEMON_MAIN_THREAD.with(|cell| {
            cell.set(true).unwrap();
        });

        debug!("main thread");

        thread::spawn(|| {
            IS_DAEMON_MAIN_THREAD.with(|cell| {
                cell.set(false).unwrap();
            });
            debug!("worker thread");
        })
        .join()
        .unwrap();

        let out = take_log();

        assert!(out.contains("main thread"));
        assert!(out.contains("worker thread"));
    }
}
