//! `uncgi` filter — decode `%XX` percent-escapes (FR-001).

/// Decode ASCII percent-escapes in `input`. `%XX` (two hex digits) becomes the
/// corresponding single byte. Unrecognized `%`-sequences pass through.
pub fn apply(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'%' && i + 2 < input.len() {
            if let (Some(hi), Some(lo)) = (hex(input[i + 1]), hex(input[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(input[i]);
        i += 1;
    }
    out
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_percent_20_to_space() {
        assert_eq!(apply(b"hello%20world"), b"hello world");
    }

    #[test]
    fn decodes_percent_2f_to_slash() {
        assert_eq!(apply(b"a%2Fb"), b"a/b");
        assert_eq!(apply(b"a%2fb"), b"a/b");
    }

    #[test]
    fn passes_through_no_escapes() {
        assert_eq!(apply(b"plain.txt"), b"plain.txt");
    }

    #[test]
    fn unrecognized_escape_passes_through() {
        // %ZZ — invalid hex; pass through verbatim
        assert_eq!(apply(b"a%ZZb"), b"a%ZZb");
    }

    #[test]
    fn trailing_percent_at_end_passes_through() {
        assert_eq!(apply(b"foo%"), b"foo%");
        assert_eq!(apply(b"foo%A"), b"foo%A");
    }
}
