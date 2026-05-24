//! `rusty-detox` binary entry point.

use std::io::{self, Write};
use std::path::Path;

use clap::{CommandFactory, Parser};
use rusty_detox::{DetoxBuilder, Sequence};

mod cli;
mod mode;
mod output;
mod strict;

use cli::{Cli, DetoxSubcommand};
use mode::CompatibilityMode;

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let active_mode = mode::resolve(&argv);

    let exit_code = match active_mode {
        CompatibilityMode::Default => run_default(),
        CompatibilityMode::Strict => run_strict(&argv[1..]),
    };
    std::process::exit(exit_code);
}

fn run_default() -> i32 {
    let cli = Cli::parse();

    // Subcommand path: completions.
    if let Some(DetoxSubcommand::Completions { shell }) = cli.subcommand {
        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        clap_complete::generate(shell, &mut cmd, name, &mut io::stdout());
        return 0;
    }

    if cli.list_sequences {
        for name in &["default", "iso8859_1", "utf_8"] {
            println!("{name}");
        }
        return 0;
    }

    if cli.paths.is_empty() {
        eprintln!("rusty-detox: no input paths");
        return 1;
    }

    let sequence = match cli.sequence.as_deref() {
        Some("default") | None => Sequence::default(),
        Some("iso8859_1") => Sequence::iso8859_1(),
        Some("utf_8") => Sequence::utf_8(),
        Some(other) => {
            eprintln!("rusty-detox: sequence '{other}' not found");
            return 1;
        }
    };

    let detox = DetoxBuilder::new()
        .sequence(sequence)
        .verbose(cli.verbose)
        .dry_run(cli.dry_run)
        .recursive(cli.recursive)
        .build();

    let mut had_error = false;
    for path_str in &cli.paths {
        let path = Path::new(path_str);
        let plan = detox.plan(path);
        for entry in &plan {
            if cli.dry_run || cli.verbose {
                let line = output::rename_line(&entry.source, &entry.target);
                let _ = io::stdout().write_all(line.as_bytes());
            }
        }
        if !cli.dry_run {
            for entry in &plan {
                if let Err(e) = std::fs::rename(&entry.source, &entry.target) {
                    eprintln!("rusty-detox: {}: {e}", entry.source.display());
                    had_error = true;
                }
            }
        }
    }
    if had_error { 1 } else { 0 }
}

fn run_strict(args: &[String]) -> i32 {
    let parsed = match strict::parse(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    if parsed.help {
        eprintln!("Usage: rusty-detox [OPTIONS] [PATHS...]");
        return 0;
    }
    if parsed.version {
        println!("rusty-detox {}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    if parsed.list_sequences {
        for name in &["default", "iso8859_1", "utf_8"] {
            println!("{name}");
        }
        return 0;
    }
    if parsed.paths.is_empty() {
        eprintln!("rusty-detox: no input paths");
        return 1;
    }

    let sequence = match parsed.sequence.as_deref() {
        Some("default") | None => Sequence::default(),
        Some("iso8859_1") => Sequence::iso8859_1(),
        Some("utf_8") => Sequence::utf_8(),
        Some(other) => {
            eprintln!("rusty-detox: sequence '{other}' not found");
            return 1;
        }
    };

    let detox = DetoxBuilder::new()
        .sequence(sequence)
        .verbose(parsed.verbose)
        .dry_run(parsed.dry_run)
        .recursive(parsed.recursive)
        .build();

    let mut had_error = false;
    for path_str in &parsed.paths {
        let path = Path::new(path_str);
        let plan = detox.plan(path);
        for entry in &plan {
            if parsed.dry_run || parsed.verbose {
                let line = output::rename_line(&entry.source, &entry.target);
                let _ = io::stdout().write_all(line.as_bytes());
            }
        }
        if !parsed.dry_run {
            for entry in &plan {
                if let Err(e) = std::fs::rename(&entry.source, &entry.target) {
                    eprintln!("rusty-detox: {}: {e}", entry.source.display());
                    had_error = true;
                }
            }
        }
    }
    if had_error { 1 } else { 0 }
}
