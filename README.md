# Viper

An analysis engine for [Difflicit](https://github.com/ivxlabs/difflicit). Ask Difflicit to analyze two artifacts,
pick Viper, and it writes up the **visual changes** between them as a [diffmd](https://ivx.run/docs/difflicit/diffmd/)
note:

- for text and source: every changed file with its additions and deletions, then each file's changes as a patch;
- for binaries: the functions that changed (least alike first), and those added and removed.

Viper is open source under the MIT license, so it's also a starting point for your own engine: fork it, and
release yours under any license you like, open source or proprietary. **HyperViper** is the full version, built
on Viper.

## Install

```sh
scripts/package.sh        # builds it into dist/viper
```

In Difflicit: **Analyze → Manage engines… → Install engine…**, and pick `dist/viper`. Then choose **Viper** as the
engine when you analyze.

## How it works

Difflicit runs `bin/viper` once per analysis, with a JSON request on standard input (the two sides and Difflicit's
diff of them). Viper writes the analysis to standard output, and any problem to standard error with a failing
exit status. The protocol is documented at https://ivx.run/docs/difflicit/engines/.

| Path | What |
| --- | --- |
| `engine.json` | What Difflicit reads to install and run the engine: its id, name, version and command. |
| `src/main.rs` | The program: request in, analysis out. |
| `src/lib.rs` | The analysis, written as diffmd. |
| `src/diff.rs` | Reading the unified diff in the request. |

```sh
cargo test                # unit tests, and the engine run the way Difflicit runs it
```

## License

MIT. See [LICENSE](LICENSE). (Difflicit itself is GPL-3.0; engines are separate programs, so that doesn't
reach them.)
