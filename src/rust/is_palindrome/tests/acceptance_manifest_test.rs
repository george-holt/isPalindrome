//! Direct manifest-driven tests: ``fixtures/acceptance_manifest.json`` executed in-process.
//! Coverage for ``lib.rs`` is collected via ``llvm-cov`` in ``tools/coverage/coverage_html.sh``:
//! Bazel's merged LCOV for this ``rust_test`` target is often empty across the rlib boundary.

use is_palindrome::{is_palindrome, is_palindrome_from_utf8};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Manifest {
    cases: Vec<Value>,
}

fn manifest_path() -> PathBuf {
    if let Ok(rf) = std::env::var("RUNFILES_DIR") {
        let ws = std::env::var("TEST_WORKSPACE").unwrap_or_else(|_| "_main".into());
        for rel in [
            format!("{ws}/fixtures/acceptance_manifest.json"),
            "fixtures/acceptance_manifest.json".to_string(),
        ] {
            let p = PathBuf::from(&rf).join(&rel);
            if p.is_file() {
                return p;
            }
        }
    }
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let p = d.join("fixtures/acceptance_manifest.json");
        if p.is_file() {
            return p;
        }
        d = d.parent().expect("repo root").to_path_buf();
    }
}

fn applies(case: &Value, lang: &str) -> bool {
    if case.get("expected").is_none() {
        return false;
    }
    match case.get("applies_to") {
        None => true,
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .any(|s| s == lang),
        _ => false,
    }
}

fn parse_custom(case: &Value) -> Option<HashSet<u8>> {
    let opts = case.get("options")?;
    if opts.get("invalid_mode")?.as_str()? != "custom" {
        return None;
    }
    let arr = opts.get("invalid_bytes_hex")?.as_array()?;
    if arr.is_empty() {
        return None;
    }
    let mut s = HashSet::new();
    for h in arr {
        let hex = h.as_str()?;
        s.insert(u8::from_str_radix(hex, 16).ok()?);
    }
    Some(s)
}

fn uses_string_api(case: &Value) -> bool {
    if case.get("input_unicode_scalar").is_some() {
        return true;
    }
    case.get("category")
        .and_then(|c| c.as_str())
        .is_some_and(|c| c == "string_api")
}

fn unicode_scalar_to_char(spec: &str) -> char {
    let p = spec.trim().to_uppercase();
    let rest = p.strip_prefix("U+").expect("unicode scalar");
    let cp = u32::from_str_radix(rest, 16).expect("hex");
    char::from_u32(cp).expect("scalar")
}

fn run_case(case: &Value) {
    let id = case.get("id").and_then(|v| v.as_str()).unwrap_or("?");
    let custom_owned = parse_custom(case);
    let custom = custom_owned.as_ref();
    let exp = case.get("expected").expect("expected");

    if uses_string_api(case) {
        let s: String = if let Some(sc) = case.get("input_unicode_scalar").and_then(|v| v.as_str()) {
            unicode_scalar_to_char(sc).to_string()
        } else {
            case.get("input_ascii")
                .and_then(|v| v.as_str())
                .expect("input_ascii")
                .to_string()
        };
        match is_palindrome_from_utf8(&s, custom) {
            Ok(got) => {
                assert_eq!(exp.get("kind").and_then(|k| k.as_str()), Some("boolean"), "{id}");
                let want = exp.get("value").and_then(|v| v.as_bool()).expect("value");
                assert_eq!(want, got, "{id}");
            }
            Err(e) => {
                assert_eq!(exp.get("kind").and_then(|k| k.as_str()), Some("error"), "{id}");
                let code = exp.get("code").and_then(|c| c.as_str()).expect("code");
                assert_eq!(code, e.code, "{id}");
                assert_eq!(e.to_string(), code, "{id}: Display matches manifest code");
            }
        }
        return;
    }

    let bytes: Vec<u8> = if let Some(a) = case.get("input_ascii").and_then(|v| v.as_str()) {
        a.as_bytes().to_vec()
    } else if let Some(h) = case.get("input_hex").and_then(|v| v.as_str()) {
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect()
    } else {
        panic!("{id}: no input");
    };

    let got = is_palindrome(&bytes, custom);
    assert_eq!(exp.get("kind").and_then(|k| k.as_str()), Some("boolean"), "{id}");
    let want = exp.get("value").and_then(|v| v.as_bool()).expect("value");
    assert_eq!(want, got, "{id}");
}

#[test]
fn acceptance_manifest_cases() {
    let path = manifest_path();
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let m: Manifest = serde_json::from_str(&text).expect("manifest json");
    for case in &m.cases {
        if !applies(case, "rust") {
            continue;
        }
        run_case(case);
    }
}
