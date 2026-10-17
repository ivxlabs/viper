# Viper

An analysis engine for [Difflicit](https://github.com/ivxlabs/difflicit): give it two versions of something — files,
folders, binaries — and it writes up what changed between them as [diffmd](https://ivx.run/docs/difflicit/diffmd/),
Markdown with the patches in it.

- **Text and source**: every changed file with its additions and deletions, then each file's changes as a patch.
- **Binaries**: the functions that changed (least alike first), and those added and removed (with radare2).

```sh
brew install ivxlabs/tap/viper
```

## Use it

On its own:

```sh
viper old/libfoo new/libfoo                  # two folders (or files): the analysis on stdout
viper old/libfoo new/libfoo -o analysis.diffmd
git diff v1.0 v1.1 | viper --patch -         # a patch you already have
```

With Difflicit: install it, and pick **Viper** as the engine when you **Analyze**. Difflicit finds it on your PATH
and opens the analysis as a note, next to the diff.

Viper compares files with `git` and binaries with `radiff2` (`brew install radare2`); without radare2, a changed
binary is reported as just that.

## Build your own engine

Viper is MIT-licensed so you can build on it: fork it and release your engine under any license you like, open
source or proprietary. An engine is a command that answers two things (protocol:
https://ivx.run/docs/difflicit/engines/):

- `--about`: what it is, as JSON (name, version, license, `"protocol": 1`);
- `--difflicit`: a JSON request on stdin (the two sides, and Difflicit's diff of them), the analysis as diffmd on
  stdout; problems on stderr, with a failing exit status.

| Path | What |
| --- | --- |
| `about.json` | What `--about` prints: the engine's name, version, license. |
| `src/main.rs` | The command line. |
| `src/compare.rs` | Comparing two paths (or reading a patch) on its own, into the same request Difflicit sends. |
| `src/lib.rs` | The analysis, written as diffmd. |
| `src/diff.rs` | Reading a unified diff. |

```sh
cargo test                      # unit tests, and the command run as people and Difflicit run it
cargo install --path .          # build it from source into ~/.cargo/bin
```

Releases: bump the version in `Cargo.toml` and `about.json`, push a `vX.Y.Z` tag, and publish the draft release
it builds; that also updates the Homebrew formula in ivxlabs/homebrew-tap.

## License

MIT. See [LICENSE](LICENSE). (Difflicit itself is GPL-3.0; engines are separate programs, so that doesn't
reach them.)
