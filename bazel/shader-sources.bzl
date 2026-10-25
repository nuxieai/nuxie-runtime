"""Immutable upstream inputs for the build-only shader generator."""

_REVISION = "74e5d3f635c3c463fe10e4327261705549ee5919"
_SHA256 = "407a0f2df213b96c08799104a8bb807f8ef7ea86d0c43b875f67840d7553544f"

def _impl(ctx):
    manifest = ctx.read(ctx.attr.manifest)
    data = json.decode(manifest)
    if data["revision"] != _REVISION:
        fail("Shader source manifest revision does not match the source archive")
    ctx.download_and_extract(
        type = "tar.gz",
        url = "https://codeload.github.com/rive-app/rive-runtime/tar.gz/" + _REVISION,
        sha256 = _SHA256,
        stripPrefix = "rive-runtime-" + _REVISION,
    )
    ctx.file("source-manifest.json", manifest)
    ctx.file("BUILD.bazel", '\n'.join([
        'package(default_visibility = ["//visibility:public"])',
        'filegroup(name = "sources", srcs = ' + json.encode(sorted(data["files"].keys())) + ')',
        'filegroup(name = "manifest", srcs = ["source-manifest.json"])',
        '',
    ]))
    return ctx.repo_metadata(reproducible = True)

_repository = repository_rule(
    implementation = _impl,
    attrs = {"manifest": attr.label(default = Label(":shader-source-manifest.json"))},
)

def _extension_impl(_ctx):
    _repository(name = "runtime_shader_sources")

shader_sources = module_extension(implementation = _extension_impl)
