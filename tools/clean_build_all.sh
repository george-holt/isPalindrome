#!/usr/bin/env bash
# Remove native build outputs, then run tools/build_all.sh (full clean rebuild).
# Safe on POSIX (Linux, macOS, WSL); use Git Bash on Windows if bash is available.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> clean: cpp/build, c/build, Rust target, node_modules, .NET artifacts"
rm -rf cpp/build cpp/out c/build c/out
( cd rust/is_palindrome && cargo clean )
rm -rf nodejs/is-palindrome/node_modules

dotnet clean "$ROOT/cs/IsPalindrome.sln" --nologo -v minimal || true
dotnet clean "$ROOT/cs/PalCli/PalCli.csproj" --nologo -v minimal || true

echo "==> rebuild"
exec "$ROOT/tools/build_all.sh"
