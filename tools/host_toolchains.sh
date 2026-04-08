#!/usr/bin/env bash
# Fails if required host tools for language matrix tests are missing (see CI setup-*).
set -euo pipefail
missing=0
for cmd in python3 cargo rustc node npm dotnet; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing on PATH: $cmd" >&2
    missing=1
  fi
done
if [[ -n "${COVERAGE:-}" && -n "${COVERAGE_DIR:-}" ]]; then
  printf '%s\n' "TN:" "end_of_record" >"${COVERAGE_DIR}/_sh_tools_host_toolchains.dat"
fi
exit "$missing"
