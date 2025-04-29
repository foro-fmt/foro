fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");

    let dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(Into::into)
        .unwrap_or_else(|| {
            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            std::path::PathBuf::from(manifest).join("target")
        });
    println!("cargo:rustc-env=TARGET_DIR={}", dir.display());
}
