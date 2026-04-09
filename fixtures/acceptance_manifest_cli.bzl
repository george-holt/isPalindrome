load("@rules_shell//shell:sh_test.bzl", "sh_test")
load("//:visibility.bzl", "ROOT_TEST_SUITE")

# Bazel shards for manifest CLI acceptance (one sh_test per backend for parallel CI).


def acceptance_manifest_cli_suite():
    """Subprocess CLI + adapter acceptance (``is_palindrome_cli``). Supplementary only.

    Canonical behavior checks are the per-language ``acceptance_test`` targets that load the
    same manifest in-process. These shards are tagged ``manual`` so ``bazel test //...`` runs
    the direct tests only; C# remains on ``rules_dotnet`` separately.
    """
    data = [
        "//CLI:is_palindrome_cli",
        "//CLI:stdin_json_adapter",
        "//src/c:stdin_json_adapter",
        "//src/cpp:stdin_json_adapter",
        "//src/cs:stdin_json_adapter",
        "//fixtures:acceptance_manifest.json",
        "bazel_acceptance_main.py",
    ] + native.glob(
        [
            "__init__.py",
            "cli/__init__.py",
            "cli/**/*.py",
        ],
    )

    # (backend_id, size); C# adapter is `//src/cs:stdin_json_adapter` (rules_dotnet) + host `dotnet`.
    shards = (
        ("c", "small"),
        ("cpp", "small"),
        ("cs", "small"),
        ("nodejs", "large"),  # manifest + c8 (npx) can be slow on cold cache / network
        ("py", "small"),
        ("rust", "small"),
    )

    tests = []
    for backend, size in shards:
        name = "acceptance_manifest_cli_" + backend
        sh_test(
            name = name,
            srcs = ["acceptance_manifest_cli_test.sh"],
            args = [backend],
            data = data,
            tags = [
                "acceptance",
                "manual",
            ],
            size = size,
        )
        tests.append(":" + name)

    native.test_suite(
        name = "acceptance_manifest_cli",
        tests = tests,
        tags = [
            "acceptance",
            "manual",
        ],
        visibility = ROOT_TEST_SUITE,
    )
