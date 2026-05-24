//! `utf_8` filter — UTF-8 codepoint to ASCII (FR-003).

use crate::tables::lookup_utf8;

/// Translate each UTF-8 codepoint via the vendored `Table.utf_8`. Codepoints
/// not in the table pass through as their original UTF-8 byte sequence
/// (FR-003); invalid UTF-8 fragments pass through byte-by-byte.
pub fn apply(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let first = input[i];
        if first < 0x80 {
            // ASCII fast path
            out.push(first);
            i += 1;
            continue;
        }
        // Decode one UTF-8 sequence (or pass an invalid byte through).
        let (cp, consumed) = decode_one(&input[i..]);
        if let (Some(cp), consumed) = (cp, consumed) {
            if let Some(repl) = lookup_utf8(cp) {
                out.extend_from_slice(repl);
            } else {
                // Pass through the original byte sequence.
                out.extend_from_slice(&input[i..i + consumed]);
            }
            i += consumed;
        } else {
            // Invalid byte — pass it through verbatim.
            out.push(first);
            i += 1;
        }
    }
    out
}

/// Decode one UTF-8 codepoint at the start of `bytes`. Returns
/// `(Some(cp), n)` on success where `n` is the number of bytes consumed, or
/// `(None, 0)` on invalid UTF-8.
fn decode_one(bytes: &[u8]) -> (Option<u32>, usize) {
    if bytes.is_empty() {
        return (None, 0);
    }
    let b = bytes[0];
    let (expected_len, mut cp): (usize, u32) = if b < 0x80 {
        return (Some(b as u32), 1);
    } else if (b & 0b1110_0000) == 0b1100_0000 {
        (2, (b & 0b0001_1111) as u32)
    } else if (b & 0b1111_0000) == 0b1110_0000 {
        (3, (b & 0b0000_1111) as u32)
    } else if (b & 0b1111_1000) == 0b1111_0000 {
        (4, (b & 0b0000_0111) as u32)
    } else {
        return (None, 0);
    };
    if bytes.len() < expected_len {
        return (None, 0);
    }
    for &cont in &bytes[1..expected_len] {
        if (cont & 0b1100_0000) != 0b1000_0000 {
            return (None, 0);
        }
        cp = (cp << 6) | ((cont & 0b0011_1111) as u32);
    }
    (Some(cp), expected_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_passes_through() {
        assert_eq!(apply(b"hello.txt"), b"hello.txt");
    }

    #[test]
    fn utf8_e_acute_to_e() {
        // U+00E9 = é = 0xC3 0xA9 in UTF-8
        assert_eq!(apply("café".as_bytes()), b"cafe");
    }

    #[test]
    fn unmapped_codepoint_passes_through_bytes() {
        // U+4E2D (中) not in our seed table → original bytes preserved
        let middle = "中".as_bytes();
        assert_eq!(apply(middle), middle);
    }

    #[test]
    fn invalid_utf8_byte_passes_through() {
        // 0xFF alone is not a valid start byte
        assert_eq!(apply(&[b'a', 0xFF, b'b']), &[b'a', 0xFF, b'b']);
    }
}
