"""Declare the C compiler used by the signature crate's Wasm build script."""

load("@rules_rust//cargo:defs.bzl", _cargo_build_script = "cargo_build_script")

_WASM = str(Label("//bazel:wasm"))
_CLANG = str(Label("@nuxie_wasm_c_tools//:clang"))
_AR = str(Label("@nuxie_wasm_c_tools//:llvm-ar"))
_SUPPORT = str(Label("@nuxie_wasm_c_tools//:compiler_support"))

def cargo_build_script(build_script_env = {}, data = [], **kwargs):
    wasm_env = dict(build_script_env)
    # rules_rust expands execpaths before entering the authored crate directory.
    # Target-qualified keys take precedence over rules_rust's no_cc fallback.
    wasm_env.update({
        "CC_wasm32_unknown_unknown": "$(execpath " + _CLANG + ")",
        "AR_wasm32_unknown_unknown": "$(execpath " + _AR + ")",
    })
    _cargo_build_script(
        build_script_env = select({_WASM: wasm_env, "//conditions:default": build_script_env}),
        # The tools attribute populates the script binary's runfiles. These
        # files belong to the execution action itself for absolute execpaths.
        data = data + select({_WASM: [_CLANG, _AR, _SUPPORT], "//conditions:default": []}),
        **kwargs
    )
