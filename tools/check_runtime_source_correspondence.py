#!/usr/bin/env python3
"""Check runtime file coverage, not semantic parity or review completion.

The pinned upstream tree is the inventory. Header/implementation pairs share
one Rust owner; four upstream naming exceptions follow their Rust modules.
Explicit adapted and deferred owners are reported separately, never as mirrors.
No generated ledger or stored completion count is required.
"""

import argparse
from pathlib import Path, PurePosixPath
import subprocess
import sys


OWNER_ROOT = Path("crates/nuxie-runtime/src/mechanical_port/source")
RENAMED_OWNERS = {
    "nested_animation": "animation/nested_animation",
    "property_recorder": "animation/property_recorder",
    "shapes/shape_paint_path": "shapes/paint/shape_paint_path",
    "text_engine": "text/text_engine",
}
# 845a82a9 extracts these implementations from existing Lua owners. The Rust
# backend-neutral traits and native Luau implementation already own this code.
ADAPTED_OWNERS = {
    "lua/lua_asset_reference": Path("crates/nuxie-scripting/src/vm/lua_blob.rs"),
    "lua/lua_atoms": Path("crates/nuxie-scripting/src/vm.rs"),
    "lua/lua_script_backend": Path("crates/nuxie-scripting/src/vm.rs"),
    "lua/lua_transition": Path("crates/nuxie-scripting/src/vm/lua_transition.rs"),
    "scripted/script_backend": Path("crates/nuxie-runtime/src/scripting.rs"),
}
# C++ object-layout padding across preprocessor configurations has no Rust ABI
# counterpart. This is not an exemption for scripting lifecycle behavior.
CXX_ONLY_OWNERS = {"scripting_slots"}
# d4fe1022 editor-native contracts are outside this runtime's feature surface.
# Exact names only: future editor headers still require a scope review.
EDITOR_ONLY_OWNERS = {
    "core/fractional_index", "core/field_types/core_fractional_index_type",
    "editor/core_handle", "editor/object_arena",
}
# Runtime expansion of editor_hooks leaves ordinary generated callbacks intact;
# editor journaling/validation hooks disappear. No editor API is implemented.
PREPROCESSOR_ADAPTED_OWNERS = {
    "core/editor_hooks": OWNER_ROOT / "generated/component_base.rs",
}
# User-deferred execution lane, UNIV-3728. Enumerate exact owners so new upstream
# files still fail this check until their scope is examined.
DEFERRED_OWNERS = {
    "wasm/artboard_wire", "wasm/browser_scripting_vm", "wasm/data_convert_wire",
    "wasm/gamepad_wire", "wasm/listener_wire", "wasm/module/gpu_proxy",
    "wasm/module/module_context", "wasm/module/render_proxy", "wasm/module_render",
    "wasm/module_tier_ladder", "wasm/path_effect_wire", "wasm/prelinked_aot", "wasm/wamr_state_transplant",
    "wasm/wasm_scripting_vm",
}


def upstream_owners(paths: list[str]) -> set[str]:
    owners = set()
    for path in paths:
        for prefix, extension in (("include/rive/", ".hpp"), ("src/", ".cpp")):
            if path.startswith(prefix) and path.endswith(extension):
                owners.add(str(PurePosixPath(path[len(prefix):]).with_suffix("")))
    return owners


def missing_owners(repo: Path, owners: set[str]) -> list[str]:
    missing = []
    for owner in sorted(owners):
        if owner in DEFERRED_OWNERS or owner in CXX_ONLY_OWNERS or owner in EDITOR_ONLY_OWNERS:
            continue
        target = PREPROCESSOR_ADAPTED_OWNERS.get(owner) or ADAPTED_OWNERS.get(owner,
            OWNER_ROOT / (RENAMED_OWNERS.get(owner, owner) + ".rs"))
        if not (repo / target).is_file() or not (repo / target).read_text().strip():
            missing.append(f"{owner} -> {target}")
    return missing


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--upstream-ref", required=True)
    args = parser.parse_args()
    paths = subprocess.check_output(
        ["git", "-C", str(args.upstream_root), "ls-tree", "-r", "--name-only",
         args.upstream_ref, "--", "include/rive", "src"], text=True,
    ).splitlines()
    owners = upstream_owners(paths)
    if not owners:
        raise ValueError("pinned upstream contains no runtime header/source owners")
    missing = missing_owners(args.repo_root, owners)
    if missing:
        print("Missing Rust source owners:\n" + "\n".join(missing), file=sys.stderr)
        return 1
    adapted = owners & ADAPTED_OWNERS.keys()
    deferred = owners & DEFERRED_OWNERS
    cxx_only = owners & CXX_ONLY_OWNERS
    editor_only = owners & EDITOR_ONLY_OWNERS
    preprocessor = owners & PREPROCESSOR_ADAPTED_OWNERS.keys()
    mirrored = len(owners) - len(adapted) - len(deferred) - len(cxx_only) - len(editor_only) - len(preprocessor)
    print(f"Runtime source correspondence: {mirrored} mirrored, {len(adapted)} adapted, "
          f"{len(cxx_only)} C++-ABI-only, {len(preprocessor)} preprocessor-adapted, "
          f"{len(editor_only)} editor-only, {len(deferred)} deferred "
          "(structural coverage only; not proof of behavioral parity).")
    for owner in sorted(adapted):
        print(f"Adapted: {owner} -> {ADAPTED_OWNERS[owner]}")
    for owner in sorted(cxx_only):
        print(f"C++-ABI-only: {owner}")
    for owner in sorted(preprocessor):
        print(f"Preprocessor-adapted (runtime callbacks only): {owner} -> {PREPROCESSOR_ADAPTED_OWNERS[owner]}")
    for owner in sorted(editor_only):
        print(f"Editor-only (unsupported WITH_RIVE_EDITOR): {owner}")
    for owner in sorted(deferred):
        print(f"Deferred (UNIV-3728): {owner}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Runtime source correspondence failed: {error}", file=sys.stderr)
        raise SystemExit(1)
