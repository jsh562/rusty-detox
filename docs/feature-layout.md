# rusty-detox — v0.2.0 Feature Layout

**Status**: implementation draft for the v0.2.0 Cargo features convention
backfill (spec 00011, Phase 8 — rusty-detox).

**Authority**:
- `specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md` (why)
- `project-instructions.md` §Cargo Feature Surface (what)
- This document — the per-port carving + WHY for each leaf, per HINT-003
  + HINT-009 of spec 00011.

**Reference port**: rusty-figlet v0.2.0 — see `../../rusty-figlet/docs/feature-layout.md`
(FROZEN reference port) for the format anchor. rusty-detox conforms to the
same shape with the minimum-convention surface dictated by its
single-capability scope. The companion sibling ports rusty-sponge v0.2.0,
rusty-vipe v0.2.0, rusty-pee v0.2.0, and rusty-pwgen v0.2.0 (see
`../../rusty-sponge/docs/feature-layout.md`, `../../rusty-vipe/docs/feature-layout.md`,
`../../rusty-pee/docs/feature-layout.md`, `../../rusty-pwgen/docs/feature-layout.md`)
established the zero-leaf precedent for single-capability ports with a
companion / aliased binary.

**Iteration model**: v0.2.0 is a **purely additive** SemVer-minor release.
Every v0.1.x feature name and composition is preserved verbatim; new
umbrellas (`full`, `detox-classic`, `detox-minimal`) are layered on top
without renaming or narrowing the existing `cli` / `default` /
`inline-detox` features. Library and binary API surfaces are unchanged.

## Single-capability port — spec 00011 §Scope Edge Cases

rusty-detox is a **single-capability port**: it has exactly one documented
capability — sanitize messy filenames through a configurable filter
pipeline (a Rust port of Doug Harple's `detox(1)` v3.0.1). Spec 00011
§Scope Edge Cases dictates that single-capability ports apply the
**minimum convention**:

> ports with only one capability adopt the minimum convention:
> `full = ["cli"]` and `<port>-classic = ["cli"]` are the required
> umbrellas; ZERO leaves carved beyond those required umbrellas.

This document records the carving exercise and the explicit decision
to NOT split orthogonal sub-capabilities into leaves — every additional
behavior of `rusty-detox` (the `uncgi` / `iso8859_1` / `utf_8` / `safe` /
`wipeup` / `max_length` / `safe_platform` filters, the three built-in
sequences, the `detoxrc` external config parser, the recursive walker,
the collision-safe batch rename, the EXDEV cross-device rename fallback,
the Strict-mode argv pre-scanner, the `completions` subcommand) is part
of the single core capability surface and removing any of them would
break the documented public CLI / library contract or change byte-output
for the upstream-pinned v3.0.1 translation tables (the table chain is
locked at v0.1.0 per the lockstep-SemVer policy).

## Source-tree walk

`src/` modules (v0.1.0, post-Phase-1 baseline):

| Module                  | Always-on?     | CLI-only deps                          | Notes                                                                       |
|-------------------------|---------------:|----------------------------------------|-----------------------------------------------------------------------------|
| `error.rs`              | yes            | (thiserror — always-on)                | `DetoxError` enum; library + binary need it.                                |
| `tables.rs`             | yes            | none                                   | Vendored `Table.utf_8` / `Table.iso8859_1` translation tables (v3.0.1).     |
| `sequence.rs`           | yes            | none                                   | `Sequence` builder + the three built-in sequences (default/iso8859_1/utf_8).|
| `filter/mod.rs`         | yes            | none                                   | `Filter` enum dispatcher.                                                   |
| `filter/uncgi.rs`       | yes            | none                                   | Percent-escape decoder.                                                     |
| `filter/iso8859_1.rs`   | yes            | none                                   | ISO-8859-1 → ASCII transliterator (via vendored table).                     |
| `filter/utf8.rs`        | yes            | none                                   | UTF-8 → ASCII transliterator (via vendored table).                          |
| `filter/safe.rs`        | yes            | none                                   | Shell-metachar / control-char scrub.                                        |
| `filter/wipeup.rs`      | yes            | none                                   | Collapse repeated underscores; trim trailing underscores.                   |
| `filter/max_length.rs`  | yes            | none                                   | Length cap (preserves extension).                                           |
| `filter/safe_platform.rs`| yes           | none                                   | Windows-reserved-name / illegal-char scrub (auto on Windows, opt-in else).  |
| `planner.rs`            | yes            | none                                   | `RenamePlanEntry` builder; collision-safe `_N` suffix logic.                |
| `renamer.rs`            | yes            | none                                   | `rename_with_fallback`; EXDEV (cross-device) copy+fsync+rename+unlink.      |
| `config.rs`             | yes            | none                                   | `detoxrc` hand-rolled recursive descent parser.                             |
| `lib.rs`                | yes            | none                                   | Public API (`Detox`, `DetoxBuilder`, `Sequence`, `Filter`, etc.).           |
| `walker.rs`             | no — `cli`     | walkdir                                | Recursive directory walker (depth-first leaves-up).                         |
| `cli.rs`                | no — `cli`     | clap                                   | clap-derive `Cli` struct + `Subcommand::Completions`.                       |
| `mode.rs`               | no — `cli`     | none (gated by `cli`)                  | CompatibilityMode resolver (`--strict` > env > argv[0]).                    |
| `output.rs`             | no — `cli`     | none (gated by `cli`)                  | `rename_line` formatter for `-v` / `-n` output.                             |
| `strict.rs`             | no — `cli`     | (clap pulled by `cli`)                 | Hand-rolled Strict-mode argv pre-scanner + byte-equal upstream dispatcher.  |
| `main.rs`               | no — `cli`     | clap, clap_complete, anyhow, terminal_size, walkdir | Binary entry; gated by `required-features = ["cli"]`.       |
| `bin/inline_detox.rs`   | no — `inline-detox` | (inherits `cli`)                  | `inline-detox` companion binary; gated by `required-features = ["inline-detox"]`. |

## Leaf-carving criteria (HINT-009)

A capability becomes a leaf when ALL of the following hold:

1. It is **self-containable** — gated cleanly via `#[cfg(feature = "<leaf>")]`
   at the module or top-level item boundary (HINT-004).
2. Either (a) it has a **sole optional dependency** that no other leaf needs
   (HINT-005), OR (b) it is a pure-cfg-gate of an internal module worth
   exposing as a knob.
3. Disabling it does NOT break any always-on library/CLI surface.

A capability does NOT become a leaf when:

- It is foundational (the filter pipeline cores: uncgi/iso8859_1/utf_8/
  safe/wipeup/max_length/safe_platform — each is a discrete filter in the
  documented `Sequence` composition; disabling any one would break the
  built-in `Sequence::default()`/`utf_8()`/`iso8859_1()` constructors).
- It is part of the single documented capability surface (filter pipeline,
  three built-in sequences, `detoxrc` config parser, recursive walking,
  collision-safe batch rename, EXDEV fallback, Strict-mode dispatcher,
  completions subcommand).
- It would create more than ~6 leaves (FR-007 + HINT-003 envelope).

## v0.2.0 Carved Leaves

**ZERO new leaves carved at v0.2.0**. Every capability inside rusty-detox
is either:

1. Foundational always-on library code (the seven `Filter` variants, the
   three built-in `Sequence` constructors, the vendored translation
   tables, the planner, the renamer with EXDEV fallback, `detoxrc`
   config parser, error types) — cannot be stripped without breaking
   the public surface.
2. Already gated by the v0.1.x `cli` umbrella (clap-derived argument
   parsing, completions subcommand, mode resolver, output formatter,
   Strict-mode argv pre-scanner, recursive walker).
3. Already gated by the v0.1.x `inline-detox` feature (the second
   `inline-detox` binary entry).

### Leaves intentionally NOT carved

The following candidate leaves were considered + rejected:

- **`filter-uncgi` / `filter-utf8` / `filter-iso8859_1` / `filter-safe` /
  `filter-wipeup` / `filter-max_length` / `filter-safe_platform`**: Each
  filter is a discrete enum variant of `Filter` consumed by the built-in
  `Sequence::default()`, `Sequence::utf_8()`, and `Sequence::iso8859_1()`
  constructors. Removing any one would break the byte-output of the
  built-in sequences and silently produce different sanitized names
  for stored / shell-script invocations — a MAJOR-bump behavior change
  disguised as a feature flag. Rejected per HINT-009 criterion 3.

- **`config-detoxrc`**: The hand-rolled `detoxrc` parser is part of the
  documented `-f <file>` CLI surface and the `$XDG_CONFIG_HOME/detoxrc`
  / `~/.detoxrc` / `/etc/detoxrc` precedence chain. Carving it would
  create a CLI mode where `-f` exists but produces an error, which
  is worse than not having the flag at all. Rejected per HINT-009
  criteria 1 + 3.

- **`walker-recursive`**: The recursive directory walker IS already
  gated by the v0.1.x `cli` umbrella (it pulls `walkdir` which is in
  the `cli` dep list). Carving it out separately would require
  splitting `walkdir` away from `cli` and renaming the existing `cli`
  feature — both of which violate the additive-v0.2.0 contract.
  The library API's `Detox::plan()` and `Detox::execute()` paths use
  the walker only behind `#[cfg(feature = "cli")]` (already in place
  in v0.1.x lib.rs) — library consumers without `cli` get the
  single-path behavior.

- **`strict-compat`**: rusty-detox's Strict mode dispatches inline in
  `main.rs` via `mode::resolve` and the hand-rolled getopt mirror in
  `src/strict.rs`. Both are gated by the `cli` umbrella in v0.1.x
  (since they consume `clap` for the `--strict` flag itself). Carving
  out a separate `strict-compat` leaf would require splitting
  `strict.rs` away from `cli.rs`, which is more refactoring than the
  additive v0.2.0 release allows. The capability survives untouched
  inside the existing `cli` composition. (Note: rusty-figlet carves
  `strict-compat` because its Strict-mode parser is dep-free hand-rolled
  getopt; rusty-detox's Strict dispatcher consumes `clap` for the
  `--strict` flag itself, so it cannot stand alone without `cli`.)

- **`completions`**: Could be carved as `["dep:clap_complete"]`, but per
  spec 00011 §Scope Edge Cases minimum-convention single-capability
  ports declare ZERO new leaves. `clap_complete` is bundled into the
  v0.1.x `cli` umbrella verbatim. Carving it would either rename `cli`
  (breaking SemVer additivity) or duplicate the surface.

- **`exdev-fallback`**: The cross-device rename fallback (copy + fsync
  + rename + unlink) is the headline correctness guarantee of the
  renamer — disabling it would let users hit an unhelpful "Invalid
  cross-device link" error on otherwise-working `cargo install`
  defaults. Foundational always-on. Rejected per HINT-009 criterion 3.

- **`inline-detox`**: This v0.1.x feature ships a second binary named
  `inline-detox` that reads stdin / writes stdout (FR-044/FR-045
  upstream-canonical companion). It IS retained verbatim per the
  v0.2.0 SemVer additive contract — but it is NOT one of the 2
  required preset bundles per FR-007 (those are `detox-classic` and
  `detox-minimal` below). Documented separately as an
  installation-time convenience knob.

## Preset bundles (FR-007 — 2 required for single-capability ports)

Per spec 00011 §Scope Edge Cases + FR-007, even single-capability ports
declare 2 preset bundles to give the keep-list workaround documentation
something concrete to point at.

### `detox-classic` (REQUIRED — bare port, 1:1 with upstream detox 3.0.1)

```toml
detox-classic = ["cli"]
```

- Includes `cli` so the binary exists.
- Single-capability port; the `cli` umbrella IS the bare-port surface.
- Use case: `cargo install rusty-detox --no-default-features --features detox-classic`
  for a detox 3.0.1 drop-in replacement (Strict mode is invoked via
  the `--strict` flag, `RUSTY_DETOX_STRICT` env var, or `detox` /
  `detox-alias` argv[0] auto-detect — none of these require additional
  features).

### `detox-minimal`

```toml
detox-minimal = ["cli"]
```

- Same composition as `detox-classic` (single-capability port has no
  smaller subset to carve).
- Use case: explicit "minimal CLI install" alias for users who prefer
  the `<port>-minimal` naming convention seen across other Rusty ports
  (figlet-minimal, ts-minimal, sponge-minimal, vipe-minimal,
  pee-minimal, pwgen-minimal).
  Documented as an intentional semantic alias rather than a distinct
  composition.

### `inline-detox` (v0.1.x feature retained, NOT a convention preset)

`inline-detox = ["cli"]` from v0.1.0 ships an additional `inline-detox`
binary alongside `rusty-detox`. It is retained verbatim per the v0.2.0
SemVer additive contract — but it is NOT one of the 2 required preset
bundles per FR-007 (those are `detox-classic` and `detox-minimal`
above). `inline-detox` is documented separately as an
installation-time convenience knob, not a capability subset.

## Cross-port glossary candidates

No leaves carved → no cross-port glossary contributions from rusty-detox
in this iteration. If a future minor release adds an orthogonal
capability (e.g., a `transliterate-deunicode` leaf adding the
deferred richer non-Latin coverage from v0.1.0 known-limitations, or
a `filter-special` leaf adding the deferred `--special` sockets /
devices / named-pipes detoxer), the leaf would be a candidate for
promotion to `docs/feature-vocabulary.md` per FR-053.

## CI matrix shape (FR-010..FR-014)

Per plan §Per-Port v0.2.0 CI Matrix, scaled to a zero-leaf port:

- **Tier 1 — `test-default`**: full DDR-003 cross-compile matrix
  (5 targets). Post-v0.2.0 `default = ["full"]` and `full = ["cli"]`,
  so the kitchen-sink test resolves to the same set as v0.1.0
  `default = ["cli"]` — no regression in coverage.
- **Tier 2 — `test-no-default`**: Linux x86_64 only. `cargo test
  --no-default-features --lib` + dep-tree audit (SC-001 evidence).
- **Tier 3 — `test-<bundle>`**: one job per preset bundle. Linux only.
  - `test-detox-classic`
  - `test-detox-minimal`
- **Tier 4 — `check-leaf-<leaf>`**: SKIPPED. Zero leaves → no
  per-leaf compile-check jobs. A placeholder comment in `ci.yml`
  documents why this tier is empty.
- **Tier 5 — `lint-convention`**: single Linux job invoking the
  vendored `tools/feature-lint/run.sh` script.

Per FR-014, bundle/lint jobs are constrained to Linux x86_64.

## Vendored feature-lint

Per spec 00011 §Phase 2 iteration 6 precedent (rusty-figlet vendored
the lint script because the umbrella `jsh562/rustylib` is private and
cross-repo `actions/checkout` cannot reach it), rusty-detox vendors
`tools/feature-lint/{lint.sh,run.sh,README.md}` from the umbrella into
the port repo. The vendored copy is byte-equal to the umbrella source
of truth as of the freeze commit (post the dev-tooling-allowlist +
benches/tests-search + additive-CHANGELOG-support fixes from rusty-ts
v0.2.0 / E011 Phase 3 iteration 2, the path-sanitization fixes from
rusty-sponge v0.2.0 / E011 Phase 4, and the additional sibling-port
iterations through rusty-pwgen v0.2.0 / E011 Phase 7).

## Why no new leaves — explicit rationale

Spec 00011 §Scope Edge Cases anticipates this case verbatim:

> Some ports have only one orthogonal capability. Those ports adopt the
> minimum convention: `full = ["cli"]` and `<port>-classic = ["cli"]`
> as aliases; the convention SHAPE is consistent across the portfolio
> even when the per-port leaf carving yields zero leaves.

rusty-detox deliberately chooses the zero-leaf path because:

1. The vendored upstream translation tables (`Table.utf_8`,
   `Table.iso8859_1`) are **frozen at v3.0.1** per the lockstep-SemVer
   stability policy; any change is a MAJOR bump because consumers have
   stored sanitized filenames that depend on byte-for-byte identical
   transliteration. Carving any of the filter pipeline machinery into
   an opt-out leaf would let users silently break stored-name
   reproducibility — a MAJOR-bump behavior change disguised as a
   feature flag.
2. The cost of carving a speculative leaf (cfg-gate scaffolding,
   per-leaf CI matrix entry, README/CHANGELOG row, glossary candidacy)
   outweighs the value when no orthogonal capability exists to gate.
3. The portfolio-wide convention shape (umbrella set, README "Cargo
   Features" section, lint compliance) is preserved verbatim — a
   downstream library consumer reading the README for rusty-detox
   gets the same one-glance feature matrix UX as one reading
   rusty-figlet, rusty-ts, rusty-sponge, rusty-vipe, rusty-pee, or
   rusty-pwgen.
4. v0.2.0 is **purely additive**. Every v0.1.x feature is preserved
   verbatim; no SemVer break. Future minor releases can add leaves
   without breaking the v0.2.0 contract: a hypothetical
   `transliterate-deunicode` v0.3.0 feature would slot in as
   `transliterate-deunicode = ["dep:deunicode"]` alongside the
   existing umbrellas with zero migration cost.
