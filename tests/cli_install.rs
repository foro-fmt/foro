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

/// config 変更後は再 install が必要か
#[test]
fn test_cli_install_config_change_invalidates_marker() {
    let env = TestEnvBuilder::new()
        .fixture_path("./tests/fixtures/cli_install/")
        .cache_dir("cache")
        .build_without_install();

    env.foro(&["install"]);

    // install 後は format が通る
    let out = env.foro_cmd(&["format", "main.rs"]).output().unwrap();
    assert!(out.status.success());

    // config を別の内容に書き換える
    // 内容は同じだが bytes が違う（スペース追加）→ ハッシュが変わりマーカー無効化
    std::fs::write(env.config_file.path(), b"{ \"rules\": [] }\n").unwrap();

    // format が再びエラーになるか
    let out = env.foro_cmd(&["format", "main.rs"]).output().unwrap();
    assert!(
        !out.status.success(),
        "format should fail after config change"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("foro install"),
        "error should mention `foro install`, got: {stderr}"
    );
}
