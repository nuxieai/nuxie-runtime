"""Executable pool packaging adapter; SDK compilation never runs on the caller."""

def _packager_impl(ctx):
    executable = ctx.actions.declare_file(ctx.label.name)
    script = ctx.file.script
    ctx.actions.write(
        executable,
        "#!/bin/sh\nset -eu\nexec /usr/bin/python3 \"$0.runfiles/%s/%s\" \"$@\"\n" % (ctx.workspace_name, script.short_path),
        is_executable = True,
    )
    return [DefaultInfo(
        executable = executable,
        runfiles = ctx.runfiles(files = [script] + ctx.files.data),
    )]

audio_apk_packager = rule(
    implementation = _packager_impl,
    executable = True,
    attrs = {
        "script": attr.label(allow_single_file = True, mandatory = True),
        "data": attr.label_list(allow_files = True),
    },
)
