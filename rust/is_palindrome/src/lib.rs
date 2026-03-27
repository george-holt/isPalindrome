//! Core palindrome check (SPEC §2–§3). Standard library only.

pub mod cli;

use std::collections::HashSet;

/// String API: any Unicode scalar &gt; U+007F (SPEC §3).
#[derive(Debug, PartialEq, Eq)]
pub struct PalindromeError {
    pub code: &'static str,
}

impl std::fmt::Display for PalindromeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl std::error::Error for PalindromeError {}

fn is_ascii_alnum(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9')
}

fn is_ascii_letter(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z')
}

fn bytes_match(a: u8, b: u8) -> bool {
    if is_ascii_letter(a) && is_ascii_letter(b) {
        (a | 32) == (b | 32)
    } else {
        a == b
    }
}

fn is_valid_byte(b: u8, custom: Option<&HashSet<u8>>) -> bool {
    if !is_ascii_alnum(b) {
        return false;
    }
    if let Some(set) = custom {
        if !set.is_empty() && set.contains(&b) {
            return false;
        }
    }
    true
}

/// Byte mode. `custom` delimiter bytes are skipped when non-empty (SPEC §2).
pub fn from_bytes(data: &[u8], custom: Option<&HashSet<u8>>) -> bool {
    if data.is_empty() {
        return true;
    }
    let mut l: i32 = 0;
    let mut r: i32 = data.len() as i32 - 1;
    loop {
        while l <= r && !is_valid_byte(data[l as usize], custom) {
            l += 1;
        }
        while l <= r && !is_valid_byte(data[r as usize], custom) {
            r -= 1;
        }
        if l >= r {
            return true;
        }
        if !bytes_match(data[l as usize], data[r as usize]) {
            return false;
        }
        l += 1;
        r -= 1;
    }
}

/// UTF-8 string: rejects any scalar &gt; U+007F (SPEC §3).
pub fn from_string(s: &str, custom: Option<&HashSet<u8>>) -> Result<bool, PalindromeError> {
    for c in s.chars() {
        if c as u32 > 0x7F {
            return Err(PalindromeError {
                code: "NON_ASCII_STRING_INPUT",
            });
        }
    }
    Ok(from_bytes(s.as_bytes(), custom))
}
