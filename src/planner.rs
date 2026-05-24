//! Rename planner (FR-023/FR-024, AD-011, SC-029).
//!
//! Pre-computes the full source→target mapping for a batch of entries, with
//! monotonic `_N` suffix disambiguation for collisions. The suffix is inserted
//! BEFORE the final extension token (Q1 + AD-011).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::RenamePlanEntry;
use crate::error::DetoxError;
use crate::sequence::Sequence;

/// Plan the rename(s) for a batch of paths sharing the same parent directory.
///
/// `entries` is the list of source paths (e.g., from the walker). `sequence`
/// is applied to each basename. Collisions are resolved with a monotonic
/// counter scoped to this batch: the first collision picks `_1`, the second
/// `_2`, etc. (SC-029). Returns a vector of `RenamePlanEntry` with
/// `collision_suffix` set when disambiguation was applied.
pub fn plan_directory(
    entries: &[PathBuf],
    sequence: &Sequence,
    collision_cap: u32,
) -> Result<Vec<RenamePlanEntry>, DetoxError> {
    let mut plan = Vec::new();
    let mut taken: BTreeSet<PathBuf> = BTreeSet::new();

    // Pre-load `taken` with siblings that already exist but are NOT in
    // `entries` (otherwise an unrelated existing file could be silently
    // overwritten).
    for src in entries {
        if let Some(parent) = src.parent() {
            if parent.exists() {
                for sibling in std::fs::read_dir(parent).into_iter().flatten().flatten() {
                    taken.insert(sibling.path());
                }
            }
        }
    }

    for src in entries {
        let basename = src
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let target_bytes = sequence.apply(basename.as_bytes());
        let target_name = String::from_utf8_lossy(&target_bytes).into_owned();
        if target_name == basename {
            // Already clean — skip; remove from `taken` set since the source
            // itself doesn't claim the slot for someone else.
            taken.insert(src.clone());
            continue;
        }
        let target = src.with_file_name(&target_name);
        // Source itself is allowed to "occupy" its own slot — remove it from
        // taken so collision detection only fires for foreign occupants.
        taken.remove(src);
        let (final_target, suffix) = resolve_collision(&target, &taken, collision_cap)?;
        taken.insert(final_target.clone());
        plan.push(RenamePlanEntry {
            source: src.clone(),
            target: final_target,
            collision_suffix: suffix,
        });
    }

    Ok(plan)
}

/// Resolve a target collision by appending `_N` suffix before the final
/// extension. Monotonic counter starts at 1.
///
/// Returns `(resolved_target, suffix_applied)`. Suffix is `None` when no
/// collision occurred.
pub fn resolve_collision(
    target: &Path,
    taken: &BTreeSet<PathBuf>,
    cap: u32,
) -> Result<(PathBuf, Option<u32>), DetoxError> {
    if !taken.contains(target) {
        return Ok((target.to_path_buf(), None));
    }

    let parent = target.parent();
    let name = target
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (stem, ext) = split_extension(&name);

    for n in 1..=cap {
        let candidate_name = match &ext {
            Some(e) => format!("{stem}_{n}.{e}"),
            None => format!("{stem}_{n}"),
        };
        let candidate = match parent {
            Some(p) => p.join(&candidate_name),
            None => PathBuf::from(&candidate_name),
        };
        if !taken.contains(&candidate) {
            return Ok((candidate, Some(n)));
        }
    }
    Err(DetoxError::Collision {
        path: target.to_path_buf(),
        attempts: cap,
    })
}

/// Split `name` into `(stem, Option<extension>)` at the LAST `.` boundary
/// (matching `max_length` extension-preservation philosophy).
fn split_extension(name: &str) -> (String, Option<String>) {
    if let Some(dot) = name.rfind('.') {
        // Leading-dot files (".bashrc") have no extension — treat the whole
        // name as stem.
        if dot == 0 {
            return (name.to_string(), None);
        }
        let stem = name[..dot].to_string();
        let ext = name[dot + 1..].to_string();
        (stem, Some(ext))
    } else {
        (name.to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_collision_returns_original() {
        let taken = BTreeSet::new();
        let (res, suf) = resolve_collision(Path::new("foo.txt"), &taken, 1000).unwrap();
        assert_eq!(res, PathBuf::from("foo.txt"));
        assert_eq!(suf, None);
    }

    #[test]
    fn collision_suffix_before_extension() {
        let mut taken = BTreeSet::new();
        taken.insert(PathBuf::from("foo.txt"));
        let (res, suf) = resolve_collision(Path::new("foo.txt"), &taken, 1000).unwrap();
        assert_eq!(res, PathBuf::from("foo_1.txt"));
        assert_eq!(suf, Some(1));
    }

    #[test]
    fn monotonic_counter_sc029() {
        let mut taken = BTreeSet::new();
        taken.insert(PathBuf::from("foo.txt"));
        taken.insert(PathBuf::from("foo_1.txt"));
        taken.insert(PathBuf::from("foo_2.txt"));
        let (res, suf) = resolve_collision(Path::new("foo.txt"), &taken, 1000).unwrap();
        assert_eq!(res, PathBuf::from("foo_3.txt"));
        assert_eq!(suf, Some(3));
    }

    #[test]
    fn no_extension_appends_underscore_n() {
        let mut taken = BTreeSet::new();
        taken.insert(PathBuf::from("README"));
        let (res, _) = resolve_collision(Path::new("README"), &taken, 1000).unwrap();
        assert_eq!(res, PathBuf::from("README_1"));
    }

    #[test]
    fn split_extension_last_dot_wins() {
        assert_eq!(
            split_extension("foo.tar.gz"),
            ("foo.tar".into(), Some("gz".into()))
        );
        assert_eq!(split_extension("foo"), ("foo".into(), None));
        assert_eq!(split_extension(".bashrc"), (".bashrc".into(), None));
    }

    #[test]
    fn collision_cap_exceeded_returns_err() {
        let mut taken = BTreeSet::new();
        taken.insert(PathBuf::from("foo.txt"));
        for n in 1..=5 {
            taken.insert(PathBuf::from(format!("foo_{n}.txt")));
        }
        let err = resolve_collision(Path::new("foo.txt"), &taken, 5).unwrap_err();
        assert!(matches!(err, DetoxError::Collision { attempts: 5, .. }));
    }
}
