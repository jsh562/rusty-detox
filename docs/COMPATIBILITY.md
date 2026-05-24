# Compatibility — rusty-detox vs upstream `detox(1)` v3.0.1

This document enumerates every flag and behavior with Default-mode and Strict-mode rows. Populated during Polish phase (T130+). For the v0.1.0 release.

## Flag Surface

| Flag | Upstream | rusty-detox Default | rusty-detox Strict |
|---|---|---|---|
| `-n` / `--dry-run` | ✓ | ✓ | ✓ |
| `-r` / `--recursive` | ✓ | ✓ | ✓ |
| `-s <name>` / `--sequence` | ✓ | ✓ | ✓ |
| `-f <path>` / `--config-file` | ✓ | ✓ | ✓ |
| `-L` / `--list-sequences` | ✓ | ✓ | ✓ |
| `-v` / `--verbose` | ✓ | ✓ | ✓ |
| `-h` / `--help` | ✓ | ✓ (clap-styled) | ✓ (byte-equal) |
| `-V` / `--version` | ✓ | ✓ | ✓ |
| `--special` | ✓ | ✗ (deferred to v0.2.0) | ✗ |
| `--inline` | ✓ (via inline-detox companion) | ✓ (via `inline-detox` cargo feature) | ✓ |
| `completions <shell>` subcommand | ✗ | ✓ (Default-mode only) | ✗ (treated as unparseable positional) |

## Behavioral Divergences

### Default-mode (BREAKING-CHANGE)

- Conflicting flag pairs (e.g., `-s default -s utf_8`) — rusty-detox Default rejects at parse time via `clap` `conflicts_with`. Upstream uses last-wins. Strict mode mirrors upstream.

### Stream policy

- Rename lines (driven by `-n` or `-v`) → STDOUT
- Diagnostics, warnings, errors → STDERR regardless of `-v`

### Library API

- The library exposes `Detox`, `DetoxBuilder`, `Sequence`, `Filter`, `DetoxError`, `RenamePlanEntry`, `DetoxReport`. `default-features = false` strips all CLI deps.
- `Filter::Safe { unsafe_chars }` requires explicit construction; `Filter::safe_default()` provides the v0.1.0 default unsafe-char set (FR-004).
