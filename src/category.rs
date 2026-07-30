use crate::checker::{Violation, is_default_allowed};

/// Classifies a non-ASCII violation by the remediation a coding agent should
/// take, not by which Unicode block the character happens to live in.
///
/// The whole point of this enum is that "what should I do about this
/// character" does not line up with Unicode block boundaries:
/// - Some characters are deterministic, mechanical substitutions
///   (`Fullwidth`).
/// - Some are natural-language text that needs a translation pass, not a
///   substitution (`Cjk`).
/// - Some merely look like Latin letters and are a security concern rather
///   than a language concern (`Homoglyph`).
/// - Some are already tolerated by the default allowlist and only appear
///   as violations under `--strict` (`Symbol`).
/// - Everything left over, including bytes that could not be decoded as
///   UTF-8, falls into a catch-all (`Other`).
///
/// The declaration order below is also the `derive(Ord)` sort order. It is
/// chosen to match the alphabetical order of the `label()` strings (cjk,
/// fullwidth, homoglyph, symbol, other), not Unicode code point order, so
/// keep the two in sync if either changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    /// Natural-language CJK/Hangul script text: needs translation, not a
    /// mechanical substitution.
    Cjk,
    /// Deterministically machine-replaceable with an ASCII equivalent.
    ///
    /// U+3000 IDEOGRAPHIC SPACE technically belongs to the CJK Symbols and
    /// Punctuation block, but it is classified here rather than under `Cjk`
    /// because it has one unambiguous ASCII replacement (a regular space)
    /// and carries no natural-language content to translate.
    Fullwidth,
    /// Resembles a Latin letter closely enough to be a homoglyph attack
    /// vector.
    ///
    /// This is deliberately not folded into `Cjk`: a homoglyph is a human
    /// security-review concern, not a translation task, and mixing the two
    /// risks a Greek or Cyrillic look-alike being "fixed" by translating it
    /// as if it were prose, which would silently paper over the attack.
    Homoglyph,
    /// Already accepted by the default allowlist ([`is_default_allowed`]);
    /// this variant only ever appears when scanning with `--strict`.
    Symbol,
    /// Everything else, including bytes that failed to decode as UTF-8.
    Other,
}

impl Category {
    /// Classifies a violation by remediation strategy.
    ///
    /// `None` (an undecodable byte) always maps to [`Category::Other`].
    ///
    /// Match arm order matters here: `Fullwidth` and `Cjk` are checked
    /// first because they are exact, well-known ranges. The Greek/Cyrillic
    /// `Homoglyph` check runs next, but explicitly excludes U+0394, U+03BC
    /// and U+03C0 -- those three are on the [`is_default_allowed`] allowlist
    /// as math symbols (Delta, mu, pi), and without the exclusion this arm
    /// would shadow the `Symbol` arm below and misclassify them as
    /// homoglyphs.
    #[must_use]
    pub const fn classify(v: &Violation) -> Self {
        let Some(c) = v.char_display else {
            return Self::Other;
        };
        match c {
            '\u{00A0}' // NO-BREAK SPACE
            | '\u{3000}' // IDEOGRAPHIC SPACE
            | '\u{FF01}'..='\u{FF5E}' => Self::Fullwidth, // Fullwidth ASCII variants

            '\u{1100}'..='\u{11FF}' // Hangul Jamo
            | '\u{3001}'..='\u{303F}' // CJK Symbols and Punctuation (U+3000 handled above)
            | '\u{3040}'..='\u{30FF}' // Hiragana + Katakana
            | '\u{3400}'..='\u{4DBF}' // CJK Unified Ideographs Extension A
            | '\u{4E00}'..='\u{9FFF}' // CJK Unified Ideographs
            | '\u{AC00}'..='\u{D7A3}' // Hangul Syllables
            | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
            | '\u{FF61}'..='\u{FF9F}' => Self::Cjk, // Halfwidth Katakana

            // Delta/mu/pi excluded: they are on the default math allowlist,
            // not homoglyph risks -- see the doc comment above.
            '\u{0370}'..='\u{03FF}' // Greek and Coptic
            | '\u{0400}'..='\u{04FF}' // Cyrillic
                if !matches!(c, '\u{0394}' | '\u{03BC}' | '\u{03C0}') =>
            {
                Self::Homoglyph
            }

            _ if is_default_allowed(c) => Self::Symbol,
            _ => Self::Other,
        }
    }

    /// Returns the lowercase, machine-stable label for this category.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cjk => "cjk",
            Self::Fullwidth => "fullwidth",
            Self::Homoglyph => "homoglyph",
            Self::Symbol => "symbol",
            Self::Other => "other",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn violation(char_display: Option<char>) -> Violation {
        Violation {
            line: 1,
            column: 1,
            byte: 0,
            char_display,
        }
    }

    #[test]
    fn cjk_representatives() {
        // U+4E00 CJK UNIFIED IDEOGRAPH, U+3042 HIRAGANA LETTER A,
        // U+30A2 KATAKANA LETTER A, U+AC00 HANGUL SYLLABLE.
        for c in ['\u{4E00}', '\u{3042}', '\u{30A2}', '\u{AC00}'] {
            assert_eq!(
                Category::classify(&violation(Some(c))),
                Category::Cjk,
                "expected {c:?} to classify as Cjk"
            );
        }
    }

    #[test]
    fn fullwidth_representatives() {
        // U+FF21 FULLWIDTH LATIN CAPITAL LETTER A, U+3000 IDEOGRAPHIC SPACE,
        // U+00A0 NO-BREAK SPACE.
        for c in ['\u{FF21}', '\u{3000}', '\u{00A0}'] {
            assert_eq!(
                Category::classify(&violation(Some(c))),
                Category::Fullwidth,
                "expected {c:?} to classify as Fullwidth"
            );
        }
    }

    #[test]
    fn homoglyph_representatives() {
        // U+0430 CYRILLIC SMALL LETTER A, U+03B1 GREEK SMALL LETTER ALPHA.
        for c in ['\u{0430}', '\u{03B1}'] {
            assert_eq!(
                Category::classify(&violation(Some(c))),
                Category::Homoglyph,
                "expected {c:?} to classify as Homoglyph"
            );
        }
    }

    #[test]
    fn arm_order_boundary_u3000_is_fullwidth() {
        assert_eq!(
            Category::classify(&violation(Some('\u{3000}'))),
            Category::Fullwidth
        );
    }

    #[test]
    fn arm_order_boundary_u3001_is_cjk() {
        assert_eq!(
            Category::classify(&violation(Some('\u{3001}'))),
            Category::Cjk
        );
    }

    #[test]
    fn arm_order_boundary_uff5e_is_fullwidth() {
        assert_eq!(
            Category::classify(&violation(Some('\u{FF5E}'))),
            Category::Fullwidth
        );
    }

    #[test]
    fn arm_order_boundary_uff61_is_cjk() {
        assert_eq!(
            Category::classify(&violation(Some('\u{FF61}'))),
            Category::Cjk
        );
    }

    #[test]
    fn arm_order_boundary_gap_between_fullwidth_and_cjk_is_other() {
        // U+FF5F and U+FF60 fall in the gap between the Fullwidth ASCII
        // variants block and the Halfwidth Katakana block, and are not on
        // the default allowlist, so they fall through to Other.
        assert_eq!(
            Category::classify(&violation(Some('\u{FF5F}'))),
            Category::Other
        );
        assert_eq!(
            Category::classify(&violation(Some('\u{FF60}'))),
            Category::Other
        );
    }

    #[test]
    fn default_allowlist_chars_are_symbol() {
        // U+2014 EM DASH and an emoji code point are both accepted by
        // is_default_allowed(), so they surface only under --strict.
        for c in ['\u{2014}', '\u{1F600}'] {
            assert_eq!(
                Category::classify(&violation(Some(c))),
                Category::Symbol,
                "expected {c:?} to classify as Symbol"
            );
        }
    }

    #[test]
    fn allowlisted_greek_math_symbols_are_symbol_not_homoglyph() {
        // U+0394 GREEK CAPITAL LETTER DELTA, U+03BC GREEK SMALL LETTER MU,
        // U+03C0 GREEK SMALL LETTER PI are all in the Greek and Coptic
        // range that Homoglyph would otherwise claim, but is_default_allowed
        // treats them as math notation. The Homoglyph arm carves these three
        // out explicitly so they fall through to the Symbol arm instead.
        for c in ['\u{0394}', '\u{03BC}', '\u{03C0}'] {
            assert_eq!(
                Category::classify(&violation(Some(c))),
                Category::Symbol,
                "expected {c:?} to classify as Symbol, not Homoglyph"
            );
        }
    }

    #[test]
    fn undecodable_byte_is_other() {
        assert_eq!(Category::classify(&violation(None)), Category::Other);
    }

    #[test]
    fn label_returns_expected_strings() {
        assert_eq!(Category::Cjk.label(), "cjk");
        assert_eq!(Category::Fullwidth.label(), "fullwidth");
        assert_eq!(Category::Homoglyph.label(), "homoglyph");
        assert_eq!(Category::Symbol.label(), "symbol");
        assert_eq!(Category::Other.label(), "other");
    }
}
