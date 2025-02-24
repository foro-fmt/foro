use crate::app_dir::AppDirResolver;
use std::path::PathBuf;

pub(crate) struct MacOSAppDirResolver;

impl AppDirResolver for MacOSAppDirResolver {
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
        // fixme: this is not best place
        //   /var/run/ have permission problem
        let mut a = dirs::config_dir()?;
        a.push("foro-socket-tmp");
        Some(a)
    }

    fn log_dir(&self) -> Option<PathBuf> {
        let mut a = self.socket_dir()?;
        a.push("log");
        Some(a)
    }
}
