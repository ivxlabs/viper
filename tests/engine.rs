// SPDX-License-Identifier: MIT
//! The engine's command as people and Difflicit run it.

use std::io::Write;
use std::process::{Command, Stdio};

fn viper(args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME"))))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

#[test]
fn answers_difflicit() {
    let req = r#"{"protocol":1,"project":"/p","before":{"artifact":"x","kind":"git","version":"v1","repo":"/p/x"},
        "after":{"artifact":"x","kind":"git","version":"v2","repo":"/p/x"},
        "diff":{"kind":"unified","text":"diff --git a/f b/f\n--- a/f\n+++ b/f\n@@ -1 +1 @@\n-1\n+2\n"}}"#;
    let out = viper(&["--difflicit"], req);
    assert!(out.status.success());
    assert!(stdout(&out).starts_with("# x: v1 → v2\n"));
}

#[test]
fn says_what_it_is() {
    let out = viper(&["--about"], "");
    let about: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(about["protocol"].as_u64(), Some(1));
    assert!(about["name"].as_str().is_some_and(|n| !n.is_empty()));
}

#[test]
fn reads_a_patch_from_stdin() {
    let out = viper(&["--patch", "-"], "--- a/k.c\n+++ b/k.c\n@@ -1 +1 @@\n-1\n+2\n");
    assert!(out.status.success());
    assert!(stdout(&out).contains("| `k.c` | modified | 1 | 1 |"));
}

#[test]
fn compares_two_files_into_a_note() {
    let dir = std::env::temp_dir().join(format!("viper-cli-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("old")).unwrap();
    std::fs::create_dir_all(dir.join("new")).unwrap();
    std::fs::write(dir.join("old/check.c"), "return n < LIMIT;\n").unwrap();
    std::fs::write(dir.join("new/check.c"), "return n <= LIMIT;\n").unwrap();
    let note = dir.join("note.diffmd");
    let (old, new, note_s) = (dir.join("old/check.c"), dir.join("new/check.c"), note.to_string_lossy().into_owned());
    let out = viper(&[&old.to_string_lossy(), &new.to_string_lossy(), "-o", &note_s], "");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let md = std::fs::read_to_string(&note).unwrap();
    assert!(md.starts_with("# check.c: old → new\n"), "{md}");
    assert!(md.contains("-return n < LIMIT;\n+return n <= LIMIT;"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn explains_mistakes() {
    let out = viper(&["--difflicit"], "not json");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("isn't valid JSON"));
    let out = viper(&["--frobnicate"], "");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage:"));
}
