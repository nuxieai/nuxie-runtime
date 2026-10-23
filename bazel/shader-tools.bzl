"""Checksum-pinned, build-only renderer shader compiler sources."""

def _sources_impl(ctx):
    ctx.download_and_extract(url = ctx.attr.url, sha256 = ctx.attr.sha256, stripPrefix = ctx.attr.prefix, type = "tar.gz")
    if ctx.attr.headers_url:
        ctx.download_and_extract(
            url = ctx.attr.headers_url,
            sha256 = ctx.attr.headers_sha256,
            stripPrefix = "SPIRV-Headers-vulkan-sdk-1.4.341.0",
            output = "external/spirv-headers",
        )
        # CMake consumes the complete source tree, not a nested Bazel package.
        ctx.delete("external/spirv-headers/BUILD.bazel")
    ctx.file("BUILD.bazel", """package(default_visibility = ["//visibility:public"])
exports_files(glob(["**"], exclude = ["**/BUILD", "**/BUILD.bazel"]))
filegroup(name = "sources", srcs = glob(["**"], exclude = ["**/BUILD", "**/BUILD.bazel"]))
""")
    return ctx.repo_metadata(reproducible = True)

_sources = repository_rule(
    implementation = _sources_impl,
    attrs = {
        "url": attr.string(mandatory = True),
        "sha256": attr.string(mandatory = True),
        "prefix": attr.string(mandatory = True),
        "headers_url": attr.string(),
        "headers_sha256": attr.string(),
    },
)

def _aliases_impl(ctx):
    ctx.file("BUILD.bazel", "\n".join([
        'package(default_visibility = ["//visibility:public"])',
        'alias(name = "naga", actual = "{}")'.format(ctx.attr.naga),
        'alias(name = "glslangValidator", actual = "{}")'.format(ctx.attr.glslang),
        'alias(name = "spirv-opt", actual = "{}")'.format(ctx.attr.spirv),
        'alias(name = "ply", actual = "{}")'.format(ctx.attr.ply),
        'alias(name = "ply_init", actual = "{}")'.format(ctx.attr.ply_init),
    ]))
    return ctx.repo_metadata(reproducible = True)

_aliases = repository_rule(
    implementation = _aliases_impl,
    attrs = {
        "naga": attr.label(default = Label("//bazel/shader-tool-build:naga")),
        "glslang": attr.label(default = Label("//bazel/shader-tool-build:glslangValidator")),
        "spirv": attr.label(default = Label("//bazel/shader-tool-build:spirv-opt")),
        "ply": attr.label(default = Label("@runtime_shader_ply//:sources")),
        "ply_init": attr.label(default = Label("@runtime_shader_ply//:ply/__init__.py")),
    },
)

def _impl(_ctx):
    _sources(
        name = "runtime_shader_glslang",
        url = "https://github.com/KhronosGroup/glslang/archive/refs/tags/16.2.0.tar.gz",
        sha256 = "01985335785c97906a91afe3cb5ee015997696181ec6c125bab5555602ba08e2",
        prefix = "glslang-16.2.0",
    )
    _sources(
        name = "runtime_shader_spirv",
        url = "https://github.com/KhronosGroup/SPIRV-Tools/archive/refs/tags/vulkan-sdk-1.4.341.0.tar.gz",
        sha256 = "15bfb678138cdf9cd1480dfb952547bbb66b763a735b6d5582578572f5c2e6f9",
        prefix = "SPIRV-Tools-vulkan-sdk-1.4.341.0",
        headers_url = "https://github.com/KhronosGroup/SPIRV-Headers/archive/refs/tags/vulkan-sdk-1.4.341.0.tar.gz",
        headers_sha256 = "cab0a654c4917e16367483296b44cdb1d614e3120c721beafcd37e3a8580486c",
    )
    _sources(
        name = "runtime_shader_naga",
        url = "https://static.crates.io/crates/naga-cli/naga-cli-30.0.0.crate",
        sha256 = "70c089479ee0825b0786deb835b200cc2242bb8d210078837937e8ff885946c4",
        prefix = "naga-cli-30.0.0",
    )
    _sources(
        name = "runtime_shader_ply",
        url = "https://files.pythonhosted.org/packages/e5/69/882ee5c9d017149285cab114ebeab373308ef0f874fcdac9beb90e0ac4da/ply-3.11.tar.gz",
        sha256 = "00c7c1aaa88358b9c765b6d3000c6eec0ba42abca5351b095321aef446081da3",
        prefix = "ply-3.11",
    )
    _aliases(name = "runtime_shader_tools")

shader_tools = module_extension(implementation = _impl)
