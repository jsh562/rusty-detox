//! Recursive directory walker (FR-026/FR-027/FR-028, AD-014, HINT-002).
//!
//! Wraps `walkdir` with the required ordering policy:
//! - `contents_first(true)` — leaves-up so parent renames don't invalidate child paths
//! - `follow_links(false)` — symlinks-to-directories are NOT descended (FR-027)
//!
//! Filters out non-regular non-directory file types (sockets, char/block
//! devices, named pipes) per FR-028.

use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

/// One entry yielded from the walker.
#[derive(Debug, Clone)]
pub struct WalkEntry {
    /// Absolute or root-relative path of this entry.
    pub path: PathBuf,
    /// `true` if this is a directory; `false` for regular files.
    pub is_dir: bool,
}

/// Recursively walk `root` depth-first leaves-up.
///
/// Returns a `Vec<WalkEntry>` of regular files and directories under `root`.
/// Symlinks-to-directories are NOT followed (FR-027); non-regular/non-directory
/// entries (sockets/char/block/FIFOs) are silently filtered out (FR-028).
///
/// The walker collects into a `Vec` rather than streaming because the planner
/// needs the full per-directory list to compute collision-safe targets in one
/// pass.
pub fn recursive_walk(root: &Path) -> Vec<WalkEntry> {
    let mut entries = Vec::new();
    for result in WalkDir::new(root)
        .contents_first(true)
        .follow_links(false)
        .into_iter()
    {
        match result {
            Ok(entry) => {
                if is_regular_or_dir(&entry) {
                    entries.push(WalkEntry {
                        path: entry.path().to_path_buf(),
                        is_dir: entry.file_type().is_dir(),
                    });
                }
            }
            Err(_) => {
                // Skip unreadable entries silently (matches upstream).
                continue;
            }
        }
    }
    entries
}

fn is_regular_or_dir(entry: &DirEntry) -> bool {
    let ft = entry.file_type();
    ft.is_file() || ft.is_dir() || ft.is_symlink()
}
