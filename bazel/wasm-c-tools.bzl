"""Checksummed LLVM tools for the runtime's freestanding Wasm C verifier."""

_VERSION = "27.0"
_SHA256S = {
    "arm64-linux": "4cf4c553c4640e63e780442146f87d83fdff5737f988c06a6e3b2f0228e37665",
    "arm64-macos": "055c3dc2766772c38e71a05d353e35c322c7b2c6458a36a26a836f9808a550f8",
    "x86_64-linux": "b7d4d944c88503e4f21d84af07ac293e3440b1b6210bfd7fe78e0afd92c23bc2",
    "x86_64-macos": "163dfd47f989b1a682744c1ae1f0e09a83ff5c4bbac9dcd8546909ab54cda5a1",
}

def _wasm_c_tools_impl(ctx):
    arch = ctx.os.arch
    if arch in ["aarch64", "arm64"]:
        arch = "arm64"
    elif arch in ["amd64", "x86_64"]:
        arch = "x86_64"
    else:
        fail("Unsupported Wasm C compiler host architecture: " + arch)
    if ctx.os.name == "mac os x":
        os = "macos"
    elif ctx.os.name == "linux":
        os = "linux"
    else:
        fail("The freestanding Wasm C verifier requires macOS or Linux")
    host = arch + "-" + os
    archive = "wasi-sdk-" + _VERSION + "-" + host
    ctx.download_and_extract(
        url = "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-27/" + archive + ".tar.gz",
        sha256 = _SHA256S[host],
        stripPrefix = archive,
    )
    # This distribution supplies LLVM executables and builtin headers only.
    # The authored verifier owns its freestanding headers and links no WASI libc.
    ctx.delete("share/wasi-sysroot")
    ctx.file("bin/nuxie-wasm-clang", """#!/bin/sh
set -eu
compiler_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec "$compiler_dir/clang" --target=wasm32-unknown-unknown -nostdlibinc "$@"
""", executable = True)
    ctx.template("BUILD.bazel", ctx.attr.build_file)
    return ctx.repo_metadata(reproducible = True)

wasm_c_tools_repository = repository_rule(
    implementation = _wasm_c_tools_impl,
    attrs = {"build_file": attr.label(default = Label(":wasm-c-tools.BUILD.bazel"))},
)

def _wasm_c_tools_extension_impl(_ctx):
    wasm_c_tools_repository(name = "nuxie_wasm_c_tools")

wasm_c_tools = module_extension(implementation = _wasm_c_tools_extension_impl)
