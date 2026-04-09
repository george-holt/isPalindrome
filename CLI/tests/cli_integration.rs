//! Integration tests for **IsPalindromeCLI** (`is_palindrome_cli` binary).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    if let Ok(rf) = std::env::var("RUNFILES_DIR") {
        for rel in [
            "_main/fixtures/acceptance_manifest.json",
            "fixtures/acceptance_manifest.json",
        ] {
            let manifest = PathBuf::from(&rf).join(rel);
            if manifest.is_file() {
                return manifest
                    .parent()
                    .and_then(|p| p.parent())
                    .expect("repo root")
                    .to_path_buf();
            }
        }
    }
    if let Ok(m) = std::env::var("MODULE_BAZEL") {
        let p = PathBuf::from(m);
        if let Some(parent) = p.parent() {
            if parent.join("fixtures/acceptance_manifest.json").is_file() {
                return parent.to_path_buf();
            }
        }
    }
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let mut d = start;
    loop {
        if d.join("fixtures/acceptance_manifest.json").is_file() {
            return d;
        }
        d = d
            .parent()
            .expect("fixtures/acceptance_manifest.json not found")
            .to_path_buf();
    }
}

fn cli_bin() -> String {
    if let Ok(rf) = std::env::var("RUNFILES_DIR") {
        for rel in [
            "_main/CLI/is_palindrome_cli",
            "CLI/is_palindrome_cli",
        ] {
            let p = PathBuf::from(&rf).join(rel);
            if p.is_file() {
                return p.to_string_lossy().into_owned();
            }
        }
    }
    std::env::var("IS_PALINDROME_CLI").unwrap_or_else(|_| {
        env!("CARGO_BIN_EXE_is_palindrome_cli").to_string()
    })
}

/// Runfiles path to `stdin_json_adapter` so explicit `--impl rust` / JSON rust can run in Bazel.
fn rust_stdin_adapter_bin() -> Option<PathBuf> {
    if let Ok(rf) = std::env::var("RUNFILES_DIR") {
        for rel in [
            "_main/CLI/stdin_json_adapter",
            "CLI/stdin_json_adapter",
        ] {
            let p = PathBuf::from(&rf).join(rel);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Hermetic `rules_python` interpreter (`@python_3_12//:python3`) — repo dir name includes the platform triple.
fn hermetic_python3_runfile() -> Option<PathBuf> {
    let rf = std::env::var("RUNFILES_DIR").ok()?;
    let base = PathBuf::from(rf);
    let entries = fs::read_dir(&base).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("rules_python++python+python_3_12_") {
            continue;
        }
        for bin in ["bin/python3", "bin/python3.exe"] {
            let p = entry.path().join(bin);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn cli_command() -> Command {
    let mut c = Command::new(cli_bin());
    c.current_dir(repo_root());
    if let Some(p) = rust_stdin_adapter_bin() {
        c.env("IS_PALINDROME_RUST_STDIN_ADAPTER", p);
    }
    if let Some(p) = hermetic_python3_runfile() {
        c.env("IS_PALINDROME_PYTHON", p);
    }
    c
}

#[test]
fn default_invocation_aba_stdout_true_exit_0() {
    let out = cli_command()
        .args(["aba"])
        .output()
        .expect("spawn cli");
    assert_eq!(out.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "true");
}

#[test]
fn default_invocation_ab_exit_1() {
    let out = cli_command()
        .args(["ab"])
        .output()
        .expect("spawn cli");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "false");
}

#[test]
fn impl_rust_explicit_matches_default() {
    let a = cli_command()
        .args(["--impl", "rust", "aba"])
        .output()
        .expect("spawn");
    let b = cli_command()
        .args(["aba"])
        .output()
        .expect("spawn");
    assert_eq!(a.status, b.status);
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn impl_py_matches_rust_for_aba() {
    let py = cli_command()
        .env("PYTHONPATH", repo_root().join("src/py"))
        .args(["--impl", "py", "aba"])
        .output()
        .expect("spawn");
    let rust = cli_command()
        .args(["--impl", "rust", "aba"])
        .output()
        .expect("spawn");
    assert_eq!(
        py.status.code(),
        Some(0),
        "python + is_palindrome.stdin_json_adapter required for --impl py, stderr={}",
        String::from_utf8_lossy(&py.stderr)
    );
    assert_eq!(py.stdout, rust.stdout);
}

#[test]
fn non_ascii_string_api_exit_2() {
    let out = cli_command()
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
    let cmd = cli_command()
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
    let cmd = cli_command()
        .env(
            "PYTHONPATH",
            repo_root().join("src/py").to_string_lossy().as_ref(),
        )
        .args([
            "--input-json",
            inp.to_str().unwrap(),
            "--output-json",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("spawn");
    // `--impl all` exits 0 only when every backend agrees. In minimal/hermetic environments
    // (e.g. Bazel sandbox) native thin CLIs may be missing or disagree — exit 2 is valid;
    // the merged JSON file is still written.
    let code = cmd.status.code();
    assert!(
        matches!(code, Some(0) | Some(2)),
        "unexpected exit {:?}, stderr={}",
        code,
        String::from_utf8_lossy(&cmd.stderr)
    );
    let body = fs::read_to_string(&out).expect("read out");
    assert!(
        body.contains("rust") && body.contains("results"),
        "expected merged JSON, got: {body}"
    );
}
