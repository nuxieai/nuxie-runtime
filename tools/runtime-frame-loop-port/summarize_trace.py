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
        "rust": (
            "<nuxie_runtime::artboard::ArtboardInstance>"
            "::update_pass_with_script_mode"
        ),
    },
    "artboard_update_components": {
        "cpp": "rive::Artboard::updateComponents()",
        "rust": (
            "<nuxie_runtime::artboard::ArtboardInstance>"
            "::update_components_with_hook_recording::"
            "<<nuxie_runtime::artboard::ArtboardInstance>"
            "::update_pass_with_script_mode::{closure#0}>"
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
        "cpp": {
            "source": "src/component.cpp",
            "anchor": "m_Dirt |= value;",
        },
        "rust": {
            "sum": [
                {
                    "source": "crates/nuxie-runtime/src/artboard.rs",
                    "anchor": "component.dirt |= dirt;",
                },
                {
                    "source": "crates/nuxie-runtime/src/artboard.rs",
                    "anchor": "component.dirt |= ComponentDirt::COMPONENTS;",
                },
            ],
        },
    },
    "component_dirt_consumptions": {
        "cpp": {
            "source": "src/artboard.cpp",
            "anchor": "component->m_Dirt = ComponentDirt::None;",
        },
        "rust": {
            "source": "crates/nuxie-runtime/src/artboard.rs",
            "anchor": "scheduled_component.dirt = ComponentDirt::NONE;",
        },
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
    "internal_owner_rediscovery": {
        "cpp": 0,
        "rust": 0,
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
    "component_owner_resolutions": {
        "cpp": "rive::Component::onAddedDirty(rive::CoreContext*)",
        "rust": {
            "sum": [
                {
                    "source": "crates/nuxie-runtime/src/artboard.rs",
                    "anchor": (
                        '.context("authored Component handle is missing")?;'
                    ),
                    "occurrence": 1,
                },
                {
                    "source": "crates/nuxie-runtime/src/objects.rs",
                    "anchor": "occurrence.path_composer_handle = Some(handle);",
                },
                {
                    "source": "crates/nuxie-runtime/src/objects.rs",
                    "anchor": (
                        "occurrence.text_variation_helper_handle = Some(handle);"
                    ),
                },
            ],
        },
    },
    "dependency_builds": {
        "cpp": {
            "source": "src/artboard.cpp",
            "anchor": "object->as<Component>()->buildDependencies();",
        },
        "rust": {
            "source": "crates/nuxie-runtime/src/artboard.rs",
            "anchor": (
                '.context("authored Component handle is missing during '
                'dependency construction")?;'
            ),
        },
    },
    "dependency_sorts": {
        # DependencySorter::sort is also used for DrawTarget ordering. Count
        # the Artboard dependency-owner call specifically so the landmark
        # compares one retained component schedule per occurrence.
        "cpp": {
            "source": "src/artboard.cpp",
            "anchor": "sorter.sort(this, m_DependencyOrder);",
        },
        "rust": (
            "<nuxie_runtime::objects::InstanceObjectArena>"
            "::sort_dependencies_from_root"
        ),
    },
}


MECHANISM_LANDMARKS = {
    "component_add_dirt": LANDMARKS["component_add_dirt"],
    "component_dirt_consumptions": LANDMARKS["component_dirt_consumptions"],
    "constraint_applications": {
        "cpp": [
            "rive::DistanceConstraint::constrain(rive::TransformComponent*)",
            "rive::FollowPathConstraint::constrain(rive::TransformComponent*)",
            "rive::IKConstraint::constrain(rive::TransformComponent*)",
            "rive::RotationConstraint::constrain(rive::TransformComponent*)",
            "rive::ScaleConstraint::constrain(rive::TransformComponent*)",
            "rive::ScrollConstraint::constrain(rive::TransformComponent*)",
            "rive::ScrollBarConstraint::constrain(rive::TransformComponent*)",
            "rive::TransformConstraint::constrain(rive::TransformComponent*)",
            "rive::TranslationConstraint::constrain(rive::TransformComponent*)",
        ],
        "rust": "nuxie_runtime::constraints::apply_constraint",
    },
    "follow_path_measure_rebuilds": {
        "cpp": {
            "source": "src/constraints/follow_path_constraint.cpp",
            "anchor": "m_pathMeasure = PathMeasure(&m_rawPath);",
        },
        "rust": {
            "source": "crates/nuxie-runtime/src/constraints.rs",
            "anchor": (
                "retained.path_measure = "
                "RuntimePathMeasure::from_raw_path(&retained.raw_path);"
            ),
        },
    },
    "scroll_physics_advances": {
        "cpp": [
            "rive::ClampedScrollPhysics::advance(float)",
            "rive::ElasticScrollPhysics::advance(float)",
        ],
        "rust": (
            "<nuxie_runtime::components::RuntimeScrollPhysicsState>::advance"
        ),
    },
    "scroll_child_applies": {
        "cpp": (
            "rive::ScrollConstraint::constrainChild("
            "rive::LayoutNodeProvider*)"
        ),
        "rust": "nuxie_runtime::constraints::apply_scroll_constraint_child",
    },
    "scroll_virtualizer_settlements": {
        "cpp": (
            "rive::ScrollVirtualizer::constrain("
            "rive::ScrollConstraint*, "
            "std::__1::vector<rive::LayoutNodeProvider*, "
            "std::__1::allocator<rive::LayoutNodeProvider*>>&, float, "
            "rive::VirtualizedDirection)"
        ),
        "rust": {
            "source": "crates/nuxie-runtime/src/constraints.rs",
            "anchor": "let computed_layout_bounds = artboard",
            "occurrence": 1,
        },
    },
    "skin_buffer_rebuilds": {
        "cpp": "rive::Skin::update(rive::ComponentDirt)",
        "rust": {
            "source": "crates/nuxie-runtime/src/artboard.rs",
            "anchor": "let tendon_count = self",
        },
    },
    "advancing_dispatches": {
        "cpp": {
            "source": "src/artboard.cpp",
            "anchor": "if (adv->advanceComponent(elapsedSeconds, flags))",
        },
        "rust": {
            "source": "crates/nuxie-runtime/src/artboard.rs",
            "anchor": "let entry = self.advancing_components[index];",
            "occurrence": 2,
        },
    },
    "resetting_dispatches": {
        "cpp": {
            "source": "src/artboard.cpp",
            "anchor": "obj->reset();",
        },
        "rust": {
            "source": "crates/nuxie-runtime/src/artboard.rs",
            "anchor": "let entry = self.resetting_components[index];",
        },
    },
    "internal_owner_rediscovery": LANDMARKS["internal_owner_rediscovery"],
}


MECHANISM_CONSTRUCTION_LANDMARKS = {
    "component_owner_resolutions": CONSTRUCTION_LANDMARKS[
        "component_owner_resolutions"
    ],
    "dependency_builds": CONSTRUCTION_LANDMARKS["dependency_builds"],
    "dependency_sorts": CONSTRUCTION_LANDMARKS["dependency_sorts"],
    "ik_chain_builds": {
        "cpp": "rive::IKConstraint::onAddedClean(rive::CoreContext*)",
        "rust": {
            "source": "crates/nuxie-runtime/src/artboard.rs",
            "anchor": "retained.chain = chain;",
        },
    },
}


STEADY_LANDMARKS = {
    "component_dirt_consumptions": LANDMARKS["component_dirt_consumptions"],
    "constraint_applications": MECHANISM_LANDMARKS["constraint_applications"],
    "follow_path_measure_rebuilds": MECHANISM_LANDMARKS[
        "follow_path_measure_rebuilds"
    ],
    "skin_buffer_rebuilds": MECHANISM_LANDMARKS["skin_buffer_rebuilds"],
    "draw_order_sort": LANDMARKS["draw_order_sort"],
    "clipping_redundancy_clear": LANDMARKS["clipping_redundancy_clear"],
    "layout_compute": LANDMARKS["layout_compute"],
    "internal_owner_rediscovery": LANDMARKS["internal_owner_rediscovery"],
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
    include_zero: bool = False,
) -> dict[str, list[dict[str, int | str]]]:
    document = json.loads(path.read_text(encoding="utf-8"))
    functions = [
        row
        for row in document["data"][0]["functions"]
        if include_zero or int(row["count"]) > 0
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


def exact_source_line_count(
    coverage: dict[str, Any],
    *,
    source_root: pathlib.Path,
    source: str,
    anchor: str,
    occurrence: int = 1,
) -> int:
    source_path = source_root / source
    lines = source_path.read_text(encoding="utf-8").splitlines()
    matching_lines = [
        index
        for index, line in enumerate(lines, start=1)
        if anchor in line
    ]
    if occurrence < 1 or occurrence > len(matching_lines):
        raise ValueError(
            f"source landmark {source}:{anchor!r} occurrence {occurrence} "
            f"matched {len(matching_lines)} lines"
        )
    target_line = matching_lines[occurrence - 1]
    expected_path = source_path.resolve()
    matching_files = [
        row
        for row in coverage["data"][0].get("files", [])
        if pathlib.Path(str(row["filename"])).resolve() == expected_path
    ]
    if len(matching_files) != 1:
        raise ValueError(
            f"source landmark {source}:{target_line} matched "
            f"{len(matching_files)} coverage files"
        )
    segments = [
        segment
        for segment in matching_files[0].get("segments", [])
        if int(segment[0]) == target_line and bool(segment[3])
    ]
    if segments:
        return max(int(segment[2]) for segment in segments)
    preceding = [
        segment
        for segment in matching_files[0].get("segments", [])
        if (int(segment[0]), int(segment[1])) <= (target_line, 1)
    ]
    if not preceding or not bool(preceding[-1][3]):
        raise ValueError(
            f"source landmark {source}:{target_line} has no counted segment"
        )
    return int(preceding[-1][2])


def landmark_count(
    *,
    functions: dict[str, list[dict[str, int | str]]],
    coverage: dict[str, Any],
    pattern: int | str | list[str] | dict[str, Any],
    source_root: pathlib.Path,
) -> int:
    if isinstance(pattern, dict):
        if "sum" in pattern:
            summands = pattern["sum"]
            if not isinstance(summands, list) or not summands:
                raise ValueError("summed landmark requires a non-empty list")
            return sum(
                landmark_count(
                    functions=functions,
                    coverage=coverage,
                    pattern=summand,
                    source_root=source_root,
                )
                for summand in summands
            )
        return exact_source_line_count(
            coverage,
            source_root=source_root,
            source=str(pattern["source"]),
            anchor=str(pattern["anchor"]),
            occurrence=int(pattern.get("occurrence", 1)),
        )
    return exact_function_count(functions, pattern)


def summarize_landmarks(
    *,
    patterns: dict[str, dict[str, Any]],
    cpp_functions: dict[str, list[dict[str, int | str]]],
    rust_functions: dict[str, list[dict[str, int | str]]],
    cpp_coverage: dict[str, Any],
    rust_coverage: dict[str, Any],
    upstream: pathlib.Path,
    repo_root: pathlib.Path,
) -> dict[str, dict[str, int]]:
    return {
        name: {
            "cpp": landmark_count(
                functions=cpp_functions,
                coverage=cpp_coverage,
                pattern=pattern["cpp"],
                source_root=upstream,
            ),
            "rust": landmark_count(
                functions=rust_functions,
                coverage=rust_coverage,
                pattern=pattern["rust"],
                source_root=repo_root,
            ),
        }
        for name, pattern in patterns.items()
    }


def summarize(args: argparse.Namespace) -> dict[str, Any]:
    ledger = tomllib.loads(args.ledger.read_text(encoding="utf-8"))
    repo_root = args.ledger.parent.parent
    scope, source_sets = source_scope(ledger, args.upstream)
    cpp_coverage = json.loads(args.cpp_coverage.read_text(encoding="utf-8"))
    rust_coverage = json.loads(args.rust_coverage.read_text(encoding="utf-8"))
    cpp_full_coverage = json.loads(
        args.cpp_full_coverage.read_text(encoding="utf-8")
    )
    rust_full_coverage = json.loads(
        args.rust_full_coverage.read_text(encoding="utf-8")
    )
    cpp_mechanism_coverage = json.loads(
        args.cpp_mechanism_coverage.read_text(encoding="utf-8")
    )
    rust_mechanism_coverage = json.loads(
        args.rust_mechanism_coverage.read_text(encoding="utf-8")
    )
    cpp_mechanism_full_coverage = json.loads(
        args.cpp_mechanism_full_coverage.read_text(encoding="utf-8")
    )
    rust_mechanism_full_coverage = json.loads(
        args.rust_mechanism_full_coverage.read_text(encoding="utf-8")
    )
    cpp_steady_coverage = json.loads(
        args.cpp_steady_coverage.read_text(encoding="utf-8")
    )
    rust_steady_coverage = json.loads(
        args.rust_steady_coverage.read_text(encoding="utf-8")
    )
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
    cpp_mechanism = coverage_functions(
        path=args.cpp_mechanism_coverage,
        side="cpp",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
        scope_only=False,
        include_zero=True,
    )
    rust_mechanism = coverage_functions(
        path=args.rust_mechanism_coverage,
        side="rust",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
        scope_only=False,
        include_zero=True,
    )
    cpp_mechanism_full = coverage_functions(
        path=args.cpp_mechanism_full_coverage,
        side="cpp",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
        scope_only=False,
        include_zero=True,
    )
    rust_mechanism_full = coverage_functions(
        path=args.rust_mechanism_full_coverage,
        side="rust",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
        scope_only=False,
        include_zero=True,
    )
    cpp_steady = coverage_functions(
        path=args.cpp_steady_coverage,
        side="cpp",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
        scope_only=False,
        include_zero=True,
    )
    rust_steady = coverage_functions(
        path=args.rust_steady_coverage,
        side="rust",
        upstream=args.upstream,
        scope=scope,
        demangler=args.demangler,
        scope_only=False,
        include_zero=True,
    )
    reached_by_source_set = collections.Counter(
        source_sets[path] for path in cpp
    )
    landmarks = summarize_landmarks(
        patterns=LANDMARKS,
        cpp_functions=cpp,
        rust_functions=rust,
        cpp_coverage=cpp_coverage,
        rust_coverage=rust_coverage,
        upstream=args.upstream,
        repo_root=repo_root,
    )
    construction_landmarks = summarize_landmarks(
        patterns=CONSTRUCTION_LANDMARKS,
        cpp_functions=cpp_full,
        rust_functions=rust_full,
        cpp_coverage=cpp_full_coverage,
        rust_coverage=rust_full_coverage,
        upstream=args.upstream,
        repo_root=repo_root,
    )
    mechanism_landmarks = summarize_landmarks(
        patterns=MECHANISM_LANDMARKS,
        cpp_functions=cpp_mechanism,
        rust_functions=rust_mechanism,
        cpp_coverage=cpp_mechanism_coverage,
        rust_coverage=rust_mechanism_coverage,
        upstream=args.upstream,
        repo_root=repo_root,
    )
    mechanism_construction_landmarks = summarize_landmarks(
        patterns=MECHANISM_CONSTRUCTION_LANDMARKS,
        cpp_functions=cpp_mechanism_full,
        rust_functions=rust_mechanism_full,
        cpp_coverage=cpp_mechanism_full_coverage,
        rust_coverage=rust_mechanism_full_coverage,
        upstream=args.upstream,
        repo_root=repo_root,
    )
    steady_landmarks = summarize_landmarks(
        patterns=STEADY_LANDMARKS,
        cpp_functions=cpp_steady,
        rust_functions=rust_steady,
        cpp_coverage=cpp_steady_coverage,
        rust_coverage=rust_steady_coverage,
        upstream=args.upstream,
        repo_root=repo_root,
    )
    allocation_counts = json.loads(
        args.allocation_counts.read_text(encoding="utf-8")
    )
    landmarks["per_frame_allocations"] = {
        side: sum(int(value) for value in allocation_counts[side].values())
        for side in ("cpp", "rust")
    }
    mechanism_allocation_counts = json.loads(
        args.mechanism_allocation_counts.read_text(encoding="utf-8")
    )
    mechanism_landmarks["per_frame_allocations"] = {
        side: sum(
            int(value) for value in mechanism_allocation_counts[side].values()
        )
        for side in ("cpp", "rust")
    }
    steady_allocation_counts = json.loads(
        args.steady_allocation_counts.read_text(encoding="utf-8")
    )
    steady_landmarks["per_frame_allocations"] = {
        side: sum(int(value) for value in steady_allocation_counts[side].values())
        for side in ("cpp", "rust")
    }
    return {
        "schema": "nuxie-runtime-frame-loop-trace/v2",
        "upstream_ref": ledger["upstream_ref"],
        "rust_ref": args.rust_ref,
        "mode": "frame-only counters reset after construction and before samples",
        "corpus": args.corpus_id,
        "mechanism_corpus": args.mechanism_corpus_id,
        "steady_corpus": args.steady_corpus_id,
        "artifacts": {
            "cpp_coverage_sha256": sha256(args.cpp_coverage),
            "rust_coverage_sha256": sha256(args.rust_coverage),
            "cpp_mechanism_coverage_sha256": sha256(
                args.cpp_mechanism_coverage
            ),
            "rust_mechanism_coverage_sha256": sha256(
                args.rust_mechanism_coverage
            ),
            "cpp_steady_coverage_sha256": sha256(args.cpp_steady_coverage),
            "rust_steady_coverage_sha256": sha256(args.rust_steady_coverage),
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
        "mechanism_landmarks": mechanism_landmarks,
        "mechanism_construction_landmarks": mechanism_construction_landmarks,
        "steady_landmarks": steady_landmarks,
        "golden_stream_operations": {
            "cpp": stream_counts(args.stream_directory, "cpp"),
            "rust": stream_counts(args.stream_directory, "rust"),
        },
        "mechanism_golden_stream_operations": {
            "cpp": stream_counts(args.mechanism_stream_directory, "cpp"),
            "rust": stream_counts(args.mechanism_stream_directory, "rust"),
        },
        "allocation_counts": allocation_counts,
        "mechanism_allocation_counts": mechanism_allocation_counts,
        "steady_allocation_counts": steady_allocation_counts,
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
    result.add_argument(
        "--cpp-mechanism-coverage", type=pathlib.Path, required=True
    )
    result.add_argument(
        "--rust-mechanism-coverage", type=pathlib.Path, required=True
    )
    result.add_argument(
        "--cpp-mechanism-full-coverage", type=pathlib.Path, required=True
    )
    result.add_argument(
        "--rust-mechanism-full-coverage", type=pathlib.Path, required=True
    )
    result.add_argument("--cpp-steady-coverage", type=pathlib.Path, required=True)
    result.add_argument("--rust-steady-coverage", type=pathlib.Path, required=True)
    result.add_argument("--cpp-binary", type=pathlib.Path, required=True)
    result.add_argument("--rust-binary", type=pathlib.Path, required=True)
    result.add_argument("--stream-directory", type=pathlib.Path, required=True)
    result.add_argument(
        "--mechanism-stream-directory", type=pathlib.Path, required=True
    )
    result.add_argument("--allocation-counts", type=pathlib.Path, required=True)
    result.add_argument(
        "--mechanism-allocation-counts", type=pathlib.Path, required=True
    )
    result.add_argument(
        "--steady-allocation-counts", type=pathlib.Path, required=True
    )
    result.add_argument("--demangler", type=pathlib.Path, required=True)
    result.add_argument("--rust-ref", required=True)
    result.add_argument("--corpus-id", action="append", default=[])
    result.add_argument("--mechanism-corpus-id", action="append", default=[])
    result.add_argument("--steady-corpus-id", action="append", default=[])
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
