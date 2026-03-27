//! Loads `fixtures/acceptance_manifest.json` (SPEC §4).

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use is_palindrome::{from_bytes, from_string};
use serde_json::Value;

fn manifest_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/acceptance_manifest.json")
}

fn decode_hex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0, "odd hex length");
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn unicode_scalar_to_string(manifest_scalar: &str) -> String {
    let prefix = "U+";
    assert!(manifest_scalar.len() > prefix.len());
    assert!(manifest_scalar.to_uppercase().starts_with(prefix));
    let hex = &manifest_scalar[prefix.len()..];
    let cp = u32::from_str_radix(hex, 16).unwrap();
    char::from_u32(cp)
        .map(|c| c.to_string())
        .unwrap_or_else(|| panic!("invalid scalar {manifest_scalar}"))
}

fn applies_to_rust(case: &Value) -> bool {
    match case.get("applies_to") {
        None => true,
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .any(|s| s == "rust"),
        _ => false,
    }
}

fn parse_custom_delimiters<'a>(
    case: &Value,
    buf: &'a mut HashSet<u8>,
) -> Option<&'a HashSet<u8>> {
    buf.clear();
    let opts = case.get("options")?;
    if opts.get("invalid_mode")?.as_str()? != "custom" {
        return None;
    }
    let arr = opts.get("invalid_bytes_hex")?.as_array()?;
    for h in arr {
        let hex = h.as_str()?;
        assert_eq!(hex.len(), 2);
        buf.insert(u8::from_str_radix(hex, 16).unwrap());
    }
    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

enum InputKind {
    Bytes(Vec<u8>),
    String(String),
}

fn build_input(case: &Value, string_api: bool) -> InputKind {
    if let Some(ascii) = case.get("input_ascii").and_then(|v| v.as_str()) {
        if string_api {
            return InputKind::String(ascii.to_string());
        }
        return InputKind::Bytes(ascii.as_bytes().to_vec());
    }
    if let Some(hex) = case.get("input_hex").and_then(|v| v.as_str()) {
        return InputKind::Bytes(decode_hex(hex));
    }
    if let Some(sc) = case.get("input_unicode_scalar").and_then(|v| v.as_str()) {
        return InputKind::String(unicode_scalar_to_string(sc));
    }
    panic!("case has no recognized input field");
}

#[test]
fn acceptance_manifest_matches_spec() {
    let path = manifest_path();
    assert!(path.exists(), "missing {}", path.display());
    let json = fs::read_to_string(&path).unwrap();
    let root: Value = serde_json::from_str(&json).unwrap();
    let cases = root.get("cases").unwrap().as_array().unwrap();

    let mut custom_buf = HashSet::new();

    for case in cases {
        let id = case.get("id").and_then(|v| v.as_str()).unwrap();
        if id == "pal-stream-note-001" {
            continue;
        }
        if !applies_to_rust(case) {
            continue;
        }

        let custom = parse_custom_delimiters(case, &mut custom_buf);
        let expected = case.get("expected").unwrap();
        let kind = expected.get("kind").and_then(|v| v.as_str()).unwrap();

        let string_api = case.get("applies_to").is_some();

        if kind == "boolean" {
            let want = expected.get("value").and_then(|v| v.as_bool()).unwrap();
            match build_input(case, string_api) {
                InputKind::Bytes(b) => {
                    assert_eq!(from_bytes(&b, custom), want, "{id}");
                }
                InputKind::String(s) => {
                    assert_eq!(from_string(&s, custom).unwrap(), want, "{id}");
                }
            }
        } else if kind == "error" {
            let code = expected.get("code").and_then(|v| v.as_str()).unwrap();
            match build_input(case, true) {
                InputKind::String(s) => {
                    let err = from_string(&s, custom).unwrap_err();
                    assert_eq!(err.code, code, "{id}");
                }
                InputKind::Bytes(_) => panic!("{id}: expected string input for error case"),
            }
        } else {
            panic!("unknown expected.kind in {id}");
        }
    }
}
