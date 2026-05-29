//! `inline-detox` companion binary (FR-044/FR-045).
//!
//! Reads stdin to EOF as a single name byte-sequence, applies the configured
//! sequence, writes the sanitized name to stdout.
//!
//! Gated at the file level by `#[cfg(feature = "inline-detox")]` (in
//! addition to `required-features = ["inline-detox"]` in Cargo.toml) so the
//! portfolio-wide feature-lint phantom-leaf sub-check (FR-008 / FR-052
//! sub-rule 3) recognizes the v0.1.x `inline-detox` feature as
//! source-gated. The `required-features` attribute is the binding gate for
//! the binary entry; this `cfg` is a no-op for build purposes (the binary
//! is only compiled when the feature is enabled either way).

#![cfg(feature = "inline-detox")]

use std::io::{self, Read, Write};

use clap::Parser;
use rusty_detox::{DetoxBuilder, Sequence};

/// Stream-detox a single name from stdin to stdout.
#[derive(Parser, Debug)]
#[command(name = "inline-detox", about, version)]
struct Args {
    /// Sequence to apply (default `default`).
    #[arg(short = 's', long = "sequence", value_name = "NAME")]
    sequence: Option<String>,

    /// Override config-file resolution (currently ignored in v0.1.0 — see US6).
    #[arg(short = 'f', long = "config-file", value_name = "PATH")]
    _config_file: Option<String>,
}

fn main() {
    let args = Args::parse();
    let sequence = match args.sequence.as_deref() {
        Some("default") | None => Sequence::default(),
        Some("iso8859_1") => Sequence::iso8859_1(),
        Some("utf_8") => Sequence::utf_8(),
        Some(other) => {
            eprintln!("inline-detox: sequence '{other}' not found");
            std::process::exit(1);
        }
    };
    let detox = DetoxBuilder::new().sequence(sequence).build();

    let mut buf = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut buf) {
        eprintln!("inline-detox: {e}");
        std::process::exit(1);
    }
    let out = detox.sanitize_bytes(&buf);
    if let Err(e) = io::stdout().write_all(&out) {
        eprintln!("inline-detox: {e}");
        std::process::exit(1);
    }
}
