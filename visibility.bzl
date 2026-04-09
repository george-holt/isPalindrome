"""Named visibility widenings for the repo dependency graph.

Use these constants on `visibility` attributes so cross-package edges stay
explicit and Bazel rejects invalid graphs at analysis time.

Package groups are defined in //:BUILD.bazel; this file only holds stable
aliases for labels used from other packages' BUILD files.
"""

# Aggregated at `//:all_tests` (root test suite).
ROOT_TEST_SUITE = ["//:root_test_suite"]

# JSON manifest shared by language acceptance tests and the Rust CLI tests.
ACCEPTANCE_MANIFEST_JSON = ["//:acceptance_manifest_json_consumers"]

# `acceptance_manifest.json` wrapped for rules_js (Node acceptance test).
ACCEPTANCE_MANIFEST_JS = ["//:acceptance_manifest_js_consumers"]

# `fixtures/manifest_cases.py` — Python acceptance only.
MANIFEST_CASES_PY = ["//:manifest_cases_py_consumers"]

# Rust core library; only the polyglot CLI links it today.
CLI_RUST_CORE = ["//:cli_rust_core_consumers"]

# Python sources bundled into CLI runfiles (`--impl py`).
CLI_PY_SOURCES = ["//:cli_py_sources_consumers"]

# `MODULE.bazel` as a runfile for CLI integration/unit tests.
CLI_MODULE_BAZEL = ["//:cli_module_bazel_consumers"]

# Subprocess CLI acceptance harness under //fixtures (binaries + stdin adapters as runfiles).
FIXTURES_CLI_ACCEPTANCE = ["//:fixtures_cli_acceptance_consumers"]

# Scripts and exported files under //tools/coverage consumed by //tools tests.
TOOLS_COVERAGE = ["//:tools_coverage_consumers"]
