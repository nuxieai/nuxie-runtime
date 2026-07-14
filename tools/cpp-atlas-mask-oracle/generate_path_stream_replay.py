#!/usr/bin/env python3
"""Compile a path-only golden stream into exact C++ renderer calls."""

import argparse
import dataclasses
import hashlib
import pathlib
import re


NUMBER = r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?"
POINT_RE = re.compile(rf"\(({NUMBER}),({NUMBER})\)")
TRANSFORM_RE = re.compile(
    rf"transform matrix=\[({NUMBER}),({NUMBER}),({NUMBER}),"
    rf"({NUMBER}),({NUMBER}),({NUMBER})\]"
)
PATH_RE = re.compile(
    r"\{id=(\d+),fillRule=(\d+),path=\{verbs=\[([^]]*)\],points=\[([^]]*)\]\}\}"
)
PAINT_RE = re.compile(
    rf"\{{id=(\d+),style=(fill|stroke),color=(0x[0-9a-f]{{8}}),"
    rf"thickness=({NUMBER}),join=(\d+),cap=(\d+),feather=({NUMBER}),"
    r"blendMode=(\d+),shader=(\d+)\}"
)
SOURCE_RE = re.compile(r'^source file="([^"]*)" artboard="([^"]*)" scene="([^"]*)"$')


@dataclasses.dataclass(frozen=True)
class PathSnapshot:
    object_id: int
    fill_rule: int
    records: tuple[tuple[str, tuple[str, ...]], ...]


@dataclasses.dataclass(frozen=True)
class PaintSnapshot:
    object_id: int
    style: str
    color: str
    thickness: str
    join: int
    cap: int
    feather: str
    blend_mode: int
    shader: int


def parse_path(text: str) -> PathSnapshot:
    match = PATH_RE.fullmatch(text)
    if match is None:
        raise ValueError(f"invalid path snapshot: {text[:120]}")
    object_id, fill_rule_text, verbs_text, points_text = match.groups()
    verbs = [] if not verbs_text else verbs_text.split(",")
    points = POINT_RE.findall(points_text)
    if ",".join(f"({x},{y})" for x, y in points) != points_text:
        raise ValueError(f"path {object_id} contains a noncanonical point literal")
    arity = {"move": 1, "line": 1, "quad": 2, "cubic": 3, "close": 0}
    records = []
    point_index = 0
    for verb in verbs:
        if verb not in arity:
            raise ValueError(f"path {object_id} has unsupported verb {verb!r}")
        end = point_index + arity[verb]
        if end > len(points):
            raise ValueError(f"path {object_id} verb stream overruns its points")
        values = tuple(value for point in points[point_index:end] for value in point)
        records.append((verb, values))
        point_index = end
    if point_index != len(points):
        raise ValueError(f"path {object_id} has unconsumed points")
    return PathSnapshot(int(object_id), int(fill_rule_text), tuple(records))


def parse_paint(text: str) -> PaintSnapshot:
    match = PAINT_RE.fullmatch(text)
    if match is None:
        raise ValueError(f"invalid paint snapshot: {text[:120]}")
    (
        object_id,
        style,
        color,
        thickness,
        join,
        cap,
        feather,
        blend_mode,
        shader,
    ) = match.groups()
    return PaintSnapshot(
        int(object_id),
        style,
        color,
        thickness,
        int(join),
        int(cap),
        feather,
        int(blend_mode),
        int(shader),
    )


def float_literal(value: str) -> str:
    return f"{value}f" if any(char in value for char in ".eE") else f"{value}.f"


def parse_expected_counts(values: list[str]) -> dict[str, int]:
    counts = {}
    for value in values:
        name, separator, count = value.partition("=")
        if not separator or not name or name in counts:
            raise ValueError(f"invalid or duplicate expected count {value!r}")
        counts[name] = int(count)
    return counts


def generate_include(
    stream: pathlib.Path,
    profile: str,
    expected_sha256: str,
    expected_source: str | None,
    expected_source_suffix: str | None,
    expected_artboard: str,
    expected_scene: str,
    expected_width: int,
    expected_height: int,
    expected_clear_color: str | None,
    expected_sample_seconds: str | None,
    expected_counts: dict[str, int],
    function_name: str,
    blend_mode_override: int | None,
    function_attribute: str | None = None,
) -> str:
    if expected_clear_color is not None and re.fullmatch(
        r"0x[0-9a-f]{8}", expected_clear_color
    ) is None:
        raise ValueError(
            f"expected clear color must be canonical 0xRRGGBBAA: {expected_clear_color!r}"
        )
    raw = stream.read_bytes()
    actual_sha256 = hashlib.sha256(raw).hexdigest()
    if actual_sha256 != expected_sha256:
        raise ValueError(
            "path-only stream sha256 drifted: "
            f"expected {expected_sha256}, got {actual_sha256}"
        )
    lines = raw.decode("utf-8").splitlines()
    if not lines or lines[0] != "rive-golden-stream-v1":
        raise ValueError("path-only stream header or clear-color contract drifted")
    if profile == "gm":
        assert expected_source is not None
        assert expected_clear_color is not None
        expected_header = [
            f'source file="{expected_source}" artboard="" scene="{expected_scene}"',
            f"frameSize width={expected_width} height={expected_height}",
            f"clearColor value={expected_clear_color}",
        ]
        metadata_index = 1
        while metadata_index < len(lines) and (
            lines[metadata_index].startswith("makeEmptyRenderPath ")
            or lines[metadata_index].startswith("makeRenderPaint ")
        ):
            metadata_index += 1
        if lines[metadata_index : metadata_index + 3] != expected_header:
            raise ValueError("path-only stream header or clear-color contract drifted")
        command_lines = lines[1:metadata_index] + lines[metadata_index + 3 : -1]
    elif profile == "riv":
        metadata_index = 1
        while metadata_index < len(lines) and (
            lines[metadata_index].startswith("makeEmptyRenderPath ")
            or lines[metadata_index].startswith("makeRenderPaint ")
        ):
            metadata_index += 1
        if metadata_index + 2 >= len(lines):
            raise ValueError("RIV profile header contract drifted")
        source = SOURCE_RE.fullmatch(lines[metadata_index])
        if source is None:
            raise ValueError("RIV profile header contract drifted")
        source_file, artboard, scene = source.groups()
        if (
            (expected_source is not None and source_file != expected_source)
            or (expected_source_suffix is not None and not source_file.endswith(expected_source_suffix))
            or artboard != expected_artboard
            or scene != expected_scene
            or lines[metadata_index + 1]
            != f"frameSize width={expected_width} height={expected_height}"
            or lines[metadata_index + 2] != f"sample seconds={expected_sample_seconds}"
            or any(line.startswith("clearColor ") for line in lines)
        ):
            raise ValueError("RIV profile header contract drifted")
        command_lines = lines[1:metadata_index] + lines[metadata_index + 3:-1]
    else:
        raise ValueError(f"unsupported stream profile: {profile}")
    if not command_lines or lines[-1] != "frame" or "frame" in lines[:-1]:
        raise ValueError("path-only replay requires exactly one terminal frame marker")

    function_declaration = (
        f"{function_attribute} void {function_name}"
        if function_attribute is not None
        else f"void {function_name}"
    )
    output = [
        f"// Generated by {pathlib.Path(__file__).name}; do not edit.",
        f"// Source: {stream.name} sha256={actual_sha256}.",
        f"{function_declaration}(rive::RiveRenderer* renderer, rive::gpu::RenderContext* context)",
        "{",
    ]
    if blend_mode_override is not None:
        output.insert(
            2, f"// Diagnostic paint blend-mode override={blend_mode_override}."
        )
    paths: dict[int, PathSnapshot | None] = {}
    paints: set[int] = set()
    counts: dict[str, int] = {}
    save_depth = 0

    def count(name: str) -> None:
        counts[name] = counts.get(name, 0) + 1

    def materialize_path(path: PathSnapshot) -> None:
        if path.object_id not in paths:
            raise ValueError("path snapshot references an undeclared path")
        if paths[path.object_id] is None:
            methods = {
                "move": "moveTo",
                "line": "lineTo",
                "quad": "quadTo",
                "cubic": "cubicTo",
            }
            for verb, values in path.records:
                if verb == "close":
                    output.append(f"    path{path.object_id}->close();")
                else:
                    arguments = ", ".join(float_literal(value) for value in values)
                    output.append(
                        f"    path{path.object_id}->{methods[verb]}({arguments});"
                    )
            paths[path.object_id] = path
        elif paths[path.object_id].records != path.records:
            raise ValueError(
                f"path {path.object_id} mutates after its first snapshot; unsupported by this oracle"
            )

    for line in command_lines:
        if line == "save":
            save_depth += 1
            count("save")
            output.append("    renderer->save();")
            continue
        if line == "restore":
            if save_depth == 0:
                raise ValueError("path-only stream restores past the renderer stack root")
            save_depth -= 1
            count("restore")
            output.append("    renderer->restore();")
            continue
        transform = TRANSFORM_RE.fullmatch(line)
        if transform is not None:
            count("transform")
            values = ", ".join(float_literal(value) for value in transform.groups())
            output.append(f"    renderer->transform(rive::Mat2D({values}));")
            continue
        if line.startswith("makeEmptyRenderPath "):
            snapshot = parse_path(line.removeprefix("makeEmptyRenderPath "))
            if snapshot.object_id in paths or snapshot.records:
                raise ValueError("makeEmptyRenderPath must declare a unique empty path")
            paths[snapshot.object_id] = None
            count("makeEmptyRenderPath")
            output.append(f"    auto path{snapshot.object_id} = context->makeEmptyRenderPath();")
            continue
        if line.startswith("makeRenderPaint "):
            snapshot = parse_paint(line.removeprefix("makeRenderPaint "))
            if snapshot.object_id in paints:
                raise ValueError("makeRenderPaint must declare a unique paint")
            if snapshot.shader != 0:
                raise ValueError("path-only replay does not support paint shaders")
            paints.add(snapshot.object_id)
            count("makeRenderPaint")
            output.append(f"    auto paint{snapshot.object_id} = context->makeRenderPaint();")
            continue
        if line.startswith("clipPath path="):
            path = parse_path(line.removeprefix("clipPath path="))
            output.append(
                f"    path{path.object_id}->fillRule(static_cast<rive::FillRule>({path.fill_rule}));"
            )
            materialize_path(path)
            count("clipPath")
            output.append(f"    renderer->clipPath(path{path.object_id}.get());")
            continue
        if line.startswith("drawPath path="):
            body = line.removeprefix("drawPath path=")
            path_text, separator, paint_text = body.rpartition(" paint=")
            if not separator:
                raise ValueError("drawPath is missing its paint snapshot")
            path = parse_path(path_text)
            paint = parse_paint(paint_text)
            if path.object_id not in paths or paint.object_id not in paints:
                raise ValueError("drawPath references an undeclared path or paint")
            if paint.shader != 0:
                raise ValueError("path-only replay does not support paint shaders")
            output.append(
                f"    path{path.object_id}->fillRule(static_cast<rive::FillRule>({path.fill_rule}));"
            )
            materialize_path(path)
            style = "fill" if paint.style == "fill" else "stroke"
            output.extend(
                [
                    f"    paint{paint.object_id}->style(rive::RenderPaintStyle::{style});",
                    f"    paint{paint.object_id}->color({paint.color});",
                    f"    paint{paint.object_id}->thickness({float_literal(paint.thickness)});",
                    f"    paint{paint.object_id}->join(static_cast<rive::StrokeJoin>({paint.join}));",
                    f"    paint{paint.object_id}->cap(static_cast<rive::StrokeCap>({paint.cap}));",
                    f"    paint{paint.object_id}->feather({float_literal(paint.feather)});",
                    f"    paint{paint.object_id}->blendMode(static_cast<rive::BlendMode>({blend_mode_override if blend_mode_override is not None else paint.blend_mode}));",
                    f"    renderer->drawPath(path{path.object_id}.get(), paint{paint.object_id}.get());",
                ]
            )
            count("drawPath")
            continue
        raise ValueError(f"unsupported path-only stream command: {line[:120]}")

    if save_depth != 0:
        raise ValueError(f"path-only stream leaves {save_depth} unmatched saves")
    if counts != expected_counts:
        raise ValueError(f"path-only stream counts drifted: expected {expected_counts}, got {counts}")
    output.append("}")
    return "\n".join(output) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stream", type=pathlib.Path, required=True)
    parser.add_argument("--profile", choices=("gm", "riv"), default="gm")
    parser.add_argument("--expected-sha256", required=True)
    source_group = parser.add_mutually_exclusive_group(required=True)
    source_group.add_argument("--expected-source")
    source_group.add_argument("--expected-source-suffix")
    parser.add_argument("--expected-artboard", default="")
    parser.add_argument("--expected-scene")
    parser.add_argument("--expected-width", type=int, required=True)
    parser.add_argument("--expected-height", type=int, required=True)
    parser.add_argument("--expected-clear-color")
    parser.add_argument("--expected-sample-seconds")
    parser.add_argument("--expected-count", action="append", default=[])
    parser.add_argument("--override-blend-mode", type=int, choices=range(29))
    parser.add_argument("--function", required=True)
    parser.add_argument("--output", type=pathlib.Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if bool(args.output) == args.check:
        parser.error("specify exactly one of --output or --check")
    if args.profile == "gm":
        if args.expected_source is None:
            parser.error("GM profile requires --expected-source")
        expected_scene = args.expected_scene or args.expected_source.removeprefix("gm:")
        expected_clear_color = args.expected_clear_color or "0x00000000"
    else:
        if args.expected_scene is None:
            parser.error("RIV profile requires --expected-scene")
        if args.expected_sample_seconds is None:
            parser.error("RIV profile requires --expected-sample-seconds")
        if args.expected_clear_color is not None:
            parser.error("RIV profile has an implicit transparent clear color")
        expected_scene = args.expected_scene
        expected_clear_color = None
    generated = generate_include(
        args.stream,
        args.profile,
        args.expected_sha256,
        args.expected_source,
        args.expected_source_suffix,
        args.expected_artboard,
        expected_scene,
        args.expected_width,
        args.expected_height,
        expected_clear_color,
        args.expected_sample_seconds,
        parse_expected_counts(args.expected_count),
        args.function,
        args.override_blend_mode,
    )
    if args.output:
        args.output.write_text(generated)


if __name__ == "__main__":
    main()
