//! Default-mode CLI (clap-derive) (FR-014..FR-022).

use clap::{Parser, Subcommand};

/// Sanitize filenames through the configured filter pipeline. A Rust port of
/// Doug Harple's `detox(1)`.
#[derive(Parser, Debug)]
#[command(name = "rusty-detox", about, version, arg_required_else_help = false)]
pub struct Cli {
    /// Dry-run: plan and report renames without issuing any rename syscalls.
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// Recursive: descend into directories depth-first leaves-up.
    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    /// Sequence to apply (default: `default`).
    #[arg(short = 's', long = "sequence", value_name = "NAME")]
    pub sequence: Option<String>,

    /// Override config-file resolution with an explicit path.
    #[arg(short = 'f', long = "config-file", value_name = "PATH")]
    pub config_file: Option<String>,

    /// List all loaded sequence names to stdout, one per line.
    #[arg(short = 'L', long = "list-sequences")]
    pub list_sequences: bool,

    /// Verbose: emit one rename line per change to stdout (FR-019).
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Activate Strict-compat mode (byte-equal upstream stderr, last-wins flag resolution).
    #[arg(long = "strict")]
    pub strict: bool,

    /// Disable Strict-compat mode even when argv[0] or env would activate it.
    #[arg(long = "no-strict", conflicts_with = "strict")]
    pub no_strict: bool,

    /// Optional subcommand.
    #[command(subcommand)]
    pub subcommand: Option<DetoxSubcommand>,

    /// Paths to detox (zero or more).
    pub paths: Vec<String>,
}

/// Subcommands exposed in Default mode only (FR-021/FR-034).
#[derive(Subcommand, Debug)]
pub enum DetoxSubcommand {
    /// Emit a shell-completion script to stdout.
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
