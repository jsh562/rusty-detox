//! US5 (inline-detox companion binary) integration tests (T106–T110).

mod common;

#[test]
fn inline_default_sequence_sanitizes_space() {
    let assert = common::inline_detox_cmd()
        .write_stdin("hello world.txt")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert_eq!(stdout, "hello_world.txt");
}

#[test]
fn inline_utf8_sequence_transliterates() {
    let assert = common::inline_detox_cmd()
        .arg("-s")
        .arg("utf_8")
        .write_stdin("café.pdf")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert_eq!(stdout, "cafe.pdf");
}

#[test]
fn inline_empty_stdin_produces_empty_stdout() {
    let assert = common::inline_detox_cmd()
        .write_stdin("")
        .assert()
        .success();
    assert!(assert.get_output().stdout.is_empty());
}

#[test]
fn inline_unknown_sequence_errors() {
    common::inline_detox_cmd()
        .arg("-s")
        .arg("nosuchsequence")
        .write_stdin("foo.txt")
        .assert()
        .failure();
}

#[test]
fn inline_matches_rusty_detox_sanitization() {
    // SC-013 — inline-detox produces identical output to rusty-detox's
    // single-file sanitization for the same input under the same sequence.
    let inline = common::inline_detox_cmd()
        .arg("-s")
        .arg("utf_8")
        .write_stdin("My Résumé.pdf")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inline_str = String::from_utf8_lossy(&inline).into_owned();
    // rusty-detox -n via dry-run produces "<src> -> <tgt>\n"; extract the target.
    let detox = common::rusty_detox_cmd()
        .arg("-n")
        .arg("-s")
        .arg("utf_8")
        .arg("My Résumé.pdf")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let detox_str = String::from_utf8_lossy(&detox).into_owned();
    let extracted_target = detox_str
        .split(" -> ")
        .nth(1)
        .map(|s| s.trim_end_matches('\n'))
        .unwrap_or("");
    assert_eq!(inline_str, extracted_target);
}
