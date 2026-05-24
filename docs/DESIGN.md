# rusty-detox — Design Notes

Authoritative spec/plan: [`specs/00006-detox-port/`](../../rusty/specs/00006-detox-port/) in the umbrella repo.

## Upstream Dependency Status

E003 (reusable `port-ci.yml` workflow) is not yet shipped in the umbrella repo at v0.1.0 time. Inline workflows (`ci.yml`, `release.yml`) are duplicated verbatim from `rusty-pwgen` as a pragmatic-path solution. When E003 v1.0.0 lands, these files are replaced by thin callers pinned to that tag.

Tracked as portfolio tech debt; not a blocker for v0.1.0.

## Vendored Translation Tables

`Table.utf_8` and `Table.iso8859_1` are vendored from upstream `dharple/detox` at v3.0.1 (commit hash TBD at table-capture time). Stored as sorted-by-codepoint static slices in `src/tables.rs`; binary search at runtime via `slice::binary_search_by_key`.

Freeze policy: re-vendoring (sync with later upstream tags) is a MAJOR semver bump in rusty-detox. README + CHANGELOG document this expectation.

## SemVer Bump Policy

- **MAJOR**: re-vendor translation tables; remove/rename a public API symbol; change `Filter` / `DetoxError` variant payload; change Strict-mode byte-exact output format.
- **MINOR**: add a new `Filter` variant (enabled by `#[non_exhaustive]`); add a new `DetoxError` variant; add a new public method or `RenamePlanEntry`/`DetoxReport` field; add a new CLI flag.
- **PATCH**: bug fixes that do not alter documented behavior; performance improvements; doc-only changes.

## Build / Feature Matrix

| Feature combination | Binary | Library deps |
|---|---|---|
| `default = ["cli"]` | `rusty-detox` | clap, clap_complete, anyhow, terminal_size, walkdir |
| `default-features = false` | none | thiserror only |
| `default + inline-detox` | `rusty-detox`, `inline-detox` | as `default` |

Verified by `tests/lib_api.rs::default_features_off_dep_tree`.
