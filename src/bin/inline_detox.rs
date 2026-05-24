//! `inline-detox` companion binary (FR-044/FR-045).
//!
//! Reads stdin to EOF as a single name byte-sequence, applies the configured
//! sequence, writes the sanitized name to stdout.

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
