//! Hand-rolled argv parser for Strict-compat mode (AD-008).
//!
//! Implements upstream `detox(1)` flag semantics:
//! - Last-wins on repeated flags (FR-033)
//! - Grouped short flags (`-rv` = `-r -v`; FR-035)
//! - Byte-equal stderr for unknown-flag rejection (FR-032)
//! - No `completions` subcommand (FR-034)
//!
//! Used by `src/main.rs` when [`mode::resolve`](crate::mode::resolve) returns
//! [`CompatibilityMode::Strict`](crate::mode::CompatibilityMode::Strict).

/// Parsed Strict-mode arguments.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct StrictArgs {
    /// Dry-run flag (`-n`).
    pub dry_run: bool,
    /// Recursive flag (`-r`).
    pub recursive: bool,
    /// Verbose flag (`-v`).
    pub verbose: bool,
    /// `-L`: list sequences.
    pub list_sequences: bool,
    /// `-s <name>` sequence selector (last-wins).
    pub sequence: Option<String>,
    /// `-f <path>` config-file override (last-wins).
    pub config_file: Option<String>,
    /// Positional path arguments.
    pub paths: Vec<String>,
    /// Help flag (`-h`).
    pub help: bool,
    /// Version flag (`-V`).
    pub version: bool,
}

/// Errors returned from Strict-mode argv parsing.
#[derive(Debug, PartialEq, Eq)]
pub enum StrictParseError {
    /// Unknown short flag (e.g., `-Z`).
    UnknownFlag(char),
    /// Unknown long flag (e.g., `--unknown`).
    UnknownLongFlag(String),
    /// Flag expected an argument but reached end of argv.
    MissingArgument(String),
}

impl std::fmt::Display for StrictParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrictParseError::UnknownFlag(c) => {
                write!(f, "rusty-detox: invalid option -- '{c}'")
            }
            StrictParseError::UnknownLongFlag(s) => {
                write!(f, "rusty-detox: unrecognized option '--{s}'")
            }
            StrictParseError::MissingArgument(s) => {
                write!(f, "rusty-detox: option requires an argument -- '{s}'")
            }
        }
    }
}

impl std::error::Error for StrictParseError {}

/// Parse Strict-mode argv (argv[0] excluded).
///
/// Implements last-wins on repeated `-s` / `-f` flags (FR-033) and grouped
/// short flags (FR-035). Treats the `completions` token as a regular
/// positional argument (FR-034 — Strict mode does NOT expose subcommands).
pub fn parse(args: &[String]) -> Result<StrictArgs, StrictParseError> {
    let mut out = StrictArgs::default();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--" {
            // End-of-options; remaining args are paths.
            out.paths.extend(args[i + 1..].iter().cloned());
            break;
        } else if let Some(long) = arg.strip_prefix("--") {
            match long {
                "dry-run" => out.dry_run = true,
                "recursive" => out.recursive = true,
                "verbose" => out.verbose = true,
                "list-sequences" => out.list_sequences = true,
                "help" => out.help = true,
                "version" => out.version = true,
                "strict" | "no-strict" => {} // consumed already by mode::resolve
                "sequence" => {
                    i += 1;
                    out.sequence = Some(
                        args.get(i)
                            .cloned()
                            .ok_or_else(|| StrictParseError::MissingArgument("sequence".into()))?,
                    );
                }
                "config-file" => {
                    i += 1;
                    out.config_file =
                        Some(args.get(i).cloned().ok_or_else(|| {
                            StrictParseError::MissingArgument("config-file".into())
                        })?);
                }
                _ => return Err(StrictParseError::UnknownLongFlag(long.to_string())),
            }
        } else if let Some(shorts) = arg.strip_prefix('-') {
            if shorts.is_empty() {
                // Bare `-` is a positional path (stdin convention).
                out.paths.push(arg.clone());
            } else {
                // Grouped short flags: `-rv` = `-r -v` (FR-035)
                let mut chars = shorts.chars();
                while let Some(c) = chars.next() {
                    match c {
                        'n' => out.dry_run = true,
                        'r' => out.recursive = true,
                        'v' => out.verbose = true,
                        'L' => out.list_sequences = true,
                        'h' => out.help = true,
                        'V' => out.version = true,
                        's' => {
                            // `-s` takes an argument — either glued or next argv
                            let rest: String = chars.clone().collect();
                            if !rest.is_empty() {
                                out.sequence = Some(rest);
                                break;
                            }
                            i += 1;
                            out.sequence =
                                Some(args.get(i).cloned().ok_or_else(|| {
                                    StrictParseError::MissingArgument("s".into())
                                })?);
                            break;
                        }
                        'f' => {
                            let rest: String = chars.clone().collect();
                            if !rest.is_empty() {
                                out.config_file = Some(rest);
                                break;
                            }
                            i += 1;
                            out.config_file =
                                Some(args.get(i).cloned().ok_or_else(|| {
                                    StrictParseError::MissingArgument("f".into())
                                })?);
                            break;
                        }
                        unk => return Err(StrictParseError::UnknownFlag(unk)),
                    }
                }
            }
        } else {
            // Positional path — `completions` and friends end up here in Strict.
            out.paths.push(arg.clone());
        }
        i += 1;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouped_short_flags() {
        let p = parse(&["-rv".into()]).unwrap();
        assert!(p.recursive && p.verbose);
    }

    #[test]
    fn last_wins_sequence_flag() {
        let p = parse(&["-s".into(), "default".into(), "-s".into(), "utf_8".into()]).unwrap();
        assert_eq!(p.sequence.as_deref(), Some("utf_8"));
    }

    #[test]
    fn glued_short_flag_with_arg() {
        let p = parse(&["-sutf_8".into()]).unwrap();
        assert_eq!(p.sequence.as_deref(), Some("utf_8"));
    }

    #[test]
    fn unknown_short_flag_rejected() {
        let err = parse(&["-Z".into()]).unwrap_err();
        assert_eq!(err, StrictParseError::UnknownFlag('Z'));
    }

    #[test]
    fn unknown_long_flag_rejected() {
        let err = parse(&["--unknown".into()]).unwrap_err();
        assert_eq!(err, StrictParseError::UnknownLongFlag("unknown".into()));
    }

    #[test]
    fn completions_token_is_a_positional() {
        // FR-034: Strict mode treats `completions` as an unknown positional path
        let p = parse(&["completions".into(), "bash".into()]).unwrap();
        assert_eq!(p.paths, vec!["completions", "bash"]);
    }

    #[test]
    fn unknown_flag_message_format() {
        let err = StrictParseError::UnknownFlag('Z');
        assert_eq!(err.to_string(), "rusty-detox: invalid option -- 'Z'");
    }
}
