//! Integration tests for **IsPalindromeCLI** (`is_palindrome_cli` binary).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_is_palindrome_cli")
}

#[test]
fn default_invocation_aba_stdout_true_exit_0() {
    let out = Command::new(cli_bin())
        .current_dir(repo_root())
        .args(["aba"])
        .output()
        .expect("spawn cli");
    assert_eq!(out.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "true");
}

#[test]
fn default_invocation_ab_exit_1() {
    let out = Command::new(cli_bin())
        .current_dir(repo_root())
        .args(["ab"])
        .output()
        .expect("spawn cli");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "false");
}

#[test]
fn impl_rust_explicit_matches_default() {
    let a = Command::new(cli_bin())
        .current_dir(repo_root())
        .args(["--impl", "rust", "aba"])
        .output()
        .expect("spawn");
    let b = Command::new(cli_bin())
        .current_dir(repo_root())
        .args(["aba"])
        .output()
        .expect("spawn");
    assert_eq!(a.status, b.status);
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn non_ascii_string_api_exit_2() {
    let out = Command::new(cli_bin())
        .current_dir(repo_root())
        .env("PYTHONPATH", repo_root())
        .args(["--impl", "rust", "é"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn json_roundtrip_string_mode() {
    let dir = tempfile::tempdir().expect("tempdir");
    let inp = dir.path().join("in.json");
    let out = dir.path().join("out.json");
    fs::write(
        &inp,
        r#"{"mode":"string","text":"aba","custom":[],"impl":"rust"}"#,
    )
    .expect("write");
    let cmd = Command::new(cli_bin())
        .current_dir(repo_root())
        .args([
            "--input-json",
            inp.to_str().unwrap(),
            "--output-json",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("spawn");
    assert_eq!(cmd.status.code(), Some(0), "{}", String::from_utf8_lossy(&cmd.stderr));
    let body = fs::read_to_string(&out).expect("read out");
    assert!(body.contains("\"result\":true") || body.contains("\"result\": true"));
}

#[test]
fn impl_all_json_contains_rust_and_status() {
    let dir = tempfile::tempdir().expect("tempdir");
    let inp = dir.path().join("in.json");
    let out = dir.path().join("out.json");
    fs::write(
        &inp,
        r#"{"mode":"string","text":"aba","custom":[],"impl":"all"}"#,
    )
    .expect("write");
    let cmd = Command::new(cli_bin())
        .current_dir(repo_root())
        .env("PYTHONPATH", repo_root())
        .args([
            "--input-json",
            inp.to_str().unwrap(),
            "--output-json",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("spawn");
    assert_eq!(cmd.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&cmd.stderr));
    let body = fs::read_to_string(&out).expect("read out");
    assert!(
        body.contains("rust") && body.contains("results"),
        "expected merged JSON, got: {body}"
    );
}
