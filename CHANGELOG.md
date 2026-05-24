# Changelog

All notable changes to `rusty-detox` are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-MM-DD

### Added

- Initial release. Faithful Rust port of Doug Harple's `detox(1)` v3.0.1.
- Filter pipeline: `uncgi`, `iso8859_1`, `utf_8`, `safe`, `wipeup`, `max_length`, `safe_platform` (Windows auto, non-Windows opt-in).
- Three built-in sequences: `default` (= `safe` + `wipeup`), `iso8859_1`, `utf_8`.
- `detoxrc` external config parser (hand-rolled recursive descent; path precedence: `-f` > `$XDG_CONFIG_HOME/detoxrc` > `~/.detoxrc` > `/etc/detoxrc` > built-in).
- Recursive directory walking (depth-first leaves-up via `walkdir`) with collision-safe batch rename (monotonic `_N` suffix before the final extension token).
- Cross-device rename (EXDEV) fallback: copy + fsync + rename + unlink with best-effort metadata preservation.
- CLI: `-n`/`-r`/`-s`/`-f`/`-L`/`-v`/`--help`/`--version` plus the `completions` subcommand (bash/zsh/fish/powershell).
- Strict-compat mode: `--strict` / `RUSTY_DETOX_STRICT=1` / argv[0] = `detox`/`detox-alias`; byte-equal stderr against captured upstream v3.0.1 snapshots; last-wins flag resolution; grouped short flags.
- Library API: `Detox`, `DetoxBuilder`, `Sequence`, `Filter`, `DetoxError`, `RenamePlanEntry`, `DetoxReport`; all `#[non_exhaustive]` where appropriate; `default-features = false` strips clap/walkdir/etc.
- Companion `inline-detox` binary (gated behind cargo feature `inline-detox`).
- Pre-generated shell completions committed under `completions/` with drift tests.
- MSRV: Rust 1.85 (edition 2024).

### Notes

- `Table.utf_8` and `Table.iso8859_1` are vendored at upstream `dharple/detox` v3.0.1 — future re-vendoring is a MAJOR semver bump in rusty-detox.
- `--special` flag (upstream detoxing of sockets/devices/named pipes) deferred to v0.2.0+.
- `--transliterate=deunicode` opt-in (richer non-Latin coverage at the cost of byte-equal compat) deferred to v0.2.0+.

[Unreleased]: https://github.com/jsh562/rusty-detox/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jsh562/rusty-detox/releases/tag/v0.1.0
