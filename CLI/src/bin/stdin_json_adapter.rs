//! Stdin JSON adapter binary (JSON on stdin) for subprocess checks against the Rust library.

use is_palindrome::{is_palindrome, is_palindrome_from_utf8};
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{self, Read};

#[derive(Deserialize)]
struct Req {
    mode: String,
    hex: Option<String>,
    text: Option<String>,
    custom: Vec<u8>,
}

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("stdin");
    let req: Req = match serde_json::from_str(buf.trim()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let custom: Option<HashSet<u8>> = if req.custom.is_empty() {
        None
    } else {
        Some(req.custom.iter().copied().collect())
    };
    match req.mode.as_str() {
        "hex" => {
            let hex = match req.hex {
                Some(h) => h,
                None => {
                    eprintln!("missing hex");
                    std::process::exit(2);
                }
            };
            let mut v = Vec::new();
            for i in (0..hex.len()).step_by(2) {
                let b = u8::from_str_radix(&hex[i..i + 2], 16).expect("hex pair");
                v.push(b);
            }
            let r = is_palindrome(&v, custom.as_ref());
            print!("{}\n", if r { "true" } else { "false" });
            std::process::exit(if r { 0 } else { 1 });
        }
        "string" => {
            let text = match req.text {
                Some(t) => t,
                None => {
                    eprintln!("missing text");
                    std::process::exit(2);
                }
            };
            match is_palindrome_from_utf8(&text, custom.as_ref()) {
                Ok(r) => {
                    print!("{}\n", if r { "true" } else { "false" });
                    std::process::exit(if r { 0 } else { 1 });
                }
                Err(e) => {
                    eprintln!("{}", e.code);
                    eprintln!("Input contains a scalar value > U+007F.");
                    std::process::exit(2);
                }
            }
        }
        _ => {
            eprintln!("unknown mode");
            std::process::exit(2);
        }
    }
}
