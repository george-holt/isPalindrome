#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${TEST_SRCDIR:-}" ]]; then
  ROOT="${TEST_SRCDIR}/${TEST_WORKSPACE:-_main}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi
bash -n "$ROOT/tools/bazel-coverage.sh"
bash -n "$ROOT/tools/coverage/coverage_html.sh"
bash -n "$ROOT/tools/coverage_report.sh"
bash -n "$ROOT/tools/bazel_coverage_html.sh"
bash -n "$ROOT/tools/normalize_coverage_paths_test.sh"
python3 -m py_compile "$ROOT/tools/normalize_coverage_paths.py"
if [[ -n "${COVERAGE:-}" && -n "${COVERAGE_DIR:-}" ]]; then
  printf '%s\n' "TN:" "end_of_record" >"${COVERAGE_DIR}/_sh_tools_coverage_syntax.dat"
fi
exit 0
