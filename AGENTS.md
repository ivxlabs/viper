# AGENTS.md

Instructions for AI coding agents working on Viper. Humans: see [README.md](README.md).

Viper is an analysis engine for Difflicit: a command-line tool that writes up the changes between two versions as
diffmd. People run it on its own (`viper before after`); Difflicit runs it as `viper --difflicit` with one JSON
request on stdin. The protocol is https://ivx.run/docs/difflicit/engines/; keep to it exactly. Both ways must give
the same analysis: the command line builds the same request Difflicit sends (`src/compare.rs`).

HyperViper, the full version, is a private fork of this repository and merges from it, so keep changes here
general, and keep the code free of the name "viper": the library is `engine`, and the command's name comes from
`Cargo.toml`. Only the identity files differ between the two (`Cargo.toml`'s package, `about.json`, `LICENSE`,
`README.md`, this file, the license header, and the Homebrew packaging).

- **Keep it simple, don't repeat yourself.** The engine's name, version and license live in `about.json` (the
  version also in `Cargo.toml`; a test checks they agree).
- **Don't add dependencies** when a few lines will do.
- **License headers.** Every source file starts with `SPDX-License-Identifier: MIT` in a comment. Viper is MIT on
  purpose, so anyone can build an engine on it, proprietary ones included: don't add GPL code or dependencies.
- **The output is for people.** Lead with what matters, and put the lines each point rests on in `diff` blocks.

Before you finish: `cargo clippy --all-targets -- -D warnings && cargo test`.
