#!/usr/bin/env python3
"""Compile a strict golden stream into exact C++ renderer calls."""

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
FRAME_SIZE_RE = re.compile(r"^frameSize width=(\d+) height=(\d+)$")
SAMPLE_RE = re.compile(rf"^sample seconds=({NUMBER})$")
DECODE_IMAGE_RE = re.compile(
    r"decodeImage id=(\d+) width=(\d+) height=(\d+) data=([0-9a-f]+)"
)
DRAW_IMAGE_RE = re.compile(
    rf"drawImage image=(\d+) sampler=\{{wrapX=(\d+),wrapY=(\d+),"
    rf"filter=(\d+),key=(\d+)\}} blendMode=(\d+) opacity=({NUMBER})"
)
LINEAR_GRADIENT_RE = re.compile(
    rf"makeLinearGradient id=(\d+) start=\(({NUMBER}),({NUMBER})\) "
    rf"end=\(({NUMBER}),({NUMBER})\) stops=(\[.*\])"
)
RADIAL_GRADIENT_RE = re.compile(
    rf"makeRadialGradient id=(\d+) center=\(({NUMBER}),({NUMBER})\) "
    rf"radius=({NUMBER}) stops=(\[.*\])"
)
GRADIENT_STOP_RE = re.compile(
    rf"\{{color=(0x[0-9a-f]{{8}}),stop=({NUMBER})\}}"
)


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


@dataclasses.dataclass(frozen=True)
class GradientStop:
    color: str
    offset: str


@dataclasses.dataclass(frozen=True)
class RivFrameSelection:
    source_file: str
    artboard: str
    scene: str
    width: int
    height: int
    sample_seconds: str
    command_lines: tuple[str, ...]


RETAINED_DECLARATION_PREFIXES = (
    "makeEmptyRenderPath ",
    "makeRenderPaint ",
    "decodeImage ",
    "makeLinearGradient ",
    "makeRadialGradient ",
    "makeRenderBuffer ",
)


def select_riv_frame(lines: list[str], frame_index: int) -> RivFrameSelection:
    if frame_index < 0:
        raise ValueError("RIV frame index must be nonnegative")
    source_indices = [
        index for index, line in enumerate(lines) if line.startswith("source file=")
    ]
    if len(source_indices) != 1:
        raise ValueError("RIV profile header contract drifted")
    source_index = source_indices[0]
    source = SOURCE_RE.fullmatch(lines[source_index])
    frame_size = (
        FRAME_SIZE_RE.fullmatch(lines[source_index + 1])
        if source_index + 1 < len(lines)
        else None
    )
    if source is None or frame_size is None:
        raise ValueError("RIV profile header contract drifted")
    if any(line.startswith("clearColor ") for line in lines):
        raise ValueError("RIV profile header contract drifted")

    sample_indices = [
        index for index, line in enumerate(lines) if SAMPLE_RE.fullmatch(line)
    ]
    frame_indices = [index for index, line in enumerate(lines) if line == "frame"]
    if (
        not sample_indices
        or len(sample_indices) != len(frame_indices)
        or frame_index >= len(sample_indices)
        or frame_indices[-1] != len(lines) - 1
    ):
        raise ValueError("RIV frame-selection contract drifted")
    for index, (sample_index, terminal_index) in enumerate(
        zip(sample_indices, frame_indices)
    ):
        next_sample = (
            sample_indices[index + 1]
            if index + 1 < len(sample_indices)
            else len(lines)
        )
        if not (source_index + 1 < sample_index < terminal_index < next_sample):
            raise ValueError("RIV frame-selection contract drifted")
        if index > 0:
            previous_terminal = frame_indices[index - 1]
            if any(
                not line.startswith("input ")
                for line in lines[previous_terminal + 1 : sample_index]
            ):
                raise ValueError("RIV frame-selection contract drifted")

    first_sample = sample_indices[0]
    prefix = lines[1:source_index] + lines[source_index + 2 : first_sample]
    prior_declarations = []
    for sample_index, terminal_index in zip(
        sample_indices[:frame_index], frame_indices[:frame_index]
    ):
        prior_declarations.extend(
            line
            for line in lines[sample_index + 1 : terminal_index]
            if line.startswith(RETAINED_DECLARATION_PREFIXES)
        )
    selected_sample = SAMPLE_RE.fullmatch(lines[sample_indices[frame_index]])
    assert selected_sample is not None
    selected_commands = lines[
        sample_indices[frame_index] + 1 : frame_indices[frame_index]
    ]
    width, height = (int(value) for value in frame_size.groups())
    return RivFrameSelection(
        *source.groups(),
        width,
        height,
        selected_sample.group(1),
        tuple([*prefix, *prior_declarations, *selected_commands]),
    )


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


def parse_gradient_stops(text: str) -> tuple[GradientStop, ...]:
    if not text.startswith("[") or not text.endswith("]"):
        raise ValueError("gradient stops must use canonical brackets")
    matches = list(GRADIENT_STOP_RE.finditer(text[1:-1]))
    canonical = ",".join(match.group(0) for match in matches)
    if canonical != text[1:-1] or len(matches) < 2:
        raise ValueError("gradient requires at least two canonical stops")
    return tuple(GradientStop(*match.groups()) for match in matches)


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
    frame_index: int = 0,
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
        if frame_index != 0:
            raise ValueError("GM replay only supports frame index 0")
        assert expected_source is not None
        assert expected_clear_color is not None
        expected_header = [
            f'source file="{expected_source}" artboard="" scene="{expected_scene}"',
            f"frameSize width={expected_width} height={expected_height}",
            f"clearColor value={expected_clear_color}",
        ]
        source_indices = [
            index for index, line in enumerate(lines) if line.startswith("source file=")
        ]
        if len(source_indices) != 1:
            raise ValueError("path-only stream header or clear-color contract drifted")
        metadata_index = source_indices[0]
        if lines[metadata_index : metadata_index + 3] != expected_header:
            raise ValueError("path-only stream header or clear-color contract drifted")
        command_lines = lines[1:metadata_index] + lines[metadata_index + 3 : -1]
        if lines[-1] != "frame" or "frame" in lines[:-1]:
            raise ValueError("path-only replay requires exactly one terminal frame marker")
    elif profile == "riv":
        selection = select_riv_frame(lines, frame_index)
        if (
            (expected_source is not None and selection.source_file != expected_source)
            or (
                expected_source_suffix is not None
                and not selection.source_file.endswith(expected_source_suffix)
            )
            or selection.artboard != expected_artboard
            or selection.scene != expected_scene
            or selection.width != expected_width
            or selection.height != expected_height
            or selection.sample_seconds != expected_sample_seconds
        ):
            raise ValueError("RIV profile header contract drifted")
        command_lines = list(selection.command_lines)
    else:
        raise ValueError(f"unsupported stream profile: {profile}")
    if not command_lines:
        raise ValueError("path-only replay requires at least one command")

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
    paint_shaders: dict[int, int] = {}
    shaders: set[int] = set()
    images: set[int] = set()
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
            paints.add(snapshot.object_id)
            paint_shaders[snapshot.object_id] = 0
            count("makeRenderPaint")
            output.append(f"    auto paint{snapshot.object_id} = context->makeRenderPaint();")
            continue
        linear_gradient = LINEAR_GRADIENT_RE.fullmatch(line)
        radial_gradient = RADIAL_GRADIENT_RE.fullmatch(line)
        if linear_gradient is not None or radial_gradient is not None:
            match = linear_gradient or radial_gradient
            assert match is not None
            shader_id = int(match.group(1))
            if shader_id in shaders:
                raise ValueError(f"gradient redeclares shader {shader_id}")
            stops = parse_gradient_stops(match.groups()[-1])
            shaders.add(shader_id)
            command = (
                "makeLinearGradient"
                if linear_gradient is not None
                else "makeRadialGradient"
            )
            count(command)
            colors = ", ".join(stop.color for stop in stops)
            offsets = ", ".join(float_literal(stop.offset) for stop in stops)
            output.extend(
                [
                    f"    static constexpr std::array<rive::ColorInt, {len(stops)}> shader{shader_id}Colors = {{{{{colors}}}}};",
                    f"    static constexpr std::array<float, {len(stops)}> shader{shader_id}Stops = {{{{{offsets}}}}};",
                ]
            )
            coordinates = ", ".join(
                float_literal(value) for value in match.groups()[1:-1]
            )
            output.append(
                f"    auto shader{shader_id} = context->{command}({coordinates}, shader{shader_id}Colors.data(), shader{shader_id}Stops.data(), shader{shader_id}Colors.size());"
            )
            output.extend(
                [
                    f"    if (shader{shader_id} == nullptr)",
                    "    {",
                    f'        fail("reference gradient {shader_id} creation drifted");',
                    "    }",
                ]
            )
            continue
        decode_image = DECODE_IMAGE_RE.fullmatch(line)
        if decode_image is not None:
            image_id_text, width_text, height_text, encoded_hex = decode_image.groups()
            image_id = int(image_id_text)
            width = int(width_text)
            height = int(height_text)
            if image_id in images:
                raise ValueError(f"decodeImage redeclares image {image_id}")
            if width <= 0 or height <= 0:
                raise ValueError("decodeImage dimensions must be positive")
            if len(encoded_hex) % 2 != 0:
                raise ValueError("decodeImage data must contain complete bytes")
            encoded = bytes.fromhex(encoded_hex)
            if not encoded:
                raise ValueError("decodeImage data must not be empty")
            images.add(image_id)
            count("decodeImage")
            output.append(
                f"    static constexpr std::array<uint8_t, {len(encoded)}> image{image_id}Encoded = {{{{"
            )
            for offset in range(0, len(encoded), 16):
                values = ", ".join(
                    f"0x{value:02x}" for value in encoded[offset : offset + 16]
                )
                output.append(f"        {values},")
            output.extend(
                [
                    "    }};",
                    f"    auto image{image_id} = context->decodeImage(rive::Span<const uint8_t>(image{image_id}Encoded.data(), image{image_id}Encoded.size()));",
                    f"    if (image{image_id} == nullptr || image{image_id}->width() != {width} || image{image_id}->height() != {height})",
                    "    {",
                    f'        fail("MSAA reference image {image_id} decode or dimensions drifted");',
                    "    }",
                ]
            )
            continue
        draw_image = DRAW_IMAGE_RE.fullmatch(line)
        if draw_image is not None:
            (
                image_id_text,
                wrap_x_text,
                wrap_y_text,
                filter_text,
                sampler_key_text,
                blend_mode_text,
                opacity,
            ) = draw_image.groups()
            image_id = int(image_id_text)
            wrap_x = int(wrap_x_text)
            wrap_y = int(wrap_y_text)
            image_filter = int(filter_text)
            sampler_key = int(sampler_key_text)
            blend_mode = int(blend_mode_text)
            if image_id not in images:
                raise ValueError(f"drawImage references undeclared image {image_id}")
            if wrap_x not in range(3) or wrap_y not in range(3):
                raise ValueError("drawImage sampler has an invalid wrap mode")
            if image_filter not in range(2):
                raise ValueError("drawImage sampler has an invalid filter mode")
            expected_sampler_key = wrap_x + wrap_y * 3 + image_filter * 9
            if sampler_key != expected_sampler_key:
                raise ValueError(
                    "drawImage sampler key is inconsistent with its fields: "
                    f"expected {expected_sampler_key}, got {sampler_key}"
                )
            if blend_mode not in range(29):
                raise ValueError("drawImage has an invalid blend mode")
            count("drawImage")
            output.append(
                f"    renderer->drawImage(image{image_id}.get(), rive::ImageSampler::SamplerFromKey({sampler_key}), static_cast<rive::BlendMode>({blend_mode}), {float_literal(opacity)});"
            )
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
            if paint.shader != 0 and paint.shader not in shaders:
                raise ValueError(f"drawPath references undeclared shader {paint.shader}")
            output.append(
                f"    path{path.object_id}->fillRule(static_cast<rive::FillRule>({path.fill_rule}));"
            )
            materialize_path(path)
            style = "fill" if paint.style == "fill" else "stroke"
            shader_binding = []
            if paint.shader != paint_shaders[paint.object_id]:
                shader_value = (
                    f"shader{paint.shader}" if paint.shader != 0 else "nullptr"
                )
                shader_binding.append(
                    f"    paint{paint.object_id}->shader({shader_value});"
                )
                paint_shaders[paint.object_id] = paint.shader
            output.extend(
                [
                    f"    paint{paint.object_id}->style(rive::RenderPaintStyle::{style});",
                    f"    paint{paint.object_id}->color({paint.color});",
                    f"    paint{paint.object_id}->thickness({float_literal(paint.thickness)});",
                    f"    paint{paint.object_id}->join(static_cast<rive::StrokeJoin>({paint.join}));",
                    f"    paint{paint.object_id}->cap(static_cast<rive::StrokeCap>({paint.cap}));",
                    f"    paint{paint.object_id}->feather({float_literal(paint.feather)});",
                    f"    paint{paint.object_id}->blendMode(static_cast<rive::BlendMode>({blend_mode_override if blend_mode_override is not None else paint.blend_mode}));",
                    *shader_binding,
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
    parser.add_argument("--frame-index", type=int, default=0)
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
        frame_index=args.frame_index,
    )
    if args.output:
        args.output.write_text(generated)


if __name__ == "__main__":
    main()
