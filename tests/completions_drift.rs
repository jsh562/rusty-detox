//! US7 (Shell Completions) drift gate (T125–T128).
//!
//! Asserts the committed `completions/` artifacts match what clap_complete
//! generates today. Regenerate via:
//!   cargo run -- completions bash > completions/rusty-detox.bash
//!   cargo run -- completions zsh > completions/_rusty-detox
//!   cargo run -- completions fish > completions/rusty-detox.fish
//!   cargo run -- completions powershell > completions/rusty-detox.ps1

mod common;

use std::fs;
use std::path::PathBuf;

fn committed(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("completions")
        .join(name);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("missing committed file {path:?}: {e}"));
    normalize(&bytes)
}

fn generate(shell: &str) -> Vec<u8> {
    let output = common::rusty_detox_cmd()
        .arg("completions")
        .arg(shell)
        .output()
        .expect("completions subcommand runs");
    assert!(
        output.status.success(),
        "completions {shell} exit code: {:?}",
        output.status
    );
    normalize(&output.stdout)
}

fn normalize(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().copied().filter(|b| *b != b'\r').collect()
}

#[test]
fn drift_bash() {
    assert_eq!(
        committed("rusty-detox.bash"),
        generate("bash"),
        "bash completion drift — regenerate with `cargo run -- completions bash > completions/rusty-detox.bash`"
    );
}

#[test]
fn drift_zsh() {
    assert_eq!(
        committed("_rusty-detox"),
        generate("zsh"),
        "zsh completion drift — regenerate"
    );
}

#[test]
fn drift_fish() {
    assert_eq!(
        committed("rusty-detox.fish"),
        generate("fish"),
        "fish completion drift — regenerate"
    );
}

#[test]
fn drift_powershell() {
    assert_eq!(
        committed("rusty-detox.ps1"),
        generate("powershell"),
        "powershell completion drift — regenerate"
    );
}

#[test]
fn strict_mode_does_not_emit_completion_script() {
    // SC-009 — Strict mode does NOT expose `completions` subcommand. The token
    // falls through to the path-processing loop as an unparseable positional;
    // the key assertion is that stdout is NOT a completion script.
    let assert = common::rusty_detox_cmd()
        .env_remove("RUSTY_DETOX_STRICT")
        .arg("--strict")
        .arg("completions")
        .arg("bash")
        .assert();
    let output = assert.get_output().clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("#!/")
            && !stdout.contains("complete -F")
            && !stdout.contains("_rusty_detox()"),
        "Strict mode must NOT emit a completion script; got {stdout:?}"
    );
}
