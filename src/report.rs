use std::borrow::Cow;
use std::io::{self, Write};
use std::path::Path;

use crate::category::Category;
use crate::checker::Violation;
use crate::cli::OutputFormat;
use crate::walker::FileResult;

/// Formats a single violation in GCC/Clang-style: `path:line:col: non-ASCII byte 0xXX ('char')`.
#[must_use]
pub fn format_violation(path: &Path, v: &Violation) -> String {
    let char_part = v
        .char_display
        .map_or_else(String::new, |c| format!(" ('{c}')"));
    format!(
        "{}:{}:{}: non-ASCII byte 0x{:02X}{}",
        path.display(),
        v.line,
        v.column,
        v.byte,
        char_part,
    )
}

/// Per-file digest of `Violation`s, used to render the compact output format.
///
/// There used to be a full list of violating line numbers here, run-length
/// compressed into ranges. It was dropped: a caller (typically a coding
/// agent) only benefits from a line number when it can jump straight to the
/// violation, which only works when the violations in a file sit on exactly
/// one line. Once they are scattered across two or more lines, the caller
/// ends up reading the whole file to fix it anyway, so a range list is dead
/// weight -- and for a file that is mostly non-ASCII, that list can dwarf
/// the rest of the output.
pub struct FileSummary {
    /// The violating line number, when every violation in the file sits on
    /// that single line; `None` when the violations span two or more
    /// lines, in which case the line number is omitted from the output (see
    /// the struct-level doc comment for why).
    pub single_line: Option<usize>,
    /// Categories present among the violations, sorted and deduped in
    /// `Category`'s derived `Ord` order.
    pub categories: Vec<Category>,
    /// Number of violations summarized (i.e. `violations.len()`).
    pub count: usize,
}

/// Builds a `FileSummary` from a single file's violations.
#[must_use]
pub fn summarize(violations: &[Violation]) -> FileSummary {
    let count = violations.len();

    let mut lines: Vec<usize> = violations.iter().map(|v| v.line).collect();
    lines.sort_unstable();
    lines.dedup();
    let single_line = match lines.as_slice() {
        [only] => Some(*only),
        _ => None,
    };

    let mut categories: Vec<Category> = violations.iter().map(Category::classify).collect();
    categories.sort_unstable();
    categories.dedup();

    FileSummary {
        single_line,
        categories,
        count,
    }
}

/// Renders a category list as `cjk,fullwidth`.
fn format_categories(categories: &[Category]) -> String {
    categories
        .iter()
        .copied()
        .map(Category::label)
        .collect::<Vec<_>>()
        .join(",")
}

/// Returns the English plural suffix ("" or "s") for `n`.
const fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Renders the `{N} file(s), {M} char(s)` summary line, e.g.
/// `1 file, 1 char` or `2 files, 5 chars`.
fn summary_line(files: usize, chars: usize) -> String {
    format!(
        "{files} file{}, {chars} char{}",
        plural(files),
        plural(chars),
    )
}

/// Appends the `... and {K} more file(s)` truncation notice, if any files
/// were hidden by a `max_files` cap.
fn write_truncation_notice(out: &mut impl Write, total: usize, shown: usize) -> io::Result<()> {
    let hidden = total.saturating_sub(shown);
    if hidden > 0 {
        writeln!(out, "... and {hidden} more file{}", plural(hidden))?;
    }
    Ok(())
}

/// Filters `violations` down to those in `only`; an empty `only` means "keep
/// everything", and is returned without cloning.
fn filter_violations<'a>(violations: &'a [Violation], only: &[Category]) -> Cow<'a, [Violation]> {
    if only.is_empty() {
        Cow::Borrowed(violations)
    } else {
        Cow::Owned(
            violations
                .iter()
                .filter(|v| only.contains(&Category::classify(v)))
                .cloned()
                .collect(),
        )
    }
}

/// Renders `results` to `out` in the given `format`.
///
/// `only` filters which categories of violation are reported (a `Violation`
/// -level filter, not a file-level one); an empty slice means "report every
/// category". Files that end up with zero violations after filtering are
/// omitted entirely.
///
/// `max_files` caps how many files are listed, in both formats; the rest are
/// summarized as a single `... and K more file(s)` trailer line.
///
/// `results` is rendered in the order given; callers that want a specific
/// file order (e.g. sorted by path) must sort before calling.
///
/// If nothing is reported (either `results` is empty or `only` filters out
/// every violation), nothing at all is written -- not even the compact
/// format's summary line.
///
/// # Errors
///
/// Returns `Err` if writing to `out` fails.
pub fn render(
    out: &mut impl Write,
    results: &[FileResult],
    format: OutputFormat,
    only: &[Category],
    max_files: Option<usize>,
) -> io::Result<bool> {
    match format {
        OutputFormat::Compact => render_compact(out, results, only, max_files),
        OutputFormat::Gcc => render_gcc(out, results, only, max_files),
    }
}

/// Renders the compact, per-file digest format (see [`render`]).
///
/// Each row is `path:line [categories] (count)` when every violation in
/// that file sits on a single line, or `path [categories] (count)`
/// otherwise -- see [`FileSummary`] for why the line number is dropped in
/// the multi-line case.
fn render_compact(
    out: &mut impl Write,
    results: &[FileResult],
    only: &[Category],
    max_files: Option<usize>,
) -> io::Result<bool> {
    let mut rows: Vec<(&Path, FileSummary)> = Vec::new();
    let mut total_chars = 0usize;

    for result in results {
        let filtered = filter_violations(&result.violations, only);
        if filtered.is_empty() {
            continue;
        }
        let summary = summarize(&filtered);
        total_chars += summary.count;
        rows.push((result.path.as_path(), summary));
    }

    let total_files = rows.len();
    if total_files == 0 {
        return Ok(false);
    }

    writeln!(out, "{}", summary_line(total_files, total_chars))?;

    let shown = max_files.map_or(total_files, |max| max.min(total_files));
    for (path, summary) in &rows[..shown] {
        let categories = format_categories(&summary.categories);
        match summary.single_line {
            Some(line) => writeln!(
                out,
                "{}:{} [{}] ({})",
                path.display(),
                line,
                categories,
                summary.count,
            )?,
            None => writeln!(
                out,
                "{} [{}] ({})",
                path.display(),
                categories,
                summary.count
            )?,
        }
    }
    write_truncation_notice(out, total_files, shown)?;

    Ok(true)
}

/// Renders the pre-0.3 GCC/Clang-compatible format (see [`render`]).
fn render_gcc(
    out: &mut impl Write,
    results: &[FileResult],
    only: &[Category],
    max_files: Option<usize>,
) -> io::Result<bool> {
    let mut files: Vec<(&Path, Cow<'_, [Violation]>)> = Vec::new();

    for result in results {
        let filtered = filter_violations(&result.violations, only);
        if filtered.is_empty() {
            continue;
        }
        files.push((result.path.as_path(), filtered));
    }

    let total_files = files.len();
    if total_files == 0 {
        return Ok(false);
    }

    let shown = max_files.map_or(total_files, |max| max.min(total_files));
    for (path, violations) in &files[..shown] {
        for v in violations.iter() {
            writeln!(out, "{}", format_violation(path, v))?;
        }
    }
    write_truncation_notice(out, total_files, shown)?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn violation(line: usize, char_display: Option<char>) -> Violation {
        Violation {
            line,
            column: 1,
            byte: 0,
            char_display,
        }
    }

    fn file_result(path: &str, violations: Vec<Violation>) -> FileResult {
        FileResult {
            path: path.into(),
            violations,
        }
    }

    /// Renders `results` and returns `(output, had_violations)`, panicking on
    /// I/O or UTF-8 failure (there is none possible against a `Vec<u8>` sink,
    /// but `.unwrap()`/`.expect()` are denied lints in this crate).
    fn render_to_string(
        results: &[FileResult],
        format: OutputFormat,
        only: &[Category],
        max_files: Option<usize>,
    ) -> (String, bool) {
        let mut out = Vec::new();
        let had = match render(&mut out, results, format, only, max_files) {
            Ok(had) => had,
            Err(e) => panic!("render returned an error: {e}"),
        };
        let text = match String::from_utf8(out) {
            Ok(text) => text,
            Err(e) => panic!("render produced non-UTF-8 output: {e}"),
        };
        (text, had)
    }

    // --- summarize ---

    #[test]
    fn summarize_single_line_when_every_violation_is_on_one_line() {
        // Two violations on the same line still collapse to one line number.
        let violations = vec![
            violation(3, Some('\u{4E00}')),
            violation(3, Some('\u{3042}')),
        ];
        let summary = summarize(&violations);
        assert_eq!(summary.single_line, Some(3));
        assert_eq!(summary.count, 2);
    }

    #[test]
    fn summarize_no_single_line_when_violations_span_multiple_lines() {
        let violations = vec![
            violation(1, Some('\u{3042}')),
            violation(5, Some('\u{3042}')),
        ];
        let summary = summarize(&violations);
        assert_eq!(summary.single_line, None);
        assert_eq!(summary.count, 2);
    }

    #[test]
    fn summarize_sorts_and_dedups_categories() {
        // U+0430 CYRILLIC SMALL LETTER A -> Homoglyph, U+4E00 CJK UNIFIED
        // IDEOGRAPH -> Cjk; two Cjk violations must not duplicate the entry.
        let violations = vec![
            violation(1, Some('\u{0430}')),
            violation(1, Some('\u{4E00}')),
            violation(2, Some('\u{4E00}')),
        ];
        let summary = summarize(&violations);
        assert_eq!(summary.categories, vec![Category::Cjk, Category::Homoglyph]);
    }

    // --- render: shared / compact ---

    #[test]
    fn render_compact_empty_results_is_empty_output_and_false() {
        let (out, had) = render_to_string(&[], OutputFormat::Compact, &[], None);
        assert!(!had);
        assert_eq!(out, "");
    }

    #[test]
    fn render_compact_singular_summary_for_one_file_one_char() {
        let results = vec![file_result("/a.md", vec![violation(1, Some('\u{3042}'))])];
        let (out, had) = render_to_string(&results, OutputFormat::Compact, &[], None);
        assert!(had);
        assert_eq!(out, "1 file, 1 char\n/a.md:1 [cjk] (1)\n");
    }

    #[test]
    fn render_compact_multiple_files_mixes_single_and_multi_line_rows() {
        // /a.md has violations on two lines, so its row omits the line
        // number entirely (just a space after the path); /b.md has all of
        // its violations on one line, so its row keeps the `:line`.
        let results = vec![
            file_result(
                "/a.md",
                vec![
                    violation(1, Some('\u{3042}')),
                    violation(2, Some('\u{3042}')),
                ],
            ),
            file_result("/b.md", vec![violation(5, Some('\u{FF21}'))]),
        ];
        let (out, had) = render_to_string(&results, OutputFormat::Compact, &[], None);
        assert!(had);
        assert_eq!(
            out,
            "2 files, 3 chars\n/a.md [cjk] (2)\n/b.md:5 [fullwidth] (1)\n"
        );
    }

    #[test]
    fn render_compact_only_filter_narrows_multiline_file_to_single_line() {
        // Unfiltered, /a.md has violations on two lines (line 1 is cjk,
        // line 2 is homoglyph), so no single line number is printed.
        // Restricting to --only cjk drops the line-2 violation, leaving
        // exactly one violating line -- the line number must reappear.
        let results = vec![file_result(
            "/a.md",
            vec![
                violation(1, Some('\u{4E00}')),
                violation(2, Some('\u{0430}')),
            ],
        )];

        let (out_unfiltered, had_unfiltered) =
            render_to_string(&results, OutputFormat::Compact, &[], None);
        assert!(had_unfiltered);
        assert_eq!(
            out_unfiltered,
            "1 file, 2 chars\n/a.md [cjk,homoglyph] (2)\n"
        );

        let (out_filtered, had_filtered) =
            render_to_string(&results, OutputFormat::Compact, &[Category::Cjk], None);
        assert!(had_filtered);
        assert_eq!(out_filtered, "1 file, 1 char\n/a.md:1 [cjk] (1)\n");
    }

    #[test]
    fn render_compact_only_filter_drops_empty_file_but_keeps_others() {
        let results = vec![
            file_result("/a.md", vec![violation(1, Some('\u{0430}'))]),
            file_result("/b.md", vec![violation(2, Some('\u{3042}'))]),
        ];
        let (out, had) = render_to_string(&results, OutputFormat::Compact, &[Category::Cjk], None);
        assert!(had);
        assert_eq!(out, "1 file, 1 char\n/b.md:2 [cjk] (1)\n");
    }

    #[test]
    fn render_compact_only_filter_removing_everything_is_empty_output_and_false() {
        let results = vec![file_result("/a.md", vec![violation(1, Some('\u{0430}'))])];
        let (out, had) = render_to_string(&results, OutputFormat::Compact, &[Category::Cjk], None);
        assert!(!had);
        assert_eq!(out, "");
    }

    #[test]
    fn render_compact_max_files_truncation_singular() {
        let results = vec![
            file_result("/a.md", vec![violation(1, Some('\u{3042}'))]),
            file_result("/b.md", vec![violation(1, Some('\u{3042}'))]),
        ];
        let (out, had) = render_to_string(&results, OutputFormat::Compact, &[], Some(1));
        assert!(had);
        assert_eq!(
            out,
            "2 files, 2 chars\n/a.md:1 [cjk] (1)\n... and 1 more file\n"
        );
    }

    #[test]
    fn render_compact_max_files_truncation_plural() {
        let results = vec![
            file_result("/a.md", vec![violation(1, Some('\u{3042}'))]),
            file_result("/b.md", vec![violation(1, Some('\u{3042}'))]),
            file_result("/c.md", vec![violation(1, Some('\u{3042}'))]),
        ];
        let (out, had) = render_to_string(&results, OutputFormat::Compact, &[], Some(1));
        assert!(had);
        assert_eq!(
            out,
            "3 files, 3 chars\n/a.md:1 [cjk] (1)\n... and 2 more files\n"
        );
    }

    // --- render: gcc ---

    #[test]
    fn render_gcc_empty_results_is_empty_output_and_false() {
        let (out, had) = render_to_string(&[], OutputFormat::Gcc, &[], None);
        assert!(!had);
        assert_eq!(out, "");
    }

    #[test]
    fn render_gcc_matches_format_violation_output() {
        let v1 = violation(1, Some('\u{3042}'));
        let v2 = violation(2, Some('\u{FF21}'));
        let results = vec![file_result("/a.md", vec![v1.clone(), v2.clone()])];
        let (out, had) = render_to_string(&results, OutputFormat::Gcc, &[], None);
        assert!(had);
        let expected = format!(
            "{}\n{}\n",
            format_violation(Path::new("/a.md"), &v1),
            format_violation(Path::new("/a.md"), &v2),
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn render_gcc_max_files_truncation_notice() {
        let v = violation(1, Some('\u{3042}'));
        let results = vec![
            file_result("/a.md", vec![v.clone()]),
            file_result("/b.md", vec![v.clone()]),
        ];
        let (out, had) = render_to_string(&results, OutputFormat::Gcc, &[], Some(1));
        assert!(had);
        let expected = format!(
            "{}\n... and 1 more file\n",
            format_violation(Path::new("/a.md"), &v)
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn render_gcc_only_filter_removing_everything_is_empty_output_and_false() {
        let results = vec![file_result("/a.md", vec![violation(1, Some('\u{0430}'))])];
        let (out, had) = render_to_string(&results, OutputFormat::Gcc, &[Category::Cjk], None);
        assert!(!had);
        assert_eq!(out, "");
    }
}
