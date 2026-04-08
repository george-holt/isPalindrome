#!/usr/bin/env bash
# One command: full-repo coverage HTML (Rust/C/C++/Python/Node + C#).
# Run: bazel run //tools/coverage:full
# Forwards args to coverage_html.sh (which runs C# coverage when building HTML).
set -euo pipefail

ROOT="${BUILD_WORKING_DIRECTORY:-}"
if [[ -z "$ROOT" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
fi
cd "$ROOT"

"$ROOT/tools/coverage/coverage_html.sh" "$@"
