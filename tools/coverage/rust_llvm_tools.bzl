"""Expose Rust toolchain ``llvm-cov`` / ``llvm-profdata`` as build outputs (no ``output_base`` scraping)."""

def _rust_llvm_coverage_tools_impl(ctx):
    toolchain = ctx.toolchains["@rules_rust//rust:toolchain_type"]
    if not toolchain.llvm_cov or not toolchain.llvm_profdata:
        fail("Rust toolchain is missing llvm-cov/llvm-profdata (need llvm-tools in the Rust dist).")
    cov = ctx.actions.declare_file("llvm-cov")
    prof = ctx.actions.declare_file("llvm-profdata")
    ctx.actions.symlink(
        output = cov,
        target_file = toolchain.llvm_cov,
        is_executable = True,
    )
    ctx.actions.symlink(
        output = prof,
        target_file = toolchain.llvm_profdata,
        is_executable = True,
    )
    return [DefaultInfo(files = depset([cov, prof]))]

rust_llvm_coverage_tools = rule(
    doc = "Symlinks the exec-config Rust toolchain's llvm-cov and llvm-profdata into this package under bazel-bin.",
    implementation = _rust_llvm_coverage_tools_impl,
    toolchains = ["@rules_rust//rust:toolchain_type"],
)
