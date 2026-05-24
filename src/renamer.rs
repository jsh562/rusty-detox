//! Atomic rename with EXDEV cross-device fallback (FR-025/FR-030, HINT-003).
//!
//! Default path: `std::fs::rename`. On `ErrorKind::CrossesDevices` (Rust 1.85
//! stable), fall back to: create temp file on target's filesystem → copy
//! bytes → fsync → rename-into-place → unlink source. Metadata copy
//! (mode bits + mtime) is best-effort per FR-030.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::DetoxError;

/// Rename `src` to `tgt`, falling back through the EXDEV chain on
/// cross-device errors.
pub fn rename_with_fallback(src: &Path, tgt: &Path) -> Result<(), DetoxError> {
    match std::fs::rename(src, tgt) {
        Ok(_) => Ok(()),
        Err(e) if is_cross_device(&e) => exdev_fallback(src, tgt),
        Err(e) => Err(DetoxError::Io {
            path: src.to_path_buf(),
            source: e,
        }),
    }
}

fn is_cross_device(e: &std::io::Error) -> bool {
    // Rust 1.85 stable: ErrorKind::CrossesDevices.
    matches!(e.kind(), std::io::ErrorKind::CrossesDevices)
}

fn exdev_fallback(src: &Path, tgt: &Path) -> Result<(), DetoxError> {
    let parent = tgt.parent().unwrap_or_else(|| Path::new("."));
    let tmp = unique_temp_path(parent);

    // Copy bytes from src into the temp file.
    {
        let mut reader = std::fs::File::open(src).map_err(|e| DetoxError::CrossDevice {
            source_path: src.to_path_buf(),
            target: tgt.to_path_buf(),
            source_err: e,
        })?;
        let mut writer = std::fs::File::create(&tmp).map_err(|e| DetoxError::CrossDevice {
            source_path: src.to_path_buf(),
            target: tgt.to_path_buf(),
            source_err: e,
        })?;
        if let Err(e) = std::io::copy(&mut reader, &mut writer) {
            let _ = std::fs::remove_file(&tmp);
            return Err(DetoxError::CrossDevice {
                source_path: src.to_path_buf(),
                target: tgt.to_path_buf(),
                source_err: e,
            });
        }
        if let Err(e) = writer.flush() {
            let _ = std::fs::remove_file(&tmp);
            return Err(DetoxError::CrossDevice {
                source_path: src.to_path_buf(),
                target: tgt.to_path_buf(),
                source_err: e,
            });
        }
        if let Err(e) = writer.sync_all() {
            let _ = std::fs::remove_file(&tmp);
            return Err(DetoxError::CrossDevice {
                source_path: src.to_path_buf(),
                target: tgt.to_path_buf(),
                source_err: e,
            });
        }
    }

    // Atomically rename temp → target (within target filesystem).
    if let Err(e) = std::fs::rename(&tmp, tgt) {
        let _ = std::fs::remove_file(&tmp);
        return Err(DetoxError::CrossDevice {
            source_path: src.to_path_buf(),
            target: tgt.to_path_buf(),
            source_err: e,
        });
    }

    // Best-effort metadata copy.
    let _ = copy_metadata(src, tgt);

    // Unlink the original.
    if let Err(e) = std::fs::remove_file(src) {
        eprintln!(
            "rusty-detox: warning: rename succeeded but could not unlink source '{}': {e}",
            src.display()
        );
    }
    Ok(())
}

fn unique_temp_path(parent: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    parent.join(format!(".rusty-detox-tmp-{pid}-{nanos}"))
}

/// Best-effort metadata copy from `src` to `tgt`. Per-field failures emit a
/// stderr warning but do NOT abort the operation (FR-030).
pub fn copy_metadata(src: &Path, tgt: &Path) -> Result<(), DetoxError> {
    let meta = std::fs::metadata(src).map_err(|e| DetoxError::Io {
        path: src.to_path_buf(),
        source: e,
    })?;
    // Permissions.
    if let Err(e) = std::fs::set_permissions(tgt, meta.permissions()) {
        eprintln!(
            "rusty-detox: warning: could not preserve permissions on '{}': {e}",
            tgt.display()
        );
    }
    // Timestamps. std::fs::FileTimes stable since 1.75.
    if let (Ok(mtime), Ok(atime)) = (meta.modified(), meta.accessed()) {
        let times = std::fs::FileTimes::new()
            .set_modified(mtime)
            .set_accessed(atime);
        if let Ok(tgt_file) = std::fs::File::options().write(true).open(tgt) {
            if let Err(e) = tgt_file.set_times(times) {
                eprintln!(
                    "rusty-detox: warning: could not preserve timestamps on '{}': {e}",
                    tgt.display()
                );
            }
        }
    }
    Ok(())
}
