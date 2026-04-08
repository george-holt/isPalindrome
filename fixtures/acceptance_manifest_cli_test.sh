#!/usr/bin/env bash
# Run manifest acceptance against the built Rust CLI.
# Bazel passes one backend id as $1 (c, cpp, cs, nodejs, py, rust); see ``acceptance_manifest_cli.bzl``.
# Manual run: no args → ``--all-backends``; or ``./acceptance_manifest_cli_test.sh rust`` for one backend.
# Implemented as sh_test so `bazel coverage` does not run rules_python's coveragepy wrapper on a
# manifest that includes the Rust binary in runfiles (see rules_python #2762: non-Python paths in
# COVERAGE_MANIFEST → "No data was collected" → failing lcov step).
set -euo pipefail

BACKEND="${1:-}"

# NuGet / `dotnet` need HOME; suppress first-run banner on stdout (breaks CLI parsing of `true`/`false`).
export HOME="${HOME:-${TEST_TMPDIR:-${TMPDIR:-/tmp}}}"
export DOTNET_CLI_HOME="${DOTNET_CLI_HOME:-${HOME}/.dotnet-cli-acceptance}"
mkdir -p "$DOTNET_CLI_HOME"
export DOTNET_NO_FIRST_TIME_EXPERIENCE=1
export DOTNET_CLI_TELEMETRY_OPTOUT=1

if [[ -n "${RUNFILES_DIR:-}" ]]; then
  R="${RUNFILES_DIR}/${TEST_WORKSPACE:-_main}"
  if [[ ! -d "$R" ]]; then
    R="${RUNFILES_DIR}/_main"
  fi
  if [[ ! -d "$R" ]]; then
    R="${RUNFILES_DIR}"
  fi
elif [[ -n "${TEST_SRCDIR:-}" ]]; then
  R="${TEST_SRCDIR}/${TEST_WORKSPACE:-_main}"
else
  R="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

export PYTHONPATH="$R"

cli=""
while IFS= read -r f; do
  # Runfiles may expose the binary as a symlink; find -type f skips those.
  if [[ -e "$f" && -x "$f" ]]; then
    cli="$f"
    break
  fi
done < <(find "$R" -name is_palindrome_cli 2>/dev/null || true)

if [[ -z "$cli" ]]; then
  echo "is_palindrome_cli not found under $R" >&2
  exit 2
fi
export IS_PALINDROME_CLI="$cli"

# Manifest-driven per-backend coverage (Python harness → CLI → thin adapters). When set, the CLI
# wraps adapters (e.g. ``coverage run`` for Python) and writes under ``IS_PALINDROME_COVERAGE_DIR``.
if [[ -n "${IS_PALINDROME_MANIFEST_COVERAGE:-}" ]]; then
  export IS_PALINDROME_COVERAGE_DIR="${IS_PALINDROME_COVERAGE_DIR:-${TEST_TMPDIR:-/tmp}/is_palindrome_manifest_cov}"
  mkdir -p "$IS_PALINDROME_COVERAGE_DIR"
fi

# Thin native adapters for strict multi-backend acceptance (env read by is_palindrome_cli).
# Preserve explicit env (e.g. coverage-instrumented paths from ``bazel test --test_env``).
IS_PALINDROME_RUST_STDIN_ADAPTER="${IS_PALINDROME_RUST_STDIN_ADAPTER:-}"
IS_PALINDROME_C_STDIN_ADAPTER="${IS_PALINDROME_C_STDIN_ADAPTER:-}"
IS_PALINDROME_CPP_STDIN_ADAPTER="${IS_PALINDROME_CPP_STDIN_ADAPTER:-}"
while IFS= read -r f; do
  [[ -e "$f" && -x "$f" ]] || continue
  case "$f" in
    *"/CLI/"* | *"/CLI"*)
      [[ -z "${IS_PALINDROME_RUST_STDIN_ADAPTER}" ]] && IS_PALINDROME_RUST_STDIN_ADAPTER="$f"
      ;;
    *"/src/c/"*)
      [[ -z "${IS_PALINDROME_C_STDIN_ADAPTER}" ]] && IS_PALINDROME_C_STDIN_ADAPTER="$f"
      ;;
    *"/src/cpp/"*)
      [[ -z "${IS_PALINDROME_CPP_STDIN_ADAPTER}" ]] && IS_PALINDROME_CPP_STDIN_ADAPTER="$f"
      ;;
  esac
done < <(find "$R" -name stdin_json_adapter 2>/dev/null || true)
export IS_PALINDROME_RUST_STDIN_ADAPTER IS_PALINDROME_C_STDIN_ADAPTER IS_PALINDROME_CPP_STDIN_ADAPTER

main_py="$R/fixtures/bazel_acceptance_main.py"
if [[ ! -f "$main_py" ]]; then
  echo "missing $main_py" >&2
  exit 2
fi

# C# adapter: Bazel-built `//src/cs:stdin_json_adapter` in runfiles (rules_dotnet), else `bazel-bin`, else `dotnet build` for manual runs.
if [[ -z "${IS_PALINDROME_CS_STDIN_ADAPTER_DLL:-}" ]]; then
  while IFS= read -r f; do
    [[ -f "$f" ]] || continue
    export IS_PALINDROME_CS_STDIN_ADAPTER_DLL="$f"
    break
  done < <(find "$R" -path '*/stdin_json_adapter/net8.0/StdinJsonAdapter.dll' -type f 2>/dev/null | sort || true)
fi
if [[ -z "${IS_PALINDROME_CS_STDIN_ADAPTER_DLL:-}" && -f "${R}/bazel-bin/src/cs/stdin_json_adapter/net8.0/StdinJsonAdapter.dll" ]]; then
  export IS_PALINDROME_CS_STDIN_ADAPTER_DLL="${R}/bazel-bin/src/cs/stdin_json_adapter/net8.0/StdinJsonAdapter.dll"
fi
if [[ -z "${IS_PALINDROME_CS_STDIN_ADAPTER_DLL:-}" ]] && command -v dotnet >/dev/null 2>&1 && [[ -d "$R/src/cs" ]]; then
  ( cd "$R/src/cs" && dotnet build StdinJsonAdapter/StdinJsonAdapter.csproj -v:q -nologo ) || true
  for cfg in Debug Release; do
    dll="$R/src/cs/StdinJsonAdapter/bin/$cfg/net8.0/StdinJsonAdapter.dll"
    if [[ -f "$dll" ]]; then
      export IS_PALINDROME_CS_STDIN_ADAPTER_DLL="$dll"
      break
    fi
  done
fi

status=0
if [[ -z "${BACKEND}" ]]; then
  python3 "$main_py" --all-backends || status=$?
else
  python3 "$main_py" --impl "${BACKEND}" || status=$?
fi

# Acceptance: with manifest coverage enabled, require real tool output (not only Bazel's COVERAGE stub).
if [[ "$status" -eq 0 && -n "${IS_PALINDROME_MANIFEST_COVERAGE:-}" ]]; then
  case "${BACKEND}" in
    py)
      shopt -s nullglob
      cov_files=( "${IS_PALINDROME_COVERAGE_DIR}"/.coverage* )
      shopt -u nullglob
      if [[ ${#cov_files[@]} -eq 0 ]]; then
        echo "IS_PALINDROME_MANIFEST_COVERAGE: expected Python coverage data under ${IS_PALINDROME_COVERAGE_DIR} (no .coverage*)" >&2
        exit 3
      fi
      ;;
    nodejs)
      if ! find "${IS_PALINDROME_COVERAGE_DIR}" -name lcov.info -print -quit | grep -q .; then
        echo "IS_PALINDROME_MANIFEST_COVERAGE: expected Node c8 lcov.info under ${IS_PALINDROME_COVERAGE_DIR}" >&2
        exit 3
      fi
      ;;
    c|cpp|rust)
      shopt -s nullglob
      profraws=( "${IS_PALINDROME_COVERAGE_DIR}"/llvm_*.profraw )
      shopt -u nullglob
      if [[ ${#profraws[@]} -eq 0 ]]; then
        echo "IS_PALINDROME_MANIFEST_COVERAGE: expected LLVM profraw (llvm_*.profraw) under ${IS_PALINDROME_COVERAGE_DIR} for ${BACKEND}" >&2
        exit 3
      fi
      ;;
    cs)
      shopt -s nullglob
      cob=( "${IS_PALINDROME_COVERAGE_DIR}"/*.cobertura.xml )
      shopt -u nullglob
      if [[ ${#cob[@]} -eq 0 ]]; then
        echo "IS_PALINDROME_MANIFEST_COVERAGE: expected *.cobertura.xml under ${IS_PALINDROME_COVERAGE_DIR} (install: dotnet tool install -g dotnet-coverage)" >&2
        exit 3
      fi
      ;;
  esac
fi

if [[ -n "${COVERAGE:-}" && -n "${COVERAGE_DIR:-}" ]]; then
  cov_tag="${BACKEND:-all}"
  printf '%s\n' "TN:" "end_of_record" >"${COVERAGE_DIR}/_sh_fixtures_acceptance_manifest_cli_${cov_tag}.dat"
fi

exit "$status"
