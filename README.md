# rusty-detox

[![crates.io](https://img.shields.io/crates/v/rusty-detox.svg)](https://crates.io/crates/rusty-detox)
[![docs.rs](https://docs.rs/rusty-detox/badge.svg)](https://docs.rs/rusty-detox)
[![license: MIT OR Apache-2.0](https://img.shields.io/crates/l/rusty-detox.svg)](#license)

Sanitize messy filenames. A Rust port of [Doug Harple's `detox(1)`](https://github.com/dharple/detox) with the full upstream filter pipeline, `detoxrc` config grammar, recursive collision-safe batch rename, EXDEV cross-device fallback, and a typed library API.

Part of the [Rusty portfolio](https://jsh562.github.io/rusty-portfolio).

## Install

```sh
cargo install rusty-detox
# or, with prebuilt binaries:
cargo binstall rusty-detox
```

## Usage

```sh
# Preview a rename (dry-run)
rusty-detox -n 'My Résumé (final v2).pdf'
# My Résumé (final v2).pdf -> My_Resume_final_v2.pdf

# Recursive batch-rename a directory tree
rusty-detox -r ./downloads/

# Pick a sequence
rusty-detox -s utf_8 ./*.pdf

# List loaded sequences
rusty-detox -L
```

## Cargo Features

| Feature | Default | What it gates |
|---|---|---|
| `cli` | yes | `clap` + `clap_complete` + `anyhow` + `terminal_size` + `walkdir` |
| `inline-detox` | no | Upstream-canonical `inline-detox` companion binary |

Library consumers can use `rusty-detox = { version = "0.1", default-features = false }` to get the filter pipeline + library API without any CLI dependencies.

## Library API

```rust,no_run
use rusty_detox::{Detox, DetoxBuilder, Sequence};

let detox = DetoxBuilder::new()
    .sequence(Sequence::utf_8())
    .build();
let cleaned = detox.sanitize("My Résumé (final v2).pdf");
// cleaned == "My_Resume_final_v2.pdf"
```

## Compatibility

`rusty-detox` has two modes:

- **Default** — clap-styled flag parser; rejects conflicting flag pairs at parse time; adds `--help`, `--version`, `completions` subcommand.
- **Strict** (`--strict`, env `RUSTY_DETOX_STRICT=1`, or argv[0] = `detox`/`detox-alias`) — byte-equal stderr against upstream v3.0.1 for documented diagnostics; last-wins flag resolution; no subcommands.

The upstream `Table.utf_8` and `Table.iso8859_1` are vendored at v3.0.1 freeze; future re-vendoring is a MAJOR semver bump.

## Concurrency

The EXDEV cross-device rename fallback (copy + fsync + rename + unlink) is not atomic across the full chain — a concurrent reader could observe both the source and target briefly. Upstream `detox(1)` has the same property. Run rusty-detox to completion on a quiescent tree if external observers are critical.

## MSRV

Rust 1.85 (edition 2024). Re-verified against stable-minus-two policy at each release.

## License

Dual-licensed under [MIT](LICENSE) or [Apache-2.0](LICENSE-APACHE).
