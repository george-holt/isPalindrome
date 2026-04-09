#!/usr/bin/env bash
# Fails if language-matrix host tools are missing on PATH.
# Run from CI after setup-* steps and from .devcontainer/post-create.sh — not via Bazel
# (Bazel uses hermetic toolchains; this only validates the outer environment).
set -euo pipefail
missing=0
for cmd in python3 cargo rustc node npm dotnet; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing on PATH: $cmd" >&2
    missing=1
  fi
done
exit "$missing"
