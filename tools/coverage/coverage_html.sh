#!/usr/bin/env bash
# One-shot coverage HTML: direct ``acceptance_test`` targets + shared manifest.
# Rust uses ``llvm-cov``/``llvm-profdata`` from ``//tools/coverage:rust_llvm_tools`` (exec Rust toolchain)
# on the instrumented test binary (Bazel merged LCOV often misses rlib lines).
# C/C++/Python/Node use ``bazel coverage --combined_report=lcov``.
# C# uses ``tools/coverage/cs_coverage.sh`` (Coverlet Cobertura + ReportGenerator), linked from the same index.
#
# Usage (from repository root — not via ``bazel run``; this script invokes Bazel):
#   ./tools/bazel-coverage.sh
#   ./tools/coverage/coverage_html.sh
#   ./tools/coverage/coverage_html.sh --no-html   # LCOV only under reports/coverage/*.info
#
set -euo pipefail

ROOT="${BUILD_WORKING_DIRECTORY:-}"
if [[ -z "$ROOT" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
fi
cd "$ROOT"

NO_HTML=false
for arg in "$@"; do
  case "$arg" in
    --no-html) NO_HTML=true ;;
    -h | --help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
  esac
done

command -v bazel >/dev/null 2>&1 || {
  echo "missing: bazel" >&2
  exit 1
}

if [[ "$NO_HTML" != true ]]; then
  command -v genhtml >/dev/null 2>&1 || {
    echo "missing: genhtml (lcov package, e.g. apt install lcov)" >&2
    exit 1
  }
fi

OUT="$ROOT/reports/coverage"
mkdir -p "$OUT"
# Prior sandbox runs can leave read-only ``*.info`` / HTML dirs; replace cleanly each run.
rm -f "$OUT"/*.info
rm -rf "$OUT"/rust "$OUT"/c "$OUT"/cpp "$OUT"/python "$OUT"/nodejs "$OUT"/cs
rm -f "$OUT"/*.lib.info "$OUT"/_rust.workspace.info

declare -A MAIN_FILE=(
  [rust]=src/rust/is_palindrome/src/lib.rs
  [c]=src/c/src/is_palindrome.c
  [cpp]=src/cpp/src/IsPalindrome.cpp
  [python]=src/py/is_palindrome/palindrome.py
  [nodejs]=src/nodejs/ispalindrome/isPalindrome.js
  [csharp]=src/cs/IsPalindrome.cs
)

COVERAGE_REPORT="$ROOT/bazel-out/_coverage/_coverage_report.dat"
instr='^//src/c[/:],^//src/cpp[/:],^//src/nodejs/ispalindrome[/:],^//src/py[/:],^//src/rust/is_palindrome[/:]'

line_pct() {
  local f="$1"
  lcov --ignore-errors empty,unused --summary "$f" 2>&1 | grep 'lines' | sed -E 's/.*: ([0-9.]+)%.*/\1/' | head -1 || echo "—"
}

# Line % for ``IsPalindrome.cs`` from Coverlet Cobertura (partial types → dedupe line numbers; else root aggregate).
cs_line_pct() {
  local cob="$1"
  python3 - "$cob" <<'PY'
import sys
import xml.etree.ElementTree as ET

def localname(tag: str) -> str:
    return tag.split("}", 1)[-1] if "}" in tag else tag

path = sys.argv[1]
root = ET.parse(path).getroot()
by_num: dict[int, int] = {}
for elem in root.iter():
    if localname(elem.tag) != "class":
        continue
    fn = (elem.get("filename") or "").replace("\\", "/")
    if not fn.endswith("IsPalindrome.cs"):
        continue
    for line in elem.iter():
        if localname(line.tag) != "line":
            continue
        num = int(line.get("number") or 0)
        hits = int(line.get("hits") or 0)
        by_num[num] = max(by_num.get(num, 0), hits)
if by_num:
    covered = sum(1 for h in by_num.values() if h > 0)
    best = covered / len(by_num)
else:
    lr = root.get("line-rate")
    best = float(lr) if lr is not None else 0.0
pct = best * 100.0
print(f"{pct:.1f}")
PY
}

collect_rust_lcov() {
  local dest="$1"
  echo "==> Rust: llvm-cov on instrumented //src/rust/is_palindrome:acceptance_test"
  bazel build \
    --collect_code_coverage \
    --instrumentation_filter="${instr}" \
    //tools/coverage:rust_llvm_tools \
    //src/rust/is_palindrome:acceptance_test \
    >/dev/null
  local bin rdir prof llvm_dir runfiles_parent bins
  llvm_dir="$ROOT/bazel-bin/tools/coverage"
  if [[ ! -x "$llvm_dir/llvm-cov" || ! -x "$llvm_dir/llvm-profdata" ]]; then
    echo "missing Rust llvm tools under $llvm_dir (build //tools/coverage:rust_llvm_tools first)" >&2
    exit 1
  fi
  shopt -s nullglob
  bins=( "$ROOT/bazel-bin/src/rust/is_palindrome"/test-*/acceptance_test )
  shopt -u nullglob
  if [[ ${#bins[@]} -eq 0 ]]; then
    echo "missing //src/rust/is_palindrome:acceptance_test under bazel-bin/src/rust/is_palindrome/test-*/acceptance_test" >&2
    exit 1
  fi
  # rules_rust can leave multiple test-* trees (e.g. different configurations); use the newest mtime (the one this build refreshed).
  bin=""
  latest=-1
  for candidate in "${bins[@]}"; do
    [[ -f "$candidate" ]] || continue
    ts="$(python3 -c 'import os,sys; print(int(os.path.getmtime(sys.argv[1])))' "$candidate" 2>/dev/null || echo 0)"
    if [[ "$ts" -gt "$latest" ]]; then
      latest="$ts"
      bin="$candidate"
    fi
  done
  if [[ -z "$bin" ]]; then
    echo "no usable acceptance_test binary among: ${bins[*]}" >&2
    exit 1
  fi
  runfiles_parent="$(dirname "$bin")/acceptance_test.runfiles"
  if [[ ! -d "$runfiles_parent" ]]; then
    echo "missing runfiles dir: $runfiles_parent" >&2
    exit 1
  fi
  if [[ -d "$runfiles_parent/_main" ]]; then
    rdir="$runfiles_parent/_main"
  else
    found=( "$runfiles_parent"/* )
    if [[ ${#found[@]} -eq 1 && -d "${found[0]}" ]]; then
      rdir="${found[0]}"
    else
      echo "could not resolve runfiles workspace under $runfiles_parent (expected _main or exactly one subdirectory)" >&2
      exit 1
    fi
  fi
  prof="$OUT/_rust_profraw_tmp"
  rm -rf "$prof"
  mkdir -p "$prof"
  (
    export RUNFILES_DIR="$rdir"
    LLVM_PROFILE_FILE="$prof/rust-%m.profraw" "$bin"
  )
  shopt -s nullglob
  local merges=( "$prof"/*.profraw )
  shopt -u nullglob
  if [[ ${#merges[@]} -eq 0 ]]; then
    echo "no LLVM profraw emitted under $prof" >&2
    exit 1
  fi
  "$llvm_dir/llvm-profdata" merge -sparse "${merges[@]}" -o "$prof/merged.profdata"
  "$llvm_dir/llvm-cov" export "$bin" \
    -instr-profile="$prof/merged.profdata" \
    -format=lcov \
    --ignore-filename-regex='^/rustc/|^/registry/src/|/\.cargo/|rules_rust~~crate~crates__|/external/' \
    >"$dest"
  rm -rf "$prof"
}

filter_rust_for_algo_html() {
  local raw="$1"
  local ws="$OUT/_rust.workspace.info"
  local algo="$2"
  lcov --ignore-errors unused,empty --remove "$raw" 'external/*' -o "$ws" >/dev/null
  lcov --ignore-errors unused,empty --extract "$ws" '*is_palindrome/src/lib.rs' -o "$algo" >/dev/null
  rm -f "$ws"
}

# Restrict LCOV to the single algorithm source (exclude Catch2 / test driver) for a fair row in the index.
filter_cc_lcov_algo() {
  local src="$1"
  local pattern="$2"
  local dest="$3"
  lcov --ignore-errors unused,empty --extract "$src" "$pattern" -o "$dest" >/dev/null
}

rows_html=""

collect_rust_lcov "$OUT/rust.info"
rust_algo="$OUT/rust.lib.info"
filter_rust_for_algo_html "$OUT/rust.info" "$rust_algo"
pct="$(line_pct "$rust_algo")"
if [[ "$NO_HTML" != true ]]; then
  genhtml --quiet --ignore-errors empty,source --branch-coverage --title "isPalindrome rust (lib.rs)" -o "$OUT/rust" "$rust_algo"
  rows_html="${rows_html}
<tr><td>rust</td><td><code>${MAIN_FILE[rust]}</code></td><td>${pct}%</td><td><a href=\"rust/index.html\">report</a></td></tr>"
fi

for lang in c cpp python nodejs; do
  case "$lang" in
    c) t="//src/c:acceptance_test" ;;
    cpp) t="//src/cpp:acceptance_test" ;;
    python) t="//src/py:acceptance_test" ;;
    nodejs) t="//src/nodejs/ispalindrome:acceptance_test" ;;
  esac
  echo "==> bazel coverage $t"
  bazel coverage \
    --nocache_test_results \
    --combined_report=lcov \
    --instrumentation_filter="${instr}" \
    "$t" \
    --test_output=errors
  dest="$OUT/${lang}.info"
  cp -f "$COVERAGE_REPORT" "$dest"
  html_src="$dest"
  case "$lang" in
    c)
      filter_cc_lcov_algo "$dest" '*/is_palindrome.c' "$OUT/c.lib.info"
      html_src="$OUT/c.lib.info"
      ;;
    cpp)
      filter_cc_lcov_algo "$dest" '*/IsPalindrome.cpp' "$OUT/cpp.lib.info"
      html_src="$OUT/cpp.lib.info"
      ;;
  esac
  pct="$(line_pct "$html_src")"
  if [[ "$NO_HTML" != true ]]; then
    genhtml --quiet --ignore-errors empty,source --branch-coverage --title "isPalindrome ${lang}" -o "$OUT/${lang}" "$html_src"
    rows_html="${rows_html}
<tr><td>${lang}</td><td><code>${MAIN_FILE[$lang]}</code></td><td>${pct}%</td><td><a href=\"${lang}/index.html\">report</a></td></tr>"
  fi
done

if [[ "$NO_HTML" != true ]]; then
  echo "==> C#: dotnet test + coverlet (see tools/coverage/cs_coverage.sh)"
  "$ROOT/tools/coverage/cs_coverage.sh"
  cob="$OUT/cs/coverage.cobertura.xml"
  if [[ ! -f "$cob" ]]; then
    echo "missing Cobertura after cs coverage: $cob" >&2
    exit 1
  fi
  pct="$(cs_line_pct "$cob")"
  rows_html="${rows_html}
<tr><td>csharp</td><td><code>${MAIN_FILE[csharp]}</code></td><td>${pct}%</td><td><a href=\"cs/html/index.html\">report</a></td></tr>"
fi

if [[ "$NO_HTML" == true ]]; then
  echo "LCOV files: $OUT/*.info"
  exit 0
fi

cat >"$OUT/index.html" <<EOF
<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>isPalindrome — coverage summary</title>
<style>
body{font-family:system-ui,sans-serif;margin:2rem;max-width:56rem}
table{border-collapse:collapse;width:100%}
th,td{border:1px solid #ccc;padding:0.4rem 0.6rem;text-align:left}
th{background:#f4f4f4}
code{font-size:0.9em}
</style>
</head>
<body>
<h1>Coverage summary (direct manifest tests)</h1>
<p>Shared cases: <code>fixtures/acceptance_manifest.json</code>. C/C++/Python/Node use <code>bazel coverage</code> merged LCOV. Rust uses <code>llvm-cov</code> on the instrumented <code>acceptance_test</code> binary so lines in <code>lib.rs</code> are attributed (Bazel&rsquo;s merged report is often empty across the rlib boundary). C# uses Coverlet Cobertura + ReportGenerator (see <code>tools/coverage/cs_coverage.sh</code>); the table links to that HTML and line % for <code>IsPalindrome.cs</code> when present in the Cobertura file.</p>
<table>
<thead><tr><th>Language</th><th>Main algorithm file</th><th>Lines (summary)</th><th>HTML</th></tr></thead>
<tbody>
${rows_html}
</tbody>
</table>
</body>
</html>
EOF

echo ""
echo "Coverage index: file://$OUT/index.html"
echo "              $OUT/index.html"
