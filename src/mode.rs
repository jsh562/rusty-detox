//! Compatibility-mode precedence ladder (FR-031, AD-007).
//!
//! Active mode is computed once at startup from (high → low):
//! 1. `--strict` flag on argv
//! 2. `RUSTY_DETOX_STRICT=1` env var
//! 3. argv[0] basename matches `detox` / `detox-alias`
//!
//! `--no-strict` overrides all three lower-precedence sources.

/// Whether to apply Default-mode extensions or Strict upstream parity.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompatibilityMode {
    /// Ergonomic Default mode (clap-styled error messages, conflicts_with).
    #[default]
    Default,
    /// Strict-compat mode (byte-equal upstream stderr, last-wins flag resolution).
    Strict,
}

/// Resolve the active mode from `argv` and environment per FR-031.
///
/// `argv[0]` is the binary invocation name; remaining args may contain
/// `--strict` or `--no-strict`. The `RUSTY_DETOX_STRICT` env var is checked
/// via `std::env::var`.
pub fn resolve(argv: &[String]) -> CompatibilityMode {
    // Explicit --no-strict wins over everything.
    if argv.iter().any(|a| a == "--no-strict") {
        return CompatibilityMode::Default;
    }
    // 1. --strict flag
    if argv.iter().any(|a| a == "--strict") {
        return CompatibilityMode::Strict;
    }
    // 2. env var
    if std::env::var("RUSTY_DETOX_STRICT").as_deref() == Ok("1") {
        return CompatibilityMode::Strict;
    }
    // 3. argv[0] basename
    if let Some(first) = argv.first() {
        let base = basename(first);
        if base.eq_ignore_ascii_case("detox") || base.eq_ignore_ascii_case("detox-alias") {
            return CompatibilityMode::Strict;
        }
    }
    CompatibilityMode::Default
}

/// Extract the basename from a path-like string. Cross-platform: strips
/// Windows `.exe` suffix and handles both `\` and `/` separators.
pub fn basename(path: &str) -> &str {
    let last = path
        .rsplit_once(['/', '\\'])
        .map(|(_, b)| b)
        .unwrap_or(path);
    last.strip_suffix(".exe").unwrap_or(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_strict_flag() {
        let argv = vec!["rusty-detox".into(), "--strict".into()];
        assert_eq!(resolve(&argv), CompatibilityMode::Strict);
    }

    #[test]
    fn no_strict_overrides() {
        let argv = vec![
            "rusty-detox".into(),
            "--strict".into(),
            "--no-strict".into(),
        ];
        assert_eq!(resolve(&argv), CompatibilityMode::Default);
    }

    #[test]
    fn argv0_detox_triggers_strict() {
        let argv = vec!["/usr/local/bin/detox".into()];
        assert_eq!(resolve(&argv), CompatibilityMode::Strict);
    }

    #[test]
    fn argv0_detox_alias_triggers_strict() {
        let argv = vec!["detox-alias".into()];
        assert_eq!(resolve(&argv), CompatibilityMode::Strict);
    }

    #[test]
    fn argv0_rusty_detox_is_default() {
        let argv = vec!["rusty-detox".into()];
        assert_eq!(resolve(&argv), CompatibilityMode::Default);
    }

    #[test]
    fn basename_strips_exe_suffix() {
        assert_eq!(basename("C:\\bin\\detox.exe"), "detox");
        assert_eq!(basename("/usr/bin/detox"), "detox");
        assert_eq!(basename("detox"), "detox");
    }
}
