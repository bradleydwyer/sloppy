//! Integration tests for the slop-detector CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("slop-detector").unwrap()
}

// ===========================================================================
// analyze subcommand
// ===========================================================================

#[test]
fn test_stdin_clean_text() {
    cmd()
        .arg("analyze")
        .write_stdin("She put the money back.")
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"));
}

#[test]
fn test_stdin_sloppy_text() {
    let text = "This groundbreaking initiative serves as a testament to the vibrant, robust, and crucial \
                work being done by renowned experts. Furthermore, the tapestry of collaboration here is \
                breathtaking, highlighting its potential.\n\nIn conclusion, we must delve deeper.";
    cmd()
        .arg("analyze")
        .write_stdin(text)
        .assert()
        .failure()
        .stdout(predicate::str::contains("FAIL"));
}

#[test]
fn test_json_output() {
    let text = "The vibrant tapestry delves into groundbreaking territory.";
    let output = cmd()
        .args(["analyze", "-f", "json"])
        .write_stdin(text)
        .output()
        .unwrap();
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(data.get("score").is_some());
    assert!(data.get("flags").is_some());
    assert!(data.get("passed").is_some());
    assert!(data["flags"].as_array().is_some());
}

#[test]
fn test_quiet_mode() {
    let output = cmd()
        .args(["analyze", "-q"])
        .write_stdin("Clean text here.")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Score:"));
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 1);
}

#[test]
fn test_custom_threshold() {
    // With very lenient threshold, sloppy text should pass
    cmd()
        .args(["analyze", "-t", "100"])
        .write_stdin("The vibrant community gathered.")
        .assert()
        .success();
}

#[test]
fn test_file_input() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.md");
    std::fs::write(&file, "She put the money back.").unwrap();
    cmd()
        .arg("analyze")
        .arg(file)
        .assert()
        .success();
}

#[test]
fn test_empty_input() {
    cmd()
        .arg("analyze")
        .write_stdin("")
        .assert()
        .failure();
}

// ===========================================================================
// voice subcommand
// ===========================================================================

#[test]
fn test_voice_generates_output() {
    cmd()
        .arg("voice")
        .assert()
        .success()
        .stdout(predicate::str::contains("LEXICAL RESTRICTIONS"));
}

// ===========================================================================
// config subcommand
// ===========================================================================

#[test]
fn test_config_dump() {
    cmd()
        .args(["config", "--dump"])
        .assert()
        .success()
        .stdout(predicate::str::contains("threshold"));
}

#[test]
fn test_config_init() {
    let dir = TempDir::new().unwrap();
    cmd()
        .args(["config", "--init"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));
    assert!(dir.path().join(".slop-detector.toml").exists());
}
