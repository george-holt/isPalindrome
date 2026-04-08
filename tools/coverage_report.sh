#!/usr/bin/env bash
# Deprecated: use ./tools/bazel-coverage.sh (single canonical coverage entry point).
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/bazel-coverage.sh" "$@"
