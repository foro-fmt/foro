mod common;

use crate::common::{uv_available, TestEnvBuilder};

#[test]
fn test_cli_bulk_format_basic() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/basic/")
        .work_dir("./input/")
        .build();

    env.foro(&["format", "."]);
    env.assert_eq("input/a.rs", "expected/a.rs");
    env.assert_eq("input/b.rs", "expected/b.rs");
}

#[test]
fn test_cli_bulk_format_single_thread() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/basic/")
        .work_dir("./input/")
        .build();

    env.foro(&["format", ".", "--threads", "1"]);
    env.assert_eq("input/a.rs", "expected/a.rs");
    env.assert_eq("input/b.rs", "expected/b.rs");
}

#[test]
fn test_cli_bulk_format_subdirectory() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/subdirectory/")
        .work_dir("./input/")
        .build();

    env.foro(&["format", "."]);
    env.assert_eq("input/top.rs", "expected/top.rs");
    env.assert_eq("input/sub/nested.rs", "expected/sub/nested.rs");
}

#[test]
fn test_cli_bulk_format_foro_ignore() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/foro_ignore/")
        .work_dir("./input/")
        .build();

    env.foro(&["format", "."]);
    env.assert_eq("input/formatted.rs", "expected/formatted.rs");
    // ignored.rs should remain unformatted because it's listed in .foro-ignore
    env.assert_eq("input/ignored.rs", "expected/ignored.rs");
}

#[test]
fn test_cli_bulk_format_default_ignore() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/default_ignore/")
        .work_dir("./input/")
        .build();

    env.foro(&["format", "."]);
    env.assert_eq("input/main.rs", "expected/main.rs");
    // node_modules/dep.rs should remain unformatted due to default ignore rules
    env.assert_eq("input/node_modules/dep.rs", "expected/node_modules/dep.rs");
}

#[test]
fn test_cli_bulk_format_multiple_paths() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/multiple_paths/")
        .work_dir("./input/")
        .build();

    env.foro(&["format", "./dir1", "./dir2"]);
    env.assert_eq("input/dir1/a.rs", "expected/dir1/a.rs");
    env.assert_eq("input/dir2/b.rs", "expected/dir2/b.rs");
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "CommandIO is unsupported on Windows")]
fn test_cli_bulk_format_no_rule_match() {
    if !uv_available() {
        eprintln!("skipping test_cli_bulk_format_no_rule_match: uv is not available");
        return;
    }

    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/no_rule_match/")
        .work_dir("./input/")
        .build();

    let mut cmd = env.foro_cmd(&["format", "."]);
    let output = std::process::Command::output(&mut cmd).unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("2 files processed."));
    assert!(stderr.contains("1 changed"));
    assert!(stderr.contains("0 unchanged"));
    assert!(stderr.contains("1 ignored"));
    assert!(stderr.contains("0 errors"));

    env.assert_eq("input/main.rs", "expected/main.rs");
    // .txt has no matching rule, so it should be left untouched
    env.assert_eq("input/readme.txt", "expected/readme.txt");
}

#[test]
fn test_cli_bulk_format_foro_ignore_glob() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/foro_ignore_glob/")
        .work_dir("./input/")
        .build();

    env.foro(&["format", "."]);
    env.assert_eq("input/main.rs", "expected/main.rs");
    // *.generated.rs should be excluded by .foro-ignore glob pattern
    env.assert_eq("input/types.generated.rs", "expected/types.generated.rs");
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "CommandIO is unsupported on Windows")]
fn test_cli_bulk_format_error_count() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/error_count/")
        .work_dir("./input/")
        .build();

    let mut cmd = env.foro_cmd(&["format", "."]);
    let output = std::process::Command::output(&mut cmd).unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Formatted with errors:"));
    assert!(stderr.contains("1 files processed."));
    assert!(stderr.contains("0 changed"));
    assert!(stderr.contains("0 unchanged"));
    assert!(stderr.contains("0 ignored"));
    assert!(stderr.contains("1 error"));

    // File should stay untouched because formatter execution failed.
    env.assert_eq("input/main.txt", "expected/main.txt");
}
