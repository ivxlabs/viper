// SPDX-License-Identifier: GPL-3.0-only
//! Reading the unified diff Difflicit sends (`git diff`, `git diff --no-index`) into files and hunks.

#[derive(Debug, Default, PartialEq)]
pub struct File {
    /// The path before and after (the same unless renamed; `/dev/null` when added or deleted).
    pub old: String,
    pub new: String,
    pub hunks: Vec<Hunk>,
    pub added: usize,
    pub removed: usize,
    /// "Binary files … differ": changed, but with no lines to show.
    pub binary: bool,
}

#[derive(Debug, Default, PartialEq)]
pub struct Hunk {
    /// The `@@ -a,b +c,d @@ …` line.
    pub header: String,
    /// Its lines, each starting with ' ', '+', '-' or '\'.
    pub lines: Vec<String>,
}

impl File {
    /// The path to show: the new one, or the old one when it was deleted.
    pub fn path(&self) -> &str {
        if self.new == "/dev/null" { &self.old } else { &self.new }
    }

    pub fn status(&self) -> &'static str {
        match (self.old.as_str(), self.new.as_str()) {
            ("/dev/null", _) => "added",
            (_, "/dev/null") => "deleted",
            (a, b) if a != b => "renamed",
            _ => "modified",
        }
    }
}

/// `a/x` → `x`, and anything after a tab (timestamps from `diff -u`) dropped.
fn clean(path: &str) -> String {
    let path = path.split('\t').next().unwrap_or(path);
    path.strip_prefix("a/").or_else(|| path.strip_prefix("b/")).unwrap_or(path).to_string()
}

/// The files of a unified diff, in order. Lenient: lines it doesn't understand are skipped.
pub fn parse(text: &str) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            // "a/old b/new": split at " b/", which holds for paths without that sequence in them.
            let (old, new) = rest.split_once(" b/").map(|(a, b)| (clean(a), b.to_string())).unwrap_or_default();
            files.push(File { old, new, ..File::default() });
        } else if let Some(path) = line.strip_prefix("--- ") {
            // A file with no `diff --git` line (plain `diff -u`), or its first hunk yet to come.
            if files.last().is_none_or(|f| !f.hunks.is_empty()) {
                files.push(File::default());
            }
            files.last_mut().unwrap().old = clean(path);
        } else if let Some(path) = line.strip_prefix("+++ ") {
            if let Some(f) = files.last_mut() {
                f.new = clean(path);
            }
        } else if line.starts_with("@@") {
            if let Some(f) = files.last_mut() {
                f.hunks.push(Hunk { header: line.to_string(), lines: vec![] });
            }
        } else if line.starts_with("Binary files ") && line.ends_with(" differ") {
            if let Some(f) = files.last_mut() {
                f.binary = true;
            }
        } else if let Some(f) = files.last_mut()
            && let Some(h) = f.hunks.last_mut()
            && matches!(line.chars().next(), Some(' ' | '+' | '-' | '\\') | None)
        {
            match line.chars().next() {
                Some('+') => f.added += 1,
                Some('-') => f.removed += 1,
                _ => {}
            }
            h.lines.push(if line.is_empty() { " ".into() } else { line.to_string() });
        }
    }
    files.retain(|f| !f.old.is_empty() || !f.new.is_empty());
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_git_diffs() {
        let text = "diff --git a/k.c b/k.c\nindex 1..2 100644\n--- a/k.c\n+++ b/k.c\n@@ -1,3 +1,3 @@ int f()\n int a;\n-int b;\n+long b;\n int c;\n\
                    diff --git a/new.h b/new.h\nnew file mode 100644\n--- /dev/null\n+++ b/new.h\n@@ -0,0 +1 @@\n+#pragma once\n\
                    diff --git a/blob b/blob\nBinary files a/blob and b/blob differ\n";
        let files = parse(text);
        assert_eq!(files.len(), 3);
        assert_eq!((files[0].path(), files[0].status(), files[0].added, files[0].removed), ("k.c", "modified", 1, 1));
        assert_eq!(files[0].hunks[0].lines, vec![" int a;", "-int b;", "+long b;", " int c;"]);
        assert_eq!((files[1].path(), files[1].status()), ("new.h", "added"));
        assert!(files[2].binary);
    }

    #[test]
    fn reads_plain_diffs() {
        let files = parse("--- a/x\t2024-01-01\n+++ b/x\t2024-01-02\n@@ -1 +1 @@\n-1\n+2\n");
        assert_eq!((files.len(), files[0].path(), files[0].added), (1, "x", 1));
    }
}
