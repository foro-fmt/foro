use std::io::Write;
mod common;

use std::io::stdout;
use crate::common::{TestEnv, TestEnvBuilder};
use assert_cmd::prelude::*;
use regex::Regex;
use serial_test::serial;

#[test]
fn test_cli_daemon() {
    let env = TestEnv::new();

    let res = env.foro_stderr(&["daemon", "start"]);
    assert!(res.contains("Daemon started"));
    
    let res = env.foro_stdout(&["daemon", "ping"]);
    let pid_0 = Regex::new(r"daemon pid: (\d+)").unwrap()
        .captures(&res)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();

    let res = env.foro_stderr(&["daemon", "stop"]);
    assert!(res.contains("Daemon stopped"));

    // `foro restart` when daemon is stopped should start normally
    let res = env.foro_stderr(&["daemon", "restart"]);
    assert!(res.contains("Daemon is not running"));
    assert!(res.contains("Daemon started"));

    let res = env.foro_stdout(&["daemon", "ping"]);
    let pid_1 = Regex::new(r"daemon pid: (\d+)").unwrap()
        .captures(&res)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();
    
    assert_ne!(pid_0, pid_1);

    let res = env.foro_stderr(&["daemon", "restart"]);
    assert!(res.contains("Daemon stopped"));
    assert!(res.contains("Daemon started"));
    
    let res = env.foro_stdout(&["daemon", "ping"]);
    let pid_2 = Regex::new(r"daemon pid: (\d+)").unwrap()
        .captures(&res)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();
    
    assert_ne!(pid_1, pid_2);
}

#[test]
fn test_cli_daemon_lock() {
    let env = TestEnv::new();
    
    let mut proc_0 = env.foro_cmd(&["daemon", "start"]).spawn().unwrap();
    let mut proc_1 = env.foro_cmd(&["daemon", "start"]).spawn().unwrap();

    assert!(proc_0.wait().unwrap().success());
    assert!(proc_1.wait().unwrap().success());
}
