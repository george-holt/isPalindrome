#!/usr/bin/env bash
# Build all compiled ports: C++, C, Rust, Node deps, .NET (library, tests, PalCli).
# Run from anywhere; assumes repo root contains cpp/, c/, rust/, nodejs/, cs/.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> CMake: cpp"
cmake -S cpp -B cpp/build
cmake --build cpp/build

echo "==> CMake: c"
cmake -S c -B c/build
cmake --build c/build

echo "==> Rust (cargo build)"
( cd rust/is_palindrome && cargo build )

echo "==> Node (npm install)"
( cd nodejs/is-palindrome && npm install )

echo "==> .NET"
dotnet build "$ROOT/cs/IsPalindrome.sln" --nologo -v minimal
dotnet build "$ROOT/cs/PalCli/PalCli.csproj" --nologo -v minimal

echo "==> build_all: done"
