// SPDX-License-Identifier: GPL-3.0-only
//! Viper, an analysis engine for Difflicit: it writes up the visual changes between two artifacts — what changed,
//! and where, file by file and function by function — as diffmd. Difflicit runs it with one JSON request on stdin
//! and reads the analysis from stdout (protocol: https://ivx.run/docs/difflicit/engines/).

pub mod diff;

use serde_json::Value;

/// The engine's own description; its name and version head every analysis.
const MANIFEST: &str = include_str!("../engine.json");

/// The request format this engine speaks.
pub const PROTOCOL: u64 = 1;

/// How much of a diff an analysis shows: files with a section of their own, and lines in each.
const MAX_FILES: usize = 50;
const MAX_LINES: usize = 300;
/// Functions listed per table for binaries.
const MAX_FUNCTIONS: usize = 100;

/// This engine's name and version, from engine.json.
pub fn identity() -> (String, String) {
    let m: Value = serde_json::from_str(MANIFEST).expect("engine.json is valid JSON");
    let field = |k: &str| m.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    (field("name"), field("version"))
}

/// The analysis of a request, as diffmd.
pub fn analyze(req: &Value) -> Result<String, String> {
    let protocol = req.get("protocol").and_then(Value::as_u64).unwrap_or(0);
    if protocol != PROTOCOL {
        return Err(format!("Difflicit sent protocol {protocol}; this engine speaks {PROTOCOL}. Update the engine or Difflicit."));
    }
    let side = |k: &str| {
        let s = req.get(k).ok_or(format!("The request has no {k:?}."))?;
        let field = |f: &str| s.get(f).and_then(Value::as_str).unwrap_or_default().to_string();
        Ok::<_, String>((field("artifact"), field("version")))
    };
    let ((before_artifact, before), (after_artifact, after)) = (side("before")?, side("after")?);
    let title = if before_artifact == after_artifact {
        format!("{after_artifact}: {before} → {after}")
    } else {
        format!("{before_artifact} {before} → {after_artifact} {after}")
    };

    let (name, version) = identity();
    let mut out = vec![
        format!("# {title}"),
        format!(
            "> Written by {name} {version} from the diff of the two: what changed, and where. Check it against the diff, and make these notes your own."
        ),
    ];
    let diff = req.get("diff").ok_or("The request has no \"diff\".")?;
    match diff.get("kind").and_then(Value::as_str) {
        Some("unified") => unified(diff.get("text").and_then(Value::as_str).unwrap_or_default(), &mut out),
        Some("binary") => binary(diff.get("functions").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default(), &mut out),
        other => return Err(format!("Don't know how to read a {other:?} diff.")),
    }
    Ok(out.join("\n\n") + "\n")
}

/// Text in a table cell or inline code: no pipes or backticks to break out of it.
fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('`', "'")
}

/// A fenced block that can't be closed early by what's in it: one backtick more than its longest run.
fn fence(lang: &str, body: &str) -> String {
    let longest = body.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let ticks = "`".repeat(longest.max(2) + 1);
    format!("{ticks}{lang}\n{body}\n{ticks}")
}

fn plural(n: usize, what: &str) -> String {
    format!("{n} {what}{}", if n == 1 { "" } else { "s" })
}

/// Text: an overview of the files, then each file's hunks as a patch.
fn unified(text: &str, out: &mut Vec<String>) {
    let files = diff::parse(text);
    if files.is_empty() {
        out.push("The two are identical: there's nothing to show.".into());
        return;
    }
    let (added, removed) = files.iter().fold((0, 0), |(a, r), f| (a + f.added, r + f.removed));
    out.push(format!("**{} changed** · +{added} −{removed}", plural(files.len(), "file")));

    let mut table = vec!["| File | Change | + | − |".to_string(), "| --- | --- | --: | --: |".to_string()];
    table.extend(files.iter().map(|f| format!("| `{}` | {} | {} | {} |", cell(f.path()), f.status(), f.added, f.removed)));
    out.push("## Overview".into());
    out.push(table.join("\n"));

    for f in files.iter().take(MAX_FILES) {
        out.push(format!("## `{}`", cell(f.path())));
        if f.status() == "renamed" {
            out.push(format!("Renamed from `{}`.", cell(&f.old)));
        }
        if f.binary {
            out.push("A binary file: it changed, but has no lines to show.".into());
            continue;
        }
        let mut body = vec![format!("--- a/{}", f.old), format!("+++ b/{}", f.new)];
        let mut shown = 0;
        let total: usize = f.hunks.iter().map(|h| h.lines.len()).sum();
        for h in &f.hunks {
            if shown >= MAX_LINES {
                break;
            }
            let take = h.lines.len().min(MAX_LINES - shown);
            // A hunk cut short is the block's last, so readers that go by its header's counts just stop early.
            body.push(h.header.clone());
            body.extend(h.lines[..take].iter().cloned());
            shown += take;
        }
        out.push(fence("diff", &body.join("\n")));
        if shown < total {
            out.push(format!("…and {} more in the diff.", plural(total - shown, "line")));
        }
    }
    if files.len() > MAX_FILES {
        out.push(format!("…and {} more, listed in the overview.", plural(files.len() - MAX_FILES, "file")));
    }
}

/// Binaries: radare2's comparison of their functions, as tables of what changed, appeared and went.
fn binary(functions: &[Value], out: &mut Vec<String>) {
    let field = |f: &Value, k: &str| f.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    let with = |status: &str| functions.iter().filter(|f| field(f, "status") == status).collect::<Vec<_>>();
    let (mut changed, added, removed, same) = (with("changed"), with("added"), with("removed"), with("same"));
    if functions.is_empty() {
        out.push("radare2 found no functions to compare in these binaries.".into());
        return;
    }
    out.push(format!(
        "**{} changed, {} added, {} removed** · {} unchanged",
        plural(changed.len(), "function"),
        added.len(),
        removed.len(),
        same.len()
    ));
    // Least alike first: the biggest changes.
    let similarity = |f: &Value| f.get("similarity").and_then(Value::as_f64).unwrap_or(1.0);
    changed.sort_by(|a, b| similarity(a).total_cmp(&similarity(b)));

    let section = |title: &str, rows: &[&Value], head: &str, row: &dyn Fn(&Value) -> String, out: &mut Vec<String>| {
        if rows.is_empty() {
            return;
        }
        let mut table = vec![head.to_string(), format!("|{}", " --- |".repeat(head.matches('|').count() - 1))];
        table.extend(rows.iter().take(MAX_FUNCTIONS).map(|f| row(f)));
        out.push(format!("## {title}"));
        out.push(table.join("\n"));
        if rows.len() > MAX_FUNCTIONS {
            out.push(format!("…and {} more.", plural(rows.len() - MAX_FUNCTIONS, "function")));
        }
    };
    section(
        "Changed functions",
        &changed,
        "| Function | Alike | Before | After |",
        &|f| {
            format!(
                "| `{}` | {:.0}% | `{}` | `{}` |",
                cell(&field(f, "name")),
                similarity(f) * 100.0,
                field(f, "base_addr"),
                field(f, "head_addr")
            )
        },
        out,
    );
    section("Added functions", &added, "| Function | At |", &|f| format!("| `{}` | `{}` |", cell(&field(f, "name")), field(f, "head_addr")), out);
    section("Removed functions", &removed, "| Function | Was at |", &|f| format!("| `{}` | `{}` |", cell(&field(f, "name")), field(f, "base_addr")), out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(diff: Value) -> Value {
        json!({
            "protocol": 1,
            "project": "/p",
            "before": { "artifact": "lib", "kind": "file", "version": "1.0", "path": "/p/a" },
            "after": { "artifact": "lib", "kind": "file", "version": "1.1", "path": "/p/b" },
            "diff": diff,
        })
    }

    #[test]
    fn writes_up_text() {
        let text = "diff --git a/k.c b/k.c\n--- a/k.c\n+++ b/k.c\n@@ -1,2 +1,2 @@\n int a;\n-int b;\n+long b;\n";
        let md = analyze(&request(json!({ "kind": "unified", "text": text }))).unwrap();
        assert!(md.starts_with("# lib: 1.0 → 1.1\n"));
        assert!(md.contains("**1 file changed** · +1 −1"));
        assert!(md.contains("| `k.c` | modified | 1 | 1 |"));
        assert!(md.contains("```diff\n--- a/k.c\n+++ b/k.c\n@@ -1,2 +1,2 @@\n int a;\n-int b;\n+long b;\n```"));
    }

    #[test]
    fn writes_up_binaries() {
        let functions = json!([
            { "name": "sym.check", "status": "changed", "similarity": 0.9, "base_addr": "0x10", "head_addr": "0x20" },
            { "name": "sym.parse", "status": "changed", "similarity": 0.4, "base_addr": "0x30", "head_addr": "0x40" },
            { "name": "sym.new", "status": "added", "similarity": 0.0, "head_addr": "0x50" },
            { "name": "sym.same", "status": "same", "similarity": 1.0 },
        ]);
        let md = analyze(&request(json!({ "kind": "binary", "functions": functions }))).unwrap();
        assert!(md.contains("**2 functions changed, 1 added, 0 removed** · 1 unchanged"));
        // Least alike first.
        assert!(md.find("sym.parse").unwrap() < md.find("sym.check").unwrap());
        assert!(md.contains("| `sym.parse` | 40% | `0x30` | `0x40` |"));
        assert!(md.contains("## Added functions"));
        assert!(!md.contains("## Removed functions"));
    }

    #[test]
    fn fences_hold() {
        assert_eq!(fence("diff", "+```"), "````diff\n+```\n````");
    }

    #[test]
    fn refuses_other_protocols() {
        let mut req = request(json!({ "kind": "unified", "text": "" }));
        req["protocol"] = json!(2);
        assert!(analyze(&req).is_err());
    }

    #[test]
    fn manifest_matches_the_crate() {
        // engine.json describes this crate: same id, same version, and the binary scripts/package.sh puts in bin/.
        let m: Value = serde_json::from_str(MANIFEST).unwrap();
        let (name, version) = identity();
        assert!(!name.is_empty());
        assert_eq!(m["id"], env!("CARGO_PKG_NAME"), "engine.json's id and Cargo.toml's name differ");
        assert_eq!(version, env!("CARGO_PKG_VERSION"), "engine.json and Cargo.toml disagree on the version");
        assert_eq!(m["command"], serde_json::json!([format!("bin/{}", env!("CARGO_PKG_NAME"))]));
    }
}
