#!/usr/bin/env bash
# ``tools/bazel-coverage.sh`` delegates to ``tools/coverage/coverage_html.sh`` (Bazel ``js_test`` / V8 coverage; no host pnpm for c8).
set -euo pipefail
if [[ -n "${TEST_SRCDIR:-}" ]]; then
  ROOT="${TEST_SRCDIR}/${TEST_WORKSPACE:-_main}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi
SCRIPT="$ROOT/tools/bazel-coverage.sh"
if ! grep -q 'coverage_html.sh' "$SCRIPT"; then
  echo "expected $SCRIPT to delegate to tools/coverage/coverage_html.sh" >&2
  exit 1
fi
if [[ -n "${COVERAGE:-}" && -n "${COVERAGE_DIR:-}" ]]; then
  printf '%s\n' "TN:" "end_of_record" >"${COVERAGE_DIR}/_sh_tools_coverage_pnpm_bootstrap.dat"
fi
exit 0
