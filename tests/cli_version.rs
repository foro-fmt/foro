use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("foro").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with(format!(
            "foro {}\n",
            env!("CARGO_PKG_VERSION")
        )))
        .stderr(predicate::str::is_empty());
}
