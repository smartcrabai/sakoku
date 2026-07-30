use std::path::PathBuf;

use clap::Parser;

const LONG_ABOUT: &str = "\
Detect non-ASCII bytes in source files.

Output format:
  By default (0.3.0+), output is compact: one line per file. The line
  number is shown only when every violation in that file sits on a single
  line -- once violations span two or more lines, a caller (typically a
  coding agent) ends up reading the whole file to fix it anyway, so the
  line number is left out instead. For example:
    path/to/file.md:12 [cjk] (3)
    path/to/other.md [cjk,fullwidth] (287)
  Use --format gcc to get the pre-0.3 GCC/Clang-compatible format instead
  (one line per violating character). If there are no violations, nothing
  is printed.

  Categories (see --only):
    - cjk: CJK / Hangul text that needs translation
    - fullwidth: full-width ASCII, ideographic space, NBSP -- mechanically
      replaceable
    - homoglyph: Cyrillic / Greek -- possible homoglyph attack, needs human
      security review
    - symbol: characters on the default allowlist (only reported under
      --strict)
    - other: everything else

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

/// Output format for reporting violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Compact per-file digest, the default as of 0.3.0: one line per file,
    /// e.g. `path/to/file.md:12 [cjk] (3)` when every violation sits on one
    /// line, or `path/to/other.md [cjk,fullwidth] (287)` when they span more.
    #[default]
    Compact,
    /// GCC/Clang-compatible, one line per violating character; the format
    /// used by default before 0.3.0.
    Gcc,
}

/// Character category used to filter reported violations via `--only`.
///
/// This mirrors `crate::category::Category` but is defined separately so
/// that `clap::ValueEnum` (and its CLI-facing string conversions) do not
/// leak into the classification module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CategoryArg {
    /// CJK / Hangul text that needs translation.
    Cjk,
    /// Full-width ASCII, ideographic space, NBSP -- mechanically
    /// replaceable.
    Fullwidth,
    /// Cyrillic / Greek -- possible homoglyph attack, needs human security
    /// review.
    Homoglyph,
    /// Characters on the default allowlist (only reported under --strict).
    Symbol,
    /// Everything else.
    Other,
}

impl From<CategoryArg> for crate::category::Category {
    fn from(value: CategoryArg) -> Self {
        match value {
            CategoryArg::Cjk => Self::Cjk,
            CategoryArg::Fullwidth => Self::Fullwidth,
            CategoryArg::Homoglyph => Self::Homoglyph,
            CategoryArg::Symbol => Self::Symbol,
            CategoryArg::Other => Self::Other,
        }
    }
}

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

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Compact)]
    pub format: OutputFormat,

    /// Only report characters in these categories (comma-separated).
    ///
    /// When empty (the default), violations in every category are reported.
    /// If this filter causes zero violations to remain, the run exits 0 --
    /// that outcome means "no violations in the requested categories", not
    /// an error.
    ///
    /// Repeat the flag or use a comma-separated list to select several
    /// categories: `--only cjk,fullwidth` or `--only cjk --only fullwidth`.
    /// Space-separated values are deliberately not accepted, because a
    /// variadic option would swallow the trailing path arguments.
    #[arg(long, value_enum, value_delimiter = ',')]
    pub only: Vec<CategoryArg>,

    /// Maximum number of files to list.
    ///
    /// Unlimited by default. When the limit is exceeded, the remaining
    /// files are summarized as "... and K more files" instead of being
    /// silently dropped from the output.
    #[arg(long)]
    pub max_files: Option<usize>,
}
