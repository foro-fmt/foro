use anyhow::Result;
use log::{debug, info, warn};
use std::time::SystemTime;
use std::{fs, io, path::Path, path::PathBuf, thread, time::Duration};

pub struct StartupLock {
    path: PathBuf,
}

impl StartupLock {
    pub fn acquire(socket_dir: &Path) -> Result<Self> {
        let path = socket_dir.join("daemon-start.lock");
        let mut taken_lock_started: Option<SystemTime> = None;

        loop {
            match fs::create_dir(&path) {
                Ok(()) => {
                    debug!("startup-lock acquired");
                    return Ok(Self { path });
                }
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    match taken_lock_started {
                        None => {
                            taken_lock_started = Some(path.metadata()?.modified()?);
                            info!("startup-lock held, waiting...");
                        }
                        Some(t) => {
                            if t.elapsed()?.as_secs_f32() > 1.0 {
                                warn!("startup-lock stale, releasing...");
                                let _ = fs::remove_dir_all(&path);
                                continue;
                            }
                        }
                    }

                    thread::sleep(Duration::from_micros(10));
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

impl Drop for StartupLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
        debug!("startup-lock released");
    }
}
