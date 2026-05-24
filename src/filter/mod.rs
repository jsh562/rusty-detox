//! The Filter pipeline (FR-001..FR-007 + FR-039).
//!
//! Each `Filter` variant is a single-step byte-sequence transformation. A
//! [`Sequence`](crate::Sequence) is an ordered `Vec<Filter>` consumed
//! left-to-right when [`Detox::sanitize`](crate::Detox::sanitize) runs.

pub mod iso8859_1;
pub mod max_length;
pub mod safe;
pub mod safe_platform;
pub mod uncgi;
pub mod utf8;
pub mod wipeup;

/// Default unsafe-character set for [`Filter::Safe`] when callers want the
/// upstream-compatible default. Includes path-separator byte `/` per
/// clarification Q10 + FR-004.
pub const DEFAULT_UNSAFE_CHARS: &[u8] = b" ()[]{}<>\'\"!@#$&*?;|\\/\x7f";

/// Default separator byte for [`Filter::Wipeup`] (the byte that runs are
/// collapsed and trimmed of). Matches upstream's `_` default.
pub const DEFAULT_SEPARATOR: u8 = b'_';

/// One transformation step in the [`Sequence`](crate::Sequence) pipeline.
///
/// `#[non_exhaustive]` is required (FR-039) so SemVer-minor releases can add
/// new variants such as a future `--transliterate=deunicode` opt-in.
///
/// # Construction shortcut
///
/// For [`Filter::Safe`] with the default unsafe-character set, prefer
/// [`Filter::safe_default()`] over enumerating the byte set manually.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// Decode CGI percent-escapes (`%XX` → single byte). FR-001.
    Uncgi,
    /// Translate Latin-1 high bytes (0x80–0xFF) to ASCII via the vendored
    /// `Table.iso8859_1`. FR-002.
    Iso8859_1,
    /// Translate UTF-8 codepoints to ASCII via the vendored `Table.utf_8`.
    /// Unmapped codepoints pass through. FR-003.
    Utf8,
    /// Replace each unsafe-set byte with `replacement`. FR-004.
    Safe {
        /// Replacement byte (default `b'_'`).
        replacement: u8,
        /// Bytes considered unsafe. See [`DEFAULT_UNSAFE_CHARS`] for the
        /// v0.1.0 default; callers MAY pass any byte set.
        unsafe_chars: Vec<u8>,
    },
    /// Collapse runs of `separator` into one occurrence; when
    /// `remove_trailing` is true, also trim leading/trailing runs. FR-005.
    Wipeup {
        /// Separator byte (default [`DEFAULT_SEPARATOR`]).
        separator: u8,
        /// When true, trim leading and trailing runs of `separator`.
        remove_trailing: bool,
    },
    /// Truncate to `limit` bytes while preserving the final extension token
    /// (everything after the last `.`). FR-006.
    MaxLength {
        /// Maximum total byte length of the basename.
        limit: usize,
    },
    /// Rewrite Windows-reserved device names (CON, PRN, AUX, NUL, COM1–9,
    /// LPT1–9) by suffixing the basename with `_`, and rewrite Windows-
    /// reserved characters (`< > : " | ? *`) and ASCII control bytes using
    /// the same replacement as [`Filter::Safe`]. FR-007.
    ///
    /// Auto-enabled on Windows builds; opt-in elsewhere via a sequence entry.
    SafePlatform,
}

impl Filter {
    /// Construct a [`Filter::Safe`] with the v0.1.0 default unsafe-character
    /// set and `b'_'` replacement (FR-004 + clarification Q10). Convenience
    /// constructor for callers who want the upstream-compatible default
    /// without enumerating the byte set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_detox::Filter;
    ///
    /// let safe = Filter::safe_default();
    /// assert!(matches!(safe, Filter::Safe { replacement: b'_', .. }));
    /// ```
    pub fn safe_default() -> Self {
        Filter::Safe {
            replacement: b'_',
            unsafe_chars: DEFAULT_UNSAFE_CHARS.to_vec(),
        }
    }

    /// Construct a [`Filter::Wipeup`] with `b'_'` separator and trailing
    /// trimming enabled (matches upstream's `default` sequence).
    pub fn wipeup_default() -> Self {
        Filter::Wipeup {
            separator: DEFAULT_SEPARATOR,
            remove_trailing: true,
        }
    }

    /// Apply this single filter to `input`, returning the transformed bytes.
    pub fn apply(&self, input: &[u8]) -> Vec<u8> {
        match self {
            Filter::Uncgi => uncgi::apply(input),
            Filter::Iso8859_1 => iso8859_1::apply(input),
            Filter::Utf8 => utf8::apply(input),
            Filter::Safe {
                replacement,
                unsafe_chars,
            } => safe::apply(input, *replacement, unsafe_chars),
            Filter::Wipeup {
                separator,
                remove_trailing,
            } => wipeup::apply(input, *separator, *remove_trailing),
            Filter::MaxLength { limit } => max_length::apply(input, *limit),
            Filter::SafePlatform => safe_platform::apply(input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_default_matches_fr004() {
        match Filter::safe_default() {
            Filter::Safe {
                replacement,
                unsafe_chars,
            } => {
                assert_eq!(replacement, b'_');
                assert!(unsafe_chars.contains(&b' '));
                assert!(unsafe_chars.contains(&b'/')); // Q10
                assert!(unsafe_chars.contains(&b'('));
            }
            _ => panic!("safe_default must return Filter::Safe"),
        }
    }

    #[test]
    fn wipeup_default_collapses_underscores() {
        let f = Filter::wipeup_default();
        assert_eq!(f.apply(b"a__b___c"), b"a_b_c");
    }
}
