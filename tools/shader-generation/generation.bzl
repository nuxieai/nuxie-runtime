"""One cacheable tree artifact for all portable renderer shader products."""

def _generate_impl(ctx):
    output = ctx.actions.declare_directory(ctx.label.name)
    args = ctx.actions.args()
    args.add(ctx.file._driver)
    args.add("--manifest", ctx.file.manifest)
    args.add("--generator", ctx.file.generator)
    args.add("--overlay", ctx.file.overlay)
    args.add("--ply-init", ctx.file.ply_init)
    args.add("--naga", ctx.executable.naga)
    args.add("--glslang", ctx.executable.glslang)
    args.add("--spirv-opt", ctx.executable.spirv_opt)
    args.add("--output", output.path)
    ctx.actions.run(
        executable = "/usr/bin/python3",
        arguments = [args],
        inputs = depset(
            [ctx.file._driver, ctx.file._verifier, ctx.file.manifest, ctx.file.generator, ctx.file.overlay, ctx.file.ply_init] + ctx.files.sources + ctx.files.ply,
        ),
        tools = [tool[DefaultInfo].files_to_run for tool in [ctx.attr.naga, ctx.attr.glslang, ctx.attr.spirv_opt]],
        outputs = [output],
        env = {"PATH": "/usr/bin:/bin", "PYTHONDONTWRITEBYTECODE": "1", "PYTHONHASHSEED": "0"},
        use_default_shell_env = False,
        mnemonic = "RendererShaders",
        progress_message = "Generating pinned WGSL, GLSL and SPIR-V renderer shaders",
    )
    return [DefaultInfo(files = depset([output]))]

renderer_shaders = rule(
    implementation = _generate_impl,
    attrs = {
        "sources": attr.label(mandatory = True),
        "manifest": attr.label(allow_single_file = True, mandatory = True),
        "generator": attr.label(allow_single_file = True, mandatory = True),
        "overlay": attr.label(allow_single_file = True, mandatory = True),
        "ply_init": attr.label(allow_single_file = True, mandatory = True),
        "ply": attr.label(mandatory = True),
        "naga": attr.label(executable = True, cfg = "exec", mandatory = True),
        "glslang": attr.label(executable = True, cfg = "exec", mandatory = True),
        "spirv_opt": attr.label(executable = True, cfg = "exec", mandatory = True),
        "_driver": attr.label(default = ":generate.py", allow_single_file = True),
        "_verifier": attr.label(default = ":verify_inputs.py", allow_single_file = True),
    },
)
