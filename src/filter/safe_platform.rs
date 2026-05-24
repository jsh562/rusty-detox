//! `safe_platform` filter — Windows-reserved-name rewriter (FR-007 + FR-046/047).

const RESERVED_BASES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

const RESERVED_CHARS: &[u8] = b"<>:\"|?*";

/// Rewrite Windows-reserved names and characters.
///
/// `CON.txt` → `CON_.txt` (basename suffixed with `_` before the extension).
/// `<>:"|?*` and ASCII control bytes (0x00–0x1F) → `_` (matches default `safe`
/// replacement byte).
pub fn apply(input: &[u8]) -> Vec<u8> {
    // First pass: replace reserved chars + control bytes.
    let mut out: Vec<u8> = input
        .iter()
        .map(|&b| {
            if RESERVED_CHARS.contains(&b) || b < 0x20 {
                b'_'
            } else {
                b
            }
        })
        .collect();

    // Second pass: rewrite reserved device names (case-insensitive, with or
    // without extension). Operate only on the basename portion.
    let dot_pos = out.iter().position(|&b| b == b'.');
    let basename_end = dot_pos.unwrap_or(out.len());
    let basename = &out[..basename_end];
    if let Ok(name) = std::str::from_utf8(basename) {
        if RESERVED_BASES.iter().any(|r| name.eq_ignore_ascii_case(r)) {
            // Insert `_` at the end of the basename.
            let mut new_out = Vec::with_capacity(out.len() + 1);
            new_out.extend_from_slice(basename);
            new_out.push(b'_');
            new_out.extend_from_slice(&out[basename_end..]);
            out = new_out;
        }
    }

    // Third pass: strip trailing dots and spaces (Windows strips them silently).
    while let Some(&last) = out.last() {
        if last == b'.' || last == b' ' {
            out.pop();
        } else {
            break;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn con_dot_txt_becomes_con_underscore_dot_txt() {
        assert_eq!(apply(b"CON.txt"), b"CON_.txt");
    }

    #[test]
    fn case_insensitive_reserved_name() {
        assert_eq!(apply(b"con.txt"), b"con_.txt");
    }

    #[test]
    fn reserved_chars_become_underscores() {
        assert_eq!(apply(b"a<b>c"), b"a_b_c");
    }

    #[test]
    fn control_bytes_become_underscores() {
        assert_eq!(apply(&[b'a', 0x01, b'b']), b"a_b");
    }

    #[test]
    fn trailing_dot_stripped() {
        assert_eq!(apply(b"foo."), b"foo");
        assert_eq!(apply(b"foo ."), b"foo");
    }

    #[test]
    fn plain_name_unchanged() {
        assert_eq!(apply(b"hello.txt"), b"hello.txt");
    }
}
