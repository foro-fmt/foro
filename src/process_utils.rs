#[cfg(target_os = "linux")]
mod on_linux;
#[cfg(target_os = "linux")]
pub use on_linux::*;

#[cfg(target_os = "macos")]
mod on_macos;
#[cfg(target_os = "macos")]
pub use on_macos::*;

#[cfg(windows)]
mod on_windows;
#[cfg(windows)]
pub use on_windows::*;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_is_alive() {
        let my_pid = std::process::id();
        assert!(is_alive(my_pid));
    }

    #[test]
    fn test_is_alive_error() {
        let random_pid = 123456789;
        assert!(!is_alive(random_pid));
    }

    #[test]
    fn test_get_start_time() {
        let my_pid = std::process::id();
        let start_time = get_start_time(my_pid);
        assert!(start_time.is_ok());
    }
}
