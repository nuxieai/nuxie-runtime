"""Checksum-pinned Wasm packaging tools shared through the Bazel caches."""

_VERSION = "0.2.126"
_SHA256 = {
    "aarch64-apple-darwin": "7df536babe345deb68828148dbdc71179118afdab42d83547c7cebfbf1426bd5",
    "x86_64-apple-darwin": "6014dca993c8bf8a6ec10b6fccfbeabf599842d011f58a4abb7669afef784422",
    "aarch64-unknown-linux-musl": "2245120254a9f6c9a9adf3601f3d52bb31309219e9ceab7696e74e24885c440a",
    "x86_64-unknown-linux-musl": "064948d58e2d6c0a745216477a639ba696216d6309aaa902939d1b865b1d869d",
}

def _impl(ctx):
    arch = ctx.os.arch
    if arch in ["arm64", "aarch64"]:
        arch = "aarch64"
    elif arch in ["amd64", "x86_64"]:
        arch = "x86_64"
    else:
        fail("Unsupported wasm-bindgen host architecture: " + arch)
    if ctx.os.name == "mac os x":
        host = arch + "-apple-darwin"
    elif ctx.os.name == "linux":
        host = arch + "-unknown-linux-musl"
    else:
        fail("Browser qualification requires macOS or Linux: " + ctx.os.name)
    archive = "wasm-bindgen-{}-{}".format(_VERSION, host)
    ctx.download_and_extract(
        url = "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/{}/{}.tar.gz".format(_VERSION, archive),
        sha256 = _SHA256[host],
        stripPrefix = archive,
    )
    ctx.file("BUILD.bazel", 'package(default_visibility = ["//visibility:public"])\nexports_files(["wasm-bindgen"])\n')

_repository = repository_rule(implementation = _impl)

def _extension_impl(_ctx):
    _repository(name = "runtime_browser_tools")

browser_tools = module_extension(implementation = _extension_impl)
