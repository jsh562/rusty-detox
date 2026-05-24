//! `detoxrc` config-file parser (FR-009/FR-010/FR-011, AD-010, US6).
//!
//! Hand-rolled recursive-descent parser for the upstream v3.0.1 grammar. Zero
//! parser-crate dependency to preserve `default-features = false` zero-CLI-dep
//! promise (AD-010).
//!
//! Grammar (subset):
//!
//! ```text
//! config       := (sequence-block | comment | whitespace)*
//! sequence-block := "sequence" ident "{" filter-entry+ "}"
//! filter-entry := "filter" ident (";" | "{" param+ "}")
//! param        := ident "=" value ";"
//! value        := single-quoted-byte
//! comment      := "#" .*? newline
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::DetoxError;
use crate::filter::{DEFAULT_SEPARATOR, DEFAULT_UNSAFE_CHARS, Filter};
use crate::sequence::Sequence;

/// Parsed `detoxrc` representation. Maps sequence-name → `Sequence`.
#[derive(Debug, Clone, Default)]
pub struct DetoxConfig {
    /// User-defined and built-in sequences keyed by name.
    pub sequences: BTreeMap<String, Sequence>,
}

impl DetoxConfig {
    /// Build a config containing only the three built-in sequences (`default`,
    /// `iso8859_1`, `utf_8`). Used as the fallback when no `detoxrc` is found.
    #[must_use]
    pub fn built_in() -> Self {
        let mut sequences = BTreeMap::new();
        sequences.insert("default".to_string(), Sequence::default());
        sequences.insert("iso8859_1".to_string(), Sequence::iso8859_1());
        sequences.insert("utf_8".to_string(), Sequence::utf_8());
        DetoxConfig { sequences }
    }

    /// Look up a sequence by name. Returns `None` when not present.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Sequence> {
        self.sequences.get(name)
    }

    /// List loaded sequence names alphabetically. Used by `-L`/`--list-sequences`.
    #[must_use]
    pub fn list_names(&self) -> Vec<&str> {
        self.sequences.keys().map(String::as_str).collect()
    }
}

/// Resolve the active config-file path per FR-009 precedence (first match wins).
///
/// CLI override > `$XDG_CONFIG_HOME/detoxrc` > `~/.detoxrc` > `/etc/detoxrc`.
/// Returns `None` when no candidate exists; caller falls back to built-in
/// sequences.
pub fn resolve_path(cli_override: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = cli_override {
        return Some(p.to_path_buf());
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let candidate = PathBuf::from(xdg).join("detoxrc");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    if let Some(home) = home_dir() {
        let candidate = home.join(".detoxrc");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    let etc = PathBuf::from("/etc/detoxrc");
    if etc.exists() {
        return Some(etc);
    }
    None
}

fn home_dir() -> Option<PathBuf> {
    // Cross-platform: prefer USERPROFILE on Windows, HOME elsewhere.
    if let Ok(v) = std::env::var("HOME") {
        if !v.is_empty() {
            return Some(PathBuf::from(v));
        }
    }
    if let Ok(v) = std::env::var("USERPROFILE") {
        if !v.is_empty() {
            return Some(PathBuf::from(v));
        }
    }
    None
}

/// Parse `detoxrc` source text into a [`DetoxConfig`]. Three built-in
/// sequences are always present; user-defined sequences supplement (or
/// override on name collision).
pub fn parse(src: &str, path: &Path) -> Result<DetoxConfig, DetoxError> {
    let mut p = Parser::new(src, path);
    let mut cfg = DetoxConfig::built_in();
    p.skip_trivia();
    while !p.eof() {
        if p.starts_with("sequence") {
            let (name, seq) = p.parse_sequence()?;
            cfg.sequences.insert(name, seq);
        } else {
            return Err(p.error(format!(
                "expected 'sequence' block, found {:?}",
                p.peek_line()
            )));
        }
        p.skip_trivia();
    }
    Ok(cfg)
}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
    path: PathBuf,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str, path: &Path) -> Self {
        Parser {
            src: src.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
            path: path.to_path_buf(),
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(b)
    }

    fn starts_with(&self, kw: &str) -> bool {
        self.src[self.pos..].starts_with(kw.as_bytes())
    }

    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(b) if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' => {
                    self.bump();
                }
                Some(b'#') => {
                    while let Some(b) = self.peek() {
                        if b == b'\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                _ => break,
            }
        }
    }

    fn expect(&mut self, kw: &str) -> Result<(), DetoxError> {
        if self.starts_with(kw) {
            for _ in 0..kw.len() {
                self.bump();
            }
            Ok(())
        } else {
            Err(self.error(format!("expected '{kw}', found {:?}", self.peek_line())))
        }
    }

    fn expect_byte(&mut self, b: u8) -> Result<(), DetoxError> {
        self.skip_trivia();
        if self.peek() == Some(b) {
            self.bump();
            Ok(())
        } else {
            Err(self.error(format!(
                "expected '{}', found {:?}",
                b as char,
                self.peek().map(|c| c as char)
            )))
        }
    }

    fn parse_ident(&mut self) -> Result<String, DetoxError> {
        self.skip_trivia();
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                self.bump();
            } else {
                break;
            }
        }
        if start == self.pos {
            return Err(self.error("expected identifier".into()));
        }
        Ok(String::from_utf8_lossy(&self.src[start..self.pos]).into_owned())
    }

    fn parse_single_quoted_byte(&mut self) -> Result<u8, DetoxError> {
        self.skip_trivia();
        if self.peek() != Some(b'\'') {
            return Err(self.error("expected single-quoted byte literal".into()));
        }
        self.bump();
        let b = self
            .bump()
            .ok_or_else(|| self.error("unterminated byte literal".into()))?;
        if self.peek() != Some(b'\'') {
            return Err(self.error("expected closing single quote".into()));
        }
        self.bump();
        Ok(b)
    }

    fn parse_sequence(&mut self) -> Result<(String, Sequence), DetoxError> {
        self.expect("sequence")?;
        let name = self.parse_ident()?;
        self.expect_byte(b'{')?;
        let mut seq = Sequence::new(&name);
        loop {
            self.skip_trivia();
            if self.peek() == Some(b'}') {
                self.bump();
                break;
            }
            let filter = self.parse_filter_entry()?;
            seq = seq.push(filter);
        }
        Ok((name, seq))
    }

    fn parse_filter_entry(&mut self) -> Result<Filter, DetoxError> {
        self.expect("filter")?;
        let name = self.parse_ident()?;
        self.skip_trivia();
        let mut params: BTreeMap<String, u8> = BTreeMap::new();
        match self.peek() {
            Some(b'{') => {
                self.bump();
                loop {
                    self.skip_trivia();
                    if self.peek() == Some(b'}') {
                        self.bump();
                        break;
                    }
                    let key = self.parse_ident()?;
                    self.expect_byte(b'=')?;
                    let value = self.parse_single_quoted_byte()?;
                    self.expect_byte(b';')?;
                    params.insert(key, value);
                }
                self.skip_trivia();
                if self.peek() == Some(b';') {
                    self.bump();
                }
            }
            Some(b';') => {
                self.bump();
            }
            _ => {}
        }
        self.build_filter(&name, params)
    }

    fn build_filter(&self, name: &str, params: BTreeMap<String, u8>) -> Result<Filter, DetoxError> {
        match name {
            "uncgi" => Ok(Filter::Uncgi),
            "iso8859_1" => Ok(Filter::Iso8859_1),
            "utf_8" => Ok(Filter::Utf8),
            "safe" => Ok(Filter::Safe {
                replacement: params.get("replace").copied().unwrap_or(b'_'),
                unsafe_chars: DEFAULT_UNSAFE_CHARS.to_vec(),
            }),
            "wipeup" => Ok(Filter::Wipeup {
                separator: params.get("sep").copied().unwrap_or(DEFAULT_SEPARATOR),
                remove_trailing: true,
            }),
            "max_length" => Ok(Filter::MaxLength {
                limit: params.get("limit").copied().map(usize::from).unwrap_or(255),
            }),
            "safe_platform" => Ok(Filter::SafePlatform),
            other => Err(DetoxError::Parse {
                path: self.path.clone(),
                line: self.line,
                column: self.col,
                message: format!("unknown filter '{other}'"),
            }),
        }
    }

    fn peek_line(&self) -> String {
        let rest = &self.src[self.pos..];
        let end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
        String::from_utf8_lossy(&rest[..end.min(40)]).into_owned()
    }

    fn error(&self, message: String) -> DetoxError {
        DetoxError::Parse {
            path: self.path.clone(),
            line: self.line,
            column: self.col,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_has_three_sequences() {
        let cfg = DetoxConfig::built_in();
        assert!(cfg.get("default").is_some());
        assert!(cfg.get("iso8859_1").is_some());
        assert!(cfg.get("utf_8").is_some());
    }

    #[test]
    fn parse_empty_returns_built_ins_only() {
        let cfg = parse("", Path::new("test")).unwrap();
        assert_eq!(cfg.list_names().len(), 3);
    }

    #[test]
    fn parse_comment_skipped() {
        let cfg = parse("# this is a comment\n# another\n", Path::new("test")).unwrap();
        assert_eq!(cfg.list_names().len(), 3);
    }

    #[test]
    fn parse_custom_sequence() {
        let src = r#"
sequence url_style {
    filter safe { replace = '-'; }
    filter wipeup;
}
"#;
        let cfg = parse(src, Path::new("test")).unwrap();
        let seq = cfg.get("url_style").expect("custom sequence parsed");
        assert_eq!(seq.filters().len(), 2);
        match &seq.filters()[0] {
            Filter::Safe { replacement, .. } => assert_eq!(*replacement, b'-'),
            _ => panic!("expected Safe filter"),
        }
    }

    #[test]
    fn parse_syntax_error_returns_line_col() {
        let src = "not_a_sequence_keyword";
        let err = parse(src, Path::new("test")).unwrap_err();
        match err {
            DetoxError::Parse { line, column, .. } => {
                assert_eq!(line, 1);
                assert!(column >= 1);
            }
            _ => panic!("expected DetoxError::Parse, got {err:?}"),
        }
    }

    #[test]
    fn parse_unknown_filter_errors() {
        let src = "sequence x { filter nosuchfilter; }";
        let err = parse(src, Path::new("test")).unwrap_err();
        match err {
            DetoxError::Parse { message, .. } => {
                assert!(message.contains("unknown filter"));
            }
            _ => panic!("expected Parse error"),
        }
    }
}
