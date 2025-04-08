pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn get_build_id() -> String {
    let version = built_info::PKG_VERSION;
    let target = built_info::TARGET;
    let profile = built_info::PROFILE;
    
    let git_suffix = if let Some(git_version) = built_info::GIT_VERSION {
        format!("-{}", git_version)
    } else {
        String::new()
    };
    
    format!("{}{}-{}-{}", version, git_suffix, target, profile)
}
