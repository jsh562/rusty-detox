//! US2 (Recursive batch rename) integration tests.

mod common;

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use rusty_detox::Sequence;
use rusty_detox::planner::{plan_directory, resolve_collision};
use tempfile::TempDir;

#[test]
fn recursive_renames_nested_tree() {
    let dir = TempDir::new().unwrap();
    let inner = dir.path().join("Album (2024)");
    fs::create_dir(&inner).unwrap();
    fs::write(inner.join("pic 1.jpg"), b"x").unwrap();
    fs::write(inner.join("pic 2.jpg"), b"x").unwrap();

    common::rusty_detox_cmd()
        .arg("-r")
        .arg(dir.path())
        .assert()
        .success();

    // Inner directory was renamed.
    let renamed_dir = dir.path().join("Album_2024");
    assert!(
        renamed_dir.exists(),
        "Album (2024) should rename to Album_2024"
    );
    // Inner files were renamed.
    assert!(renamed_dir.join("pic_1.jpg").exists());
    assert!(renamed_dir.join("pic_2.jpg").exists());
}

#[test]
fn recursive_dry_run_leaves_tree_intact() {
    let dir = TempDir::new().unwrap();
    let inner = dir.path().join("Album (2024)");
    fs::create_dir(&inner).unwrap();
    fs::write(inner.join("pic 1.jpg"), b"x").unwrap();

    let assert = common::rusty_detox_cmd()
        .arg("-r")
        .arg("-n")
        .arg(dir.path())
        .assert()
        .success();

    // Disk untouched.
    assert!(inner.exists(), "dry-run preserves source dir");
    assert!(inner.join("pic 1.jpg").exists(), "dry-run preserves files");

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(stdout.contains(" -> "), "dry-run prints planned renames");
}

#[test]
fn monotonic_collision_counter_sc029() {
    // Build 3 sources that all collapse to the same sanitized target. Multiple
    // spaces are collapsed by wipeup, so these all map to `a_b.txt`.
    let dir = TempDir::new().unwrap();
    let sources: Vec<PathBuf> = vec![
        dir.path().join("a b.txt"),
        dir.path().join("a  b.txt"),
        dir.path().join("a   b.txt"),
    ];
    for src in &sources {
        fs::write(src, b"x").unwrap();
    }

    let plan = plan_directory(&sources, &Sequence::default(), 1000).unwrap();
    assert_eq!(plan.len(), 3, "all 3 sources should produce plan entries");

    let suffixes: Vec<Option<u32>> = plan.iter().map(|e| e.collision_suffix).collect();
    // First source picks the bare target; subsequent two get _1 and _2.
    let collision_count = suffixes.iter().filter(|s| s.is_some()).count();
    assert_eq!(
        collision_count, 2,
        "expected exactly 2 collisions out of 3 sources; got suffixes: {suffixes:?}"
    );

    // Suffixes are monotonic — each new suffix is strictly greater than the previous.
    let actual_nums: Vec<u32> = suffixes.iter().filter_map(|s| *s).collect();
    for window in actual_nums.windows(2) {
        assert!(
            window[1] > window[0],
            "monotonic counter must increase; got {actual_nums:?}"
        );
    }
}

#[test]
fn resolve_collision_when_target_pre_exists() {
    let mut taken = BTreeSet::new();
    taken.insert(PathBuf::from("/tmp/foo.txt"));
    let (resolved, suffix) =
        resolve_collision(&PathBuf::from("/tmp/foo.txt"), &taken, 1000).unwrap();
    assert_eq!(resolved, PathBuf::from("/tmp/foo_1.txt"));
    assert_eq!(suffix, Some(1));
}

#[test]
fn deterministic_dry_run_output_sc002() {
    // Two consecutive dry-run invocations must produce byte-identical stdout
    // (SC-002 + HINT-008: planner uses BTreeMap-derived ordering).
    let dir = TempDir::new().unwrap();
    for name in &["b world.txt", "a hello.txt", "c third.txt"] {
        fs::write(dir.path().join(name), b"x").unwrap();
    }

    let first = common::rusty_detox_cmd()
        .arg("-r")
        .arg("-n")
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let second = common::rusty_detox_cmd()
        .arg("-r")
        .arg("-n")
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(first, second, "dry-run must be deterministic across runs");
}
