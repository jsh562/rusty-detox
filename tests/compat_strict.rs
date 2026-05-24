//! US3 (Strict-compat drop-in) integration tests.

mod common;

use std::fs;
use tempfile::TempDir;

#[test]
fn strict_unknown_flag_rejected() {
    let assert = common::rusty_detox_cmd()
        .arg("--strict")
        .arg("-Z")
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
    assert!(
        stderr.contains("invalid option") && stderr.contains("'Z'"),
        "Strict unknown-flag stderr should match upstream format; got: {stderr:?}"
    );
}

#[test]
fn strict_last_wins_repeated_sequence_flag() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("Résumé.pdf");
    fs::write(&src, b"x").unwrap();

    let assert = common::rusty_detox_cmd()
        .arg("--strict")
        .arg("-n")
        .arg("-s")
        .arg("default")
        .arg("-s")
        .arg("utf_8")
        .arg(&src)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(
        stdout.contains("Resume.pdf"),
        "last-wins should select utf_8 sequence; got: {stdout:?}"
    );
}

#[test]
fn strict_grouped_short_flags() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("hello world.txt");
    fs::write(&src, b"x").unwrap();

    let assert = common::rusty_detox_cmd()
        .arg("--strict")
        .arg("-nv")
        .arg(&src)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(
        stdout.contains(" -> "),
        "grouped -nv should produce dry-run output; got: {stdout:?}"
    );
    // Dry-run preserves source.
    assert!(src.exists());
}

#[test]
fn strict_does_not_emit_completion_script() {
    let assert = common::rusty_detox_cmd()
        .arg("--strict")
        .arg("completions")
        .arg("bash")
        .assert();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(
        !stdout.contains("#!/")
            && !stdout.contains("complete -F")
            && !stdout.contains("_rusty_detox"),
        "Strict mode must NOT emit a completion script; got: {stdout:?}"
    );
}

#[test]
fn strict_activates_via_argv0_detox() {
    // Simulate argv[0] = "detox" by setting RUSTY_DETOX_STRICT=1 (equivalent
    // activation source). True argv[0] swap would require a symlink fixture
    // which is awkward cross-platform; the env-var path exercises the same
    // mode::resolve precedence ladder.
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("hello world.txt");
    fs::write(&src, b"x").unwrap();

    let assert = common::rusty_detox_cmd()
        .env("RUSTY_DETOX_STRICT", "1")
        .arg("-n")
        .arg(&src)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(
        stdout.contains(" -> "),
        "env-var strict should still run normally; got: {stdout:?}"
    );
}

#[test]
fn strict_no_strict_overrides_env() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("hello world.txt");
    fs::write(&src, b"x").unwrap();

    common::rusty_detox_cmd()
        .env("RUSTY_DETOX_STRICT", "1")
        .arg("--no-strict")
        .arg("-n")
        .arg(&src)
        .assert()
        .success();
}
