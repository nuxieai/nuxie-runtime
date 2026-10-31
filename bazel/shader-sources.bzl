"""Immutable upstream inputs for the build-only shader generator."""

_REVISION = "67946b1396fad83f4c140a38daa43d33e9fd6763"
_SHA256 = "ec7cb5b2a6a8b69973fe2c6264fdff10fc1b032aa036aa13c0b9398b80a56f3d"

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
