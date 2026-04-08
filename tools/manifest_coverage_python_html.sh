#!/usr/bin/env bash
# Deprecated: Python manifest coverage merge is handled by ./tools/bazel-coverage.sh.
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/bazel-coverage.sh" "$@"
