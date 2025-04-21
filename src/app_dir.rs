#[cfg(target_os = "linux")]
mod linux_resolver;
#[cfg(target_os = "macos")]
mod macos_resolver;
#[cfg(target_os = "windows")]
mod windows_resolver;

use anyhow::{Context, Result};
use std::path::PathBuf;

pub(crate) trait AppDirResolver {
    fn config_file(&self) -> Option<PathBuf>;
    fn cache_dir(&self) -> Option<PathBuf>;
    fn socket_dir(&self) -> Option<PathBuf>;
    fn log_dir(&self) -> Option<PathBuf>;

    fn config_file_res(&self) -> Result<PathBuf> {
        self.config_file()
            .context("Failed to get default config file")
    }

    fn cache_dir_res(&self) -> Result<PathBuf> {
        self.cache_dir().context("Failed to get default cache dir")
    }

    fn socket_dir_res(&self) -> Result<PathBuf> {
        self.socket_dir()
            .context("Failed to get default socket dir")
    }

    fn log_dir_res(&self) -> Result<PathBuf> {
        self.log_dir().context("Failed to get default log dir")
    }
}

#[cfg(target_os = "linux")]
pub(crate) type DefaultAppDirResolver = linux_resolver::LinuxAppDirResolver;

#[cfg(target_os = "macos")]
pub(crate) type DefaultAppDirResolver = macos_resolver::MacOSAppDirResolver;

#[cfg(target_os = "windows")]
pub(crate) type DefaultAppDirResolver = windows_resolver::WindowsAppDirResolver;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_dir_paths() {
        let r = DefaultAppDirResolver {};

        let cfg = r.config_file_res().unwrap();
        assert!(cfg.ends_with("foro.json"));

        let cache = r.cache_dir_res().unwrap();
        assert!(cache.ends_with("foro"));

        let sock = r.socket_dir_res().unwrap();
        assert!(sock.ends_with("foro"));

        let log = r.log_dir_res().unwrap();
        assert!(log.ends_with("foro/log"));
    }
}
