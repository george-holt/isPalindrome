#!/usr/bin/env bash
# Deprecated: use ./tools/bazel-coverage.sh (manifest acceptance + CLI tests + unified HTML).
set -euo pipefail
exec "$(cd "$(dirname "$0")" && pwd)/bazel-coverage.sh" "$@"
