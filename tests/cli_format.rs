mod common;

use crate::common::{TestEnv, TestEnvBuilder};
use assert_cmd::prelude::*;

#[test]
fn test_cli_format_rust_basic() {
    let env = TestEnv::new("./tests/fixtures/cli_format_rust/basic/");

    env.foro(&["format", "./main.rs"]);
    env.assert_eq("main.rs", "expected.rs");
}

#[test]
fn test_cli_format_rust_ignore() {
    let env = TestEnv::new("./tests/fixtures/cli_format_rust/ignore/");

    env.foro(&["format", "./main.rs"]);
    env.assert_eq("main.rs", "expected.rs");
}

#[test]
fn test_cli_format_rust_with_config() {
    let env = TestEnv::new("./tests/fixtures/cli_format_rust/with_config/");

    env.foro(&["format", "./main.rs"]);
    env.assert_eq("main.rs", "expected.rs");
}

#[test]
fn test_cli_format_rust_overwrite_config() {
    // rustfmt overwrites the config file.
    // In other words, it uses the rustfmt.toml file in the closest ancestor directory and
    // ignores rustfmt.toml files further away.

    let env = TestEnv::new("./tests/fixtures/cli_format_rust/overwrite_config/");

    env.foro(&["format", "./src/main.rs"]);
    env.assert_eq("src/main.rs", "src/expected.rs");
}

#[test]
fn test_cli_format_rust_nested_config() {
    let env = TestEnv::new("./tests/fixtures/cli_format_rust/nested_config/");

    env.foro(&["format", "./nest/src/main.rs"]);
    env.assert_eq("nest/src/main.rs", "nest/src/expected.rs");
}

#[test]
fn test_cli_format_cpp() {
    let env = TestEnv::new("./tests/fixtures/cli_format_cpp/basic/");

    env.foro(&["format", "./main.cpp"]);
    env.assert_eq("main.cpp", "expected.cpp");
}

#[test]
fn test_cli_format_go() {
    let env = TestEnv::new("./tests/fixtures/cli_format_go/basic/");

    env.foro(&["format", "./main.go"]);
    env.assert_eq("main.go", "expected.go");
}

#[test]
fn test_cli_format_ts_basic() {
    let env = TestEnv::new("./tests/fixtures/cli_format_ts/basic/");

    env.foro(&["format", "./main.ts"]);
    env.assert_eq("main.ts", "expected.ts");
}

#[test]
fn test_cli_format_ts_ignore() {
    let env = TestEnv::new("./tests/fixtures/cli_format_ts/ignore/");

    env.foro(&["format", "./main.ts"]);
    env.assert_eq("main.ts", "expected.ts");
}

#[test]
fn test_cli_format_ts_with_config() {
    let env = TestEnv::new("./tests/fixtures/cli_format_ts/with_config/");

    env.foro(&["format", "./main.ts"]);
    env.assert_eq("main.ts", "expected.ts");
}

#[test]
fn test_cli_format_ts_nested_config() {
    // biome does not read biome.json located deeper than the current directory.
    // biome plugin for foro follows this specification.

    let env = TestEnv::new("./tests/fixtures/cli_format_ts/nested_config/");

    env.foro(&["format", "./nest/src/main.ts"]);
    env.assert_eq("nest/src/main.ts", "nest/src/expected.ts");
}

#[test]
fn test_cli_format_ts_overwrite_config() {
    let env = TestEnvBuilder::new("./tests/fixtures/cli_format_ts/overwrite_config/")
        .work_dir("./root/")
        .build();

    env.foro(&["format", "./main.ts"]);
    env.assert_eq("root/main.ts", "root/expected.ts");
}

#[test]
fn test_cli_format_ts_extend_config() {
    let env = TestEnvBuilder::new("./tests/fixtures/cli_format_ts/extend_config/")
        .work_dir("./root/")
        .build();

    env.foro(&["format", "./main.ts"]);
    env.assert_eq("root/main.ts", "root/expected.ts");
}

#[test]
fn test_cli_format_python() {
    let env = TestEnv::new("./tests/fixtures/cli_format_python/basic/");

    env.foro(&["format", "./main.py"]);
    env.assert_eq("main.py", "expected.py");
}

#[test]
fn test_cli_format_rules() {
    let env = TestEnv::new("./tests/fixtures/cli_format_rules/");

    let output = env.foro_cmd(&["format", "./main.rs"]).unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("File ignored. reason: No rule matched"))
}

#[test]
#[ignore]
fn test_test_cli_format_no_cache() {
    // This test is ignored because it's really slow! (~30s)

    let env = TestEnvBuilder::new("./tests/fixtures/cli_format_rust/basic/")
        .cache_dir("./cache/")
        .build();

    env.foro(&["format", "./main.rs"]);
    env.assert_eq("main.rs", "expected.rs");
}
