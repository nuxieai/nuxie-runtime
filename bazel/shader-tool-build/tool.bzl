"""Expose a foreign CMake binary as a declared executable tool."""

def _impl(ctx):
    binary = ctx.file.binary
    executable = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.symlink(output = executable, target_file = binary, is_executable = True)
    return [DefaultInfo(
        executable = executable,
        files = depset([executable]),
        runfiles = ctx.runfiles(files = [binary]),
    )]

compiler_tool = rule(
    implementation = _impl,
    executable = True,
    attrs = {"binary": attr.label(allow_single_file = True)},
)
