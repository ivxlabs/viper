// SPDX-License-Identifier: MIT
//! The engine's command (`viper`): compare two files or folders, read a patch, or answer Difflicit. The analysis goes to
//! stdout (or a file), problems to stderr with a failing exit status.

use std::path::Path;
use std::process::ExitCode;

/// The command's name, as installed.
const NAME: &str = env!("CARGO_PKG_NAME");

fn usage() -> String {
    format!(
        "\
Writes up the changes between two versions of something as diffmd (Markdown with patch diffs).

Usage:
  {NAME} <before> <after> [-o <file>]    compare two files or folders
  {NAME} --patch <file|-> [-o <file>]    analyze a unified diff (- reads stdin): git diff v1 v2 | {NAME} --patch -
  {NAME} --difflicit                     answer Difflicit: a request on stdin, the analysis on stdout
  {NAME} --about                         what this engine is, as JSON
  {NAME} --version | --help

Binaries are compared function by function when radare2 (radiff2) is installed.
More: https://ivx.run/docs/difflicit/engines/"
    )
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::FAILURE
        }
    }
}

fn run(mut args: Vec<String>) -> Result<(), String> {
    // -o <file>: where the analysis goes, instead of stdout.
    let out = match args.iter().position(|a| a == "-o" || a == "--output") {
        Some(i) if i + 1 < args.len() => Some(args.drain(i..=i + 1).nth(1).unwrap()),
        Some(_) => return Err("-o needs a file to write to.".into()),
        None => None,
    };
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let request = match args.as_slice() {
        ["--help" | "-h"] | [] => return print(&usage()),
        ["--version" | "-V"] => return print(&format!("{NAME} {}", env!("CARGO_PKG_VERSION"))),
        ["--about"] => return print(engine::ABOUT.trim()),
        ["--difflicit"] => {
            let mut input = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut input).map_err(|e| format!("Couldn't read the request: {e}"))?;
            serde_json::from_str(&input).map_err(|e| format!("The request isn't valid JSON: {e}"))?
        }
        ["--patch", source] => engine::compare::patch(source)?,
        [before, after] if !before.starts_with('-') && !after.starts_with('-') => engine::compare::paths(Path::new(before), Path::new(after))?,
        _ => return Err(format!("Don't know what to do with {}.\n\n{}", args.join(" "), usage())),
    };
    let diffmd = engine::analyze(&request)?;
    match out {
        Some(file) => std::fs::write(&file, diffmd).map_err(|e| format!("Couldn't write {file}: {e}")),
        None => {
            print!("{diffmd}");
            Ok(())
        }
    }
}

fn print(text: &str) -> Result<(), String> {
    println!("{text}");
    Ok(())
}
