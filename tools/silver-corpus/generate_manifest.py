#!/usr/bin/env python3
"""Generate silver-corpus.toml from the pinned upstream C++ producers.

Literal matches() calls are discovered mechanically. The six layout-scroll
names assembled through helper arguments are deliberately listed below, and
the two producerless files are deliberately classified as provenance-unknown.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

UPSTREAM_REF = "d788e8ec6e8b598526607d6a1e8818e8b637b60c"
LITERAL_MATCH = re.compile(
    r'(?:silver\.matches|serializer\(\)->matches)\(\s*"([^"]+)"', re.MULTILINE
)
TEST_CASE = re.compile(r"\bTEST_CASE\s*\(")
RIV_STRING = re.compile(r'"([^"\n]*\.riv)"')
ARTBOARD_NAME = re.compile(r'artboardNamed\(\s*"([^"]+)"')
ANIMATION_NAME = re.compile(r'animationNamed\(\s*"([^"]+)"')
STATE_MACHINE_NAME = re.compile(r'stateMachineNamed\(\s*"([^"]+)"')
SAMPLE_TIME = re.compile(
    r"(?:advanceAndApply|advance)\(\s*([0-9]+(?:\.[0-9]+)?)(?:f)?\s*\)"
)


@dataclass(frozen=True)
class Producer:
    id: str
    source: str
    dependencies: tuple[str, ...]
    artboard: str
    animation: str
    state_machine: str
    lane: str
    deterministic: str
    random: str
    view_model: str
    sample_times: tuple[float, ...]
    actions: str
    status: str
    producer_class: str
    provenance_file: str
    provenance_test: str
    producer_line: int
    note: str


DYNAMIC_LAYOUT_SCROLL = (
    (
        "layout_scroll_snap_padding_layouts",
        "layout/layout_scroll_snap_padding.riv",
        "ScrollLayouts",
        "Scroll snap respects viewport padding (layouts)",
        550,
    ),
    (
        "layout_scroll_snap_padding_list",
        "layout/layout_scroll_snap_padding.riv",
        "ScrollList",
        "Scroll snap respects viewport padding (list)",
        556,
    ),
    (
        "layout_scroll_snap_padding_virtualized",
        "layout/layout_scroll_snap_padding.riv",
        "ScrollListVirtualized",
        "Scroll snap respects viewport padding (virtualized list)",
        562,
    ),
    (
        "layout_scroll_drag_multiplier_layouts",
        "layout/layout_scroll_drag_multiplier.riv",
        "ScrollLayouts",
        "Scroll drag multiplier (layouts)",
        569,
    ),
    (
        "layout_scroll_drag_multiplier_list",
        "layout/layout_scroll_drag_multiplier.riv",
        "ScrollList",
        "Scroll drag multiplier (list)",
        575,
    ),
    (
        "layout_scroll_drag_multiplier_virtualized",
        "layout/layout_scroll_drag_multiplier.riv",
        "ScrollListVirtualized",
        "Scroll drag multiplier (virtualized list)",
        581,
    ),
)

PROVENANCE_UNKNOWN = ("interpolator", "multitouch_debug")


def quoted(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def unique(values: list[str]) -> tuple[str, ...]:
    return tuple(dict.fromkeys(values))


def strip_asset_prefix(value: str) -> str:
    return value.removeprefix("assets/")


def test_chunks(source: str) -> list[tuple[str, int, str]]:
    starts = list(TEST_CASE.finditer(source))
    chunks: list[tuple[str, int, str]] = []
    for index, start in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(source)
        chunk = source[start.start() : end]
        name = re.search(r'"([^"]+)"', chunk)
        if name is None:
            continue
        line = source.count("\n", 0, start.start()) + 1
        chunks.append((name.group(1), line, chunk))
    return chunks


def infer_selector(pattern: re.Pattern[str], chunk: str, default_marker: str) -> str:
    names = unique(pattern.findall(chunk))
    if names:
        return " | ".join(names)
    return default_marker


def literal_producers(runtime_dir: Path) -> list[Producer]:
    runtime_tests = runtime_dir / "tests/unit_tests/runtime"
    files = sorted(runtime_tests.glob("*.cpp")) + sorted(
        (runtime_tests / "scripting").glob("*.cpp")
    )
    producers: list[Producer] = []
    for path in files:
        relative = path.relative_to(runtime_dir).as_posix()
        source = path.read_text(encoding="utf-8", errors="replace")
        for test_name, test_line, chunk in test_chunks(source):
            for match in LITERAL_MATCH.finditer(chunk):
                silver_id = match.group(1)
                riv_sources = unique(
                    [strip_asset_prefix(value) for value in RIV_STRING.findall(chunk)]
                )
                if silver_id == "gamepad_test":
                    # Its fixture is opened by openReadyStateMachine above the test.
                    riv_sources = ("gamepad_test.riv",)
                primary = riv_sources[0] if riv_sources else "inline-script"
                dependencies = riv_sources[1:]
                lane = "scripted" if "/scripting/" in f"/{relative}" else "runtime"
                serialized = path.name == "serialized_rendering_test.cpp"
                producer_class = (
                    "serialized-rendering"
                    if serialized
                    else "scripted-literal"
                    if lane == "scripted"
                    else "runtime-literal"
                )
                deterministic = (
                    "enabled"
                    if "deterministicMode = true" in chunk
                    else "cpp-test-defined"
                )
                sample_times = tuple(float(value) for value in SAMPLE_TIME.findall(chunk))
                artboard = infer_selector(
                    ARTBOARD_NAME,
                    chunk,
                    "default" if "artboard" in chunk else "cpp-test-defined",
                )
                animation = infer_selector(
                    ANIMATION_NAME,
                    chunk,
                    "default" if "defaultAnimation" in chunk else "none",
                )
                state_machine = infer_selector(
                    STATE_MACHINE_NAME,
                    chunk,
                    "default"
                    if "defaultStateMachine" in chunk or "stateMachineAt" in chunk
                    else "none",
                )
                view_model = (
                    "cpp-test-defined"
                    if "ViewModel" in chunk
                    or "viewModel" in chunk
                    or "bindViewModelInstance" in chunk
                    else "none"
                )
                line = test_line + chunk.count("\n", 0, match.start())
                note = (
                    "Serialized-rendering producer is catalogued from the full upstream test "
                    "body; shared action-DSL translation and Rust replay remain pending."
                    if serialized
                    else "Literal upstream producer is catalogued; shared action-DSL "
                    "translation and Rust replay remain pending."
                )
                if lane == "scripted":
                    note = (
                        "Scripted producer provenance is catalogued; scripted action/output "
                        "replay is explicitly deferred to the next adoption step."
                    )
                producers.append(
                    Producer(
                        id=silver_id,
                        source=primary,
                        dependencies=dependencies,
                        artboard=artboard,
                        animation=animation,
                        state_machine=state_machine,
                        lane=lane,
                        deterministic=deterministic,
                        random="deterministic"
                        if deterministic == "enabled"
                        else "cpp-test-defined",
                        view_model=view_model,
                        sample_times=sample_times,
                        actions="cpp-test-body",
                        status="pending-scripted" if lane == "scripted" else "pending",
                        producer_class=producer_class,
                        provenance_file=relative,
                        provenance_test=test_name,
                        producer_line=line,
                        note=note,
                    )
                )
    return producers


def dynamic_producers() -> list[Producer]:
    return [
        Producer(
            id=silver_id,
            source=source,
            dependencies=(),
            artboard=artboard,
            animation="none",
            state_machine="default",
            lane="runtime",
            deterministic="enabled",
            random="deterministic",
            view_model="bind-default-if-present",
            sample_times=(0.0, 0.016),
            actions="cpp-test-body",
            status="pending",
            producer_class="layout-scroll-dynamic",
            provenance_file="tests/unit_tests/runtime/layout_scroll_test.cpp",
            provenance_test=test_name,
            producer_line=line,
            note=(
                "Dynamically named layout-scroll producer is hand-authored from its helper "
                "arguments; shared action-DSL translation and Rust replay remain pending."
            ),
        )
        for silver_id, source, artboard, test_name, line in DYNAMIC_LAYOUT_SCROLL
    ]


def unknown_producers() -> list[Producer]:
    return [
        Producer(
            id=silver_id,
            source="provenance-unknown",
            dependencies=(),
            artboard="provenance-unknown",
            animation="provenance-unknown",
            state_machine="provenance-unknown",
            lane="unknown",
            deterministic="provenance-unknown",
            random="provenance-unknown",
            view_model="provenance-unknown",
            sample_times=(),
            actions="provenance-unknown",
            status="provenance-unknown",
            producer_class="provenance-unknown",
            provenance_file="",
            provenance_test="",
            producer_line=0,
            note="No producer/reference exists in the pinned upstream runtime tests.",
        )
        for silver_id in PROVENANCE_UNKNOWN
    ]


def discover(runtime_dir: Path) -> list[Producer]:
    producers = literal_producers(runtime_dir) + dynamic_producers() + unknown_producers()
    ids = [producer.id for producer in producers]
    duplicates = sorted({silver_id for silver_id in ids if ids.count(silver_id) > 1})
    if duplicates:
        raise ValueError(f"duplicate producer ids: {duplicates}")

    silver_ids = sorted(
        path.stem
        for path in (runtime_dir / "tests/unit_tests/silvers").glob("*.sriv")
    )
    if sorted(ids) != silver_ids:
        unrepresented = sorted(set(silver_ids) - set(ids))
        without_file = sorted(set(ids) - set(silver_ids))
        raise ValueError(
            f"producer/silver mismatch: unrepresented={unrepresented}, "
            f"without_file={without_file}"
        )
    return sorted(producers, key=lambda producer: producer.id)


def verify_upstream_ref(runtime_dir: Path) -> None:
    try:
        actual = subprocess.run(
            ["git", "-C", str(runtime_dir), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise ValueError(f"cannot resolve upstream ref at {runtime_dir}: {error}") from error
    if actual != UPSTREAM_REF:
        raise ValueError(f"upstream ref is {actual}; expected {UPSTREAM_REF}")


def render(producers: list[Producer]) -> str:
    runtime = sum(producer.lane == "runtime" for producer in producers)
    scripted = sum(producer.lane == "scripted" for producer in producers)
    unknown = sum(producer.status == "provenance-unknown" for producer in producers)
    if (len(producers), runtime, scripted, unknown) != (238, 195, 41, 2):
        raise ValueError(
            "ratchet mismatch: "
            f"entries={len(producers)} runtime={runtime} scripted={scripted} unknown={unknown}"
        )

    lines = [
        "# Generated by tools/silver-corpus/generate_manifest.py.",
        "# Runtime entries are catalogued but pending shared action-DSL/Rust replay.",
        "",
        "[corpus]",
        "version = 1",
        f"upstream_ref = {quoted(UPSTREAM_REF)}",
        "expected_entries = 238",
        "expected_runtime = 195",
        "expected_scripted = 41",
        "max_provenance_unknown = 2",
        "min_cpp_rust_exact = 0",
        "cpp_rust_exact_ids = []",
        "",
    ]
    for producer in producers:
        lines.extend(
            [
                "[[case]]",
                f"id = {quoted(producer.id)}",
                "expected = "
                + quoted(f"tests/unit_tests/silvers/{producer.id}.sriv"),
                f"source = {quoted(producer.source)}",
                "dependencies = ["
                + ", ".join(quoted(value) for value in producer.dependencies)
                + "]",
                f"artboard = {quoted(producer.artboard)}",
                f"animation = {quoted(producer.animation)}",
                f"state_machine = {quoted(producer.state_machine)}",
                f"lane = {quoted(producer.lane)}",
                f"deterministic = {quoted(producer.deterministic)}",
                f"random = {quoted(producer.random)}",
                f"view_model = {quoted(producer.view_model)}",
                "sample_times = ["
                + ", ".join(format(value, ".9g") for value in producer.sample_times)
                + "]",
                f"actions = {quoted(producer.actions)}",
                'verification = "sriv-v1-epsilon"',
                f"status = {quoted(producer.status)}",
                f"producer_class = {quoted(producer.producer_class)}",
                f"provenance_file = {quoted(producer.provenance_file)}",
                f"provenance_test = {quoted(producer.provenance_test)}",
                f"producer_line = {producer.producer_line}",
                f"note = {quoted(producer.note)}",
                "",
            ]
        )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--rive-runtime-dir",
        type=Path,
        default=Path("/Users/levi/dev/oss/rive-runtime"),
    )
    parser.add_argument("--output", type=Path, default=Path("silver-corpus.toml"))
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--skip-ref-check", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if not args.skip_ref_check:
        verify_upstream_ref(args.rive_runtime_dir)
    generated = render(discover(args.rive_runtime_dir))
    if args.check:
        existing = args.output.read_text(encoding="utf-8")
        if existing != generated:
            raise SystemExit(
                f"{args.output} is stale; run tools/silver-corpus/generate_manifest.py"
            )
        print(f"{args.output}: generated manifest is current")
        return 0
    args.output.write_text(generated, encoding="utf-8")
    print(f"wrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
