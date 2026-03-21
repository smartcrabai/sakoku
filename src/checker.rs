const IGNORE_MARKER: &[u8] = b"sakoku-ignore-next-line";

fn find_suppressed_lines(content: &[u8]) -> Vec<usize> {
    let mut suppressed = Vec::new();
    let mut line_num: usize = 1;
    for line in content.split(|&b| b == b'\n') {
        if line
            .windows(IGNORE_MARKER.len())
            .any(|w| w == IGNORE_MARKER)
        {
            suppressed.push(line_num + 1);
        }
        line_num += 1;
    }
    suppressed
}

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

/// Returns `true` if the Unicode character is on the allowlist of common symbols.
///
/// These characters are widely used in documentation, diagrams, and code comments
/// and are exempt from the non-ASCII lint check.
#[must_use]
pub const fn is_allowed_unicode(c: char) -> bool {
    matches!(
        c,
        '\u{00B0}' // ° DEGREE SIGN
        | '\u{00B1}' // ± PLUS-MINUS SIGN
        | '\u{00D7}' // × MULTIPLICATION SIGN
        | '\u{2013}' // – EN DASH
        | '\u{2014}' // — EM DASH
        | '\u{2022}' // • BULLET
        | '\u{2026}' // … HORIZONTAL ELLIPSIS
        | '\u{2190}' // ← LEFTWARDS ARROW
        | '\u{2191}' // ↑ UPWARDS ARROW
        | '\u{2192}' // → RIGHTWARDS ARROW
        | '\u{2193}' // ↓ DOWNWARDS ARROW
        | '\u{21BA}' // ↺ ANTICLOCKWISE OPEN CIRCLE ARROW
        | '\u{21BB}' // ↻ CLOCKWISE OPEN CIRCLE ARROW
        | '\u{23F8}' // ⏸ DOUBLE VERTICAL BAR
        | '\u{2260}' // ≠ NOT EQUAL TO
        | '\u{2264}' // ≤ LESS-THAN OR EQUAL TO
        | '\u{2265}' // ≥ GREATER-THAN OR EQUAL TO
        | '\u{25B6}' // ▶ BLACK RIGHT-POINTING TRIANGLE
        | '\u{25CB}' // ○ WHITE CIRCLE
        | '\u{25CF}' // ● BLACK CIRCLE
        | '\u{26A0}' // ⚠ WARNING SIGN
        | '\u{2500}' // ─ BOX DRAWINGS LIGHT HORIZONTAL
        | '\u{2502}' // │ BOX DRAWINGS LIGHT VERTICAL
        | '\u{250C}' // ┌ BOX DRAWINGS LIGHT DOWN AND RIGHT
        | '\u{2510}' // ┐ BOX DRAWINGS LIGHT DOWN AND LEFT
        | '\u{2514}' // └ BOX DRAWINGS LIGHT UP AND RIGHT
        | '\u{2518}' // ┘ BOX DRAWINGS LIGHT UP AND LEFT
        | '\u{251C}' // ├ BOX DRAWINGS LIGHT VERTICAL AND RIGHT
        | '\u{2524}' // ┤ BOX DRAWINGS LIGHT VERTICAL AND LEFT
        | '\u{252C}' // ┬ BOX DRAWINGS LIGHT DOWN AND HORIZONTAL
        | '\u{2534}' // ┴ BOX DRAWINGS LIGHT UP AND HORIZONTAL
        | '\u{253C}' // ┼ BOX DRAWINGS LIGHT VERTICAL AND HORIZONTAL
        | '\u{2713}' // ✓ CHECK MARK
        | '\u{2717}' // ✗ BALLOT X
        | '\u{2807}' // ⠇ BRAILLE PATTERN DOTS-123
        | '\u{280B}' // ⠋ BRAILLE PATTERN DOTS-124
        | '\u{280F}' // ⠏ BRAILLE PATTERN DOTS-1234
        | '\u{2819}' // ⠙ BRAILLE PATTERN DOTS-145
        | '\u{2826}' // ⠦ BRAILLE PATTERN DOTS-236
        | '\u{2827}' // ⠧ BRAILLE PATTERN DOTS-1236
        | '\u{2834}' // ⠴ BRAILLE PATTERN DOTS-356
        | '\u{2838}' // ⠸ BRAILLE PATTERN DOTS-456
        | '\u{2839}' // ⠹ BRAILLE PATTERN DOTS-1245
        | '\u{283C}' // ⠼ BRAILLE PATTERN DOTS-3456
    )
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
///
/// Lines preceded by a `sakoku-ignore-next-line` marker are excluded from results.
#[must_use]
pub fn check_bytes(content: &[u8]) -> Vec<Violation> {
    let suppressed = find_suppressed_lines(content);
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
            if char_display.is_some_and(is_allowed_unicode) {
                col += advance;
                i += advance;
                continue;
            }
            if !suppressed.contains(&line) {
                violations.push(Violation {
                    line,
                    column: col,
                    byte,
                    char_display,
                });
            }
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

    #[test]
    fn allowed_unicode_not_reported() {
        // All characters on the allowlist should produce no violations
        let allowed = "°±×–—•…←↑→↓↺↻⏸≠≤≥▶○●⚠─│┌┐└┘├┤┬┴┼✓✗⠇⠋⠏⠙⠦⠧⠴⠸⠹⠼";
        let violations = check_bytes(allowed.as_bytes());
        assert!(
            violations.is_empty(),
            "unexpected violations for allowed unicode: {violations:?}"
        );
    }

    #[test]
    fn allowed_unicode_mixed_with_disallowed() {
        // → (allowed) and あ (disallowed) on the same line
        let content = "result → \u{3042}".as_bytes();
        let violations = check_bytes(content);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].char_display, Some('あ'));
    }
}
