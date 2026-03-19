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
        // "café" — é = U+00E9 = 0xC3 0xA9 in UTF-8
        stdin.write_all("caf\u{00E9}\n".as_bytes())?;
    }
    let output = child.wait_with_output()?;
    assert_eq!(output.status.code(), Some(1), "expected exit 1");
    Ok(())
}

#[test]
fn no_args_exits_two() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin()).output()?;
    assert_eq!(output.status.code(), Some(2), "expected exit 2");
    Ok(())
}
