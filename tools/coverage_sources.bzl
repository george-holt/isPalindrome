"""Rules that expose source files to Bazel's coverage manifest for shell-driven tests."""

def _instrumented_sources_impl(ctx):
    return [
        DefaultInfo(files = depset(ctx.files.srcs)),
        coverage_common.instrumented_files_info(
            ctx,
            source_attributes = ["srcs"],
        ),
    ]

instrumented_sources = rule(
    implementation = _instrumented_sources_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = True,
            mandatory = True,
        ),
    },
    fragments = ["coverage"],
)
