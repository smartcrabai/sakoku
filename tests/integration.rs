use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sakoku"))
}

#[test]
fn clean_file_exits_zero() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("tests/fixtures/clean.txt")
        .output()?;
    assert!(
        output.status.success(),
        "expected exit 0, got: {:?}",
        output.status
    );
    assert!(output.stdout.is_empty(), "expected no output");
    Ok(())
}

#[test]
// Regression test for the pre-0.3.0 GCC output format (compact is now the default).
fn dirty_file_exits_one() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("--format")
        .arg("gcc")
        .arg("tests/fixtures/dirty.txt")
        .output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("0xE3"), "expected 0xE3 in output: {stdout}");
    Ok(())
}

#[test]
fn stdin_clean_exits_zero() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(bin())
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"let x = 42;\n")?;
    }
    let output = child.wait_with_output()?;
    assert!(output.status.success(), "expected exit 0");
    Ok(())
}

#[test]
fn stdin_dirty_exits_one() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(bin())
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        // CJK is detected at the default level (typo / homoglyph source).
        stdin.write_all("let x = \"\u{3042}\";\n".as_bytes())?;
    }
    let output = child.wait_with_output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    Ok(())
}

#[test]
fn stdin_default_allows_cafe() -> Result<(), Box<dyn std::error::Error>> {
    // "café" — é (U+00E9) is in the default allowlist (Latin-1 Supplement).
    let mut child = Command::new(bin())
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all("caf\u{00E9}\n".as_bytes())?;
    }
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "default mode should allow 'café', got: {:?}\nstdout: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(())
}

#[test]
fn stdin_strict_detects_cafe() -> Result<(), Box<dyn std::error::Error>> {
    // Under --strict the original 0.1.x behavior is restored: every non-ASCII
    // byte (including é) is reported.
    let mut child = Command::new(bin())
        .arg("--stdin")
        .arg("--strict")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all("caf\u{00E9}\n".as_bytes())?;
    }
    let output = child.wait_with_output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    Ok(())
}

#[test]
fn strict_alias_no_default_allowlist() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(bin())
        .arg("--stdin")
        .arg("--no-default-allowlist")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all("caf\u{00E9}\n".as_bytes())?;
    }
    let output = child.wait_with_output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "--no-default-allowlist must behave like --strict"
    );
    Ok(())
}

#[test]
fn no_args_exits_two() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin()).output()?;
    assert_eq!(output.status.code(), Some(2), "expected exit 2");
    Ok(())
}

#[test]
// Regression test for the pre-0.3.0 GCC output format (compact is now the default).
fn ignore_next_line_suppresses_line2_not_line3() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("--format")
        .arg("gcc")
        .arg("tests/fixtures/ignored.txt")
        .output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        !stdout.contains("ignored.txt:2:"),
        "line 2 should be suppressed: {stdout}"
    );
    assert!(
        stdout.contains("ignored.txt:3:"),
        "line 3 should be reported: {stdout}"
    );
    Ok(())
}

#[test]
fn allowed_unicode_file_exits_zero() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("tests/fixtures/allowed_unicode.txt")
        .output()?;
    assert!(
        output.status.success(),
        "expected exit 0 for allowed unicode, got: {:?}\nstdout: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stdout.is_empty(), "expected no output");
    Ok(())
}

#[test]
fn regression_pinned_chars_allowed_at_default() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("tests/fixtures/regression_default_allowed.txt")
        .output()?;
    assert!(
        output.status.success(),
        "regression fixture must be clean at default, got: {:?}\nstdout: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(())
}

#[test]
// Regression test for the pre-0.3.0 GCC output format (compact is now the default).
fn regression_pinned_chars_detected_in_strict() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("--strict")
        .arg("--format")
        .arg("gcc")
        .arg("tests/fixtures/regression_default_allowed.txt")
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "regression fixture must be dirty under --strict"
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("non-ASCII byte"),
        "expected violation lines, got: {stdout}"
    );
    Ok(())
}

#[test]
// Regression test for the pre-0.3.0 GCC output format (compact is now the default).
fn cjk_detected_at_default() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("--format")
        .arg("gcc")
        .arg("tests/fixtures/cjk_still_detected.txt")
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "CJK must be detected even at default"
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("non-ASCII byte"),
        "expected violations in output: {stdout}"
    );
    Ok(())
}

#[test]
// Regression test for the pre-0.3.0 GCC output format (compact is now the default).
fn stdin_ignore_next_line() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(bin())
        .arg("--stdin")
        .arg("--format")
        .arg("gcc")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        // line 1: marker, line 2: suppressed (U+3042), line 3: reported (U+3044)
        stdin.write_all(
            "sakoku-ignore-next-line\nlet x = \"\u{3042}\";\nlet y = \"\u{3044}\";\n".as_bytes(),
        )?;
    }
    let output = child.wait_with_output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        !stdout.contains("<stdin>:2:"),
        "line 2 should be suppressed: {stdout}"
    );
    assert!(
        stdout.contains("<stdin>:3:"),
        "line 3 should be reported: {stdout}"
    );
    Ok(())
}

#[test]
fn compact_is_default() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("tests/fixtures/dirty.txt")
        .output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("file") && stdout.contains("char"),
        "expected a compact summary line: {stdout}"
    );
    assert!(
        stdout.contains("[cjk]"),
        "expected a cjk category tag: {stdout}"
    );
    assert!(
        // dirty.txt's only violations are all on line 1, so the line
        // number must be printed.
        stdout.contains("dirty.txt:1 [cjk]"),
        "expected the single violating line number: {stdout}"
    );
    assert!(
        !stdout.contains("0xE3"),
        "compact format must not leak the old GCC byte format: {stdout}"
    );
    Ok(())
}

#[test]
fn compact_omits_line_numbers_when_multiline() -> Result<(), Box<dyn std::error::Error>> {
    // sparse_lines.txt has non-ASCII on lines 1, 2, 3, 7, 9, 10, 11, 12 --
    // scattered across many lines, so no line number (nor a range list)
    // should appear at all, just the path, categories and total count.
    let output = Command::new(bin())
        .arg("tests/fixtures/sparse_lines.txt")
        .output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        !stdout.contains("1-3"),
        "no line range must appear: {stdout}"
    );
    assert!(
        !stdout.contains(":1"),
        "no single line number must appear for a multi-line file: {stdout}"
    );
    assert!(
        stdout.contains("[cjk]"),
        "expected a cjk category tag: {stdout}"
    );
    assert!(
        stdout.contains("(8)"),
        "expected 8 total violating characters: {stdout}"
    );
    Ok(())
}

#[test]
fn compact_singular_summary() -> Result<(), Box<dyn std::error::Error>> {
    // A single stdin char (U+3042) is the simplest way to force exactly one
    // file and one violation, to prove singular/plural wording is chosen
    // independently for "file" and "char".
    let mut child = Command::new(bin())
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all("\u{3042}".as_bytes())?;
    }
    let output = child.wait_with_output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8(output.stdout)?;
    let first_line = stdout.lines().next().unwrap_or_default();
    assert_eq!(first_line, "1 file, 1 char", "unexpected summary: {stdout}");
    Ok(())
}

#[test]
fn only_filters_to_cjk() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("--only")
        .arg("cjk")
        .arg("tests/fixtures/multi_category.txt")
        .output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("multi_category.txt:2 [cjk] (7)"),
        "expected line 2 reported as cjk only: {stdout}"
    );
    assert!(
        !stdout.contains("fullwidth"),
        "fullwidth line 3 must be filtered out: {stdout}"
    );
    assert!(
        !stdout.contains("homoglyph"),
        "homoglyph line 4 must be filtered out: {stdout}"
    );

    // Unfiltered, multi_category.txt has violations spread across three
    // lines (2, 3, 4), so no single line number should be printed for it.
    let unfiltered = Command::new(bin())
        .arg("tests/fixtures/multi_category.txt")
        .output()?;
    let unfiltered_stdout = String::from_utf8(unfiltered.stdout)?;
    assert!(
        !unfiltered_stdout.contains("multi_category.txt:"),
        "expected no line number when violations span multiple lines: {unfiltered_stdout}"
    );
    Ok(())
}

#[test]
fn only_filters_to_homoglyph() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("--only")
        .arg("homoglyph")
        .arg("tests/fixtures/multi_category.txt")
        .output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("multi_category.txt:4 [homoglyph] (1)"),
        "expected line 4 reported as homoglyph only: {stdout}"
    );
    assert!(
        !stdout.contains("cjk"),
        "cjk line 2 must be filtered out: {stdout}"
    );
    assert!(
        !stdout.contains("fullwidth"),
        "fullwidth line 3 must be filtered out: {stdout}"
    );
    Ok(())
}

#[test]
fn only_with_no_match_exits_zero() -> Result<(), Box<dyn std::error::Error>> {
    // dirty.txt only contains cjk violations, so filtering to homoglyph
    // must report "no violations in the requested category", i.e. exit 0
    // with empty output -- not an error.
    let output = Command::new(bin())
        .arg("--only")
        .arg("homoglyph")
        .arg("tests/fixtures/dirty.txt")
        .output()?;
    assert!(
        output.status.success(),
        "expected exit 0, got: {:?}",
        output.status
    );
    assert!(output.stdout.is_empty(), "expected no output");
    Ok(())
}

#[test]
fn max_files_truncates() -> Result<(), Box<dyn std::error::Error>> {
    let full_output = Command::new(bin()).arg("tests/fixtures/").output()?;
    let full_stdout = String::from_utf8(full_output.stdout)?;
    let full_summary = full_stdout.lines().next().unwrap_or_default();

    let truncated_output = Command::new(bin())
        .arg("--max-files")
        .arg("1")
        .arg("tests/fixtures/")
        .output()?;
    assert_eq!(truncated_output.status.code(), Some(1), "expected exit 1");
    let truncated_stdout = String::from_utf8(truncated_output.stdout)?;
    let truncated_summary = truncated_stdout.lines().next().unwrap_or_default();

    assert_eq!(
        full_summary, truncated_summary,
        "summary line must report the true total, unaffected by --max-files"
    );
    assert!(
        truncated_stdout.contains("... and") && truncated_stdout.contains("more file"),
        "expected a truncation notice: {truncated_stdout}"
    );
    Ok(())
}

#[test]
fn clean_file_prints_nothing_in_compact() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("tests/fixtures/clean.txt")
        .output()?;
    assert!(
        output.status.success(),
        "expected exit 0, got: {:?}",
        output.status
    );
    assert!(output.stdout.is_empty(), "expected no output");
    Ok(())
}

#[test]
fn gcc_format_still_lists_every_character() -> Result<(), Box<dyn std::error::Error>> {
    // dirty.txt is `let x = "<3 CJK chars>";`, so the old GCC format must
    // still list all 3 characters, one per line, proving it is fully
    // preserved behind --format gcc.
    let output = Command::new(bin())
        .arg("--format")
        .arg("gcc")
        .arg("tests/fixtures/dirty.txt")
        .output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    let stdout = String::from_utf8(output.stdout)?;
    let line_count = stdout.lines().count();
    assert_eq!(line_count, 3, "expected 3 lines, got: {stdout}");
    Ok(())
}
