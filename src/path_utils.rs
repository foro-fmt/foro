use anyhow::{Context, Result};
use std::path::PathBuf;

// Suppress unused warnings in unix
#[allow(unused)]
fn strip_windows_path(path_str: &str) -> Result<String> {
    // Remove `\\?\`.
    let path_str = &path_str[4..];

    Ok(path_str.to_string())
}

#[allow(unused)]
fn convert_windows_path(path_str: &str) -> Result<String> {
    // strip
    let path_str = strip_windows_path(path_str)?;

    // Get name of drive, like `C`.
    let drive = path_str.chars().next().context("Failed to get drive")?;

    let path_str = format!(
        "/{}{}",
        drive.to_lowercase(),
        &path_str[2..].replace("\\", "/")
    );

    Ok(path_str.to_string())
}

#[cfg(not(windows))]
pub fn normalize_path(path: &PathBuf) -> Result<String> {
    path.canonicalize()?
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string"))
}

#[cfg(windows)]
pub fn normalize_path(path: &PathBuf) -> Result<String> {
    use anyhow::Context;

    let abs = path.canonicalize()?;

    // This is like be `\\?\C:\\Users\...`.
    let path_str = abs.to_str().context("Failed to convert path to string")?;

    strip_windows_path(path_str)
}

#[cfg(not(windows))]
pub fn to_wasm_path(path: &PathBuf) -> Result<String> {
    path.canonicalize()?
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string"))
}

#[cfg(windows)]
pub fn to_wasm_path(path: &PathBuf) -> Result<String> {
    use anyhow::Context;

    let abs = path.canonicalize()?;

    // This is like be `\\?\C:\\Users\...`.
    let path_str = abs.to_str().context("Failed to convert path to string")?;

    convert_windows_path(path_str)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_strip_windows_path() {
        assert_eq!(
            strip_windows_path(r"\\?\C:\Users\test").unwrap(),
            r"C:\Users\test"
        );
    }

    #[test]
    fn test_convert_windows_path() {
        assert_eq!(
            convert_windows_path(r"\\?\C:\Users\test").unwrap(),
            r"/c/Users/test"
        );
    }

    #[cfg_attr(not(windows), ignore)]
    #[test]
    fn test_normalize_path_windows() {
        assert_eq!(
            normalize_path(&PathBuf::from(r"C:\Users\test")).unwrap(),
            r"C:\Users\test"
        );
    }

    #[cfg_attr(not(windows), ignore)]
    #[test]
    fn test_to_wasm_path_windows() {
        assert_eq!(
            to_wasm_path(&PathBuf::from(r"C:\Users\test")).unwrap(),
            r"/c/Users/test"
        );
    }

    #[cfg_attr(not(unix), ignore)]
    #[test]
    fn test_normalize_path_unix() {
        let pwd = std::env::current_dir().unwrap();
        let pwd_str = pwd.to_str().unwrap();

        assert_eq!(normalize_path(&pwd).unwrap(), pwd_str);
    }

    #[cfg_attr(not(unix), ignore)]
    #[test]
    fn test_to_wasm_path_unix() {
        let pwd = std::env::current_dir().unwrap();
        let pwd_str = pwd.to_str().unwrap();

        assert_eq!(to_wasm_path(&pwd).unwrap(), pwd_str);
    }
}
