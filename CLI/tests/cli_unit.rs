//! Unit tests for `is_palindrome_cli` — exercise the library in-process so coverage
//! attributes execution to `lib.rs` (the `is_palindrome_cli` binary runs in a separate process).

use is_palindrome_cli::{
    decode_hex, find_repo_root, parse_check_argv, parse_hex_byte, parsed_to_json_payload,
    print_help, resolve_text, run, validate_impl, ParsedCheck, BACKENDS_ORDERED,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// Serialize JSON tests that toggle `IS_PALINDROME_RUST_STDIN_ADAPTER` (global env).
static RUST_ADAPTER_ENV_LOCK: Mutex<()> = Mutex::new(());

/// Runfiles or Cargo-built path to `stdin_json_adapter` for JSON `impl: rust`.
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
    if let Some(p) = option_env!("CARGO_BIN_EXE_stdin_json_adapter") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    None
}

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

#[test]
fn parse_argv_error_display() {
    let e = parse_hex_byte("gg").unwrap_err();
    assert_eq!(
        e.to_string(),
        "Each --custom value must be two hex digits."
    );
}

#[test]
fn parse_hex_byte_accepts_valid() {
    assert_eq!(parse_hex_byte("2c").unwrap(), 0x2c);
}

#[test]
fn decode_hex_accepts_pairs_and_rejects_odd_length() {
    assert_eq!(decode_hex("6162").unwrap(), vec![0x61, 0x62]);
    assert_eq!(decode_hex("61").unwrap(), vec![0x61]);
    assert!(decode_hex("611").unwrap_err().to_string().contains("even"));
    assert!(decode_hex("61a").unwrap_err().to_string().contains("even"));
}

#[test]
fn parse_check_argv_help_flags() {
    for flag in ["-h", "--help"] {
        let p = parse_check_argv(&[flag.into()]).unwrap();
        assert!(p.show_help);
        assert!(!p.hex_mode);
        assert!(p.positional.is_empty());
    }
}

#[test]
fn parse_check_argv_stdin_hex_custom_double_dash() {
    let p = parse_check_argv(
        &[
            "--stdin".into(),
            "--hex".into(),
            "6162".into(),
            "--custom".into(),
            "20".into(),
            "--".into(),
            "ignored".into(),
            "words".into(),
        ],
    )
    .unwrap();
    assert!(p.use_stdin);
    assert!(p.hex_mode);
    assert_eq!(p.hex_payload.as_deref(), Some("6162"));
    assert!(p.custom_bytes.contains(&0x20));
    assert_eq!(
        p.positional,
        vec![String::from("ignored"), String::from("words")]
    );
}

#[test]
fn parse_check_argv_errors_for_missing_hex_or_custom_value() {
    assert!(parse_check_argv(&["--hex".into()])
        .unwrap_err()
        .to_string()
        .contains("--hex"));
    assert!(parse_check_argv(&["--custom".into()])
        .unwrap_err()
        .to_string()
        .contains("--custom"));
}

#[test]
fn resolve_text_prefers_positional_when_present() {
    assert_eq!(
        resolve_text("", false, false, &[String::from("a"), String::from("b")]),
        "a b"
    );
}

#[test]
fn validate_impl_accepts_documented_backends() {
    for b in ["py", "cpp", "c", "rust", "cs", "nodejs", "all"] {
        assert!(validate_impl(b), "{b}");
    }
    assert!(!validate_impl("fortran"));
}

#[test]
fn backends_ordered_matches_help_text() {
    assert_eq!(
        BACKENDS_ORDERED,
        &["c", "cpp", "cs", "nodejs", "py", "rust"]
    );
}

#[test]
fn parsed_to_json_payload_string_and_hex() {
    let mut p = ParsedCheck {
        show_help: false,
        hex_mode: false,
        hex_payload: None,
        use_stdin: false,
        positional: vec![],
        custom_bytes: [0x20u8].into_iter().collect(),
    };
    let v = parsed_to_json_payload(&p, "aba");
    assert_eq!(v["mode"], serde_json::json!("string"));
    assert_eq!(v["text"], serde_json::json!("aba"));

    p.hex_mode = true;
    p.hex_payload = Some("6162".into());
    let v = parsed_to_json_payload(&p, "");
    assert_eq!(v["mode"], serde_json::json!("hex"));
    assert_eq!(v["hex"], serde_json::json!("6162"));
}

#[test]
fn print_help_does_not_panic() {
    print_help();
}

#[test]
fn find_repo_root_sees_acceptance_manifest() {
    let root = find_repo_root().expect("manifest walk");
    assert!(root.join("fixtures/acceptance_manifest.json").is_file());
}

#[test]
fn run_requires_argv0() {
    let repo = repo_root();
    assert_eq!(run(vec![], &repo), 2);
}

#[test]
fn run_help_zero() {
    let repo = repo_root();
    assert_eq!(run(vec!["cli".into(), "--help".into()], &repo), 0);
}

#[test]
fn run_default_aba_stdout_true() {
    let repo = repo_root();
    assert_eq!(run(vec!["cli".into(), "aba".into()], &repo), 0);
}

#[test]
fn run_json_string_mode_roundtrip() {
    let _g = RUST_ADAPTER_ENV_LOCK.lock().expect("rust adapter env lock");
    let prev_rust_adapter = std::env::var("IS_PALINDROME_RUST_STDIN_ADAPTER").ok();
    if let Some(p) = rust_stdin_adapter_bin() {
        std::env::set_var("IS_PALINDROME_RUST_STDIN_ADAPTER", p);
    }
    let dir = tempfile::tempdir().expect("tempdir");
    let inp = dir.path().join("in.json");
    let out = dir.path().join("out.json");
    fs::write(
        &inp,
        r#"{"mode":"string","text":"aba","custom":[],"impl":"rust"}"#,
    )
    .unwrap();
    let repo = repo_root();
    let code = run(
        vec![
            "cli".into(),
            "--input-json".into(),
            inp.to_string_lossy().into_owned(),
            "--output-json".into(),
            out.to_string_lossy().into_owned(),
        ],
        &repo,
    );
    match prev_rust_adapter {
        Some(v) => std::env::set_var("IS_PALINDROME_RUST_STDIN_ADAPTER", v),
        None => std::env::remove_var("IS_PALINDROME_RUST_STDIN_ADAPTER"),
    }
    assert_eq!(code, 0);
    let body = fs::read_to_string(&out).unwrap();
    assert!(body.contains("true") || body.contains("\"result\""));
}

#[test]
fn run_json_hex_mode_requires_hex_field() {
    let dir = tempfile::tempdir().expect("tempdir");
    let inp = dir.path().join("bad.json");
    fs::write(&inp, r#"{"mode":"hex","impl":"rust"}"#).unwrap();
    let repo = repo_root();
    let code = run(
        vec![
            "cli".into(),
            "--input-json".into(),
            inp.to_string_lossy().into_owned(),
        ],
        &repo,
    );
    assert_eq!(code, 2);
}

#[test]
fn run_rejects_unknown_impl_flag() {
    let repo = repo_root();
    let code = run(
        vec![
            "cli".into(),
            "--impl".into(),
            "nope".into(),
            "aba".into(),
        ],
        &repo,
    );
    assert_eq!(code, 2);
}

#[test]
fn run_impl_all_writes_json_when_paths_given() {
    let dir = tempfile::tempdir().expect("tempdir");
    let inp = dir.path().join("in.json");
    let out = dir.path().join("out.json");
    fs::write(
        &inp,
        r#"{"mode":"string","text":"aba","custom":[],"impl":"all"}"#,
    )
    .unwrap();
    let repo = repo_root();
    let code = run(
        vec![
            "cli".into(),
            "--input-json".into(),
            inp.to_string_lossy().into_owned(),
            "--output-json".into(),
            out.to_string_lossy().into_owned(),
        ],
        &repo,
    );
    assert!(matches!(code, 0 | 2));
    let body = fs::read_to_string(&out).unwrap();
    assert!(body.contains("consensus") || body.contains("results"));
}

#[test]
fn run_hex_cli_invalid_hex_exit_2() {
    let repo = repo_root();
    let code = run(
        vec![
            "cli".into(),
            "--impl".into(),
            "rust".into(),
            "--hex".into(),
            "gg".into(),
        ],
        &repo,
    );
    assert_eq!(code, 2);
}

#[test]
fn run_json_rejects_unknown_impl_in_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let inp = dir.path().join("in.json");
    fs::write(
        &inp,
        r#"{"mode":"string","text":"aba","custom":[],"impl":"not-a-backend"}"#,
    )
    .unwrap();
    let repo = repo_root();
    let code = run(
        vec![
            "cli".into(),
            "--input-json".into(),
            inp.to_string_lossy().into_owned(),
        ],
        &repo,
    );
    assert_eq!(code, 2);
}

#[test]
fn run_rejects_empty_impl_name() {
    let repo = repo_root();
    let code = run(
        vec!["cli".into(), "--impl".into(), "".into(), "aba".into()],
        &repo,
    );
    assert_eq!(code, 2);
}

#[test]
fn run_rejects_input_json_without_path() {
    let repo = repo_root();
    let code = run(vec!["cli".into(), "--input-json".into()], &repo);
    assert_eq!(code, 2);
}
