mod common;

use crate::common::TestEnvBuilder;

/// install なし → format がエラー、install 後 → format が通る
#[test]
fn test_cli_install_then_format_works() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_install/")
        .cache_dir("cache")
        .build_without_install();

    // install 前: format がエラーになり "foro install" を案内するか
    let out = env.foro_cmd(&["format", "main.rs"]).output().unwrap();
    assert!(
        !out.status.success(),
        "format should fail before install, but succeeded"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("foro install"),
        "error message should mention `foro install`, got: {stderr}"
    );

    // install 実行（空の rules なので即完了）
    env.foro(&["install"]);

    // install 後: format が通るか（rules が空なので "no rule matched" だが成功扱い）
    let out = env.foro_cmd(&["format", "main.rs"]).output().unwrap();
    assert!(
        out.status.success(),
        "format should succeed after install, stderr: {}",
        String::from_utf8(out.stderr).unwrap()
    );
}

/// install は冪等か
#[test]
fn test_cli_install_idempotent() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_install/")
        .cache_dir("cache")
        .build_without_install();

    env.foro(&["install"]);
    env.foro(&["install"]); // 2回目もエラーにならない
}

/// 空白だけの config 変更では再 install は不要か
#[test]
fn test_cli_install_whitespace_change_keeps_marker_valid() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_install/")
        .cache_dir("cache")
        .build_without_install();

    env.foro(&["install"]);

    // install 後は format が通る
    let out = env.foro_cmd(&["format", "main.rs"]).output().unwrap();
    assert!(out.status.success());

    // 空白だけ変更する。依存 URL 集合は同じなので、再 install は不要なはず。
    std::fs::write(env.config_file.path(), b"{ \"rules\": [] }\n").unwrap();

    // 依存関係が変わっていないので、そのまま format できるはず。
    let out = env.foro_cmd(&["format", "main.rs"]).output().unwrap();
    assert!(
        out.status.success(),
        "format should still succeed after whitespace-only config change"
    );
}
