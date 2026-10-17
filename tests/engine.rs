// SPDX-License-Identifier: GPL-3.0-only
//! The engine as Difflicit runs it: a request on stdin, diffmd on stdout, problems on stderr with a failing exit.

use std::io::Write;
use std::process::{Command, Stdio};

fn run(input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_viper"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn answers_a_request() {
    let req = r#"{"protocol":1,"project":"/p","before":{"artifact":"x","kind":"git","version":"v1","repo":"/p/x"},
        "after":{"artifact":"x","kind":"git","version":"v2","repo":"/p/x"},
        "diff":{"kind":"unified","text":"diff --git a/f b/f\n--- a/f\n+++ b/f\n@@ -1 +1 @@\n-1\n+2\n"}}"#;
    let out = run(req);
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().starts_with("# x: v1 → v2\n"));
}

#[test]
fn explains_a_bad_request() {
    let out = run("not json");
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains("isn't valid JSON"));
}
