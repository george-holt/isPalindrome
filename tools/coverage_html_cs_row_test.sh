#!/usr/bin/env bash
# Ensures the combined coverage index includes C# (Coverlet) alongside Bazel languages.
set -euo pipefail
if [[ -n "${TEST_SRCDIR:-}" ]]; then
  ROOT="${TEST_SRCDIR}/${TEST_WORKSPACE:-_main}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi
SCRIPT="$ROOT/tools/coverage/coverage_html.sh"
grep -q 'cs_coverage\.sh' "$SCRIPT" || {
  echo "expected coverage_html.sh to invoke cs_coverage.sh for combined summary" >&2
  exit 1
}
grep -q 'cs/html/index\.html' "$SCRIPT" || {
  echo "expected C# report link cs/html/index.html in generated index" >&2
  exit 1
}
grep -q 'IsPalindrome\.cs' "$SCRIPT" || {
  echo "expected main C# algorithm file in summary row" >&2
  exit 1
}
if grep -q 'C# is out of scope' "$SCRIPT"; then
  echo "coverage_html.sh should not claim C# is out of scope (summary includes C#)" >&2
  exit 1
fi
if grep -q 'rstrip("0")' "$SCRIPT"; then
  echo "C# line %% should keep one decimal (e.g. 100.0) like lcov summaries; do not strip trailing zeros" >&2
  exit 1
fi
exit 0
