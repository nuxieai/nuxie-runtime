"""Run dependency-free JS transport tests on the pool's provisioned Node."""

def _impl(ctx):
    launcher = ctx.actions.declare_file(ctx.label.name + ".sh")
    paths = ["\"$TEST_SRCDIR/$TEST_WORKSPACE/" + source.short_path + "\"" for source in ctx.files.tests]
    ctx.actions.write(launcher, "#!/usr/bin/env bash\nset -euo pipefail\nnode --version\nexec node --test " + " ".join(paths) + "\n", is_executable = True)
    return [DefaultInfo(executable = launcher, runfiles = ctx.runfiles(files = ctx.files.tests + ctx.files.data))]

node_test = rule(
    implementation = _impl,
    test = True,
    attrs = {
        "tests": attr.label_list(allow_files = [".mjs"]),
        "data": attr.label_list(allow_files = True),
    },
)
