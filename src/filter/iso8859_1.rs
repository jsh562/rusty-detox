//! `iso8859_1` filter — Latin-1 high byte to ASCII (FR-002).

use crate::tables::lookup_iso8859_1;

/// Translate each high byte (0x80–0xFF) via the vendored `Table.iso8859_1`.
/// Bytes below 0x80 and bytes without a mapping pass through unchanged.
pub fn apply(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    for &b in input {
        if b < 0x80 {
            out.push(b);
        } else if let Some(repl) = lookup_iso8859_1(b) {
            out.extend_from_slice(repl);
        } else {
            out.push(b);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_passes_through() {
        assert_eq!(apply(b"hello.txt"), b"hello.txt");
    }

    #[test]
    fn latin1_e_acute_to_e() {
        // 0xE9 = é in Latin-1
        assert_eq!(apply(&[b'r', 0xE9, b's']), b"res");
    }

    #[test]
    fn latin1_capital_aelig_to_ae() {
        // 0xC6 = Æ in Latin-1
        assert_eq!(apply(&[0xC6, b'O', b'N']), b"AEON");
    }
}
