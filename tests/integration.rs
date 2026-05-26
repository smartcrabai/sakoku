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
fn dirty_file_exits_one() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
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
fn ignore_next_line_suppresses_line2_not_line3() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
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
fn regression_pinned_chars_detected_in_strict() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
        .arg("--strict")
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
fn cjk_detected_at_default() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin())
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
fn stdin_ignore_next_line() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(bin())
        .arg("--stdin")
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
