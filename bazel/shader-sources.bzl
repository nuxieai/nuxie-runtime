"""Immutable upstream inputs for the build-only shader generator."""

_REVISION = "031de75dbeb257d32c6fab51c99bf63c52c155f4"
_SHA256 = "4679909f0cb494b27f0d118d76b2776f882328612c262942deb3f0a7a306286d"

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
