//! # rusty-detox
//!
//! A Rust port of Doug Harple's `detox(1)` v3.0.1: sanitize messy filenames
//! through a configurable filter pipeline.
//!
//! ## Quick start
//!
//! ```
//! use rusty_detox::{DetoxBuilder, Sequence};
//!
//! let detox = DetoxBuilder::new()
//!     .sequence(Sequence::utf_8())
//!     .build();
//! let clean = detox.sanitize("My Résumé (final v2).pdf");
//! // The closing paren before `.pdf` becomes `_`; trailing-trim only
//! // strips runs at the very ends of the basename, not before the extension.
//! assert_eq!(clean, "My_Resume_final_v2_.pdf");
//! ```
//!
//! ## Stability (lockstep SemVer)
//!
//! Library and binary share a single crate version. The vendored upstream
//! translation tables (`Table.utf_8`, `Table.iso8859_1`) are **frozen at
//! v3.0.1** — re-vendoring is a MAJOR semver bump.

#![deny(missing_docs)]

pub mod config;
pub mod error;
pub mod filter;
pub mod planner;
pub mod renamer;
pub mod sequence;
pub mod tables;

#[cfg(feature = "cli")]
pub mod walker;

pub use error::DetoxError;
pub use filter::Filter;
pub use sequence::Sequence;

use std::path::PathBuf;

/// Configured detox pipeline runner. Construct via [`DetoxBuilder`].
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Detox {
    sequence: Sequence,
    /// Verbose: emit one rename line per change to stdout.
    pub verbose: bool,
    /// Dry-run: plan and report renames without issuing any rename syscalls.
    pub dry_run: bool,
    /// Recursive: descend into directories depth-first leaves-up.
    pub recursive: bool,
    /// Maximum collision-resolution suffix attempts before giving up.
    pub collision_cap: u32,
}

impl Detox {
    /// Sanitize one UTF-8 input string, returning UTF-8 output (FR-040).
    ///
    /// Lossy reconstruction via [`String::from_utf8_lossy`] handles the rare
    /// case where the active sequence includes [`Filter::Uncgi`] and the
    /// input contains percent-escapes that decode to invalid UTF-8 fragments
    /// (e.g., a lone `%C3`). In that case the invalid byte is replaced with
    /// U+FFFD in the `&str` path. Callers needing byte-exact round-tripping
    /// for arbitrary inputs MUST use [`Detox::sanitize_bytes`].
    pub fn sanitize(&self, input: &str) -> String {
        let bytes = self.sanitize_bytes(input.as_bytes());
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// Canonical byte-oriented sanitization entry point (FR-040).
    ///
    /// Applies the configured [`Sequence`] to `input` and returns the
    /// transformed bytes. No filesystem I/O.
    pub fn sanitize_bytes(&self, input: &[u8]) -> Vec<u8> {
        self.sequence.apply(input)
    }

    /// Planned rename(s) for `path` without executing them (FR-041).
    ///
    /// Performs filesystem READS (`readdir`, `stat`) as required to enumerate
    /// directory contents and check for pre-existing collision targets, but
    /// no filesystem MUTATION. Side-effect-free with respect to writes/logs.
    pub fn plan(&self, path: &std::path::Path) -> Vec<RenamePlanEntry> {
        let entries = if self.recursive && path.is_dir() {
            #[cfg(feature = "cli")]
            {
                walker::recursive_walk(path)
                    .into_iter()
                    .map(|w| w.path)
                    .collect()
            }
            #[cfg(not(feature = "cli"))]
            {
                vec![path.to_path_buf()]
            }
        } else {
            vec![path.to_path_buf()]
        };
        planner::plan_directory(&entries, &self.sequence, self.collision_cap).unwrap_or_default()
    }

    /// Execute the rename(s) for `path` (FR-042).
    ///
    /// Produces the same plan as [`Detox::plan`] and iterates it issuing
    /// rename syscalls (with EXDEV fallback per FR-025). Returns a
    /// [`DetoxReport`] on success or a [`DetoxError`] on the first
    /// unrecoverable failure.
    pub fn execute(&self, path: &std::path::Path) -> Result<DetoxReport, DetoxError> {
        let plan = self.plan(path);
        let mut report = DetoxReport {
            planned: plan.len(),
            renamed: 0,
            skipped: 0,
            errored: 0,
        };
        if self.dry_run {
            report.skipped = plan.len();
            return Ok(report);
        }
        for entry in &plan {
            match renamer::rename_with_fallback(&entry.source, &entry.target) {
                Ok(_) => report.renamed += 1,
                Err(e) => {
                    report.errored += 1;
                    return Err(e);
                }
            }
        }
        Ok(report)
    }
}

/// Builder for [`Detox`] (FR-037).
///
/// All builder methods are OPTIONAL with documented defaults; `build()` is
/// INFALLIBLE. Set the active [`Sequence`] with [`DetoxBuilder::sequence`]
/// (default [`Sequence::default()`]).
#[derive(Debug, Clone)]
pub struct DetoxBuilder {
    sequence: Sequence,
    verbose: bool,
    dry_run: bool,
    recursive: bool,
    collision_cap: u32,
}

impl DetoxBuilder {
    /// Fresh builder with all defaults applied.
    #[must_use]
    pub fn new() -> Self {
        DetoxBuilder {
            sequence: Sequence::default(),
            verbose: false,
            dry_run: false,
            recursive: false,
            collision_cap: 1000,
        }
    }

    /// Set the active sequence (default [`Sequence::default()`]).
    #[must_use]
    pub fn sequence(mut self, s: Sequence) -> Self {
        self.sequence = s;
        self
    }

    /// Set verbose flag (default `false`).
    #[must_use]
    pub fn verbose(mut self, on: bool) -> Self {
        self.verbose = on;
        self
    }

    /// Set dry-run flag (default `false`).
    #[must_use]
    pub fn dry_run(mut self, on: bool) -> Self {
        self.dry_run = on;
        self
    }

    /// Set recursive flag (default `false`).
    #[must_use]
    pub fn recursive(mut self, on: bool) -> Self {
        self.recursive = on;
        self
    }

    /// Set the collision-resolution attempt cap (default `1000`).
    #[must_use]
    pub fn collision_cap(mut self, cap: u32) -> Self {
        self.collision_cap = cap;
        self
    }

    /// Build a configured [`Detox`]. INFALLIBLE.
    #[must_use]
    pub fn build(self) -> Detox {
        Detox {
            sequence: self.sequence,
            verbose: self.verbose,
            dry_run: self.dry_run,
            recursive: self.recursive,
            collision_cap: self.collision_cap,
        }
    }
}

impl Default for DetoxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A single source→target rename mapping (FR-041).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePlanEntry {
    /// Source path.
    pub source: PathBuf,
    /// Target path after the pipeline has been applied.
    pub target: PathBuf,
    /// Numeric collision-suffix applied (if any).
    pub collision_suffix: Option<u32>,
}

/// Summary report returned from [`Detox::execute`] (FR-042).
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DetoxReport {
    /// Number of entries the planner produced.
    pub planned: usize,
    /// Number of entries successfully renamed.
    pub renamed: usize,
    /// Number of entries skipped (e.g., dry-run or already-clean).
    pub skipped: usize,
    /// Number of entries that errored mid-execution.
    pub errored: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::{assert_impl_all, const_assert};

    assert_impl_all!(Detox: Send, Sync, Clone);
    assert_impl_all!(DetoxBuilder: Send, Sync, Clone);
    assert_impl_all!(Sequence: Send, Sync, Clone);
    assert_impl_all!(Filter: Send, Sync, Clone);
    assert_impl_all!(DetoxError: Send, Sync);

    // Compile-time check that DetoxReport has a Default impl.
    const _: fn() = || {
        let _ = DetoxReport::default();
    };
    const_assert!(std::mem::size_of::<RenamePlanEntry>() > 0);

    #[test]
    fn sanitize_default_sequence() {
        let detox = DetoxBuilder::new().build();
        assert_eq!(detox.sanitize("hello world.txt"), "hello_world.txt");
    }

    #[test]
    fn sanitize_utf8_sequence() {
        let detox = DetoxBuilder::new().sequence(Sequence::utf_8()).build();
        assert_eq!(detox.sanitize("café.pdf"), "cafe.pdf");
    }

    #[test]
    fn sanitize_bytes_parity_with_str_for_utf8_clean() {
        // SC-028: parity-by-construction for UTF-8-clean input
        let detox = DetoxBuilder::new().sequence(Sequence::utf_8()).build();
        let input = "café.pdf";
        assert_eq!(
            detox.sanitize(input).as_bytes(),
            detox.sanitize_bytes(input.as_bytes()).as_slice()
        );
    }

    #[test]
    fn clean_filename_unchanged() {
        let detox = DetoxBuilder::new().build();
        assert_eq!(detox.sanitize("clean_already.txt"), "clean_already.txt");
    }
}
