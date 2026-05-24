//! `wipeup` filter — collapse runs of `separator`; optionally trim ends (FR-005).

/// Collapse consecutive runs of `separator` into a single occurrence. When
/// `remove_trailing` is true, also trim leading and trailing runs entirely.
pub fn apply(input: &[u8], separator: u8, remove_trailing: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut prev_was_sep = false;
    for &b in input {
        if b == separator {
            if !prev_was_sep {
                out.push(b);
                prev_was_sep = true;
            }
        } else {
            out.push(b);
            prev_was_sep = false;
        }
    }
    if remove_trailing {
        // Trim leading and trailing separator bytes
        let start = out
            .iter()
            .position(|&b| b != separator)
            .unwrap_or(out.len());
        let end = out
            .iter()
            .rposition(|&b| b != separator)
            .map(|i| i + 1)
            .unwrap_or(0);
        if start < end {
            out = out[start..end].to_vec();
        } else {
            out.clear();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_runs() {
        assert_eq!(apply(b"a__b___c", b'_', false), b"a_b_c");
    }

    #[test]
    fn trims_leading_and_trailing_when_true() {
        assert_eq!(apply(b"__hello_world__", b'_', true), b"hello_world");
    }

    #[test]
    fn does_not_trim_when_false() {
        assert_eq!(apply(b"__hello_world__", b'_', false), b"_hello_world_");
    }

    #[test]
    fn all_separators_with_trim_becomes_empty() {
        assert_eq!(apply(b"____", b'_', true), b"");
    }
}
