// SPDX-License-Identifier: MIT
//! Viper on its own, outside Difflicit: comparing two files or folders (or reading a patch) into the same request
//! Difflicit would send, so both ways of using it give the same analysis.

use std::io::Read;
use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};

/// The request for comparing `before` and `after` (files or folders): named after them, with their diff.
pub fn paths(before: &Path, after: &Path) -> Result<Value, String> {
    for p in [before, after] {
        if !p.exists() {
            return Err(format!("{} doesn't exist.", p.display()));
        }
    }
    let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| p.display().to_string());
    let folder = |p: &Path| p.parent().and_then(Path::file_name).map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    // Two copies of one thing (old/libfoo, new/libfoo) are its versions; two different things are just themselves.
    let (before_side, after_side) = if name(before) == name(after) {
        ((name(before), folder(before)), (name(after), folder(after)))
    } else {
        ((name(before), String::new()), (name(after), String::new()))
    };
    let diff = if is_binary(before) && is_binary(after) {
        match radiff(before, after) {
            Some(functions) => json!({ "kind": "binary", "functions": functions? }),
            None => json!({ "kind": "unified", "text": no_index(before, after, &name(after))? }),
        }
    } else {
        json!({ "kind": "unified", "text": no_index(before, after, &name(after))? })
    };
    let side = |(artifact, version): (String, String), p: &Path| json!({ "artifact": artifact, "kind": "file", "version": version, "path": p.display().to_string() });
    Ok(json!({ "protocol": crate::PROTOCOL, "before": side(before_side, before), "after": side(after_side, after), "diff": diff }))
}

/// The request for a unified diff already made (`git diff v1 v2 | viper --patch -`): `source` is a file, or `-`.
pub fn patch(source: &str) -> Result<Value, String> {
    let (text, name) = if source == "-" {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).map_err(|e| format!("Couldn't read the patch: {e}"))?;
        (text, "The patch".to_string())
    } else {
        let p = Path::new(source);
        let text = std::fs::read_to_string(p).map_err(|e| format!("Couldn't read {source}: {e}"))?;
        (text, p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| source.to_string()))
    };
    let side = json!({ "artifact": name, "kind": "patch", "version": "" });
    Ok(json!({ "protocol": crate::PROTOCOL, "before": side, "after": side, "diff": { "kind": "unified", "text": text } }))
}

/// Whether a file is binary: a NUL byte in its first 8 KiB. Folders aren't.
fn is_binary(path: &Path) -> bool {
    let mut head = [0u8; 8192];
    std::fs::File::open(path).and_then(|mut f| f.read(&mut head)).is_ok_and(|n| path.is_file() && head[..n].contains(&0))
}

/// `git diff --no-index` of two files or folders, with paths as people read them. git names files by the path it
/// was given, so every file of two versions would look renamed; instead a folder's files are named inside it
/// (`a/src/x.c` → `b/src/x.c`), and two files by `name`.
fn no_index(a: &Path, b: &Path, name: &str) -> Result<String, String> {
    let (sa, sb) = (a.to_string_lossy(), b.to_string_lossy());
    let out = Command::new("git")
        .args(["diff", "--no-color", "--no-index", "--", &sa, &sb])
        .output()
        .map_err(|e| format!("Couldn't run git ({e}). Install Git: Viper uses it to compare files."))?;
    // 1 means "they differ", which is the point.
    if !matches!(out.status.code(), Some(0 | 1)) {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    // As git prints them: relative ones as given, absolute ones without the leading slash, / even on Windows.
    let printed = |s: &str| s.replace('\\', "/").trim_start_matches('/').to_string();
    let (pa, pb) = (printed(&sa), printed(&sb));
    let rename = |line: &str| -> String {
        if a.is_dir() {
            let mut line = line.to_string();
            // An added or deleted file is named under the other folder on both sides.
            for p in [&pa, &pb] {
                line = line.replace(&format!("a/{p}/"), "a/").replace(&format!("b/{p}/"), "b/");
            }
            line
        } else {
            line.replace(&format!("a/{pa}"), &format!("a/{name}")).replace(&format!("b/{pb}"), &format!("b/{name}"))
        }
    };
    let mut in_hunk = false;
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            // Only headers carry paths; in a hunk, "--- x" is a removed line that happens to start that way.
            if line.starts_with("diff --git ") {
                in_hunk = false;
            } else if line.starts_with("@@") {
                in_hunk = true;
            }
            let header = ["diff --git ", "--- ", "+++ "].iter().any(|h| line.starts_with(h));
            if !in_hunk && header { rename(line) } else { line.to_string() }
        })
        .collect();
    Ok(lines.join("\n") + "\n")
}

/// radare2's comparison of two binaries' functions, as Difflicit sends it; None when radare2 isn't installed.
fn radiff(a: &Path, b: &Path) -> Option<Result<Vec<Value>, String>> {
    let out = Command::new("radiff2").args(["-A", "-C", "-j"]).arg(a).arg(b).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    // No functions to compare (a stub library): radiff2 prints nothing at all.
    if text.trim().is_empty() {
        return Some(Ok(vec![]));
    }
    let rows: Vec<Value> = match serde_json::from_str(text.trim()) {
        Ok(rows) => rows,
        Err(e) => return Some(Err(format!("radiff2 gave unexpected output: {e}"))),
    };
    let hex = |v: Option<&Value>| v.and_then(Value::as_u64).map(|a| format!("0x{a:x}"));
    Some(Ok(rows
        .iter()
        .map(|r| {
            let dist = r.get("dist").and_then(Value::as_f64).unwrap_or(1.0);
            let name = r.get("name").and_then(Value::as_str).unwrap_or("");
            let other = r.get("diff_name").and_then(Value::as_str).unwrap_or("");
            let status = match (name.is_empty(), other.is_empty()) {
                (true, _) => "added",
                (_, true) => "removed",
                _ if dist >= 1.0 => "same",
                _ => "changed",
            };
            json!({
                "name": if name.is_empty() { other } else { name },
                "status": status,
                "similarity": dist,
                "base_addr": hex(r.get("addr")),
                "head_addr": hex(r.get("diff_addr")),
            })
        })
        .collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("viper-{tag}-{}-{n}", std::process::id()))
    }

    #[test]
    fn compares_versions_of_a_folder() {
        let dir = scratch("folders");
        let (old, new) = (dir.join("1.0/lib"), dir.join("1.1/lib"));
        std::fs::create_dir_all(old.join("src")).unwrap();
        std::fs::create_dir_all(new.join("src")).unwrap();
        std::fs::write(old.join("src/x.c"), "-- one\n").unwrap();
        std::fs::write(new.join("src/x.c"), "-- two\n").unwrap();
        let req = paths(&old, &new).unwrap();
        assert_eq!((req["before"]["artifact"].as_str(), req["before"]["version"].as_str()), (Some("lib"), Some("1.0")));
        let text = req["diff"]["text"].as_str().unwrap();
        assert!(text.contains("diff --git a/src/x.c b/src/x.c\n"), "{text}");
        assert!(text.contains("--- a/src/x.c\n+++ b/src/x.c\n"), "{text}");
        assert!(text.contains("\n--- one\n+-- two\n"), "{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn compares_two_files() {
        let dir = scratch("files");
        std::fs::create_dir_all(&dir).unwrap();
        let (a, b) = (dir.join("a.c"), dir.join("b.c"));
        std::fs::write(&a, "1\n").unwrap();
        std::fs::write(&b, "2\n").unwrap();
        let req = paths(&a, &b).unwrap();
        assert_eq!(req["before"]["artifact"], "a.c");
        assert!(req["diff"]["text"].as_str().unwrap().contains("--- a/b.c\n+++ b/b.c\n"));
        assert!(paths(&a, &dir.join("missing")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
