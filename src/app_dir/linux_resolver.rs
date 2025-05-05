use crate::app_dir::AppDirResolver;
use std::path::PathBuf;

pub(crate) struct LinuxAppDirResolver;

impl AppDirResolver for LinuxAppDirResolver {
    fn config_file(&self) -> Option<PathBuf> {
        let mut a = dirs::config_dir()?;
        a.push("foro.json");
        Some(a)
    }

    fn cache_dir(&self) -> Option<PathBuf> {
        let mut a = dirs::cache_dir()?;
        a.push("foro");
        Some(a)
    }

    fn socket_dir(&self) -> Option<PathBuf> {
        if let Some(mut a) = dirs::runtime_dir() {
            a.push("foro");
            Some(a)
        } else {
            let mut a = dirs::config_dir()?;
            a.push("foro-socket-tmp/foro");
            Some(a)
        }
    }

    fn log_dir(&self) -> Option<PathBuf> {
        // is this correct?
        let mut a = self.socket_dir()?;
        a.push("log");
        Some(a)
    }
}
