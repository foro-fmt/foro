use anyhow::Result;
use std::path::PathBuf;

#[cfg(not(windows))]
pub fn normalize_path(path: &PathBuf) -> Result<PathBuf> {
    path.canonicalize().map_err(|e| e.into())
}

#[cfg(windows)]
pub fn normalize_path(path: &PathBuf) -> Result<PathBuf> {
    use anyhow::Context;

    let abs = path.canonicalize()?;

    // This is like be `\\?\C:\\Users\...`.
    let path_str = abs.to_str().context("Failed to convert path to string")?;

    // Remove `\\?\`.
    let path_str = &path_str[4..];

    // Get name of drive, like `C`.
    let drive = path_str.chars().next().context("Failed to get drive")?;

    let path_str = format!(
        "/{}{}",
        drive.to_lowercase(),
        &path_str[2..].replace("\\", "/")
    );

    Ok(PathBuf::from(path_str))
}
