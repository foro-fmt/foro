pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub struct BuildInfo<'a> {
    version: &'a str,
    rustc_version: &'a str,
    target: &'a str,
    profile: &'a str,
}

fn normalize(s: &str) -> String {
    s.replace(" ", "_").replace("-", "_")
}

impl<'a> BuildInfo<'a> {
    pub fn build_id(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            normalize(self.version),
            normalize(self.rustc_version),
            normalize(self.target),
            normalize(self.profile),
        )
    }
}

pub fn get_build_id() -> String {
    BuildInfo {
        version: built_info::PKG_VERSION,
        rustc_version: built_info::RUSTC_VERSION,
        target: built_info::TARGET,
        profile: built_info::PROFILE,
    }
    .build_id()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_build_id_with_mocked_values() {
        let info = BuildInfo {
            version: "1.2.3-some",
            rustc_version: "rustc_1.88.0-nightly",
            target: "x86_64-unknown-linux-gnu",
            profile: "debug",
        };
        assert_eq!(
            info.build_id(),
            "1.2.3_some-rustc_1.88.0_nightly-x86_64_unknown_linux_gnu-debug"
        );
    }

    #[test]
    fn test_build_id_have_4_parts() {
        let info = get_build_id();
        let parts = info.split('-').collect::<Vec<_>>();
        assert_eq!(parts.len(), 4);
    }
}
