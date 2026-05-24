//! `max_length` filter — extension-preserving truncation (FR-006).

/// Truncate `input` to `limit` bytes while preserving the final extension
/// token (everything after the last `.`).
///
/// If the input is shorter than `limit`, it is returned unchanged. If the
/// extension alone is longer than `limit`, the entire extension is preserved
/// and the basename is truncated to zero — the result will still exceed
/// `limit`, but the extension semantics are intact (matches upstream).
pub fn apply(input: &[u8], limit: usize) -> Vec<u8> {
    if input.len() <= limit {
        return input.to_vec();
    }
    // Find the LAST '.' in the input — that's the extension separator.
    if let Some(dot_pos) = input.iter().rposition(|&b| b == b'.') {
        let ext = &input[dot_pos..]; // includes the dot
        if ext.len() < limit {
            let keep_base = limit - ext.len();
            let mut out = Vec::with_capacity(limit);
            out.extend_from_slice(&input[..keep_base]);
            out.extend_from_slice(ext);
            return out;
        }
        // Extension is larger than the limit — preserve it as-is.
        return ext.to_vec();
    }
    // No extension — just truncate.
    input[..limit].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_truncation_when_under_limit() {
        assert_eq!(apply(b"short.txt", 255), b"short.txt");
    }

    #[test]
    fn truncates_preserving_extension() {
        let long = b"abcdefghij.txt"; // 14 bytes; limit = 10 → keep ".txt" (4) + 6 base
        assert_eq!(apply(long, 10), b"abcdef.txt");
    }

    #[test]
    fn no_dot_just_truncates() {
        assert_eq!(apply(b"abcdefghij", 5), b"abcde");
    }

    #[test]
    fn last_dot_wins_for_extension_picker() {
        // foo.tar.gz: extension picker takes ".gz"
        assert_eq!(apply(b"foo.tar.gz", 6), b"foo.gz");
    }
}
