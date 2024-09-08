use crate::cli::format::FormatArgs;
use crate::cli::{GlobalOptions, IS_DAEMON_PROCESS};
use crate::config::{load_config_and_cache, load_config_and_socket};
use crate::daemon::client::ping;
use crate::daemon::interface::{
    DaemonCommandPayload, DaemonCommands, DaemonFormatArgs, DaemonFormatResponse, DaemonResponse,
};
use crate::handle_plugin::run::run;
use anyhow::{anyhow, Context, Result};
use log::{debug, error, info};
use nix::libc::{statx, AT_STATX_SYNC_AS_STAT, STATX_ALL};
use nix::unistd::{fork, ForkResult};
use os_pipe::PipeWriter;
use serde_json::json;
use std::env::current_dir;
use std::ffi::CString;
use std::io::{ErrorKind, Read};
use std::net::Shutdown;
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io, mem, process, thread, time};
use wasmtime::component::__internal::wasmtime_environ::wasmparser::Payload;

#[repr(C)]
#[derive(Debug)]
struct StatxTimestamp {
    tv_sec: i64,
    tv_nsec: u32,
    __reserved: i32,
}

#[repr(C)]
#[derive(Debug)]
struct Statx {
    stx_mask: u32,
    stx_blksize: u32,
    stx_attributes: u64,
    stx_nlink: u32,
    stx_uid: u32,
    stx_gid: u32,
    stx_mode: u16,
    __reserved0: [u16; 1],
    stx_ino: u64,
    stx_size: u64,
    stx_blocks: u64,
    stx_attributes_mask: u64,
    stx_atime: StatxTimestamp,
    stx_btime: StatxTimestamp,
    stx_ctime: StatxTimestamp,
    stx_mtime: StatxTimestamp,
    stx_rdev_major: u32,
    stx_rdev_minor: u32,
    stx_dev_major: u32,
    stx_dev_minor: u32,
    __reserved2: [u64; 14],
}

const AT_FDCWD: i32 = -100;
const STATX_BASIC_STATS: u32 = 0x000007ff;

fn get_file_statx(path: &str) -> Result<nix::libc::statx> {
    let c_path = CString::new(path)
        .map_err(|e| e.to_string())
        .map_err(|e| anyhow!(e))?;
    let mut statxbuf: nix::libc::statx = unsafe { mem::zeroed() };

    let ret = unsafe {
        nix::libc::statx(
            AT_FDCWD,
            c_path.as_ptr(),
            AT_STATX_SYNC_AS_STAT,
            STATX_BASIC_STATS,
            &mut statxbuf,
        )
    };

    if ret != 0 {
        Err(anyhow!("statx failed with error code: {}", ret))
    } else {
        Ok(statxbuf)
    }
}

pub fn daemon_format_execute_with_args(
    args: DaemonFormatArgs,
    current_dir: PathBuf,
    global_options: GlobalOptions,
) -> Result<()> {
    let no_quick_magic =
        std::env::var_os("ONEFMT_NO_QUICK_MAGIC").is_some_and(|s| s != "0" && s != "");

    debug!("no_quick_magic: {}", no_quick_magic);

    let target_path = current_dir.join(&args.path).canonicalize()?;
    let target_path_outer = target_path.clone();

    let (tx, rx) = mpsc::channel();

    let t = thread::spawn(move || -> Result<()> {
        let (config, cache_dir) =
            load_config_and_cache(&global_options.config_file, &global_options.cache_dir)?;

        let file = fs::File::open(&target_path)?;
        let mut buf_reader = io::BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        let res = run(
            &config.rules.first().unwrap().cmd,
            json!({
                "current-dir": current_dir.canonicalize()?.to_str().unwrap(),
                "target": target_path.to_str().unwrap(),
                "raw-target": args.path,
                "target-content": contents,
                }
            ),
            &cache_dir,
            !global_options.no_cache,
        )?;

        println!("{:?}", res);

        tx.send(0)?;

        Ok(())
    });

    let modified_time = get_file_statx(&target_path_outer.to_str().unwrap())?
        .stx_mtime
        .tv_nsec as u64;

    loop {
        let new_modified_time = get_file_statx(&target_path_outer.to_str().unwrap())?
            .stx_mtime
            .tv_nsec as u64;

        debug!("hmm, {:?} {:?}", &new_modified_time, &modified_time);

        if new_modified_time != modified_time {
            info!("quick magic detected file changed");
            break;
        }

        if rx.try_recv().is_ok() {
            info!("quick magic detected child finished");
            break;
        }

        sleep(time::Duration::from_micros(100));
    }

    info!("main process exit");

    if t.is_finished() {
        let res = t.join().unwrap();
        res?;
    }

    let now = SystemTime::now();

    // UNIXエポックからの経過時間を取得
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // 秒とナノ秒をそれぞれ取得
    let seconds = since_the_epoch.as_secs();
    let nanoseconds = since_the_epoch.subsec_nanos();

    // マイクロ秒単位の精度を計算
    let microseconds = nanoseconds / 1_000;
    println!("{}.{:06}", seconds, microseconds);

    Ok(())
}

fn serverside_exec_command(payload: DaemonCommandPayload) -> Result<DaemonResponse> {
    match payload.command {
        DaemonCommands::Format(s_args) => {
            match daemon_format_execute_with_args(
                s_args,
                payload.current_dir,
                payload.global_options,
            ) {
                Ok(_) => Ok(DaemonResponse::Format(DaemonFormatResponse::Success)),
                Err(err) => Ok(DaemonResponse::Format(DaemonFormatResponse::Error(
                    err.to_string(),
                ))),
            }
        }
        DaemonCommands::Ping => Ok(DaemonResponse::Pong),
    }
}

fn handle_client(mut stream: UnixStream) -> Result<()> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let payload: DaemonCommandPayload = serde_json::from_slice(&buf)?;

    info!("Received: {:?}", &payload);

    let response = serverside_exec_command(payload)?;

    info!("Response: {:?}", &response);

    stream.shutdown(Shutdown::Read)?;

    serde_json::to_writer(stream, &response)?;

    Ok(())
}

pub struct WrappedUnixSocket {
    path: PathBuf,
    listener: UnixListener,
}

impl WrappedUnixSocket {
    pub(crate) fn bind(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let parent = path.parent().context("Failed to get parent")?;
        fs::create_dir_all(parent)?;

        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(err) if err.kind() == ErrorKind::AddrInUse => {
                if ping(&path)? {
                    return Err(anyhow!("Daemon is already running"));
                } else {
                    info!("Removing dead socket file");
                    fs::remove_file(&path)?;
                    UnixListener::bind(&path)?
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        info!("Listening on: {}", path.display());

        Ok(Self { path, listener })
    }
}

impl Drop for WrappedUnixSocket {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn daemon_main(socket: WrappedUnixSocket) -> Result<()> {
    for stream in socket.listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                error!("Error: {}", err);
            }
        }
    }

    Ok(())
}

pub fn start_daemon(socket: &PathBuf, attach: bool) -> Result<()> {
    info!("Starting daemon (attach: {})", attach);

    let listener = WrappedUnixSocket::bind(socket)?;

    if attach {
        IS_DAEMON_PROCESS.store(true, Ordering::SeqCst);
        daemon_main(listener)?;
    } else {
    }

    Ok(())
}
