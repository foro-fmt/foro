use anstyle::Style;
use clap::builder::styling::Color;
use env_logger::fmt::style::RgbColor;
use log::LevelFilter;
use std::cell::OnceCell;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub static IS_DAEMON_PROCESS: AtomicBool = AtomicBool::new(false);
thread_local!(pub static DAEMON_THREAD_START: OnceCell<Instant> = OnceCell::new());
thread_local!(pub static IS_DAEMON_MAIN_THREAD: OnceCell<bool> = OnceCell::new());
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

pub(crate) fn init_env_logger(level_filter: LevelFilter, no_long_log: bool) {
    NO_LONG_LOG.store(no_long_log, Ordering::SeqCst);

    let start_time = Instant::now();

    env_logger::Builder::new()
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
                write!(buf, "{body}\n", body = record.args())?;
            } else {
                let elapsed = start_time.elapsed();
                let elapsed_micros = elapsed.as_micros();

                let level = record.level();
                let level_style = buf.default_level_style(level);

                let path = record.module_path().unwrap_or("");

                write!(buf, "[{elapsed_micros:>5} μs ")?;
                write!(buf, "{level_style}{level:<5}{level_style:#} ")?;
                write!(buf, "{path}] ")?;
                write!(buf, "{body}\n", body = record.args())?;
            }

            Ok(())
        })
        .init();
}

#[macro_export]
macro_rules! debug_long {
    ($($arg:tt)*) => {
        if !crate::log::NO_LONG_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            log::debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! trace_long {
    ($($arg:tt)*) => {
        if !crate::log::NO_LONG_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            log::trace!($($arg)*);
        }
    };
}
