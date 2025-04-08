pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn get_build_id() -> String {
    let version = built_info::PKG_VERSION;
    let target = built_info::TARGET;
    let profile = built_info::PROFILE;

    let rustc_version = built_info::RUSTC_VERSION.replace(" ", "_");

    format!("{}-{}-{}-{}", version, rustc_version, target, profile)
}
