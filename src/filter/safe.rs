//! `safe` filter — replace shell-hostile bytes with `replacement` (FR-004).

/// Replace each byte in `input` matching `unsafe_chars` with `replacement`.
pub fn apply(input: &[u8], replacement: u8, unsafe_chars: &[u8]) -> Vec<u8> {
    input
        .iter()
        .map(|&b| {
            if unsafe_chars.contains(&b) {
                replacement
            } else {
                b
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::DEFAULT_UNSAFE_CHARS;

    #[test]
    fn spaces_become_underscores_with_default_set() {
        assert_eq!(
            apply(b"hello world.txt", b'_', DEFAULT_UNSAFE_CHARS),
            b"hello_world.txt"
        );
    }

    #[test]
    fn parens_become_underscores() {
        assert_eq!(
            apply(b"foo (1).pdf", b'_', DEFAULT_UNSAFE_CHARS),
            b"foo__1_.pdf"
        );
    }

    #[test]
    fn decoded_slash_becomes_underscore_q10() {
        // FR-004 + clarification Q10: `/` is in the default unsafe set
        assert_eq!(apply(b"a/b", b'_', DEFAULT_UNSAFE_CHARS), b"a_b");
    }

    #[test]
    fn custom_replacement_byte() {
        assert_eq!(apply(b"a b", b'-', b" "), b"a-b");
    }
}
