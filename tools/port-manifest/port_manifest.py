#!/usr/bin/env python3
"""Generate and check the rive-runtime C++ to Rust provenance manifest."""

from __future__ import annotations

import argparse
import collections
import json
import pathlib
import re
import sys
import tomllib


STATUSES = {"ported", "partial", "absent", "not-applicable"}
FEATURE_ROWS = {
    "src/artboard.cpp": (
        "partial",
        "crates/nuxie-runtime/src/artboard.rs",
        "F1: core artboard behavior is ported; Artboard::volume remains absent.",
    ),
    "src/text/cursor.cpp": (
        "partial",
        "crates/nuxie-runtime/src/text.rs",
        "F2: TextInput rendering is ported; cursor editing behavior is partial.",
    ),
    "src/command_queue.cpp": ("absent", "", "F3: command queue is absent."),
    "src/constraints/scrolling/elastic_scroll_physics.cpp": (
        "absent",
        "",
        "F4/F10: elastic scroll physics is absent and still needs its parity fixture.",
    ),
    "src/animation/keyboard_listener_group.cpp": (
        "absent",
        "",
        "F5: keyboard listener runtime is absent.",
    ),
    "src/semantic/semantic_manager.cpp": (
        "absent",
        "",
        "F6: semantics runtime is absent.",
    ),
    "src/lua/lua_promise.cpp": ("absent", "", "F7: Lua promise binding is absent."),
    "src/lua/renderer/lua_gpu.cpp": (
        "absent",
        "",
        "F7/F8: Lua GPU binding and ORE GPU host are absent.",
    ),
    "src/joystick.cpp": (
        "partial",
        "crates/nuxie-runtime/src/animation.rs",
        "F9: joystick behavior is ported but still needs a parity fixture.",
    ),
    "src/shapes/list_path.cpp": (
        "partial",
        "crates/nuxie-runtime/src/draw.rs",
        "F10: generic handling exists but still needs a parity fixture.",
    ),
    "src/async/work_pool.cpp": ("absent", "", "F12: async work pool is absent."),
    "src/listener_group.cpp": (
        "partial",
        "crates/nuxie-runtime/src/state_machine.rs",
        "F13: advanced ListenerGroup behavior remains latent.",
    ),
    "src/core/binary_writer.cpp": (
        "not-applicable",
        "",
        "F14: binary writing is outside the read-only runtime contract.",
    ),
}
FEATURE_ROWS.update(
    {
        "src/assets/audio_asset.cpp": (
            "partial",
            "crates/nuxie-runtime/src/objects.rs",
            "F1: audio asset object data is ported, but playback is absent.",
        ),
        "src/audio_event.cpp": ("absent", "", "F1: audio event firing is absent."),
        "src/command_server.cpp": ("absent", "", "F3: command server is absent."),
        "src/constraints/scrolling/clamped_scroll_physics.cpp": (
            "partial",
            "crates/nuxie-runtime/src/constraints.rs",
            "F4/F10: clamped scrolling is partial and still needs its parity fixture.",
        ),
        "src/constraints/scrolling/scroll_bar_constraint.cpp": (
            "absent",
            "",
            "F4: scroll bar constraint behavior is absent.",
        ),
        "src/constraints/scrolling/scroll_bar_constraint_proxy.cpp": (
            "absent",
            "",
            "F4: scroll bar constraint proxy behavior is absent.",
        ),
        "src/constraints/scrolling/scroll_constraint.cpp": (
            "partial",
            "crates/nuxie-runtime/src/constraints.rs",
            "F4: core scrolling is ported; interactive momentum remains partial.",
        ),
        "src/constraints/scrolling/scroll_constraint_proxy.cpp": (
            "partial",
            "crates/nuxie-runtime/src/constraints.rs",
            "F4: core scrolling is ported; interactive proxy behavior remains partial.",
        ),
        "src/constraints/scrolling/scroll_physics.cpp": (
            "partial",
            "crates/nuxie-runtime/src/constraints.rs",
            "F4: the scroll physics seam is only partially ported.",
        ),
        "src/animation/gamepad_listener_group.cpp": (
            "absent",
            "",
            "F5: gamepad listener runtime is absent.",
        ),
        "src/animation/semantic_listener_group.cpp": (
            "absent",
            "",
            "F5/F6: semantic listener runtime is absent.",
        ),
        "src/animation/text_input_listener_group.cpp": (
            "absent",
            "",
            "F5: text-input listener runtime is absent.",
        ),
        "src/animation/listener_types/listener_input_type_gamepad.cpp": (
            "absent",
            "",
            "F5: gamepad listener input runtime is absent.",
        ),
        "src/animation/listener_types/listener_input_type_keyboard.cpp": (
            "absent",
            "",
            "F5: keyboard listener input runtime is absent.",
        ),
        "src/animation/listener_types/listener_input_type_semantic.cpp": (
            "absent",
            "",
            "F5/F6: semantic listener input runtime is absent.",
        ),
        "src/input/gamepad_batch.cpp": (
            "absent",
            "",
            "F5: gamepad batch input runtime is absent.",
        ),
        "src/inputs/gamepad_input.cpp": (
            "absent",
            "",
            "F5: gamepad input runtime is absent.",
        ),
        "src/inputs/keyboard_input.cpp": (
            "absent",
            "",
            "F5: keyboard input runtime is absent.",
        ),
        "src/inputs/semantic_input.cpp": (
            "absent",
            "",
            "F5/F6: semantic input runtime is absent.",
        ),
        "src/profiler/profiler.cpp": ("absent", "", "F12: profiler is absent."),
        "src/profiler/rive_profile.cpp": ("absent", "", "F12: profiler is absent."),
        "src/nested_artboard.cpp": (
            "partial",
            "crates/nuxie-runtime/src/artboard.rs",
            "F13: nested artboards are ported; latent hit-propagation ceilings remain.",
        ),
        "src/data_bind/context/context_value_artboard.cpp": (
            "partial",
            "crates/nuxie-runtime/src/view_model.rs",
            "F13: artboard context values are ported; live nested-host ceilings remain.",
        ),
        "src/text/text_modifier.cpp": (
            "partial",
            "crates/nuxie-runtime/src/text.rs",
            "F13: static text modifiers are ported with richer modifier ceilings.",
        ),
        "src/core/binary_data_reader.cpp": (
            "not-applicable",
            "",
            "F14: C++ binary data reader plumbing is outside the Rust importer shape.",
        ),
        "src/static_scene.cpp": (
            "not-applicable",
            "",
            "F14: static scene helper is outside the supported runtime contract.",
        ),
        "src/hittest_command_path.cpp": (
            "not-applicable",
            "",
            "F14: command-path helper is outside the supported runtime architecture.",
        ),
        "src/intrinsically_sizeable.cpp": (
            "not-applicable",
            "",
            "F14: intrinsic-size helper is represented by consolidated layout code.",
        ),
    }
)

for _path in {
    "src/text/raw_text_input.cpp",
    "src/text/text_input.cpp",
    "src/text/text_input_cursor.cpp",
    "src/text/text_input_drawable.cpp",
    "src/text/text_input_selected_text.cpp",
    "src/text/text_input_selection.cpp",
    "src/text/text_input_text.cpp",
    "src/text/text_selection_path.cpp",
}:
    FEATURE_ROWS[_path] = (
        "partial",
        "crates/nuxie-runtime/src/text.rs",
        "F2: TextInput rendering is ported; editing behavior remains partial.",
    )

for _path in {
    "src/semantic/semantic_data.cpp",
    "src/semantic/semantic_inference_registry.cpp",
    "src/semantic/semantic_provider.cpp",
}:
    FEATURE_ROWS[_path] = ("absent", "", "F6: semantics runtime is absent.")

for _path in {
    "src/lua/lua_audio.cpp",
    "src/lua/lua_buffer_ext.cpp",
    "src/lua/lua_data_context.cpp",
    "src/lua/lua_data_value.cpp",
    "src/lua/lua_image_decode.cpp",
    "src/lua/lua_scripted_context.cpp",
    "src/lua/lua_state.cpp",
    "src/lua/math/lua_color.cpp",
    "src/lua/math/lua_input.cpp",
    "src/lua/renderer/lua_blob.cpp",
    "src/lua/renderer/lua_gradient.cpp",
    "src/lua/renderer/lua_image.cpp",
    "src/lua/renderer/lua_mesh.cpp",
}:
    FEATURE_ROWS[_path] = ("absent", "", "F7: this Lua binding is absent.")
PREFIX_MODULES = (
    ("src/animation/", "crates/nuxie-runtime/src/animation.rs"),
    ("src/assets/", "crates/nuxie-runtime/src/objects.rs"),
    ("src/bones/", "crates/nuxie-runtime/src/components.rs"),
    ("src/constraints/", "crates/nuxie-runtime/src/constraints.rs"),
    ("src/core/", "crates/nuxie-binary/src/lib.rs"),
    ("src/data_bind/", "crates/nuxie-runtime/src/artboard_data_bind.rs"),
    ("src/importers/", "crates/nuxie-runtime/src/objects.rs"),
    ("src/input/", "crates/nuxie-runtime/src/focus.rs"),
    ("src/inputs/", "crates/nuxie-runtime/src/state_machine.rs"),
    ("src/layout/", "crates/nuxie-runtime/src/draw.rs"),
    ("src/lua/", "crates/nuxie-scripting/src/vm.rs"),
    ("src/math/", "crates/nuxie-runtime/src/components.rs"),
    ("src/scripted/", "crates/nuxie-runtime/src/scripting.rs"),
    ("src/shapes/", "crates/nuxie-runtime/src/draw.rs"),
    ("src/text/", "crates/nuxie-runtime/src/text.rs"),
    ("src/viewmodel/", "crates/nuxie-runtime/src/view_model.rs"),
)


def upstream_cpp_paths(rive_runtime_dir: pathlib.Path) -> list[str]:
    source_root = rive_runtime_dir / "src"
    paths = []
    for path in source_root.rglob("*.cpp"):
        relative = path.relative_to(rive_runtime_dir)
        if relative.parts[:2] == ("src", "generated"):
            continue
        paths.append(relative.as_posix())
    return sorted(paths)


def load_manifest(path: pathlib.Path) -> dict[str, object]:
    with path.open("rb") as source:
        return tomllib.load(source)


def feature_classification(upstream: str) -> dict[str, str] | None:
    feature_row = FEATURE_ROWS.get(upstream)
    if feature_row is None and upstream.startswith("src/audio/"):
        feature_row = ("absent", "", "F1: audio runtime is absent.")
    if feature_row is not None:
        status, rust_module, note = feature_row
        return {
            "upstream": upstream,
            "status": status,
            "rust_module": rust_module,
            "note": note,
        }
    return None


def classify(upstream: str) -> dict[str, str]:
    feature_row = feature_classification(upstream)
    if feature_row is not None:
        return feature_row
    if upstream == "src/component.cpp":
        return {
            "upstream": upstream,
            "status": "ported",
            "rust_module": "crates/nuxie-runtime/src/components.rs",
            "note": "Consolidated component runtime port.",
        }
    for prefix, rust_module in PREFIX_MODULES:
        if upstream.startswith(prefix):
            return {
                "upstream": upstream,
                "status": "ported",
                "rust_module": rust_module,
                "note": f"Consolidated Rust port for {prefix.removeprefix('src/').rstrip('/')}.",
            }
    if upstream.startswith("src/") and upstream.count("/") == 1:
        return {
            "upstream": upstream,
            "status": "ported",
            "rust_module": "crates/nuxie-runtime/src/lib.rs",
            "note": "Consolidated runtime port.",
        }
    raise ValueError(f"no classification rule for {upstream}")


def render_manifest(rows: list[dict[str, str]], upstream_ref: str) -> str:
    lines = [
        "# Generated by tools/port-manifest/port_manifest.py; edit classifications in the tool.",
        "version = 1",
        f"upstream_ref = {json.dumps(upstream_ref)}",
        'source_glob = "src/**/*.cpp"',
        'exclude_glob = "src/generated/**"',
        f"row_count = {len(rows)}",
    ]
    for row in rows:
        lines.extend(
            [
                "",
                "[[file]]",
                f"upstream = {json.dumps(row['upstream'])}",
                f"status = {json.dumps(row['status'])}",
                f"rust_module = {json.dumps(row['rust_module'])}",
                f"note = {json.dumps(row['note'])}",
            ]
        )
    return "\n".join(lines) + "\n"


def generate_manifest(
    rive_runtime_dir: pathlib.Path, upstream_ref: str, output: pathlib.Path
) -> None:
    rows = [classify(path) for path in upstream_cpp_paths(rive_runtime_dir)]
    output.write_text(render_manifest(rows, upstream_ref))


def check_manifest(
    rive_runtime_dir: pathlib.Path,
    repo_root: pathlib.Path,
    manifest_path: pathlib.Path,
    upstream_ref: str | None,
) -> None:
    upstream = set(upstream_cpp_paths(rive_runtime_dir))
    document = load_manifest(manifest_path)
    manifest_ref = document.get("upstream_ref")
    rows = document.get("file", [])
    path_counts = collections.Counter(row.get("upstream") for row in rows)
    duplicates = sorted(path for path, count in path_counts.items() if count > 1)
    if duplicates:
        raise ValueError(f"duplicate manifest rows: {', '.join(duplicates)}")
    declared = set(path_counts)
    missing = sorted(upstream - declared)
    if missing:
        raise ValueError(f"missing manifest rows: {', '.join(missing)}")
    stale = sorted(declared - upstream)
    if stale:
        raise ValueError(f"stale manifest rows: {', '.join(stale)}")
    if upstream_ref is not None:
        if not isinstance(manifest_ref, str):
            raise ValueError("manifest is missing upstream_ref")
        if manifest_ref != upstream_ref:
            raise ValueError(
                f"upstream ref mismatch: manifest {manifest_ref}, checkout {upstream_ref}"
            )
    for row in rows:
        for field in ("upstream", "status", "rust_module", "note"):
            if field not in row:
                raise ValueError(
                    f"manifest row missing field {field}: {row.get('upstream')}"
                )
        status = row.get("status")
        if status not in STATUSES:
            raise ValueError(f"invalid status for {row.get('upstream')}: {status}")
        upstream_path = row.get("upstream")
        expected = (
            feature_classification(upstream_path)
            if isinstance(upstream_path, str)
            else None
        )
        if expected is not None and (
            status != expected["status"]
            or row.get("rust_module") != expected["rust_module"]
        ):
            raise ValueError(
                f"register seed drift for {upstream_path}: "
                f"expected status={expected['status']} "
                f"rust_module={expected['rust_module']!r}"
            )
        note = row.get("note")
        if expected is not None:
            expected_feature_ids = sorted(set(re.findall(r"\bF\d+\b", expected["note"])))
            actual_feature_ids = (
                sorted(set(re.findall(r"\bF\d+\b", note)))
                if isinstance(note, str)
                else []
            )
            if actual_feature_ids != expected_feature_ids:
                raise ValueError(
                    f"register seed drift for {upstream_path}: expected feature ids "
                    f"{','.join(expected_feature_ids)}"
                )
        if status == "absent" and (
            not isinstance(note, str) or re.search(r"\bF\d+\b", note) is None
        ):
            raise ValueError(f"absent row must cite an F-row id: {row.get('upstream')}")
        rust_module = row.get("rust_module")
        if status in {"ported", "partial"} and not rust_module:
            raise ValueError(
                f"{status} row must declare a Rust module: {row.get('upstream')}"
            )
        if isinstance(rust_module, str) and rust_module and not (repo_root / rust_module).is_file():
            raise ValueError(
                f"missing Rust module for {row.get('upstream')}: {rust_module}"
            )
    status_counts = collections.Counter(row["status"] for row in rows)
    print(
        f"port-manifest: {len(rows)}/{len(upstream)} rows "
        f"(ported={status_counts['ported']}, partial={status_counts['partial']}, "
        f"absent={status_counts['absent']}, "
        f"not-applicable={status_counts['not-applicable']}); "
        "Rust module paths verified"
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    check = subparsers.add_parser("check", help="validate the checked-in manifest")
    check.add_argument("--rive-runtime-dir", required=True, type=pathlib.Path)
    check.add_argument("--repo-root", required=True, type=pathlib.Path)
    check.add_argument("--manifest", required=True, type=pathlib.Path)
    check.add_argument("--upstream-ref")
    generate = subparsers.add_parser("generate", help="generate the canonical manifest")
    generate.add_argument("--rive-runtime-dir", required=True, type=pathlib.Path)
    generate.add_argument("--upstream-ref", required=True)
    generate.add_argument("--output", required=True, type=pathlib.Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.command == "check":
            check_manifest(
                args.rive_runtime_dir,
                args.repo_root,
                args.manifest,
                args.upstream_ref,
            )
        elif args.command == "generate":
            generate_manifest(args.rive_runtime_dir, args.upstream_ref, args.output)
    except (OSError, tomllib.TOMLDecodeError, ValueError) as error:
        print(f"port-manifest {args.command} failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
