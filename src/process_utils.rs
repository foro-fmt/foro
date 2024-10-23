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
