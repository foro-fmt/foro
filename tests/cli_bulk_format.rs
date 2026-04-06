mod common;

use crate::common::TestEnvBuilder;

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
fn test_cli_bulk_format_no_rule_match() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_bulk_format/no_rule_match/")
        .work_dir("./input/")
        .build();

    env.foro(&["format", "."]);
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
