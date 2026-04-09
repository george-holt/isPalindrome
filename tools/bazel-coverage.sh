#!/usr/bin/env bash
# Canonical entry: per-language coverage HTML + index (includes C# when generating HTML).
# Run from the repository root (CI, dev shell). Do not wrap this in ``bazel run`` — the script calls
# ``bazel build`` / ``bazel coverage`` and nested Bazel is unsupported.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec "$ROOT/tools/coverage/coverage_html.sh" "$@"
