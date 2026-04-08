#!/usr/bin/env bash
# Acceptance: LCOV/Cobertura paths normalize to virtual/lang-* and virtual/cli for ReportGenerator.
set -euo pipefail
if [[ -n "${TEST_SRCDIR:-}" ]]; then
  TOOLS="${TEST_SRCDIR}/${TEST_WORKSPACE:-_main}/tools"
else
  TOOLS="$(cd "$(dirname "$0")" && pwd)"
fi
PY="$TOOLS/normalize_coverage_paths.py"
python3 -m py_compile "$PY"

REPO="$(mktemp -d)"
VIRT="$(mktemp -d)"
cleanup() { rm -rf "$REPO" "$VIRT"; }
trap cleanup EXIT

mkdir -p "$REPO/CLI/src/bin" "$REPO/src/c/src" "$REPO/src/py"
touch "$REPO/src/c/src/is_palindrome.c"
touch "$REPO/CLI/src/bin/stdin_json_adapter.rs"
touch "$REPO/src/py/palindrome.py"

python3 "$PY" ensure-layout --repo "$REPO" --virtual "$VIRT"

OUT_LCOV="$(mktemp)"
python3 "$PY" lcov --repo "$REPO" --virtual "$VIRT" \
  "$TOOLS/testdata/coverage_normalize/sample_proc_cwd.lcov" "$OUT_LCOV"
grep -Fq "SF:${VIRT}/lang-c/src/is_palindrome.c" "$OUT_LCOV" || {
  echo "expected lang-c SF: in normalized LCOV" >&2
  sed -n '1,20p' "$OUT_LCOV" >&2
  exit 1
}
grep -Fq "SF:${VIRT}/cli/src/bin/stdin_json_adapter.rs" "$OUT_LCOV" || {
  echo "expected cli SF: in normalized LCOV" >&2
  exit 1
}

OUT_XML="$(mktemp)"
python3 "$PY" cobertura --repo "$REPO" --virtual "$VIRT" \
  "$TOOLS/testdata/coverage_normalize/sample_cobertura.xml" "$OUT_XML"
grep -Fq "${VIRT}/lang-py/palindrome.py" "$OUT_XML" || {
  echo "expected lang-py path in Cobertura filename" >&2
  cat "$OUT_XML" >&2
  exit 1
}
grep -Fq 'package name="lang-py"' "$OUT_XML" || {
  echo 'expected package name="lang-py"' >&2
  cat "$OUT_XML" >&2
  exit 1
}

exit 0
