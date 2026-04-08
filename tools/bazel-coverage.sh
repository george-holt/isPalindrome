#!/usr/bin/env bash
# Backward-compatible wrapper: per-language coverage HTML + index (includes C# when generating HTML).
# Preferred: ``bazel run //tools/coverage:html``
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec "$ROOT/tools/coverage/coverage_html.sh" "$@"
