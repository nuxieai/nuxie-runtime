"""Declared provenance input used only by the distribution build scripts."""

_PROVENANCE_KEYS = [
    "NUX_RUNTIME_SOURCE_REVISION",
    "NUX_RUNTIME_BUILD_INPUTS_HASH",
    "NUX_RUNTIME_CONTRACT_FINGERPRINT",
    "NUX_RUNTIME_BUILD_PROFILE",
    "NUX_RUNTIME_RUSTC_VERSION",
    "NUX_RUNTIME_DISTRIBUTION_ROOT_PACKAGE",
]
_NUL = json.decode('"\\u0000"')

def _provenance_env_impl(ctx):
    lines = []
    for key in _PROVENANCE_KEYS:
        if key not in ctx.var:
            continue
        value = ctx.var[key]
        if _NUL in value or "\n" in value or "\r" in value or value.endswith("\\") or "${pwd}" in value:
            fail("Invalid scalar provenance value for " + key)
        lines.append(key + "=" + value)
    output = ctx.actions.declare_file(ctx.label.name + ".env")
    ctx.actions.write(output, "\n".join(lines) + ("\n" if lines else ""))
    return [DefaultInfo(files = depset([output]))]

provenance_env = rule(implementation = _provenance_env_impl)
