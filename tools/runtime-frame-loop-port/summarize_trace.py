#!/usr/bin/env python3
"""Summarize deterministic LLVM frame-loop coverage and golden-stream work."""

from __future__ import annotations

import argparse
import collections
import fnmatch
import hashlib
import json
import pathlib
import subprocess
import tomllib
from typing import Any


LANDMARKS = {
    "state_machine_advance": {
        "cpp": "rive::StateMachineInstance::advance(float, bool)",
        "rust": (
            "<nuxie_runtime::state_machine::instance::StateMachineInstance>"
            "::advance_with_report_mode"
        ),
    },
    "state_machine_layer_advance": {
        "cpp": "rive::StateMachineLayerInstance::advance(float, bool)",
        "rust": (
            "<nuxie_runtime::state_machine::StateMachineLayerInstance>::advance"
        ),
    },
    "linear_animation_advance": {
        "cpp": (
            "rive::LinearAnimationInstance::advance("
            "float, rive::KeyedCallbackReporter*)"
        ),
        "rust": "<nuxie_runtime::animation::LinearAnimationInstance>::advance",
    },
    "artboard_update_pass": {
        "cpp": "rive::Artboard::updatePass(bool)",
        "rust": "<nuxie_runtime::artboard::ArtboardInstance>::update_pass",
    },
    "artboard_update_components": {
        "cpp": "rive::Artboard::updateComponents()",
        "rust": (
            "<nuxie_runtime::artboard::ArtboardInstance>"
            "::update_components_with_hook_recording::"
            "<<nuxie_runtime::artboard::ArtboardInstance>"
            "::update_pass::{closure#0}>"
        ),
    },
    "artboard_draw": {
        "cpp": "rive::Artboard::draw(rive::Renderer*)",
        "rust": "<nuxie_runtime::artboard::ArtboardInstance>::draw_artboard",
    },
    "artboard_draw_internal": {
        "cpp": "rive::Artboard::drawInternal(rive::Renderer*)",
        "rust": (
            "<nuxie_runtime::artboard::ArtboardInstance>"
            "::draw_artboard_internal_internal_with_path_cache"
        ),
    },
    "component_add_dirt": {
        "cpp": "rive::Component::addDirt(rive::ComponentDirt, bool)",
        "rust": "<nuxie_runtime::artboard::ArtboardInstance>::add_dirt",
    },
    "keyframe_double_apply_steps": {
        "cpp": [
            (
                "rive::KeyFrameDouble::apply("
                "rive::Core*, int, float, rive::LinearAnimationInstance const*)"
            ),
            (
                "rive::KeyFrameDouble::applyInterpolation("
                "rive::Core*, int, float, rive::KeyFrame const*, float, "
                "rive::LinearAnimationInstance const*)"
            ),
        ],
        "rust": [
            (
                "nuxie_runtime::animation::apply_key_frame_double_mix::"
                "<<nuxie_runtime::animation::RuntimeLinearAnimation>"
                "::apply_with_key_frame_values::{closure#0}>"
            ),
            (
                "nuxie_runtime::animation::apply_key_frame_double_mix::"
                "<<nuxie_runtime::animation::RuntimeLinearAnimation>"
                "::apply_with_key_frame_values::{closure#1}>"
            ),
        ],
    },
    "event_apply_batch": {
        "cpp": "rive::StateMachineInstance::applyEvents()",
        "rust": (
            "<nuxie_runtime::state_machine::instance::StateMachineInstance>"
            "::apply_local_event_listeners"
        ),
    },
    "databind_artboard_batch": {
        "cpp": "rive::Artboard::updateDataBinds(bool)",
        "rust": (
            "<nuxie_runtime::artboard::ArtboardInstance>"
            "::advance_artboard_data_binds_with_elapsed"
        ),
    },
    "state_machine_transition_search": {
        "cpp": (
            "rive::StateMachineLayerInstance::findAllowedTransition("
            "rive::StateInstance*)"
        ),
        "rust": (
            "<nuxie_runtime::state_machine::StateMachineLayerInstance>"
            "::try_change_state"
        ),
    },
    "draw_order_sort": {
        "cpp": "rive::Artboard::sortDrawOrder()",
        "rust": "<nuxie_runtime::draw::RuntimeDrawableList>::sort_draw_order",
    },
    "clipping_redundancy_clear": {
        "cpp": "rive::Artboard::clearRedundantOperations()",
        "rust": (
            "<nuxie_runtime::draw::RuntimeDrawableList>"
            "::clear_redundant_operations"
        ),
    },
    "drawable_owner_lookup": {
        "cpp": 0,
        "rust": (
            "<nuxie_runtime::artboard::ArtboardInstance>"
            "::runtime_drawable_component"
        ),
    },
    "layout_compute": {
        "cpp": "rive::Artboard::calculateLayout()",
        "rust": (
            "<nuxie_runtime::draw::TaffyRuntimeLayoutEngine>"
            "::compute_layout"
        ),
    },
}

CONSTRUCTION_LANDMARKS = {
    "artboard_instance": {
        "cpp": (
            "std::__1::unique_ptr<rive::ArtboardInstance, "
            "std::__1::default_delete<rive::ArtboardInstance>> "
            "rive::Artboard::instance<rive::ArtboardInstance>() const"
        ),
        "rust": (
            "<nuxie_runtime::artboard::ArtboardInstance>::from_graph_inner"
        ),
    },
    "state_machine_instance": {
        "cpp": (
            "rive::StateMachineInstance::StateMachineInstance("
            "rive::StateMachine const*, rive::ArtboardInstance*)"
        ),
        "rust": (
            "<nuxie_runtime::state_machine::instance::StateMachineInstance>::new"
        ),
    },
    "linear_animation_instance": {
        "cpp": (
            "rive::LinearAnimationInstance::LinearAnimationInstance("
            "rive::LinearAnimation const*, rive::ArtboardInstance*, float)"
        ),
        "rust": "<nuxie_runtime::animation::LinearAnimationInstance>::new",
    },
}


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def demangle(names: list[str], executable: pathlib.Path) -> list[str]:
    result = subprocess.run(
        [str(executable)],
        input="\n".join(names),
        text=True,
        capture_output=True,
        check=True,
    )
    values = result.stdout.splitlines()
    if len(values) != len(names):
        raise ValueError(
            f"demangler returned {len(values)} names for {len(names)} inputs"
        )
    return values


def source_scope(
    ledger: dict[str, Any], upstream: pathlib.Path
) -> tuple[set[str], dict[str, str]]:
    all_files = sorted(
        path.relative_to(upstream).as_posix()
        for path in (upstream / "src").rglob("*.cpp")
        if "/generated/" not in path.as_posix()
    )
    scope: set[str] = set()
    source_set: dict[str, str] = {}
    for row in ledger.get("source_set", []):
        includes = [str(value) for value in row.get("include", [])]
        excludes = [str(value) for value in row.get("exclude", [])]
        for path in all_files:
            if any(fnmatch.fnmatchcase(path, value) for value in includes) and not any(
                fnmatch.fnmatchcase(path, value) for value in excludes
            ):
                scope.add(path)
                source_set[path] = str(row["id"])
    return scope, source_set


def coverage_functions(
    *,
    path: pathlib.Path,
    side: str,
    upstream: pathlib.Path,
    scope: set[str],
    demangler: pathlib.Path,
    scope_only: bool = True,
) -> dict[str, list[dict[str, int | str]]]:
    document = json.loads(path.read_text(encoding="utf-8"))
    functions = [
        row for row in document["data"][0]["functions"] if int(row["count"]) > 0
    ]
    names = demangle([str(row["name"]) for row in functions], demangler)
    result: dict[str, list[dict[str, int | str]]] = {}
    marker = "/crates/nuxie-runtime/src/"
    for row, name in zip(functions, names):
        absolute = str(row["filenames"][0])
        if side == "cpp":
            try:
                relative = pathlib.Path(absolute).relative_to(upstream).as_posix()
            except ValueError:
                continue
            if scope_only and relative not in scope:
                continue
        else:
            if marker not in absolute:
                continue
            relative = "crates/nuxie-runtime/src/" + absolute.split(marker, 1)[1]
        result.setdefault(relative, []).append(
            {"name": name, "count": int(row["count"])}
        )
    for rows in result.values():
        rows.sort(key=lambda row: str(row["name"]))
    return dict(sorted(result.items()))


def stream_counts(directory: pathlib.Path, side: str) -> dict[str, int]:
    counts: collections.Counter[str] = collections.Counter()
    ignored = ("rive-", "source ", "frameSize ", "sample ")
    for path in sorted(directory.glob(f"{side}-*.txt")):
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line or line.startswith(ignored):
                continue
            counts[line.split(" ", 1)[0]] += 1
    return dict(sorted(counts.items()))


def exact_function_count(
    functions: dict[str, list[dict[str, int | str]]],
    names: int | str | list[str],
) -> int:
    if isinstance(names, int):
        return names
    expected = [names] if isinstance(names, str) else names
    total = 0
    for name in expected:
        counts = [
            int(row["count"])
            for rows in functions.values()
            for row in rows
            if row["name"] == name
        ]
        if len(counts) != 1:
            raise ValueError(f"landmark {name!r} matched {len(counts)} functions")
        total += counts[0]
    return total


def summarize(args: argparse.Namespace) -> dict[str, Any]:
    ledger = tomllib.loads(args.ledger.read_text(encoding="utf-8"))
    scope, source_sets = source_scope(ledger, args.upstream)
    cpp = coverage_functions(
        path=args.cpp_coverage,
        side="cpp",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
    )
    rust = coverage_functions(
        path=args.rust_coverage,
        side="rust",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
    )
    cpp_full = coverage_functions(
        path=args.cpp_full_coverage,
        side="cpp",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
        scope_only=False,
    )
    rust_full = coverage_functions(
        path=args.rust_full_coverage,
        side="rust",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
        scope_only=False,
    )
    reached_by_source_set = collections.Counter(
        source_sets[path] for path in cpp
    )
    landmarks = {
        name: {
            "cpp": exact_function_count(cpp, patterns["cpp"]),
            "rust": exact_function_count(rust, patterns["rust"]),
        }
        for name, patterns in LANDMARKS.items()
    }
    construction_landmarks = {
        name: {
            "cpp": exact_function_count(cpp_full, patterns["cpp"]),
            "rust": exact_function_count(rust_full, patterns["rust"]),
        }
        for name, patterns in CONSTRUCTION_LANDMARKS.items()
    }
    allocation_counts = json.loads(
        args.allocation_counts.read_text(encoding="utf-8")
    )
    landmarks["per_frame_allocations"] = {
        side: sum(int(value) for value in allocation_counts[side].values())
        for side in ("cpp", "rust")
    }
    return {
        "schema": "nuxie-runtime-frame-loop-trace/v1",
        "upstream_ref": ledger["upstream_ref"],
        "rust_ref": args.rust_ref,
        "mode": "frame-only counters reset after construction and before samples",
        "corpus": args.corpus_id,
        "artifacts": {
            "cpp_coverage_sha256": sha256(args.cpp_coverage),
            "rust_coverage_sha256": sha256(args.rust_coverage),
            "cpp_binary_sha256": sha256(args.cpp_binary),
            "rust_binary_sha256": sha256(args.rust_binary),
        },
        "scope": {
            "static_cpp_files": len(scope),
            "dynamic_cpp_files": len(cpp),
            "dynamic_cpp_functions": sum(len(rows) for rows in cpp.values()),
            "dynamic_rust_files": len(rust),
            "dynamic_rust_functions": sum(len(rows) for rows in rust.values()),
            "dynamic_cpp_files_by_source_set": dict(
                sorted(reached_by_source_set.items())
            ),
        },
        "landmarks": landmarks,
        "construction_landmarks": construction_landmarks,
        "golden_stream_operations": {
            "cpp": stream_counts(args.stream_directory, "cpp"),
            "rust": stream_counts(args.stream_directory, "rust"),
        },
        "allocation_counts": allocation_counts,
        "functions": {"cpp": cpp, "rust": rust},
    }


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    result.add_argument("--ledger", type=pathlib.Path, required=True)
    result.add_argument("--upstream", type=pathlib.Path, required=True)
    result.add_argument("--cpp-coverage", type=pathlib.Path, required=True)
    result.add_argument("--rust-coverage", type=pathlib.Path, required=True)
    result.add_argument("--cpp-full-coverage", type=pathlib.Path, required=True)
    result.add_argument("--rust-full-coverage", type=pathlib.Path, required=True)
    result.add_argument("--cpp-binary", type=pathlib.Path, required=True)
    result.add_argument("--rust-binary", type=pathlib.Path, required=True)
    result.add_argument("--stream-directory", type=pathlib.Path, required=True)
    result.add_argument("--allocation-counts", type=pathlib.Path, required=True)
    result.add_argument("--demangler", type=pathlib.Path, required=True)
    result.add_argument("--rust-ref", required=True)
    result.add_argument("--corpus-id", action="append", default=[])
    result.add_argument("--output", type=pathlib.Path, required=True)
    return result


def main() -> int:
    args = parser().parse_args()
    document = summarize(args)
    args.output.write_text(
        json.dumps(document, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(
        "runtime-frame-loop-trace: "
        f"cpp={document['scope']['dynamic_cpp_files']}/"
        f"{document['scope']['static_cpp_files']} files; "
        f"rust={document['scope']['dynamic_rust_files']} modules; "
        f"landmarks={len(document['landmarks'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
