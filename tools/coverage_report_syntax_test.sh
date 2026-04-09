#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${TEST_SRCDIR:-}" ]]; then
  ROOT="${TEST_SRCDIR}/${TEST_WORKSPACE:-_main}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi
bash -n "$ROOT/tools/bazel-coverage.sh"
bash -n "$ROOT/tools/coverage/coverage_html.sh"
grep -Fq '//tools/coverage:rust_llvm_tools' "$ROOT/tools/coverage/coverage_html.sh" || {
  echo "coverage_html.sh must bazel build //tools/coverage:rust_llvm_tools (declared-path llvm-cov)" >&2
  exit 1
}
if grep -q 'find_rust_llvm_bin' "$ROOT/tools/coverage/coverage_html.sh"; then
  echo "coverage_html.sh must not use find_rust_llvm_bin (use //tools/coverage:rust_llvm_tools)" >&2
  exit 1
fi
if grep -q 'bazel info output_base' "$ROOT/tools/coverage/coverage_html.sh"; then
  echo "coverage_html.sh must not scrape bazel info output_base for llvm tools" >&2
  exit 1
fi
bash -n "$ROOT/tools/coverage_report.sh"
bash -n "$ROOT/tools/bazel_coverage_html.sh"
bash -n "$ROOT/tools/normalize_coverage_paths_test.sh"
python3 -m py_compile "$ROOT/tools/normalize_coverage_paths.py"
if [[ -n "${COVERAGE:-}" && -n "${COVERAGE_DIR:-}" ]]; then
  printf '%s\n' "TN:" "end_of_record" >"${COVERAGE_DIR}/_sh_tools_coverage_syntax.dat"
fi
exit 0
