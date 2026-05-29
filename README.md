# rusty-detox

Sanitize messy filenames through a configurable filter pipeline. Rust port of [Doug Harple's `detox(1)`](https://github.com/dharple/detox) 3.0.1.

[![crates.io](https://img.shields.io/crates/v/rusty-detox.svg)](https://crates.io/crates/rusty-detox)
[![docs.rs](https://docs.rs/rusty-detox/badge.svg)](https://docs.rs/rusty-detox)
[![CI](https://github.com/jsh562/rusty-detox/actions/workflows/ci.yml/badge.svg)](https://github.com/jsh562/rusty-detox/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](#msrv)
[![license: MIT OR Apache-2.0](https://img.shields.io/crates/l/rusty-detox.svg)](#license)

Ships the full upstream filter pipeline, `detoxrc` config grammar, recursive collision-safe batch rename, EXDEV cross-device fallback, & a typed library API. Default mode adds parse-time conflict rejection, `--help`/`--version`, & a `completions` subcommand. Strict mode reverts every observable surface to byte-equal detox 3.0.1 for drop-in migration.

Part of the [Rusty portfolio](https://jsh562.github.io/rusty-portfolio).

## Install

```sh
cargo install rusty-detox
# or, with prebuilt binaries:
cargo binstall rusty-detox
# or, download directly from GitHub Releases:
# https://github.com/jsh562/rusty-detox/releases
```

To also install the `inline-detox` companion binary that reads stdin to EOF & sanitizes the bytes as a single name:

```sh
cargo install rusty-detox --features inline-detox
```

## Usage

```sh
# Preview a rename without touching the file
rusty-detox -n 'My Résumé (final v2).pdf'
# → My Résumé (final v2).pdf -> My_Resume_final_v2.pdf

# Recursively clean a download tree (collision-safe)
rusty-detox -r ./downloads/

# Pick a specific sanitizer sequence
rusty-detox -s utf_8 ./*.pdf

# List loaded sequences so you can pick one
rusty-detox -L

# Sanitize a stream as a single name (e.g. inside a shell pipeline)
echo 'My Résumé.pdf' | inline-detox        # via inline-detox feature

# Strict detox-compat mode (drop-in detox 3.0.1 replacement)
rusty-detox --strict -r ./downloads/
RUSTY_DETOX_STRICT=1 rusty-detox -r ./downloads/
detox -r ./downloads/                       # via detox-alias argv[0] symlink

# Shell completions
rusty-detox completions bash                 # > ~/.bash_completion.d/rusty-detox
rusty-detox completions zsh                  # > ~/.zfunc/_rusty-detox
rusty-detox completions fish                 # > ~/.config/fish/completions/rusty-detox.fish
rusty-detox completions powershell
```

## Library API

The library exposes the filter pipeline, `Sequence` builder, `Detox` / `DetoxBuilder` API, & EXDEV-aware renamer without any CLI deps. Use it when you want detox's sanitizer behavior inside another tool.

```rust,no_run
use rusty_detox::{Detox, DetoxBuilder, Sequence};

let detox = DetoxBuilder::new()
    .sequence(Sequence::utf_8())
    .build();
let cleaned = detox.sanitize("My Résumé (final v2).pdf");
// cleaned == "My_Resume_final_v2.pdf"
```

For library-only consumers without CLI deps see the [Cargo Features](#cargo-features) section.

## Cargo Features

`default` enables `full`, which (for this single-capability port) resolves to the `cli` umbrella. `detox-classic` reproduces v0.1.x bare-port behavior matching upstream detox 3.0.1 1:1. To strip the CLI surface use `default-features = false` or `--no-default-features` & add the features you want.

rusty-detox is a **single-capability port**: its one documented job is "sanitize messy filenames through a configurable filter pipeline". No optional feature leaves are carved beyond the required umbrellas; see [`docs/feature-layout.md`](docs/feature-layout.md) for why.

### Feature matrix

| Feature | Description | Umbrella(s) |
|---|---|---|
| `cli` | All CLI-only dependencies (`clap`, `clap_complete`, `anyhow`, `terminal_size`, `walkdir`) and the binary entry point, mode resolver, output formatter, recursive walker, and Strict-mode pre-scanner. Library consumers strip via `default-features = false`. | `full`, `detox-classic`, `detox-minimal`, `inline-detox` |
| `inline-detox` | Installs an additional `inline-detox` binary alongside `rusty-detox`. Reads stdin to EOF, sanitizes the byte stream as a single name, writes to stdout (the upstream-canonical companion entry). | (standalone, implies `cli`) |

### Preset bundles

| Bundle | Composition | Use case |
|---|---|---|
| `detox-classic` | `cli` | Drop-in upstream detox 3.0.1 replacement. Strict mode is invoked via `--strict`, `RUSTY_DETOX_STRICT`, or `detox`/`detox-alias` argv[0] auto-detect. |
| `detox-minimal` | `cli` | Explicit minimal-CLI alias for users who prefer the `<port>-minimal` naming convention seen across other portfolio ports. Identical composition to `detox-classic`. |

### Keep-list workaround (Cargo features are union-only)

Cargo features cannot subtract from `default`. To get "everything except a specific feature," disable defaults & enumerate the features you want:

```sh
cargo install rusty-detox --no-default-features --features "cli"
# → bare CLI with no inline-detox companion.

cargo install rusty-detox --no-default-features --features "cli inline-detox"
# → CLI + the inline-detox companion binary.
```

For the common cases the named [preset bundles](#preset-bundles) are usually sufficient.

### Library-only consumers

```toml
[dependencies]
rusty-detox = { version = "0.2", default-features = false }
```

This strips `clap`, `clap_complete`, `anyhow`, `terminal_size`, & `walkdir`. The resulting build pulls only `thiserror` (required by the always-on `DetoxError` enum).

### Convention authority

This layout follows the portfolio-wide Cargo Features Convention. The "why" lives in [ADR-0006](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md); the "what" lives in [`project-instructions.md` §Cargo Feature Surface](https://github.com/jsh562/rustylib/blob/main/project-instructions.md). Every Rusty port from v0.2 onward exposes the same umbrella set (`default` / `full` / `cli` / `<port>-classic`), per-port leaves named in kebab-case, & 2 to 4 preset bundles.

## Compatibility

`rusty-detox` has two modes:

- **Default mode.** clap-styled flag parser. Conflicting flag pairs MUST be rejected at parse time. `--help`, `--version`, & the `completions` subcommand are all available.
- **Strict mode** (activated by `--strict`, `RUSTY_DETOX_STRICT=1`, or invoking the binary as `detox`/`detox-alias`). Byte-equal stderr against upstream v3.0.1 for documented diagnostics. Last-wins flag resolution. `--help`, `--version`, & `completions` MUST be rejected.

The upstream `Table.utf_8` & `Table.iso8859_1` are vendored at v3.0.1 freeze. Future re-vendoring is a MAJOR semver bump.

### Concurrency

The EXDEV cross-device rename fallback (copy + fsync + rename + unlink) is not atomic across the full chain. A concurrent reader could observe both the source & target briefly. Upstream `detox(1)` has the same property. Run rusty-detox to completion on a quiescent tree if external observers are critical.

See [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md) for the full per-flag matrix.

## What's not shipped

- **Atomic cross-device rename.** The EXDEV fallback (copy + fsync + rename + unlink) MUST NOT be relied on for atomicity across the full chain. Upstream `detox(1)` has the same property.
- **Re-vendoring of `Table.utf_8` / `Table.iso8859_1` mid-version.** Both tables are vendored at v3.0.1 freeze; any change is a MAJOR semver bump.

## MSRV

Rust **1.85** (edition 2024). Re-verified against the portfolio's stable-minus-two policy at each release.

## License

Dual-licensed under [MIT](LICENSE) or [Apache-2.0](LICENSE-APACHE) at your option.
