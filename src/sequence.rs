//! [`Sequence`] — ordered list of [`Filter`]s applied left-to-right (FR-008/FR-038).

use crate::filter::{DEFAULT_SEPARATOR, DEFAULT_UNSAFE_CHARS, Filter};

/// Ordered list of [`Filter`]s applied left-to-right to a name byte sequence.
///
/// Three built-in constructors mirror upstream's named sequences:
/// - [`Sequence::default`] (= `safe` + `wipeup`)
/// - [`Sequence::iso8859_1`] (= `iso8859_1` + `safe` + `wipeup`)
/// - [`Sequence::utf_8`] (= `utf_8` + `safe` + `wipeup`)
///
/// `Sequence::new()` + `push(Filter)` enables ad-hoc construction; `push`
/// consumes and returns owned `Self` so chained calls compile as builder-style
/// fluent code (FR-038 + clarification Q3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    pub(crate) filters: Vec<Filter>,
    pub(crate) name: String,
}

impl Sequence {
    /// Canonical inherent default constructor — returns the `default` sequence
    /// (`safe` + `wipeup`). [`impl Default for Sequence`] delegates here so
    /// both call sites produce identical values (FR-038).
    ///
    /// The inherent-method-with-same-name-as-trait pattern is a deliberate
    /// FR-038 design choice (clippy::should_implement_trait suppressed) so
    /// that `Sequence::default()` reads naturally as a named-sequence
    /// constructor parallel to `Sequence::utf_8()` and `Sequence::iso8859_1()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_detox::Sequence;
    /// let s = Sequence::default();
    /// assert_eq!(s.name(), "default");
    /// ```
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Sequence {
            name: "default".to_string(),
            filters: vec![
                Filter::Safe {
                    replacement: b'_',
                    unsafe_chars: DEFAULT_UNSAFE_CHARS.to_vec(),
                },
                Filter::Wipeup {
                    separator: DEFAULT_SEPARATOR,
                    remove_trailing: true,
                },
            ],
        }
    }

    /// Built-in `iso8859_1` sequence: `iso8859_1` + `safe` + `wipeup`.
    #[must_use]
    pub fn iso8859_1() -> Self {
        Sequence {
            name: "iso8859_1".to_string(),
            filters: vec![
                Filter::Iso8859_1,
                Filter::Safe {
                    replacement: b'_',
                    unsafe_chars: DEFAULT_UNSAFE_CHARS.to_vec(),
                },
                Filter::Wipeup {
                    separator: DEFAULT_SEPARATOR,
                    remove_trailing: true,
                },
            ],
        }
    }

    /// Built-in `utf_8` sequence: `utf_8` + `safe` + `wipeup`.
    #[must_use]
    pub fn utf_8() -> Self {
        Sequence {
            name: "utf_8".to_string(),
            filters: vec![
                Filter::Utf8,
                Filter::Safe {
                    replacement: b'_',
                    unsafe_chars: DEFAULT_UNSAFE_CHARS.to_vec(),
                },
                Filter::Wipeup {
                    separator: DEFAULT_SEPARATOR,
                    remove_trailing: true,
                },
            ],
        }
    }

    /// Empty named sequence — push filters with [`Sequence::push`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_detox::{Sequence, Filter};
    ///
    /// let seq = Sequence::new("custom")
    ///     .push(Filter::safe_default())
    ///     .push(Filter::wipeup_default());
    /// assert_eq!(seq.name(), "custom");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Sequence {
            name: name.into(),
            filters: Vec::new(),
        }
    }

    /// Append a filter and return the (consumed) sequence for fluent chaining.
    #[must_use]
    pub fn push(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Sequence name (used by `-L` listing and `-s` resolution).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Slice of filters in pipeline order.
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    /// Apply all filters in order to `input`. Returns the transformed bytes.
    pub(crate) fn apply(&self, input: &[u8]) -> Vec<u8> {
        let mut buf = input.to_vec();
        for filter in &self.filters {
            buf = filter.apply(&buf);
        }
        buf
    }
}

impl Default for Sequence {
    /// Trait default delegates to inherent [`Sequence::default()`] — both
    /// paths produce identical values (FR-038).
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_inherent_matches_trait_default() {
        let inherent = Sequence::default();
        let trait_d: Sequence = <Sequence as Default>::default();
        assert_eq!(inherent, trait_d);
    }

    #[test]
    fn default_sanitizes_space_to_underscore() {
        let s = Sequence::default();
        assert_eq!(s.apply(b"hello world.txt"), b"hello_world.txt");
    }

    #[test]
    fn utf_8_strips_e_acute() {
        let s = Sequence::utf_8();
        assert_eq!(s.apply("café résumé.pdf".as_bytes()), b"cafe_resume.pdf");
    }

    #[test]
    fn push_returns_self_for_chaining() {
        let s = Sequence::new("custom")
            .push(Filter::safe_default())
            .push(Filter::wipeup_default());
        assert_eq!(s.filters().len(), 2);
    }
}
