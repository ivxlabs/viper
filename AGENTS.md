# AGENTS.md

Instructions for AI coding agents working on Viper. Humans: see [README.md](README.md).

Viper is an analysis engine for Difflicit: a program Difflicit runs with one JSON request on stdin, answering with
an analysis in diffmd on stdout. The protocol is https://ivx.run/docs/difflicit/engines/; keep to it exactly.
HyperViper, the full version, is a private fork of this repository and merges from it, so keep changes here
general: anything HyperViper-only belongs there.

- **Keep it simple, don't repeat yourself.** Small, direct code; one source of truth for each fact. The engine's
  name, id and version live in `engine.json` (the version also in `Cargo.toml`; a test checks they agree).
- **Don't add dependencies** when a few lines will do.
- **License headers.** Every source file starts with `SPDX-License-Identifier: GPL-3.0-only` in a comment.
- **The output is for people.** Lead with what matters, and put the lines each point rests on in `diff` blocks.

Before you finish: `cargo clippy --all-targets && cargo test`, and `scripts/package.sh` if you touched packaging.
