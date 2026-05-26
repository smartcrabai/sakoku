use std::path::PathBuf;

use clap::Parser;

const LONG_ABOUT: &str = "\
Detect non-ASCII bytes in source files.

By default, a curated allowlist of common, low-risk Unicode characters is
accepted silently. Use --strict (alias: --no-default-allowlist) to flag every
non-ASCII byte instead -- this matches the 0.1.x behavior.

Default allowlist categories:
  - Typography: em/en dash, ellipsis, curly quotes, copyright/registered/
    trademark, section/pilcrow/middle dot, bullet, dagger, single/double angle
    quotes
  - Math / comparison: approx, not-equal, <=, >=, plus-minus, multiplication,
    division, infinity, square root, summation, increment, micro, pi
  - Arrows: full Unicode Arrows block (U+2190-U+21FF), plus heavy angle
    ornaments (\u{276E} \u{276F} \u{2770} \u{2771}) and media controls
    (U+23E9-U+23FA)
  - Accented Latin: Latin-1 Supplement and Latin Extended-A/B
    (U+00A1-U+024F, U+1E00-U+1EFF) -- covers cafe, Zurich, Sao Paulo,
    chionpu, Nguyen, etc.
  - Box Drawing + Block Elements + Geometric Shapes (U+2500-U+25FF)
  - Misc Symbols + Dingbats (U+2600-U+27BF)
  - Braille Patterns (U+2800-U+28FF) -- CLI spinners
  - Emoji and emoji sequence parts: U+1F300-U+1FAFF, regional indicators,
    tag characters, ZWJ, variation selectors

Deliberately NOT allowed at default (homoglyph / typo sources):
  CJK, Hangul, full-width ASCII, Cyrillic/Greek (except mu and pi),
  and U+00A0 NO-BREAK SPACE.";

/// Detect non-ASCII bytes in source files.
#[derive(Debug, Parser)]
#[command(name = "sakoku", version, about, long_about = LONG_ABOUT)]
pub struct Cli {
    /// Files or directories to check (required unless --stdin is used).
    pub paths: Vec<PathBuf>,

    /// Read from standard input instead of files.
    #[arg(long, conflicts_with = "paths")]
    pub stdin: bool,

    /// Disable the default allowlist of common Unicode characters.
    ///
    /// When set, every non-ASCII byte is flagged -- matching the 0.1.x behavior.
    #[arg(long, visible_alias = "no-default-allowlist")]
    pub strict: bool,
}
