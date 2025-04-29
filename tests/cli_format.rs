mod common;

use crate::common::{TestEnv, TestEnvBuilder};
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::path::eq_file;

#[test]
fn test_cli_format_rust() {
    let env = TestEnv::new("./tests/fixtures/cli_format_rust/");

    env.foro(&["format", "./main.rs"]);
    env.child("main.rs")
        .assert(eq_file(env.path("expected.rs")));
}

#[test]
fn test_cli_format_cpp() {
    let env = TestEnv::new("./tests/fixtures/cli_format_cpp/");

    env.foro(&["format", "./main.cpp"]);
    env.child("main.cpp")
        .assert(eq_file(env.path("expected.cpp")));
}

#[test]
fn test_cli_format_go() {
    let env = TestEnv::new("./tests/fixtures/cli_format_go/");

    env.foro(&["format", "./main.go"]);
    env.child("main.go")
        .assert(eq_file(env.path("expected.go")));
}

#[test]
fn test_cli_format_ts() {
    let env = TestEnv::new("./tests/fixtures/cli_format_ts/");

    env.foro(&["format", "./main.tsx"]);
    env.child("main.tsx")
        .assert(eq_file(env.path("expected.tsx")));
}

#[test]
fn test_cli_format_python() {
    let env = TestEnv::new("./tests/fixtures/cli_format_python/");

    env.foro(&["format", "./main.py"]);
    env.child("main.py")
        .assert(eq_file(env.path("expected.py")));
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

    let env = TestEnvBuilder::new("./tests/fixtures/cli_format_rust/")
        .cache_dir("./cache/")
        .build();

    env.foro(&["format", "./main.rs"]);
    env.child("main.rs")
        .assert(eq_file(env.path("expected.rs")));
}
