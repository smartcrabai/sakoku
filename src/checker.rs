/// A single non-ASCII byte found in the content.
#[derive(Debug, Clone)]
pub struct Violation {
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed byte column.
    pub column: usize,
    /// The offending byte value.
    pub byte: u8,
    /// The Unicode character starting at this byte, if decodable as UTF-8.
    pub char_display: Option<char>,
}

/// Returns `true` if the byte is permitted in ASCII-only source files.
///
/// Allowed: Tab (0x09), LF (0x0A), CR (0x0D), and printable ASCII (0x20–0x7E).
#[must_use]
pub const fn is_allowed(byte: u8) -> bool {
    matches!(byte, 0x09 | 0x0A | 0x0D | 0x20..=0x7E)
}

/// Tries to decode one non-ASCII UTF-8 character starting at `bytes[0]`.
///
/// Returns `None` for continuation bytes, truncated sequences, or invalid encodings.
fn try_decode_char(bytes: &[u8]) -> Option<char> {
    let first = *bytes.first()?;
    let len: usize = if first < 0xC0 {
        return None; // ASCII or UTF-8 continuation byte — invalid as a start byte
    } else if first < 0xE0 {
        2
    } else if first < 0xF0 {
        3
    } else if first < 0xF8 {
        4
    } else {
        return None;
    };
    std::str::from_utf8(bytes.get(..len)?)
        .ok()
        .and_then(|s| s.chars().next())
}

/// Scans `content` byte by byte and returns every non-ASCII violation found.
///
/// Multi-byte UTF-8 characters are reported as a single violation at the lead byte,
/// and the column counter advances by the full character width.
#[must_use]
pub fn check_bytes(content: &[u8]) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut line: usize = 1;
    let mut col: usize = 1;
    let mut i: usize = 0;

    while i < content.len() {
        let byte = content[i];
        if is_allowed(byte) {
            if byte == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            i += 1;
        } else {
            let char_display = try_decode_char(&content[i..]);
            let advance = char_display.map_or(1, char::len_utf8);
            violations.push(Violation {
                line,
                column: col,
                byte,
                char_display,
            });
            col += advance;
            i += advance;
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_only_no_violations() {
        let content = b"let x = 42;\nfn main() {}\n";
        assert!(check_bytes(content).is_empty());
    }

    #[test]
    fn japanese_chars_detected() {
        // "あいう" = E3 81 82  E3 81 84  E3 81 86 (each 3 bytes)
        let content = "let x = \"あいう\";".as_bytes();
        let violations = check_bytes(content);
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].char_display, Some('あ'));
        assert_eq!(violations[1].char_display, Some('い'));
        assert_eq!(violations[2].char_display, Some('う'));
    }

    #[test]
    fn tab_cr_lf_allowed() {
        let content = b"line1\r\nline2\ttabbed";
        assert!(check_bytes(content).is_empty());
    }

    #[test]
    fn null_byte_detected() {
        let content = b"hello\x00world";
        let violations = check_bytes(content);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].byte, 0x00);
        assert_eq!(violations[0].char_display, None);
    }

    #[test]
    fn del_detected() {
        // DEL (0x7F) is just above the printable ASCII range
        let content = b"hello\x7Fworld";
        let violations = check_bytes(content);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].byte, 0x7F);
        assert_eq!(violations[0].char_display, None);
    }

    #[test]
    fn empty_input_no_violations() {
        assert!(check_bytes(b"").is_empty());
    }

    #[test]
    fn line_and_col_tracking() {
        // "abc\nあ" — あ should be on line 2, column 1
        let content = "abc\nあ".as_bytes();
        let violations = check_bytes(content);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[0].column, 1);
        assert_eq!(violations[0].char_display, Some('あ'));
    }
}
