load("@rules_cc//cc:defs.bzl", "cc_library")

package(default_visibility = ["//visibility:public"])

cc_library(
    name = "nlohmann_json",
    hdrs = glob([
        "single_include/**/*.hpp",
        "include/**/*.hpp",
    ]),
    includes = [
        "single_include",
        "include",
    ],
)
