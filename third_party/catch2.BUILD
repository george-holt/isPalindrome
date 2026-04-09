# Catch2 v3 — Bazel-native (no CMake). Minimal user config; defaults match upstream.
load("@rules_cc//cc:defs.bzl", "cc_library")

package(default_visibility = ["//visibility:public"])

genrule(
    name = "catch_user_config_gen",
    outs = ["src/catch2/catch_user_config.hpp"],
    cmd = """
cat > $@ <<'EOF'
#ifndef CATCH_USER_CONFIG_HPP_INCLUDED
#define CATCH_USER_CONFIG_HPP_INCLUDED
#endif
EOF
""",
)

cc_library(
    name = "catch2_with_main",
    srcs = glob(["src/catch2/**/*.cpp"]),
    hdrs = glob(["src/catch2/**/*.hpp"]) + [":catch_user_config_gen"],
    includes = ["src"],
    copts = select({
        "@platforms//os:windows": ["/utf-8"],
        "//conditions:default": [],
    }),
)
