#!/usr/bin/env bash
# Host PATH checks (python3, cargo, rustc, node, npm, dotnet) are environment setup,
# not Bazel hermetic tests — see tools/validate_host_toolchains.sh and CI / devcontainer.
set -euo pipefail
if [[ -n "${TEST_SRCDIR:-}" ]]; then
  ROOT="${TEST_SRCDIR}/${TEST_WORKSPACE:-_main}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi
build_bazel="$ROOT/tools/BUILD.bazel"
if grep -E '^[[:space:]]*name[[:space:]]*=[[:space:]]*"host_toolchains"' "$build_bazel" >/dev/null; then
  echo "tools/BUILD.bazel must not define sh_test host_toolchains; use tools/validate_host_toolchains.sh from CI and .devcontainer/post-create.sh instead." >&2
  exit 1
fi
if [[ -n "${COVERAGE:-}" && -n "${COVERAGE_DIR:-}" ]]; then
  printf '%s\n' "TN:" "end_of_record" >"${COVERAGE_DIR}/_sh_tools_host_toolchains_policy.dat"
fi
