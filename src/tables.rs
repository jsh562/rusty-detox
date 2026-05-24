//! Vendored translation tables from upstream `dharple/detox` v3.0.1 (HINT-001).
//!
//! Sorted-by-codepoint static slices for binary-search lookup. Frozen at v3.0.1;
//! re-vendoring is a MAJOR semver bump in rusty-detox per `docs/DESIGN.md` §
//! "Vendored Translation Tables".
//!
//! Upstream sources:
//!   - `Table.utf_8` (large UTF-8 → ASCII map)
//!   - `Table.iso8859_1` (Latin-1 → ASCII map, 96 entries for 0xA0–0xFF)
//!
//! The full UTF-8 table will be auto-generated from upstream by a `cargo xtask
//! gen-tables` command (T023 placeholder). For v0.1.0 scaffold this module
//! ships a small seed covering the most common Latin-1 + Latin-Extended-A
//! code points; the seed is sufficient for SC-001 acceptance against the
//! v0.1.0 fixture corpus but is NOT byte-equal with upstream for arbitrary
//! input. Full table vendor-in is tracked as iter-2 work.

/// Map of Unicode codepoint → ASCII replacement bytes.
///
/// Sorted by codepoint for `binary_search_by_key` lookup. Empty replacement
/// means "delete this codepoint from the output" (rare; upstream `utf_8`
/// normally maps to non-empty ASCII).
pub static UTF_8: &[(u32, &[u8])] = &[
    // Latin-1 Supplement (0x00A0–0x00FF) — seed for v0.1.0.
    (0x00A0, b" "), // NBSP → space
    (0x00A1, b"!"),
    (0x00A2, b"c"), // ¢
    (0x00A3, b"L"), // £
    (0x00A5, b"Y"), // ¥
    (0x00A7, b"S"), // §
    (0x00A9, b"(C)"),
    (0x00AB, b"<<"),
    (0x00AD, b"-"), // soft hyphen
    (0x00AE, b"(R)"),
    (0x00B0, b"o"), // °
    (0x00B1, b"+-"),
    (0x00B5, b"u"), // µ
    (0x00B7, b"."),
    (0x00BB, b">>"),
    (0x00BF, b"?"),
    // Latin-1 letters (Uppercase A-class)
    (0x00C0, b"A"),
    (0x00C1, b"A"),
    (0x00C2, b"A"),
    (0x00C3, b"A"),
    (0x00C4, b"A"),
    (0x00C5, b"A"),
    (0x00C6, b"AE"),
    (0x00C7, b"C"),
    (0x00C8, b"E"),
    (0x00C9, b"E"),
    (0x00CA, b"E"),
    (0x00CB, b"E"),
    (0x00CC, b"I"),
    (0x00CD, b"I"),
    (0x00CE, b"I"),
    (0x00CF, b"I"),
    (0x00D0, b"D"),
    (0x00D1, b"N"),
    (0x00D2, b"O"),
    (0x00D3, b"O"),
    (0x00D4, b"O"),
    (0x00D5, b"O"),
    (0x00D6, b"O"),
    (0x00D8, b"O"),
    (0x00D9, b"U"),
    (0x00DA, b"U"),
    (0x00DB, b"U"),
    (0x00DC, b"U"),
    (0x00DD, b"Y"),
    (0x00DE, b"Th"),
    (0x00DF, b"ss"),
    // Latin-1 letters (Lowercase)
    (0x00E0, b"a"),
    (0x00E1, b"a"),
    (0x00E2, b"a"),
    (0x00E3, b"a"),
    (0x00E4, b"a"),
    (0x00E5, b"a"),
    (0x00E6, b"ae"),
    (0x00E7, b"c"),
    (0x00E8, b"e"),
    (0x00E9, b"e"),
    (0x00EA, b"e"),
    (0x00EB, b"e"),
    (0x00EC, b"i"),
    (0x00ED, b"i"),
    (0x00EE, b"i"),
    (0x00EF, b"i"),
    (0x00F0, b"d"),
    (0x00F1, b"n"),
    (0x00F2, b"o"),
    (0x00F3, b"o"),
    (0x00F4, b"o"),
    (0x00F5, b"o"),
    (0x00F6, b"o"),
    (0x00F8, b"o"),
    (0x00F9, b"u"),
    (0x00FA, b"u"),
    (0x00FB, b"u"),
    (0x00FC, b"u"),
    (0x00FD, b"y"),
    (0x00FF, b"y"),
];

/// Map of Latin-1 high byte (0x80–0xFF) → ASCII replacement bytes.
///
/// Direct 256-entry-style table; index by `byte as usize - 0x80` to get the
/// `Option<&'static [u8]>` replacement. `None` means "pass through unchanged"
/// (FR-002). Entries below 0xA0 are control codes — upstream maps these
/// conservatively; we map them all to None (pass-through) and rely on `safe`
/// to handle control bytes if needed.
pub static ISO8859_1: &[Option<&[u8]>] = &[
    // 0x80–0x9F — C1 control codes; pass through (None)
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    // 0xA0–0xFF — printable Latin-1
    Some(b" "),
    Some(b"!"),
    Some(b"c"),
    Some(b"L"),
    None,
    Some(b"Y"),
    None,
    Some(b"S"),
    None,
    Some(b"(C)"),
    None,
    Some(b"<<"),
    None,
    Some(b"-"),
    Some(b"(R)"),
    None,
    Some(b"o"),
    Some(b"+-"),
    None,
    None,
    None,
    Some(b"u"),
    None,
    Some(b"."),
    None,
    None,
    None,
    Some(b">>"),
    None,
    None,
    None,
    Some(b"?"),
    Some(b"A"),
    Some(b"A"),
    Some(b"A"),
    Some(b"A"),
    Some(b"A"),
    Some(b"A"),
    Some(b"AE"),
    Some(b"C"),
    Some(b"E"),
    Some(b"E"),
    Some(b"E"),
    Some(b"E"),
    Some(b"I"),
    Some(b"I"),
    Some(b"I"),
    Some(b"I"),
    Some(b"D"),
    Some(b"N"),
    Some(b"O"),
    Some(b"O"),
    Some(b"O"),
    Some(b"O"),
    Some(b"O"),
    None,
    Some(b"O"),
    Some(b"U"),
    Some(b"U"),
    Some(b"U"),
    Some(b"U"),
    Some(b"Y"),
    Some(b"Th"),
    Some(b"ss"),
    Some(b"a"),
    Some(b"a"),
    Some(b"a"),
    Some(b"a"),
    Some(b"a"),
    Some(b"a"),
    Some(b"ae"),
    Some(b"c"),
    Some(b"e"),
    Some(b"e"),
    Some(b"e"),
    Some(b"e"),
    Some(b"i"),
    Some(b"i"),
    Some(b"i"),
    Some(b"i"),
    Some(b"d"),
    Some(b"n"),
    Some(b"o"),
    Some(b"o"),
    Some(b"o"),
    Some(b"o"),
    Some(b"o"),
    None,
    Some(b"o"),
    Some(b"u"),
    Some(b"u"),
    Some(b"u"),
    Some(b"u"),
    Some(b"y"),
    None,
    Some(b"y"),
];

/// Look up an ASCII replacement for a Unicode codepoint via binary search in `UTF_8`.
///
/// Returns `None` for unmapped codepoints (caller passes the original UTF-8
/// bytes through, per FR-003).
pub fn lookup_utf8(cp: u32) -> Option<&'static [u8]> {
    UTF_8
        .binary_search_by_key(&cp, |&(c, _)| c)
        .ok()
        .map(|i| UTF_8[i].1)
}

/// Look up an ASCII replacement for a Latin-1 high byte (0x80–0xFF).
///
/// Returns `None` for control bytes and bytes without a mapping (caller passes
/// the byte through, per FR-002).
pub fn lookup_iso8859_1(byte: u8) -> Option<&'static [u8]> {
    if byte < 0x80 {
        return None;
    }
    ISO8859_1[(byte - 0x80) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_is_sorted_by_codepoint() {
        let mut prev = 0u32;
        for &(cp, _) in UTF_8 {
            assert!(
                cp > prev,
                "UTF_8 must be sorted ascending; {cp:#x} <= {prev:#x}"
            );
            prev = cp;
        }
    }

    #[test]
    fn iso8859_1_table_length() {
        assert_eq!(
            ISO8859_1.len(),
            128,
            "ISO8859_1 covers 0x80..=0xFF (128 entries)"
        );
    }

    #[test]
    fn latin1_resume_e_acute() {
        // é (U+00E9) → "e"
        assert_eq!(lookup_utf8(0x00E9), Some(b"e".as_slice()));
        assert_eq!(lookup_iso8859_1(0xE9), Some(b"e".as_slice()));
    }

    #[test]
    fn unmapped_returns_none() {
        // CJK character not in our seed table
        assert_eq!(lookup_utf8(0x4E2D), None);
    }

    #[test]
    fn ascii_returns_none_for_iso() {
        assert_eq!(lookup_iso8859_1(0x41), None); // 'A'
    }
}
