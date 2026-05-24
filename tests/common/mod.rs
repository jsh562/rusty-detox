//! Shared fixture helpers for integration tests (T036).

use assert_cmd::Command;

/// Command for the `rusty-detox` binary.
pub fn rusty_detox_cmd() -> Command {
    Command::cargo_bin("rusty-detox").expect("binary built")
}

/// Command for the `inline-detox` companion binary (cfg by `inline-detox` feature).
#[allow(dead_code)]
pub fn inline_detox_cmd() -> Command {
    Command::cargo_bin("inline-detox").expect("binary built (requires --features inline-detox)")
}
