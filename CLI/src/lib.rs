//! **IsPalindromeCLI** — argv parsing, JSON I/O, backend dispatch, `--impl all` merge.

use is_palindrome::{is_palindrome, is_palindrome_from_utf8, PalindromeError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// Same shape as Python `ParsedCheck`.
#[derive(Debug, Clone)]
pub struct ParsedCheck {
    pub show_help: bool,
    pub hex_mode: bool,
    pub hex_payload: Option<String>,
    pub use_stdin: bool,
    pub positional: Vec<String>,
    pub custom_bytes: HashSet<u8>,
}

#[derive(Debug)]
pub enum ParseArgvError {
    Message(String),
}

impl std::fmt::Display for ParseArgvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseArgvError::Message(s) => write!(f, "{s}"),
        }
    }
}

pub fn parse_hex_byte(h: &str) -> Result<u8, ParseArgvError> {
    if h.len() != 2 {
        return Err(ParseArgvError::Message(
            "Each --custom value must be two hex digits.".into(),
        ));
    }
    u8::from_str_radix(h, 16).map_err(|_| {
        ParseArgvError::Message("Each --custom value must be two hex digits.".into())
    })
}

pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>, ParseArgvError> {
    if hex_str.len() % 2 != 0 {
        return Err(ParseArgvError::Message(
            "Hex input must have an even number of characters.".into(),
        ));
    }
    (0..hex_str.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex_str[i..i + 2], 16).map_err(|_| {
                ParseArgvError::Message("Hex input must have an even number of characters.".into())
            })
        })
        .collect()
}

pub fn parse_check_argv(args: &[String]) -> Result<ParsedCheck, ParseArgvError> {
    let mut show_help = false;
    let mut hex_mode = false;
    let mut hex_payload: Option<String> = None;
    let mut use_stdin = false;
    let mut positional: Vec<String> = Vec::new();
    let mut custom: HashSet<u8> = HashSet::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "-h" || a == "--help" {
            show_help = true;
            return Ok(ParsedCheck {
                show_help,
                hex_mode: false,
                hex_payload: None,
                use_stdin: false,
                positional: vec![],
                custom_bytes: HashSet::new(),
            });
        }
        if a == "--stdin" {
            use_stdin = true;
            i += 1;
            continue;
        }
        if a == "--hex" {
            if i + 1 >= args.len() {
                return Err(ParseArgvError::Message("--hex requires a hex string.".into()));
            }
            hex_mode = true;
            i += 1;
            hex_payload = Some(args[i].clone());
            i += 1;
            continue;
        }
        if a == "--custom" {
            if i + 1 >= args.len() {
                return Err(ParseArgvError::Message(
                    "--custom requires a two-digit hex byte.".into(),
                ));
            }
            i += 1;
            custom.insert(parse_hex_byte(&args[i])?);
            i += 1;
            continue;
        }
        if a == "--" {
            i += 1;
            while i < args.len() {
                positional.push(args[i].clone());
                i += 1;
            }
            break;
        }
        positional.push(a.clone());
        i += 1;
    }
    Ok(ParsedCheck {
        show_help,
        hex_mode,
        hex_payload,
        use_stdin,
        positional,
        custom_bytes: custom,
    })
}

pub fn resolve_text(
    stdin_content: &str,
    use_stdin: bool,
    hex_mode: bool,
    positional: &[String],
) -> String {
    if use_stdin || (positional.is_empty() && !hex_mode) {
        stdin_content.to_string()
    } else {
        positional.join(" ")
    }
}

pub fn validate_impl(name: &str) -> bool {
    matches!(
        name,
        "py" | "cpp" | "c" | "rust" | "cs" | "nodejs" | "all"
    )
}

pub const BACKENDS_ORDERED: &[&str] = &["c", "cpp", "cs", "nodejs", "py", "rust"];

#[derive(Debug, Clone, Serialize)]
pub struct BackendRow {
    pub backend: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct JsonOutSingle {
    result: Option<bool>,
    error: Option<JsonErrBody>,
}

#[derive(Debug, Serialize)]
struct JsonErrBody {
    code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct JsonOutAll {
    #[serde(rename = "impl")]
    impl_name: String,
    results: Vec<BackendRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    consensus: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct JsonRequest {
    mode: String,
    text: Option<String>,
    hex: Option<String>,
    #[serde(default)]
    custom: Vec<u8>,
    #[serde(rename = "impl")]
    impl_name: String,
}

fn custom_from_vec(custom: &[u8]) -> Option<HashSet<u8>> {
    if custom.is_empty() {
        None
    } else {
        Some(custom.iter().copied().collect())
    }
}

fn parsed_from_json_req(req: &JsonRequest) -> Result<ParsedCheck, String> {
    let hex_mode = req.mode == "hex";
    if hex_mode && req.hex.is_none() {
        return Err("hex mode requires hex field".into());
    }
    if !hex_mode && req.mode != "string" {
        return Err("mode must be string or hex".into());
    }
    Ok(ParsedCheck {
        show_help: false,
        hex_mode,
        hex_payload: req.hex.clone(),
        use_stdin: false,
        positional: vec![],
        custom_bytes: req.custom.iter().copied().collect(),
    })
}

fn resolved_text_for_json(req: &JsonRequest, parsed: &ParsedCheck) -> Result<String, String> {
    if parsed.hex_mode {
        Ok(String::new())
    } else {
        req.text
            .clone()
            .ok_or_else(|| "string mode requires text".into())
    }
}

pub fn parsed_to_json_payload(parsed: &ParsedCheck, resolved_text: &str) -> serde_json::Value {
    let custom: Vec<u8> = parsed.custom_bytes.iter().copied().collect();
    if parsed.hex_mode {
        serde_json::json!({
            "mode": "hex",
            "hex": parsed.hex_payload.as_deref().unwrap_or(""),
            "custom": custom,
        })
    } else {
        serde_json::json!({
            "mode": "string",
            "text": resolved_text,
            "custom": custom,
        })
    }
}

fn run_rust_inprocess(parsed: &ParsedCheck, resolved_text: &str) -> Result<bool, PalindromeError> {
    let custom: Vec<u8> = parsed.custom_bytes.iter().copied().collect();
    let custom = custom_from_vec(&custom);
    if parsed.hex_mode {
        let hex = parsed.hex_payload.as_deref().unwrap_or("");
        let data = decode_hex(hex).map_err(|_| PalindromeError {
            code: "INVALID_HEX",
        })?;
        Ok(is_palindrome(&data, custom.as_ref()))
    } else {
        is_palindrome_from_utf8(resolved_text, custom.as_ref())
    }
}

fn find_cpp_stdin_json_adapter(repo_root: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("IS_PALINDROME_CPP_STDIN_ADAPTER") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    let bazel = repo_root.join("bazel-bin/src/cpp");
    [
        bazel.join("stdin_json_adapter.exe"),
        bazel.join("stdin_json_adapter"),
    ]
    .into_iter()
    .find(|c| c.is_file())
}

fn find_c_stdin_json_adapter(repo_root: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("IS_PALINDROME_C_STDIN_ADAPTER") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    let bazel = repo_root.join("bazel-bin/src/c");
    [
        bazel.join("stdin_json_adapter.exe"),
        bazel.join("stdin_json_adapter"),
    ]
    .into_iter()
    .find(|c| c.is_file())
}

fn find_rust_stdin_json_adapter(repo_root: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("IS_PALINDROME_RUST_STDIN_ADAPTER") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    let bazel = repo_root.join("bazel-bin/CLI");
    [
        bazel.join("stdin_json_adapter.exe"),
        bazel.join("stdin_json_adapter"),
    ]
    .into_iter()
    .find(|c| c.is_file())
}

/// Bazel output `bazel-bin/src/cs/stdin_json_adapter/net8.0/StdinJsonAdapter.dll`, or MSBuild output under
/// `src/cs/StdinJsonAdapter/bin`. Set `IS_PALINDROME_CS_STDIN_ADAPTER_DLL` to override (e.g. runfiles).
fn find_cs_stdin_adapter_dll(repo_root: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("IS_PALINDROME_CS_STDIN_ADAPTER_DLL") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    let bazel_dll = repo_root.join("bazel-bin/src/cs/stdin_json_adapter/net8.0/StdinJsonAdapter.dll");
    if bazel_dll.is_file() {
        return Some(bazel_dll);
    }
    let base = repo_root.join("src/cs/StdinJsonAdapter/bin");
    for rel in [
        "Debug/net8.0/StdinJsonAdapter.dll",
        "Release/net8.0/StdinJsonAdapter.dll",
    ] {
        let dll = base.join(rel);
        if dll.is_file() {
            return Some(dll);
        }
    }
    None
}

/// When `IS_PALINDROME_MANIFEST_COVERAGE=1`, thin backends are wrapped or get tool-specific env so
/// manifest acceptance can collect per-invocation artifacts under `IS_PALINDROME_COVERAGE_DIR` or
/// `TEST_TMPDIR` (parallel-safe names, e.g. `coverage.py` `--parallel-mode`).
fn manifest_coverage_enabled() -> bool {
    matches!(std::env::var("IS_PALINDROME_MANIFEST_COVERAGE").as_deref(), Ok("1"))
}

/// Unique id per thin-backend subprocess when collecting manifest coverage (c8 dirs, cobertura names).
static MANIFEST_COV_SEQ: AtomicU64 = AtomicU64::new(0);

fn next_manifest_cov_seq() -> u64 {
    MANIFEST_COV_SEQ.fetch_add(1, Ordering::SeqCst)
}

fn manifest_coverage_output_dir() -> Option<PathBuf> {
    if !manifest_coverage_enabled() {
        return None;
    }
    let base = std::env::var_os("IS_PALINDROME_COVERAGE_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("TEST_TMPDIR").map(PathBuf::from))
        .unwrap_or_else(|| std::env::temp_dir().join("is_palindrome_manifest_cov"));
    if let Err(e) = fs::create_dir_all(&base) {
        eprintln!("is_palindrome_cli: could not create IS_PALINDROME_COVERAGE_DIR {base:?}: {e}");
        return None;
    }
    Some(base)
}

fn manifest_coverage_data_file_prefix() -> Option<PathBuf> {
    Some(manifest_coverage_output_dir()?.join(".coverage"))
}

/// LLVM IR coverage: use a **relative** profile name and `current_dir` = coverage output dir so
/// profraw files land in the writable tree (e.g. Bazel `sandbox_writable_path`) even when the
/// process cwd would otherwise be read-only runfiles.
fn apply_manifest_native_llvm_coverage(cmd: &mut Command) {
    let Some(dir) = manifest_coverage_output_dir() else {
        return;
    };
    cmd.env("LLVM_PROFILE_FILE", "llvm_%p.profraw").current_dir(dir);
}

/// Last non-empty line that is exactly `true` or `false` (thin adapters; `dotnet run` may print host noise on stdout).
fn last_thin_bool_line(stdout: &str) -> Option<&str> {
    stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .rev()
        .find(|l| *l == "true" || *l == "false")
}

fn thin_stdout_to_row(backend: &str, stdout: &str, stderr: &str, code: i32) -> BackendRow {
    let line = last_thin_bool_line(stdout).unwrap_or("");
    if (code == 0 || code == 1) && (line == "true" || line == "false") {
        return BackendRow {
            backend: backend.into(),
            status: "ok".into(),
            result: Some(line == "true"),
            message: None,
            code: None,
            complete: None,
            reason: None,
        };
    }
    if code == 2 {
        return BackendRow {
            backend: backend.into(),
            status: "error".into(),
            result: None,
            message: Some(stderr.trim().to_string()),
            code: stdout.trim().lines().next().map(|s| s.to_string()),
            complete: Some(true),
            reason: None,
        };
    }
    BackendRow {
        backend: backend.into(),
        status: "error".into(),
        result: None,
        message: Some(format!("exit {code} stdout={stdout:?} stderr={stderr:?}")),
        code: None,
        complete: Some(true),
        reason: None,
    }
}

fn run_command_with_stdin_json(cmd: &mut Command, payload: &str, backend: &str) -> BackendRow {
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return BackendRow {
                backend: backend.into(),
                status: "incomplete".into(),
                result: None,
                message: Some(e.to_string()),
                code: None,
                complete: Some(false),
                reason: Some("spawn".into()),
            };
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(payload.as_bytes()) {
            return BackendRow {
                backend: backend.into(),
                status: "incomplete".into(),
                result: None,
                message: Some(e.to_string()),
                code: None,
                complete: Some(false),
                reason: Some("stdin".into()),
            };
        }
    }
    match child.wait_with_output() {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            thin_stdout_to_row(backend, &stdout, &stderr, code)
        }
        Err(e) => BackendRow {
            backend: backend.into(),
            status: "incomplete".into(),
            result: None,
            message: Some(e.to_string()),
            code: None,
            complete: Some(false),
            reason: Some("wait".into()),
        },
    }
}

fn run_thin_backend(
    impl_name: &str,
    payload: &str,
    repo_root: &Path,
) -> BackendRow {
    match impl_name {
        "nodejs" => match which::which("node") {
            Ok(node) => {
                let nodejs_root = repo_root.join("src/nodejs/ispalindrome");
                let script = nodejs_root.join("stdin-json-adapter.mjs");
                if !script.is_file() {
                    return BackendRow {
                        backend: "nodejs".into(),
                        status: "skipped".into(),
                        result: None,
                        message: Some(format!("missing {}", script.display())),
                        code: None,
                        complete: None,
                        reason: None,
                    };
                }
                let mut cmd = if let Some(cov_dir) = manifest_coverage_output_dir() {
                    let slot = cov_dir.join(format!(
                        "c8_{}_{}",
                        std::process::id(),
                        next_manifest_cov_seq()
                    ));
                    if let Err(e) = fs::create_dir_all(&slot) {
                        return BackendRow {
                            backend: "nodejs".into(),
                            status: "error".into(),
                            result: None,
                            message: Some(format!("manifest coverage: mkdir c8 slot: {e}")),
                            code: None,
                            complete: Some(true),
                            reason: None,
                        };
                    }
                    let c8_tail = [
                        "--clean",
                        "--include",
                        "stdin-json-adapter.mjs",
                        "--include",
                        "isPalindrome.js",
                        "--reporter",
                        "lcov",
                        "--reports-dir",
                    ];
                    if let Ok(pnpm) = which::which("pnpm") {
                        let mut c = Command::new(pnpm);
                        c.args(["exec", "c8"])
                            .args(c8_tail)
                            .arg(&slot)
                            .arg(node.as_os_str())
                            .arg(script.as_os_str())
                            .current_dir(&nodejs_root)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());
                        c
                    } else if which::which("npx").is_ok() {
                        let mut c = Command::new("npx");
                        c.args(["--yes", "c8"])
                            .args(c8_tail)
                            .arg(&slot)
                            .arg(node.as_os_str())
                            .arg(script.as_os_str())
                            .current_dir(&nodejs_root)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());
                        c
                    } else {
                        let mut c = Command::new(node);
                        c.arg(&script)
                            .current_dir(&nodejs_root)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());
                        c
                    }
                } else {
                    let mut c = Command::new(node);
                    c.arg(&script)
                        .current_dir(&nodejs_root)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());
                    c
                };
                run_command_with_stdin_json(&mut cmd, payload, "nodejs")
            }
            Err(_) => BackendRow {
                backend: "nodejs".into(),
                status: "skipped".into(),
                result: None,
                message: Some("node not found on PATH".into()),
                code: None,
                complete: None,
                reason: None,
            },
        },
        "cs" => match which::which("dotnet") {
            Ok(dotnet) => {
                let proj = repo_root.join("src/cs/StdinJsonAdapter/StdinJsonAdapter.csproj");
                if !proj.is_file() {
                    return BackendRow {
                        backend: "cs".into(),
                        status: "skipped".into(),
                        result: None,
                        message: Some(format!("missing {}", proj.display())),
                        code: None,
                        complete: None,
                        reason: None,
                    };
                }
                if let Some(dll) = find_cs_stdin_adapter_dll(repo_root) {
                    let dll_parent = dll.parent().map(Path::to_path_buf);
                    let mut cmd = if let Some(cov_dir) = manifest_coverage_output_dir() {
                        let out = cov_dir.join(format!(
                            "cs_{}_{}.cobertura.xml",
                            std::process::id(),
                            next_manifest_cov_seq()
                        ));
                        if let Some(dc) = which::which("dotnet-coverage").ok() {
                            let mut c = Command::new(dc);
                            c.arg("collect")
                                .arg("-f")
                                .arg("cobertura")
                                .arg("-o")
                                .arg(&out)
                                .arg(&dotnet)
                                .arg(&dll)
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped());
                            if let Some(ref dir) = dll_parent {
                                c.current_dir(dir);
                            }
                            c
                        } else {
                            let mut c = Command::new(&dotnet);
                            c.arg(&dll)
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped());
                            if let Some(ref dir) = dll_parent {
                                c.current_dir(dir);
                            }
                            c
                        }
                    } else {
                        let mut c = Command::new(&dotnet);
                        c.arg(&dll)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());
                        if let Some(ref dir) = dll_parent {
                            c.current_dir(dir);
                        }
                        c
                    };
                    run_command_with_stdin_json(&mut cmd, payload, "cs")
                } else {
                    let mut cmd = Command::new(dotnet);
                    cmd.arg("run")
                        .arg("--project")
                        .arg(&proj)
                        .current_dir(repo_root.join("src/cs"))
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());
                    run_command_with_stdin_json(&mut cmd, payload, "cs")
                }
            }
            Err(_) => BackendRow {
                backend: "cs".into(),
                status: "skipped".into(),
                result: None,
                message: Some("dotnet not found on PATH".into()),
                code: None,
                complete: None,
                reason: None,
            },
        },
        "cpp" => {
            let Some(exe) = find_cpp_stdin_json_adapter(repo_root) else {
                return BackendRow {
                    backend: "cpp".into(),
                    status: "skipped".into(),
                    result: None,
                    message: Some("stdin JSON adapter not found for cpp (build cpp)".into()),
                    code: None,
                    complete: None,
                    reason: None,
                };
            };
            let mut cmd = Command::new(&exe);
            apply_manifest_native_llvm_coverage(&mut cmd);
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            run_command_with_stdin_json(&mut cmd, payload, "cpp")
        }
        "c" => {
            let Some(exe) = find_c_stdin_json_adapter(repo_root) else {
                return BackendRow {
                    backend: "c".into(),
                    status: "skipped".into(),
                    result: None,
                    message: Some("stdin JSON adapter not found for c (build c)".into()),
                    code: None,
                    complete: None,
                    reason: None,
                };
            };
            let mut cmd = Command::new(&exe);
            apply_manifest_native_llvm_coverage(&mut cmd);
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            run_command_with_stdin_json(&mut cmd, payload, "c")
        }
        "py" => match which::which("python3").or_else(|_| which::which("python")) {
            Ok(py_exe) => {
                let py_pkg_root = repo_root.join("src/py");
                let adapter = py_pkg_root.join("is_palindrome/stdin_json_adapter.py");
                if !adapter.is_file() {
                    return BackendRow {
                        backend: "py".into(),
                        status: "skipped".into(),
                        result: None,
                        message: Some(format!("missing {}", adapter.display())),
                        code: None,
                        complete: None,
                        reason: None,
                    };
                }
                let mut cmd = Command::new(&py_exe);
                cmd.env("PYTHONPATH", py_pkg_root.as_os_str())
                    .current_dir(repo_root);
                if let Some(data_file) = manifest_coverage_data_file_prefix() {
                    cmd.arg("-m")
                        .arg("coverage")
                        .arg("run")
                        .arg("--parallel-mode")
                        .arg("--data-file")
                        .arg(data_file)
                        .arg("-m")
                        .arg("is_palindrome.stdin_json_adapter");
                } else {
                    cmd.arg("-m").arg("is_palindrome.stdin_json_adapter");
                }
                cmd.stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                run_command_with_stdin_json(&mut cmd, payload, "py")
            }
            Err(_) => BackendRow {
                backend: "py".into(),
                status: "skipped".into(),
                result: None,
                message: Some("python3 not found on PATH".into()),
                code: None,
                complete: None,
                reason: None,
            },
        },
        "rust" => {
            let Some(exe) = find_rust_stdin_json_adapter(repo_root) else {
                return BackendRow {
                    backend: "rust".into(),
                    status: "skipped".into(),
                    result: None,
                    message: Some(
                        "Rust stdin JSON adapter not found (build //CLI:stdin_json_adapter or set IS_PALINDROME_RUST_STDIN_ADAPTER)."
                            .into(),
                    ),
                    code: None,
                    complete: None,
                    reason: None,
                };
            };
            let mut cmd = Command::new(&exe);
            apply_manifest_native_llvm_coverage(&mut cmd);
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            run_command_with_stdin_json(&mut cmd, payload, "rust")
        }
        _ => BackendRow {
            backend: impl_name.into(),
            status: "error".into(),
            result: None,
            message: Some("internal: not a thin backend".into()),
            code: None,
            complete: Some(true),
            reason: None,
        },
    }
}

fn rust_row(parsed: &ParsedCheck, resolved: &str) -> BackendRow {
    match run_rust_inprocess(parsed, resolved) {
        Ok(b) => BackendRow {
            backend: "rust".into(),
            status: "ok".into(),
            result: Some(b),
            message: None,
            code: None,
            complete: None,
            reason: None,
        },
        Err(e) => BackendRow {
            backend: "rust".into(),
            status: "error".into(),
            result: None,
            message: Some(e.to_string()),
            code: Some(e.code.to_string()),
            complete: Some(true),
            reason: None,
        },
    }
}

fn stdin_needed(parsed: &ParsedCheck) -> bool {
    parsed.use_stdin || (parsed.positional.is_empty() && !parsed.hex_mode)
}

fn read_stdin_string() -> String {
    let mut s = String::new();
    let _ = io::stdin().read_to_string(&mut s);
    s
}

/// Run one backend (`--impl`); `resolved_text` is the string/hex payload after CLI parsing.
///
/// For `rust` only: `rust_use_stdin_adapter` selects the subprocess stdin JSON adapter (explicit
/// `--impl rust`, JSON `impl: rust`, or `all`). When false, the Rust palindrome library runs
/// **in-process** (default CLI invocation with no `--impl`).
pub fn dispatch_backend(
    impl_name: &str,
    parsed: &ParsedCheck,
    resolved_text: &str,
    repo_root: &Path,
    rust_use_stdin_adapter: bool,
) -> BackendRow {
    match impl_name {
        "rust" => {
            if rust_use_stdin_adapter {
                let payload = serde_json::to_string(&parsed_to_json_payload(parsed, resolved_text))
                    .unwrap_or_else(|_| "{}".into());
                run_thin_backend("rust", &payload, repo_root)
            } else {
                rust_row(parsed, resolved_text)
            }
        }
        "c" | "cpp" | "cs" | "nodejs" | "py" => {
            let payload = serde_json::to_string(&parsed_to_json_payload(parsed, resolved_text))
                .unwrap_or_else(|_| "{}".into());
            run_thin_backend(impl_name, &payload, repo_root)
        }
        _ => BackendRow {
            backend: impl_name.into(),
            status: "error".into(),
            result: None,
            message: Some("unknown backend".into()),
            code: None,
            complete: Some(true),
            reason: None,
        },
    }
}

fn all_rows(parsed: &ParsedCheck, resolved_text: &str, repo_root: &Path) -> Vec<BackendRow> {
    BACKENDS_ORDERED
        .iter()
        .map(|b| {
            let rust_thin = *b == "rust";
            dispatch_backend(b, parsed, resolved_text, repo_root, rust_thin)
        })
        .collect()
}

fn consensus_and_exit(rows: &[BackendRow]) -> (Option<bool>, i32) {
    let mut oks: Vec<bool> = Vec::new();
    for r in rows {
        match r.status.as_str() {
            "ok" => {
                if let Some(b) = r.result {
                    oks.push(b);
                } else {
                    return (None, 2);
                }
            }
            "skipped" => {}
            "error" | "incomplete" => return (None, 2),
            _ => return (None, 2),
        }
    }
    if oks.is_empty() {
        return (None, 2);
    }
    let first = oks[0];
    if oks.iter().all(|&x| x == first) {
        (Some(first), if first { 0 } else { 1 })
    } else {
        (None, 2)
    }
}

pub fn print_help() {
    print!(
        "\
IsPalindromeCLI — palindrome check (default invocation; no `check` subcommand).

Usage:
  is_palindrome_cli [--impl BACKEND] [options] [TEXT...]
  is_palindrome_cli --input-json PATH [--output-json PATH]

Backends (--impl): c, cpp, cs, nodejs, py, rust, all  (default: rust)

Options:
  --help, -h       Show this help.
  --stdin          Read UTF-8 text from stdin (string API).
  --hex HEX        Byte mode: lowercase hex pairs.
  --custom XX      Repeatable: extra delimiter byte (hex pair).
  --input-json     Read request JSON (mode, text/hex, custom, impl).
  --output-json    Write response JSON (creates/overwrites file).
  --               End of options; remaining words are TEXT.

Exit: 0 = true, 1 = false, 2 = error or partial/`all` mismatch.
Stdout: one line, \"true\" or \"false\" (or JSON file when requested).

"
    );
}

pub fn find_repo_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("IS_PALINDROME_REPO_ROOT") {
        let pb = PathBuf::from(p);
        if pb.join("fixtures/acceptance_manifest.json").is_file() {
            return Some(pb);
        }
    }
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("fixtures/acceptance_manifest.json").is_file() {
            return Some(dir);
        }
        dir = dir.parent()?.to_path_buf();
    }
}

/// Returns `(impl_name, remaining_args, explicit_impl)` where `explicit_impl` is true iff the user
/// wrote `--impl …` on the argv (so `rust` uses the stdin adapter, not in-process).
fn parse_impl_prefix(args: &[String]) -> Result<(String, Vec<String>, bool), i32> {
    if args.len() >= 2 && args[0] == "--impl" {
        if args[1].is_empty() {
            eprintln!("--impl requires a backend name");
            return Err(2);
        }
        let name = args[1].clone();
        if !validate_impl(&name) {
            eprintln!(
                "unknown --impl {name:?}; expected one of: c, cpp, cs, nodejs, py, rust, all"
            );
            return Err(2);
        }
        Ok((name, args[2..].to_vec(), true))
    } else {
        Ok(("rust".to_string(), args.to_vec(), false))
    }
}

fn strip_json_flags(
    mut args: Vec<String>,
) -> Result<(Vec<String>, Option<PathBuf>, Option<PathBuf>), i32> {
    let mut input_json: Option<PathBuf> = None;
    let mut output_json: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--input-json" {
            if i + 1 >= args.len() {
                eprintln!("--input-json requires a path");
                return Err(2);
            }
            input_json = Some(PathBuf::from(args[i + 1].clone()));
            args.remove(i);
            args.remove(i);
            continue;
        }
        if args[i] == "--output-json" {
            if i + 1 >= args.len() {
                eprintln!("--output-json requires a path");
                return Err(2);
            }
            output_json = Some(PathBuf::from(args[i + 1].clone()));
            args.remove(i);
            args.remove(i);
            continue;
        }
        i += 1;
    }
    Ok((args, input_json, output_json))
}

pub fn run(args: Vec<String>, repo_root: &Path) -> i32 {
    let mut args = args;
    if args.is_empty() {
        eprintln!("missing argv[0]");
        return 2;
    }
    args.remove(0);

    let (args, input_json, output_json) = match strip_json_flags(args) {
        Ok(x) => x,
        Err(c) => return c,
    };

    if let Some(in_path) = input_json {
        let text = match fs::read_to_string(&in_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        let req: JsonRequest = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        if !validate_impl(&req.impl_name) {
            eprintln!("unknown impl in JSON");
            return 2;
        }
        let parsed = match parsed_from_json_req(&req) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        let resolved = match resolved_text_for_json(&req, &parsed) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        return run_json_mode(
            &req.impl_name,
            &parsed,
            &resolved,
            output_json.as_ref(),
            repo_root,
        );
    }

    let (impl_name, rest, explicit_impl) = match parse_impl_prefix(&args) {
        Ok(x) => x,
        Err(c) => return c,
    };

    let parsed = match parse_check_argv(&rest) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };

    if parsed.show_help {
        print_help();
        return 0;
    }

    let stdin_buf = if stdin_needed(&parsed) {
        read_stdin_string()
    } else {
        String::new()
    };
    let resolved = resolve_text(
        &stdin_buf,
        parsed.use_stdin,
        parsed.hex_mode,
        &parsed.positional,
    );

    if impl_name == "all" {
        let rows = all_rows(&parsed, &resolved, repo_root);
        let (consensus, code) = consensus_and_exit(&rows);
        if let Some(out_path) = output_json {
            let out = JsonOutAll {
                impl_name: "all".into(),
                results: rows,
                consensus,
            };
            if let Err(e) = fs::write(&out_path, serde_json::to_string_pretty(&out).unwrap()) {
                eprintln!("{e}");
                return 2;
            }
        } else {
            match consensus {
                Some(true) => println!("true"),
                Some(false) => println!("false"),
                None => eprintln!("backends did not agree or a backend failed"),
            }
        }
        return code;
    }

    let rust_use_stdin_adapter = impl_name == "rust" && explicit_impl;
    let row = dispatch_backend(
        &impl_name,
        &parsed,
        &resolved,
        repo_root,
        rust_use_stdin_adapter,
    );
    if let Some(out_path) = output_json {
        let out = match row.status.as_str() {
            "ok" => JsonOutSingle {
                result: row.result,
                error: None,
            },
            _ => JsonOutSingle {
                result: None,
                error: Some(JsonErrBody {
                    code: row.code.clone().unwrap_or_else(|| "ERROR".into()),
                    message: row.message.clone(),
                }),
            },
        };
        if let Err(e) = fs::write(&out_path, serde_json::to_string_pretty(&out).unwrap()) {
            eprintln!("{e}");
            return 2;
        }
        return match row.status.as_str() {
            "ok" => {
                if row.result == Some(true) {
                    0
                } else if row.result == Some(false) {
                    1
                } else {
                    2
                }
            }
            _ => 2,
        };
    }

    match row.status.as_str() {
        "ok" => {
            if row.result == Some(true) {
                println!("true");
                0
            } else if row.result == Some(false) {
                println!("false");
                1
            } else {
                eprintln!("internal: ok without result");
                2
            }
        }
        "skipped" => {
            eprintln!("{}", row.message.as_deref().unwrap_or("skipped"));
            2
        }
        "error" => {
            if let Some(m) = &row.message {
                eprintln!("{m}");
            }
            if let Some(c) = &row.code {
                eprintln!("{c}");
            }
            2
        }
        "incomplete" => {
            eprintln!("{}", row.message.as_deref().unwrap_or("incomplete"));
            2
        }
        _ => {
            eprintln!("unknown status");
            2
        }
    }
}

fn run_json_mode(
    impl_name: &str,
    parsed: &ParsedCheck,
    resolved: &str,
    output_json: Option<&PathBuf>,
    repo_root: &Path,
) -> i32 {
    if impl_name == "all" {
        let rows = all_rows(parsed, resolved, repo_root);
        let (consensus, code) = consensus_and_exit(&rows);
        if let Some(out_path) = output_json {
            let out = JsonOutAll {
                impl_name: "all".into(),
                results: rows,
                consensus,
            };
            if let Err(e) = fs::write(out_path, serde_json::to_string_pretty(&out).unwrap()) {
                eprintln!("{e}");
                return 2;
            }
        }
        return code;
    }

    let rust_use_stdin_adapter = impl_name == "rust";
    let row = dispatch_backend(
        impl_name,
        parsed,
        resolved,
        repo_root,
        rust_use_stdin_adapter,
    );
    if let Some(out_path) = output_json {
        let out = match row.status.as_str() {
            "ok" => JsonOutSingle {
                result: row.result,
                error: None,
            },
            _ => JsonOutSingle {
                result: None,
                error: Some(JsonErrBody {
                    code: row.code.clone().unwrap_or_else(|| "ERROR".into()),
                    message: row.message.clone(),
                }),
            },
        };
        if let Err(e) = fs::write(out_path, serde_json::to_string_pretty(&out).unwrap()) {
            eprintln!("{e}");
            return 2;
        }
    }
    match row.status.as_str() {
        "ok" => {
            if row.result == Some(true) {
                0
            } else if row.result == Some(false) {
                1
            } else {
                2
            }
        }
        _ => 2,
    }
}

#[cfg(test)]
mod thin_stdout_tests {
    use super::{last_thin_bool_line, thin_stdout_to_row};

    #[test]
    fn last_line_wins_after_host_banner() {
        assert_eq!(
            last_thin_bool_line("\nWelcome to .NET\n\nfalse\n"),
            Some("false")
        );
        assert_eq!(last_thin_bool_line("true"), Some("true"));
    }

    #[test]
    fn thin_stdout_ok_when_true_is_last() {
        let row = thin_stdout_to_row("cs", "noise\ntrue\n", "", 0);
        assert_eq!(row.status, "ok");
        assert_eq!(row.result, Some(true));
    }
}
