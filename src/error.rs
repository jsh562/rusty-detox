//! Public error type for the rusty-detox library API (FR-042).

use std::path::PathBuf;

/// Error type returned by [`Detox`](crate::Detox) operations.
///
/// Pattern-matchable; carries structured payload (path + source-error where
/// applicable). The `#[non_exhaustive]` marker allows additive variants in
/// SemVer-minor releases.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum DetoxError {
    /// Filesystem I/O failure for a specific path.
    #[error("rusty-detox: I/O error on '{path}': {source}")]
    Io {
        /// Path being operated on when the error occurred.
        path: PathBuf,
        /// Underlying `std::io::Error`.
        #[source]
        source: std::io::Error,
    },

    /// Configuration file (`detoxrc`) could not be opened or was invalid.
    #[error("rusty-detox: config error at '{path}': {message}")]
    Config {
        /// Config file path (resolved or user-supplied).
        path: PathBuf,
        /// Human-readable explanation.
        message: String,
    },

    /// `detoxrc` parse error with line + column.
    #[error("rusty-detox: parse error at {path}:{line}:{column}: {message}")]
    Parse {
        /// Config file path.
        path: PathBuf,
        /// 1-indexed line number.
        line: usize,
        /// 1-indexed column number.
        column: usize,
        /// Parser diagnostic message.
        message: String,
    },

    /// Collision-resolution exhausted (more than `collision_cap` attempts).
    #[error("rusty-detox: cannot resolve collision for '{path}' ({attempts} attempts)")]
    Collision {
        /// Source path being detoxed.
        path: PathBuf,
        /// Number of suffix-disambiguation attempts before giving up.
        attempts: u32,
    },

    /// Cross-device rename (EXDEV) — the fallback chain also failed.
    #[error(
        "rusty-detox: cross-device rename failed from '{source_path}' to '{target}': {source_err}"
    )]
    CrossDevice {
        /// Source path.
        source_path: PathBuf,
        /// Intended target path on the other device.
        target: PathBuf,
        /// Underlying error from the fallback chain (copy/fsync/rename/unlink).
        #[source]
        source_err: std::io::Error,
    },

    /// Path was invalid for the active platform (e.g., contains characters not
    /// representable in the target OS filesystem).
    #[error("rusty-detox: invalid path '{path}': {reason}")]
    PathInvalid {
        /// Offending path.
        path: PathBuf,
        /// Human-readable explanation.
        reason: String,
    },
}
