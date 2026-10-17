// SPDX-License-Identifier: MIT
//! Difflicit runs this with one JSON request on stdin; the analysis goes to stdout, problems to stderr.

use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("Couldn't read the request: {e}");
        return ExitCode::FAILURE;
    }
    let result = serde_json::from_str(&input)
        .map_err(|e| format!("The request isn't valid JSON: {e}"))
        .and_then(|req| viper::analyze(&req));
    match result {
        Ok(diffmd) => {
            print!("{diffmd}");
            ExitCode::SUCCESS
        }
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::FAILURE
        }
    }
}
