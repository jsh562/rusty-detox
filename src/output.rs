//! Rename-line formatter (FR-032, AD-016).
//!
//! Byte-exact format: `<src>` SPACE `->` SPACE `<tgt>` LF, identical in
//! Default and Strict modes. No ANSI color codes are emitted on stdout in
//! either mode in v0.1.0.

use std::fmt::Write as _;
use std::path::Path;

/// Format one source→target rename line per FR-032.
///
/// Returns a UTF-8 string ending with `\n`. Path components are rendered via
/// `Path::display()` (lossy on non-UTF-8) — acceptable because the planner
/// always produces UTF-8 targets through the filter pipeline.
pub fn rename_line(source: &Path, target: &Path) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{} -> {}", source.display(), target.display());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn format_uses_space_arrow_space_lf() {
        let line = rename_line(
            &PathBuf::from("hello world.txt"),
            &PathBuf::from("hello_world.txt"),
        );
        assert_eq!(line, "hello world.txt -> hello_world.txt\n");
        assert!(line.ends_with('\n'));
        assert_eq!(line.matches(" -> ").count(), 1);
    }
}
