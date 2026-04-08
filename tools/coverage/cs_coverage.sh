#!/usr/bin/env bash
# C#: same manifest cases as //src/cs:acceptance_test, run under dotnet test + coverlet, then HTML via reportgenerator.
# Produces reports/coverage/cs/coverage.cobertura.xml and reports/coverage/cs/html/.
#
# Usage:
#   bazel run //tools/coverage:cs
#   ./tools/coverage/cs_coverage.sh
#
set -euo pipefail

ROOT="${BUILD_WORKING_DIRECTORY:-}"
if [[ -z "$ROOT" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
fi
cd "$ROOT"

command -v dotnet >/dev/null 2>&1 || {
  echo "missing: dotnet (SDK 8.x on PATH)" >&2
  exit 1
}

OUT="$ROOT/reports/coverage/cs"
mkdir -p "$OUT"
RESULTS="$OUT/_test_results"
rm -rf "$RESULTS"
mkdir -p "$RESULTS"

echo "==> dotnet test (coverlet collector) src/cs/AcceptanceTestCoverage/AcceptanceTestCoverage.csproj"
dotnet test "$ROOT/src/cs/AcceptanceTestCoverage/AcceptanceTestCoverage.csproj" \
  --configuration Release \
  --collect:"XPlat Code Coverage" \
  --results-directory "$RESULTS" \
  -- DataCollectionRunSettings.DataCollectors.DataCollector.Configuration.Format=cobertura

COB="$(find "$RESULTS" -name coverage.cobertura.xml -type f -print | head -1)"
if [[ -z "$COB" || ! -f "$COB" ]]; then
  echo "no coverage.cobertura.xml under $RESULTS" >&2
  exit 1
fi
cp -f "$COB" "$OUT/coverage.cobertura.xml"
echo "==> wrote $OUT/coverage.cobertura.xml"

echo "==> reportgenerator -> $OUT/html"
HTML="$OUT/html"
rm -rf "$HTML"
mkdir -p "$HTML"
(
  cd "$ROOT/tools/coverage"
  dotnet tool restore >/dev/null
  dotnet tool run reportgenerator -- \
    "-reports:$OUT/coverage.cobertura.xml" \
    "-targetdir:$HTML" \
    -reporttypes:Html
)

echo ""
echo "C# coverage (Cobertura): $OUT/coverage.cobertura.xml"
echo "C# coverage (HTML):      file://$HTML/index.html"
