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
        "P2F2: core Artboard behavior includes retained volume/audio-engine configuration, recursive propagation to nested and component-list occurrences, and Artboard-scoped sound teardown. Other historical Artboard ceilings keep this legacy row partial.",
    ),
    "src/text/cursor.cpp": (
        "ported",
        "crates/nuxie-runtime/src/text/cursor.rs",
        "FL-E6: retained cursor editing and selection behavior is ported.",
    ),
    "src/command_queue.cpp": (
        "partial",
        "crates/nuxie/src/command_queue.rs",
        "P3F: typed queued transport, callbacks, resources, view models, draw coalescing, and lifecycle commands are present; semantics and unexecuted fixture-specific parity rows remain pending.",
    ),
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
        "partial",
        "crates/nuxie-runtime/src/semantic_manager.rs",
        "F6FID: the retained manager/tree is installed and direct upstream cases pass; #LT-1 fixture differential evidence remains pending.",
    ),
    "src/lua/lua_promise.cpp": (
        "ported",
        "crates/nuxie-scripting/src/vm/promise.rs; crates/nuxie-scripting/src/vm.rs",
        "P1-i: all 47 pinned Promise scenarios plus 2 invalid-yield cases pass exact live C++/Rust VM differentials; image decode is a separate lane.",
    ),
    "src/lua/renderer/lua_gpu.cpp": (
        "partial",
        "crates/nuxie-scripting/src/gpu_canvas.rs; crates/nuxie-render-api/src/lib.rs; crates/nuxie-renderer/src/gpu_canvas.rs",
        "P3E: GPU-prefixed Lua surface is implemented through the approved wgpu adaptation; mixed-file Canvas 2D/Image:view residue remains F7/F8.",
    ),
    "src/joystick.cpp": (
        "ported",
        "crates/nuxie-runtime/src/joystick.rs",
        "FL-E4: direct occurrence-owned joystick runtime and parity fixtures.",
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
        "ported",
        "crates/nuxie-binary/src/binary_writer.rs",
        "P1-p/F14: direct exact-wire BinaryWriter owner with pinned C++ differential coverage.",
    ),
    "src/audio/audio_engine.cpp": (
        "ported",
        "crates/nuxie-audio/src/engine.rs; crates/nuxie-audio/src/device.rs",
        "P2F1/P2F3: exact headless frame clock, scheduling/clipping, manual PCM pull/sum, levels, lifecycle, and artboard stop; optional CPAL output drains that same authoritative mixer through the default device.",
    ),
    "src/audio/audio_reader.cpp": (
        "ported",
        "crates/nuxie-audio/src/source.rs",
        "P2F1/D17: Symphonia-backed independent readers are ported under the approved decoder/resampler tolerance boundary.",
    ),
    "src/audio/audio_sound.cpp": (
        "ported",
        "crates/nuxie-audio/src/engine.rs",
        "P2F1: retained sound control, volume, completion, and disposal behavior is ported.",
    ),
    "src/audio/audio_source.cpp": (
        "ported",
        "crates/nuxie-audio/src/source.rs",
        "P2F1/D17: owned WAV/MP3/FLAC and buffered sources are ported under the approved decoder/resampler tolerance boundary.",
    ),
    "src/math/hit_test.cpp": (
        "ported",
        "crates/nuxie-runtime/src/math/hit_test.rs",
        "P1-p: direct integer-cell HitTester owner consumed by HitTestCommandPath.",
    ),
}
FEATURE_ROWS.update(
    {
        "src/assets/audio_asset.cpp": (
            "ported",
            "crates/nuxie-runtime/src/assets/audio_asset.rs; crates/nuxie-runtime/src/assets/file_asset_loader.rs",
            "P2F1/P2F2: embedded and host-loaded AudioAsset bytes resolve to file-owned AudioSource values that feed dense-ordinal AudioEvent playback.",
        ),
        "src/audio_event.cpp": (
            "ported",
            "crates/nuxie-runtime/src/audio_event.rs; crates/nuxie-runtime/src/artboard.rs; crates/nuxie-runtime/src/state_machine/state_machine_instance.rs; crates/nuxie/src/lib.rs",
            "P2F2: dense-ordinal AudioEvent resolution, multiplied asset/Artboard volume, configured/default-engine event-unwind playback, and Artboard-scoped teardown are ported.",
        ),
        "src/command_server.cpp": (
            "partial",
            "crates/nuxie/src/command_server.rs",
            "P3F: the server-thread owner, handle maps, command loop, dependency cleanup, callbacks, draw scheduling, and resource/list/view-model structure are present; full 79-case promotion awaits remaining evidence and F6.",
        ),
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
            "ported",
            "crates/nuxie-runtime/src/state_machine/text_input_listener_group.rs",
            "FL-E6: direct TextInput pointer, drag, multi-click, and focus bridge.",
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
            "ported",
            "crates/nuxie-runtime/src/state_machine/gamepad_batch.rs",
            "P2-e/F5: v2 little-endian decoding, validation, per-device snapshots, focused/scripted dispatch, and public runtime/facade submission are ported.",
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
        "src/profiler/profiler.cpp": (
            "ported",
            "crates/nuxie-runtime/src/profiler.rs",
            "P1-m/F12 approved D16/PORTING-FLR-21 adaptation: the pinned MicroProfile implementation wrapper is replaced by the pluggable pure-Rust ProfileCapture seam; no C++ FFI is linked.",
        ),
        "src/profiler/rive_profile.cpp": (
            "ported",
            "crates/nuxie-runtime/src/profiler.rs; crates/nuxie-runtime/src/artboard.rs; crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs; crates/nuxie-runtime/src/state_machine/state_machine_instance.rs",
            "P1-m/F12 faithful RiveProfile records, lifecycle, nested-host paths, transition/listener hooks, and exact BinaryWriter wire bytes checked by a source-derived oracle compiled against the pinned C++ types and VectorBinaryWriter.",
        ),
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
            "ported",
            "crates/nuxie-binary/src/binary_data_reader.rs",
            "P1-p/F14: direct BinaryDataReader owner with contract and pinned C++ differential coverage.",
        ),
        "src/static_scene.cpp": (
            "ported",
            "crates/nuxie-runtime/src/static_scene.rs",
            "P1-p/F14: direct StaticScene owner with pinned C++ API-contract coverage.",
        ),
        "src/hittest_command_path.cpp": (
            "ported",
            "crates/nuxie-runtime/src/hittest_command_path.rs",
            "P1-p/F14: direct HitTestCommandPath owner with upstream fixture differentials.",
        ),
        "src/intrinsically_sizeable.cpp": (
            "ported",
            "crates/nuxie-runtime/src/intrinsically_sizeable.rs",
            "FL-E4: direct intrinsic-size dispatch and layout integration.",
        ),
    }
)

for _path, _module in {
    "src/text/raw_text_input.cpp": "crates/nuxie-runtime/src/text/raw_text_input.rs",
    "src/text/text_input.cpp": "crates/nuxie-runtime/src/text_input.rs",
    "src/text/text_input_cursor.cpp": "crates/nuxie-runtime/src/text/text_input_cursor.rs",
    "src/text/text_input_drawable.cpp": "crates/nuxie-runtime/src/text/text_input_drawable.rs",
    "src/text/text_input_selected_text.cpp": "crates/nuxie-runtime/src/text/text_input_selected_text.rs",
    "src/text/text_input_selection.cpp": "crates/nuxie-runtime/src/text/text_input_selection.rs",
    "src/text/text_input_text.cpp": "crates/nuxie-runtime/src/text/text_input_text.rs",
    "src/text/text_interface.cpp": "crates/nuxie-runtime/src/text/text_interface.rs",
    "src/text/text_selection_path.cpp": "crates/nuxie-runtime/src/text/text_selection_path.rs",
}.items():
    FEATURE_ROWS[_path] = (
        "ported",
        _module,
        "FL-E6: direct TextInput owner family and W65 behavior are ported.",
    )

FEATURE_ROWS["src/semantic/semantic_data.cpp"] = (
    "partial",
    "crates/nuxie-runtime/src/semantic_data.rs",
    "F6FID: retained semantics are integrated and focused upstream cases pass; #LT-1 fixture differential evidence remains pending.",
)
FEATURE_ROWS["src/semantic/semantic_inference_registry.cpp"] = (
    "absent",
    "",
    "F6FID: no green retained-tree upstream fixture evidence exists yet; promotion requires the #LT-1 full-diff Text-inference oracle case.",
)
FEATURE_ROWS["src/semantic/semantic_provider.cpp"] = (
    "partial",
    "crates/nuxie-runtime/src/semantic_provider.rs",
    "F6FID: mounted root/scroll bounds pass the four focus cases; #LT-1 provider differentials remain pending.",
)

for _path in {
    "src/lua/lua_buffer_ext.cpp",
    "src/lua/lua_scripted_context.cpp",
}:
    FEATURE_ROWS[_path] = ("absent", "", "F7: this Lua binding is absent.")

FEATURE_ROWS.update(
    {
        "src/lua/lua_data_context.cpp": (
            "partial",
            "crates/nuxie-scripting/src/vm/view_model.rs",
            "F7/P1G: DataContext methods are present; parent contexts without a main view model remain unrepresentable.",
        ),
        "src/lua/lua_data_value.cpp": (
            "partial",
            "crates/nuxie-scripting/src/vm.rs",
            "F7/P1G: DataValue surface is present with tracked index/newindex, coercion, and color-channel gaps.",
        ),
        "src/lua/lua_audio.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/lua_audio.rs; crates/nuxie-scripting/src/vm/view_model.rs; crates/nuxie/src/lib.rs",
            "P2F3: Context:audio, the Audio static playback/time API, AudioSource duration, and AudioSound control/query/volume bindings are direct pure-Rust Luau ports.",
        ),
        "src/lua/lua_image_decode.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/lua_image_decode.rs; crates/nuxie-image-codec/src/lib.rs; crates/nuxie-scripting/src/vm/promise.rs; crates/nuxie-runtime/src/scene.rs",
            "P2A: WorkPool-scheduled decode, root-frame VM-thread Promise settlement, cancellation, and unbounded premultiplied RGBA result/error behavior are ported.",
        ),
        "src/lua/renderer/lua_blob.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/lua_blob.rs",
            "P2B faithful candidate: Blob userdata exposes pinned name/size/fresh-copy data fields, and exact-name Context lookup preserves file order while skipping empty BlobAssets; the previously missing positive path has a live pinned-C++ differential.",
        ),
        "src/lua/renderer/lua_image.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/lua_image.rs; crates/nuxie-scripting/src/vm/view_model.rs; crates/nuxie-runtime/src/data_bind/data_bind_context.rs; crates/nuxie-runtime/src/draw.rs; crates/nuxie-runtime/src/shapes/image.rs",
            "P2A/P2B: decoded Image width/height, pre-decode nil behavior, runtime-image assignment through bound draw targets, and ImageSampler are ported; the optional ORE-only view member is omitted in this non-ORE build.",
        ),
        "src/lua/renderer/lua_mesh.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/lua_mesh.rs",
            "P2B faithful candidate under the backend-neutral render-factory adaptation: callable vertex/triangle userdata, add/reset invalidation, mapped-once native buffer upload, u16 index bounds, and renderer drawImageMesh wiring are present; both upstream cases are direct ports.",
        ),
        "src/lua/lua_state.cpp": (
            "partial",
            "crates/nuxie-scripting/src/vm/view_model.rs",
            "F7/P1G: Data initialization is ported with tracked constructor-arity gaps.",
        ),
        "src/lua/math/lua_color.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/lua_color.rs",
            "F7/P1G: the complete Color binding is ported.",
        ),
        "src/lua/renderer/lua_gradient.cpp": (
            "partial",
            "crates/nuxie-scripting/src/vm/renderer.rs",
            "F7/P1G: Gradient constructors are present with tracked non-table stop and unsigned-color conversion gaps.",
        ),
        "src/lua/logging_scripting_context.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/logging_scripting_context.rs; crates/nuxie/src/lib.rs",
            "P1G: direct host logging context and File host route are ported.",
        ),
        "src/lua/lua_rive_base.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/lua_rive_base.rs",
            "P1G: direct host-routed _G.print binding is ported.",
        ),
        "src/lua/lua_listener_invocation.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/listener_invocation.rs",
            "P1J: complete owned Invocation classification/conversion and pointer, keyboard, text, focus, reported-event, view-model, none, and gamepad payload surface.",
        ),
        "src/lua/math/lua_input.cpp": (
            "ported",
            "crates/nuxie-scripting/src/vm/listener_invocation.rs",
            "F7/P1J: PointerEvent fields, constructor, event labels, and end-to-end VM-to-state-machine tri-state hit propagation are ported.",
        ),
    }
)
PREFIX_MODULES = (
    ("src/animation/", "crates/nuxie-runtime/src/animation.rs"),
    ("src/assets/", "crates/nuxie-runtime/src/objects.rs"),
    ("src/bones/", "crates/nuxie-runtime/src/components.rs"),
    ("src/constraints/", "crates/nuxie-runtime/src/constraints.rs"),
    ("src/core/", "crates/nuxie-binary/src/lib.rs"),
    ("src/data_bind/", "crates/nuxie-runtime/src/data_bind/data_bind_context.rs"),
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
    if upstream in {"src/factory.cpp", "src/renderer.cpp"}:
        return {
            "upstream": upstream,
            "status": "ported",
            "rust_module": "crates/nuxie-render-api/src/lib.rs",
            "note": "Backend-neutral render seam owner.",
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
        modules = (
            [part.strip() for part in rust_module.split(";") if part.strip()]
            if isinstance(rust_module, str)
            else []
        )
        if isinstance(rust_module, str) and rust_module and not modules:
            raise ValueError(
                f"invalid rust_module for {row.get('upstream')}: {rust_module!r}"
            )
        if status in {"ported", "partial"} and not modules:
            raise ValueError(
                f"{status} row must declare a Rust module: {row.get('upstream')}"
            )
        missing_modules = [
            module for module in modules if not (repo_root / module).is_file()
        ]
        if missing_modules:
            raise ValueError(
                f"missing Rust module for {row.get('upstream')}: "
                f"{'; '.join(missing_modules)}"
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
