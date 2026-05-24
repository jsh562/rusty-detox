//! US1 (Default sanitization + dry-run preview) integration tests.

mod common;

use std::fs;
use tempfile::TempDir;

#[test]
fn dry_run_emits_arrow_separated_line() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("hello world.txt");
    fs::write(&src, b"x").unwrap();

    let assert = common::rusty_detox_cmd()
        .arg("-n")
        .arg(&src)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(
        stdout.contains("hello world.txt")
            && stdout.contains("hello_world.txt")
            && stdout.contains(" -> "),
        "stdout was: {stdout:?}"
    );

    // File on disk is unchanged.
    assert!(src.exists(), "dry-run must not rename");
}

#[test]
fn verbose_no_dry_run_renames_and_prints() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("hello world.txt");
    let tgt = dir.path().join("hello_world.txt");
    fs::write(&src, b"x").unwrap();

    let assert = common::rusty_detox_cmd()
        .arg("-v")
        .arg(&src)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(stdout.contains(" -> "), "verbose stdout: {stdout:?}");
    assert!(!src.exists(), "source should be renamed");
    assert!(tgt.exists(), "target should exist");
}

#[test]
fn clean_filename_silent() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("clean_already.txt");
    fs::write(&src, b"x").unwrap();

    let assert = common::rusty_detox_cmd()
        .arg("-n")
        .arg(&src)
        .assert()
        .success();
    let stdout = assert.get_output().stdout.clone();
    assert!(stdout.is_empty(), "clean name produces no output");
}

#[test]
fn empty_argv_usage_error() {
    common::rusty_detox_cmd().assert().failure();
}

#[test]
fn utf8_sequence_french_resume_dry_run() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("Résumé.pdf");
    fs::write(&src, b"x").unwrap();

    let assert = common::rusty_detox_cmd()
        .arg("-n")
        .arg("-s")
        .arg("utf_8")
        .arg(&src)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(
        stdout.contains("Resume.pdf"),
        "utf_8 stdout should contain transliterated target; got: {stdout:?}"
    );
}

#[test]
fn list_sequences_prints_all_three() {
    let assert = common::rusty_detox_cmd().arg("-L").assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(stdout.contains("default"));
    assert!(stdout.contains("iso8859_1"));
    assert!(stdout.contains("utf_8"));
}

#[test]
fn unknown_sequence_errors() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("foo.txt");
    fs::write(&src, b"x").unwrap();

    common::rusty_detox_cmd()
        .arg("-s")
        .arg("nosuchsequence")
        .arg(&src)
        .assert()
        .failure();
}

#[test]
fn version_flag_prints_version() {
    let assert = common::rusty_detox_cmd()
        .arg("--version")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(stdout.contains("rusty-detox"), "got: {stdout:?}");
}
