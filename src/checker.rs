const IGNORE_MARKER: &[u8] = b"sakoku-ignore-next-line";

fn find_suppressed_lines(content: &[u8]) -> Vec<usize> {
    let mut suppressed = Vec::new();
    for (line_num, line) in (1_usize..).zip(content.split(|&b| b == b'\n')) {
        if line
            .windows(IGNORE_MARKER.len())
            .any(|w| w == IGNORE_MARKER)
        {
            suppressed.push(line_num + 1);
        }
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

/// Configures how `check_bytes` interprets non-ASCII bytes.
#[derive(Debug, Clone, Copy, Default)]
pub struct CheckOptions {
    /// When `true`, disable the default allowlist of common Unicode
    /// characters — every non-ASCII byte is reported. This matches the
    /// original 0.1.x behavior.
    pub strict: bool,
}

/// Returns `true` if the byte is permitted in ASCII-only source files.
///
/// Allowed: Tab (0x09), LF (0x0A), CR (0x0D), and printable ASCII (0x20–0x7E).
#[must_use]
pub const fn is_allowed(byte: u8) -> bool {
    matches!(byte, 0x09 | 0x0A | 0x0D | 0x20..=0x7E)
}

/// Returns `true` if the Unicode character is on the default allowlist of
/// common, low-risk characters.
///
/// The allowlist covers:
/// - Typography: em/en dash, ellipsis, curly quotes, daggers, bullet, single
///   and double angle quotes, ™ — plus the Latin-1 typography (§ © ® ° ¶ ·)
///   covered transitively by the range below.
/// - Math / comparison: ≈ ≠ ≤ ≥ ∞ √ ∑ ∆ Δ π µ — plus ± × ÷ from Latin-1.
/// - Arrows: the full U+2190–U+21FF Arrows block, plus ❮ ❯ ❰ ❱ (prompt arrows).
/// - Media controls: U+23E9–U+23FA (⏪ ⏩ ⏸ ⏹ ⏺ …).
/// - Latin script (upper Latin-1 U+00A1–U+024F, skipping NBSP U+00A0; Latin
///   Extended Additional U+1E00–U+1EFF): naïve, café, Zürich, São, chōonpu,
///   Nguyễn, Spanish ¿¡, French «», etc.
/// - Box Drawing + Block Elements + Geometric Shapes: U+2500–U+25FF.
/// - Misc Symbols + Dingbats: U+2600–U+27BF (✓ ✔ ✘ ★ ☆ ⚠ …).
/// - Misc Symbols and Arrows: U+2B00–U+2BFF (⭐ ⬆ ⬇ ⬛ ⬜ ➕ …).
/// - Braille Patterns: U+2800–U+28FF (CLI spinners).
/// - Emoji: U+1F300–U+1FAFF, regional indicators U+1F1E6–U+1F1FF,
///   tag chars U+E0020–U+E007F, ZWJ U+200D, variation selectors U+FE0E/F.
/// - Bidi / zero-width format controls: U+200B–U+200F, U+2060, U+FEFF.
///
/// Intentionally NOT allowed (likely typo or homoglyph attack):
/// CJK, Hangul, full-width ASCII, Cyrillic, most of Greek (except Δ, µ, π),
/// and U+00A0 NO-BREAK SPACE.
#[must_use]
pub const fn is_default_allowed(c: char) -> bool {
    matches!(
        c,
        // --- General Punctuation (typography singletons in U+2000s) ---
        '\u{2013}' // – EN DASH
        | '\u{2014}' // — EM DASH
        | '\u{2018}' // ' LEFT SINGLE QUOTATION MARK
        | '\u{2019}' // ' RIGHT SINGLE QUOTATION MARK
        | '\u{201C}' // " LEFT DOUBLE QUOTATION MARK
        | '\u{201D}' // " RIGHT DOUBLE QUOTATION MARK
        | '\u{2020}' // † DAGGER
        | '\u{2021}' // ‡ DOUBLE DAGGER
        | '\u{2022}' // • BULLET
        | '\u{2023}' // ‣ TRIANGULAR BULLET
        | '\u{2026}' // … HORIZONTAL ELLIPSIS
        | '\u{2039}' // ‹ SINGLE LEFT-POINTING ANGLE QUOTATION MARK
        | '\u{203A}' // › SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
        | '\u{2122}' // ™ TRADE MARK SIGN
        // --- Math / Greek singletons outside the Latin-1 range ---
        | '\u{0394}' // Δ GREEK CAPITAL LETTER DELTA (math / physics)
        | '\u{03BC}' // μ GREEK SMALL LETTER MU
        | '\u{03C0}' // π GREEK SMALL LETTER PI
        | '\u{2206}' // ∆ INCREMENT
        | '\u{2211}' // ∑ N-ARY SUMMATION
        | '\u{221A}' // √ SQUARE ROOT
        | '\u{221E}' // ∞ INFINITY
        | '\u{2248}' // ≈ ALMOST EQUAL TO
        | '\u{2260}' // ≠ NOT EQUAL TO
        | '\u{2264}' // ≤ LESS-THAN OR EQUAL TO
        | '\u{2265}' // ≥ GREATER-THAN OR EQUAL TO
        // --- Prompt arrow ornaments (Powerline / oh-my-zsh / TUIs) ---
        | '\u{276E}' // ❮ HEAVY LEFT-POINTING ANGLE QUOTATION MARK ORNAMENT
        | '\u{276F}' // ❯ HEAVY RIGHT-POINTING ANGLE QUOTATION MARK ORNAMENT
        | '\u{2770}' // ❰ HEAVY LEFT-POINTING ANGLE BRACKET ORNAMENT
        | '\u{2771}' // ❱ HEAVY RIGHT-POINTING ANGLE BRACKET ORNAMENT
        // --- Zero-width / bidi controls + variation selectors ---
        | '\u{200B}' // ZERO WIDTH SPACE
        | '\u{200C}' // ZERO WIDTH NON-JOINER
        | '\u{200D}' // ZERO WIDTH JOINER (emoji ZWJ sequences)
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK
        | '\u{2060}' // WORD JOINER
        | '\u{FE0E}' // VARIATION SELECTOR-15 (text presentation)
        | '\u{FE0F}' // VARIATION SELECTOR-16 (emoji presentation)
        | '\u{FEFF}' // ZERO WIDTH NO-BREAK SPACE / BOM
        // --- Ranges ---
        // Upper Latin-1 Supplement + Latin Extended-A/B. Starts at U+00A1 to
        // include Spanish ¿ ¡, French « », ¢ £ ¥, and Latin-1 typography
        // (§ © ® ° ¶ · ± µ × ÷); deliberately skips U+00A0 NBSP (typo source).
        | '\u{00A1}'..='\u{024F}'
        // Latin Extended Additional -- Vietnamese precomposed (ễ ặ ờ ự …),
        // and accented Latin used in IPA, Yoruba, etc.
        | '\u{1E00}'..='\u{1EFF}'
        // Arrows block (← ↑ → ↓ ↔ ↕ ↖ ↗ ↘ ↙ ↺ ↻ ⇐ ⇑ ⇒ ⇓ …).
        | '\u{2190}'..='\u{21FF}'
        // Media control symbols (⏩ ⏪ ⏫ ⏬ ⏭ ⏮ ⏯ ⏰ ⏱ ⏲ ⏳ ⏴ ⏵ ⏶ ⏷ ⏸ ⏹ ⏺).
        | '\u{23E9}'..='\u{23FA}'
        // Box Drawing + Block Elements + Geometric Shapes.
        | '\u{2500}'..='\u{25FF}'
        // Miscellaneous Symbols + Dingbats (✓ ✔ ✘ ★ ☆ ⚠ …).
        | '\u{2600}'..='\u{27BF}'
        // Miscellaneous Symbols and Arrows (⭐ ⬆ ⬇ ⬛ ⬜ ➕ …).
        | '\u{2B00}'..='\u{2BFF}'
        // Braille Patterns -- CLI spinners.
        | '\u{2800}'..='\u{28FF}'
        // Regional Indicator Symbols (flags).
        | '\u{1F1E6}'..='\u{1F1FF}'
        // Main emoji blocks (Misc Symbols & Pictographs, Emoticons, Transport,
        // Supplemental Symbols & Pictographs, Symbols & Pictographs Extended-A).
        | '\u{1F300}'..='\u{1FAFF}'
        // Tag characters (sub-region flag tag sequences).
        | '\u{E0020}'..='\u{E007F}'
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
///
/// Characters covered by [`is_default_allowed`] are silently accepted unless
/// `options.strict` is `true`, in which case every non-ASCII byte is reported.
#[must_use]
pub fn check_bytes(content: &[u8], options: CheckOptions) -> Vec<Violation> {
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
            if !options.strict && char_display.is_some_and(is_default_allowed) {
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

    const DEFAULT: CheckOptions = CheckOptions { strict: false };
    const STRICT: CheckOptions = CheckOptions { strict: true };

    #[test]
    fn ascii_only_no_violations() {
        let content = b"let x = 42;\nfn main() {}\n";
        assert!(check_bytes(content, DEFAULT).is_empty());
        assert!(check_bytes(content, STRICT).is_empty());
    }

    #[test]
    fn japanese_chars_detected_default() {
        // CJK must remain detected at the default level (homoglyph / typo risk).
        let content = "let x = \"あいう\";".as_bytes();
        let violations = check_bytes(content, DEFAULT);
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].char_display, Some('あ'));
        assert_eq!(violations[1].char_display, Some('い'));
        assert_eq!(violations[2].char_display, Some('う'));
    }

    #[test]
    fn japanese_chars_detected_strict() {
        let content = "let x = \"あいう\";".as_bytes();
        let violations = check_bytes(content, STRICT);
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].char_display, Some('あ'));
        assert_eq!(violations[1].char_display, Some('い'));
        assert_eq!(violations[2].char_display, Some('う'));
    }

    #[test]
    fn ascii_only_no_violations_strict() {
        // The strict gate must not start flagging tab/CR/LF/printable ASCII.
        let content = b"line1\r\nline2\ttabbed\n";
        assert!(check_bytes(content, STRICT).is_empty());
    }

    #[test]
    fn tab_cr_lf_allowed() {
        let content = b"line1\r\nline2\ttabbed";
        assert!(check_bytes(content, DEFAULT).is_empty());
    }

    #[test]
    fn null_byte_detected() {
        let content = b"hello\x00world";
        let violations = check_bytes(content, DEFAULT);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].byte, 0x00);
        assert_eq!(violations[0].char_display, None);
    }

    #[test]
    fn del_detected() {
        let content = b"hello\x7Fworld";
        let violations = check_bytes(content, DEFAULT);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].byte, 0x7F);
        assert_eq!(violations[0].char_display, None);
    }

    #[test]
    fn empty_input_no_violations() {
        assert!(check_bytes(b"", DEFAULT).is_empty());
    }

    #[test]
    fn line_and_col_tracking() {
        let content = "abc\nあ".as_bytes();
        let violations = check_bytes(content, DEFAULT);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[0].column, 1);
        assert_eq!(violations[0].char_display, Some('あ'));
    }

    #[test]
    fn legacy_allowed_unicode_still_allowed_at_default() {
        // The original 0.1.x allowlist must remain a subset of the new default.
        let allowed = "°±×–—•…←↑→↓↔↕↖↗↘↙↺↻⏸≠≤≥▶○●⚠─│┌┐└┘├┤┬┴┼✓✗⠇⠋⠏⠙⠦⠧⠴⠸⠹⠼";
        let violations = check_bytes(allowed.as_bytes(), DEFAULT);
        assert!(
            violations.is_empty(),
            "unexpected violations for legacy allowlist: {violations:?}"
        );
    }

    #[test]
    fn legacy_allowed_unicode_detected_in_strict() {
        // Every char in the 0.1.x allowlist must be reported under --strict.
        let allowed = "°±×–—•…←↑→↓↔↕↖↗↘↙↺↻⏸≠≤≥▶○●⚠─│┌┐└┘├┤┬┴┼✓✗⠇⠋⠏⠙⠦⠧⠴⠸⠹⠼";
        let violations = check_bytes(allowed.as_bytes(), STRICT);
        assert_eq!(
            violations.len(),
            allowed.chars().count(),
            "strict mode must report every legacy-allowlist char"
        );
    }

    #[test]
    fn allowed_unicode_mixed_with_disallowed() {
        let content = "result → \u{3042}".as_bytes();
        let violations = check_bytes(content, DEFAULT);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].char_display, Some('あ'));
    }

    // --- Default allowlist coverage by category ---

    #[test]
    fn category_typography_allowed_at_default() {
        // §©®°¶·—–…''""•‣†‡‹›™
        let sample = "\u{00A7}\u{00A9}\u{00AE}\u{00B0}\u{00B6}\u{00B7}\u{2014}\u{2013}\u{2026}\
                     \u{2018}\u{2019}\u{201C}\u{201D}\u{2022}\u{2023}\u{2020}\u{2021}\
                     \u{2039}\u{203A}\u{2122}";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_math_allowed_at_default() {
        // ±µ×÷πμ∆∑√∞≈≠≤≥
        let sample = "\u{00B1}\u{00B5}\u{00D7}\u{00F7}\u{03C0}\u{03BC}\
                     \u{2206}\u{2211}\u{221A}\u{221E}\u{2248}\u{2260}\u{2264}\u{2265}";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_arrows_allowed_at_default() {
        // → ← ↑ ↓ ↔ ↕ ⇒ ⇐ ⇑ ⇓ ❯ ❮ ❱ ❰ › ‣
        let sample = "\u{2192}\u{2190}\u{2191}\u{2193}\u{2194}\u{2195}\
                     \u{21D2}\u{21D0}\u{21D1}\u{21D3}\u{276F}\u{276E}\u{2771}\u{2770}";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_accented_latin_allowed_at_default() {
        // naïve, café, Zürich, São, chōonpu, Ångström
        let sample = "na\u{00EF}ve caf\u{00E9} Z\u{00FC}rich S\u{00E3}o ch\u{014D}onpu \u{00C5}ngstr\u{00F6}m";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_spanish_punctuation_allowed_at_default() {
        // ¿Cómo estás? ¡Hola! — inverted question / exclamation are in upper Latin-1.
        let sample = "\u{00BF}C\u{00F3}mo est\u{00E1}s? \u{00A1}Hola!";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_vietnamese_allowed_at_default() {
        // Nguyễn Văn Đức — Latin Extended Additional precomposed.
        let sample = "Nguy\u{1EC5}n V\u{0103}n \u{0110}\u{1EE9}c";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_misc_symbols_and_arrows_allowed_at_default() {
        // ⭐ U+2B50, ⬆ U+2B06, ⬇ U+2B07, ⬛ U+2B1B, ⬜ U+2B1C
        let sample = "\u{2B50}\u{2B06}\u{2B07}\u{2B1B}\u{2B1C}";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_greek_delta_allowed_at_default() {
        // Capital Δ (U+0394) and INCREMENT ∆ (U+2206) are both math-flavored.
        let sample = "\u{0394}t = end - start; volume \u{2206}V";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_box_drawing_and_shapes_allowed_at_default() {
        // ─│┌┐└┘├┤┬┴┼ █ ▓ ▒ ░ ○ ● ▶
        let sample = "\u{2500}\u{2502}\u{250C}\u{2510}\u{2514}\u{2518}\u{251C}\u{2524}\
                     \u{252C}\u{2534}\u{253C}\u{2588}\u{2593}\u{2592}\u{2591}\
                     \u{25CB}\u{25CF}\u{25B6}";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_emoji_allowed_at_default() {
        // single emoji, ZWJ family, flag, heart with VS16
        // 👨‍👩‍👧 = man + ZWJ + woman + ZWJ + girl
        // 🇯🇵 = REGIONAL INDICATOR J + REGIONAL INDICATOR P
        // ❤️ = heart + VS16
        let sample = "\u{1F600} \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} \
                     \u{1F1EF}\u{1F1F5} \u{2764}\u{FE0F}";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn category_zero_width_allowed_at_default() {
        // U+200B ZWSP, U+200C ZWNJ, U+200E LRM, U+2060 WJ, U+FEFF BOM
        let sample = "\u{200B}\u{200C}\u{200E}\u{2060}\u{FEFF}";
        assert!(check_bytes(sample.as_bytes(), DEFAULT).is_empty());
        assert!(!check_bytes(sample.as_bytes(), STRICT).is_empty());
    }

    #[test]
    fn nbsp_still_detected_at_default() {
        // U+00A0 NO-BREAK SPACE is a common typo source — keep detected.
        let sample = "a\u{00A0}b".as_bytes();
        let violations = check_bytes(sample, DEFAULT);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].char_display, Some('\u{00A0}'));
    }

    #[test]
    fn fullwidth_ascii_still_detected_at_default() {
        // U+FF21 FULLWIDTH LATIN CAPITAL LETTER A — homoglyph risk.
        let sample = "let \u{FF21} = 1;".as_bytes();
        let violations = check_bytes(sample, DEFAULT);
        assert!(
            violations
                .iter()
                .any(|v| v.char_display == Some('\u{FF21}'))
        );
    }

    #[test]
    fn cyrillic_still_detected_at_default() {
        // U+0430 CYRILLIC SMALL LETTER A — Latin 'a' homoglyph.
        let sample = "let v\u{0430}lue = 1;".as_bytes();
        let violations = check_bytes(sample, DEFAULT);
        assert!(
            violations
                .iter()
                .any(|v| v.char_display == Some('\u{0430}'))
        );
    }

    #[test]
    fn regression_pin_seher_ts_hits() {
        // ❯ ≈ — … ≤ → ō plus a ZWJ family emoji — these are the exact
        // characters that tripped sakoku in seher-ts/sdk before the default
        // allowlist landed.
        let sample = "\u{276F} \u{2248} \u{2014} \u{2026} \u{2264} \u{2192} \u{014D} \
                     \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
        assert!(
            check_bytes(sample.as_bytes(), DEFAULT).is_empty(),
            "regression: {sample:?} must be allowed at default"
        );
    }

    #[test]
    fn ignore_marker_still_works_at_default() {
        // Marker should suppress disallowed CJK on the next line, even though
        // the allowlist is broader.
        let content = "// sakoku-ignore-next-line\nlet x = \"\u{3042}\";\n".as_bytes();
        assert!(check_bytes(content, DEFAULT).is_empty());
    }
}
