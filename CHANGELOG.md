# Changelog

All notable changes to `rusty-detox` are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-05-26

### Added (additive only — no v0.1.x behavior changed)

- Portfolio-wide [Cargo Features Convention](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md)
  layout per ADR-0006 + `project-instructions.md` §Cargo Feature Surface. rusty-detox applies the minimum convention as a **single-capability port** per spec 00011 §Scope Edge Cases.
- New umbrella features (all `["cli"]` composition for this single-cap port):
  - `full` — kitchen-sink umbrella per FR-002
  - `detox-classic` — required `<port>-classic` umbrella per FR-004; detox 3.0.1 drop-in replacement
  - `detox-minimal` — preset bundle per FR-007; explicit minimal-CLI semantic alias
- `default` now aliases to `full` instead of directly to `cli`. Resolved dependency set is identical (`full = ["cli"]`); no observable change for any consumer.
- See [`docs/feature-layout.md`](docs/feature-layout.md) for the zero-leaf rationale.

All v0.1.x feature names are preserved verbatim with identical compositions. `cli = ["dep:clap", "dep:clap_complete", "dep:anyhow", "dep:terminal_size", "dep:walkdir"]` is unchanged. `inline-detox = ["cli"]` is unchanged. Library consumers using `default-features = false` get the same CLI-stripped build. Users running `cargo install rusty-detox --features inline-detox` continue to work unchanged. The vendored upstream translation tables (`Table.utf_8`, `Table.iso8859_1`) remain frozen at v3.0.1 — byte-for-byte identical sanitized output for any stored filename.

### Notes

- See the new `## Cargo Features` section in `README.md` for the
  feature matrix, preset bundles, keep-list workaround, and convention
  authority citations.
- Reference: [ADR-0006](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md)
  (why this layout) + [`project-instructions.md` §Cargo Feature Surface](https://github.com/jsh562/rustylib/blob/main/project-instructions.md)
  (what the rules are).
- CI matrix expanded per spec 00011 FR-010..FR-014: now includes
  `test-default` (kitchen sink + cross-compile), `test-no-default`
  (bare library + dep-tree audit per SC-001), `test-detox-classic`,
  `test-detox-minimal` (preset bundles per SC-003), `test-keeplist`
  (keep-list workaround per SC-004), and `lint-convention` (vendored
  `tools/feature-lint/run.sh` invocation per FR-052). Tier 4
  (`check-leaf-<leaf>`) is intentionally empty — zero leaves carved
  per docs/feature-layout.md.
- The lint script is **vendored** into `tools/feature-lint/` (synced
  from the umbrella `jsh562/rustylib` repo) so per-port CI workflows
  do not depend on cross-repo `actions/checkout` of the private
  umbrella. Sync precedent set by rusty-figlet v0.2.0 (E011 Phase 2
  iteration 6), rusty-ts v0.2.0 (E011 Phase 3), rusty-sponge v0.2.0
  (E011 Phase 4), rusty-vipe v0.2.0 (E011 Phase 5), rusty-pee v0.2.0
  (E011 Phase 6), and rusty-pwgen v0.2.0 (E011 Phase 7).

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

[Unreleased]: https://github.com/jsh562/rusty-detox/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jsh562/rusty-detox/releases/tag/v0.2.0
[0.1.0]: https://github.com/jsh562/rusty-detox/releases/tag/v0.1.0
