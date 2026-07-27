#!/usr/bin/env python3
import hashlib
import json
import pathlib
import re
import sys


EXPECTED_FONT_SHA256 = (
    "4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf"
)
EXPECTED_WEIGHTS = [400.0, 500.0, 600.0, 700.0]


def canonical_outline(outline):
    return json.dumps(outline, separators=(",", ":")).encode()


def compare_direct(cpp_path, rust_path):
    cpp = json.loads(pathlib.Path(cpp_path).read_text())
    rust = json.loads(pathlib.Path(rust_path).read_text())
    if rust.get("font_sha256") != EXPECTED_FONT_SHA256:
        raise SystemExit(f"Rust font hash differs: {rust.get('font_sha256')}")
    if cpp["font_bytes"] != 879708 or rust["font_bytes"] != 879708:
        raise SystemExit(
            f"font length differs: cpp={cpp['font_bytes']} rust={rust['font_bytes']}"
        )
    if cpp.get("face_index") != 0 or rust.get("face_index") != 0:
        raise SystemExit(
            f"face index differs: cpp={cpp.get('face_index')} "
            f"rust={rust.get('face_index')}"
        )
    if cpp.get("axis_tag") != "wght" or rust.get("axis_tag") != "wght":
        raise SystemExit(
            f"axis tag differs: cpp={cpp.get('axis_tag')} "
            f"rust={rust.get('axis_tag')}"
        )
    cpp_weights = [entry["weight"] for entry in cpp["results"]]
    rust_weights = [entry["weight"] for entry in rust["results"]]
    if cpp_weights != EXPECTED_WEIGHTS or rust_weights != EXPECTED_WEIGHTS:
        raise SystemExit(f"weights differ: cpp={cpp_weights} rust={rust_weights}")

    max_advance_delta = 0.0
    max_outline_delta = 0.0
    glyph_count = 0
    outline_command_count = 0
    weight_outline_hashes = []
    for cpp_weight, rust_weight in zip(
        cpp["results"], rust["results"], strict=True
    ):
        if cpp_weight["axis_value"] != rust_weight["axis_value"]:
            raise SystemExit(
                f"axis differs at {cpp_weight['weight']}: "
                f"{cpp_weight['axis_value']} vs {rust_weight['axis_value']}"
            )
        if cpp_weight["text"] != rust_weight["text"]:
            raise SystemExit(f"text differs at {cpp_weight['weight']}")
        if len(cpp_weight["glyphs"]) != len(rust_weight["glyphs"]):
            raise SystemExit(
                f"glyph count differs at {cpp_weight['weight']}: "
                f"{len(cpp_weight['glyphs'])} vs {len(rust_weight['glyphs'])}"
            )
        suffix_hasher = hashlib.sha256()
        for index, (cpp_glyph, rust_glyph) in enumerate(
            zip(cpp_weight["glyphs"], rust_weight["glyphs"], strict=True)
        ):
            glyph_count += 1
            if cpp_glyph["id"] != rust_glyph["id"]:
                raise SystemExit(
                    f"glyph ID differs at weight {cpp_weight['weight']} index {index}: "
                    f"{cpp_glyph['id']} vs {rust_glyph['id']}"
                )
            advance_delta = abs(cpp_glyph["advance"] - rust_glyph["advance"])
            max_advance_delta = max(max_advance_delta, advance_delta)
            if advance_delta > 1e-6:
                raise SystemExit(
                    f"advance differs at weight {cpp_weight['weight']} index {index}: "
                    f"{cpp_glyph['advance']} vs {rust_glyph['advance']}"
                )
            cpp_outline = cpp_glyph["outline"]
            rust_outline = rust_glyph["outline"]
            if len(cpp_outline) != len(rust_outline):
                raise SystemExit(
                    f"outline command count differs at weight {cpp_weight['weight']} "
                    f"index {index}: {len(cpp_outline)} vs {len(rust_outline)}"
                )
            for command_index, (cpp_command, rust_command) in enumerate(
                zip(cpp_outline, rust_outline, strict=True)
            ):
                outline_command_count += 1
                if (
                    cpp_command[0] != rust_command[0]
                    or len(cpp_command) != len(rust_command)
                ):
                    raise SystemExit(
                        f"outline verb differs at weight {cpp_weight['weight']} "
                        f"glyph {index} command {command_index}: "
                        f"{cpp_command[0]} vs {rust_command[0]}"
                    )
                for cpp_value, rust_value in zip(
                    cpp_command[1:], rust_command[1:], strict=True
                ):
                    delta = abs(cpp_value - rust_value)
                    max_outline_delta = max(max_outline_delta, delta)
                    if delta > 1e-6:
                        raise SystemExit(
                            f"outline coordinate differs at weight "
                            f"{cpp_weight['weight']} glyph {index} command "
                            f"{command_index}: {cpp_value} vs {rust_value}"
                        )
            # Ignore the weight prefix ("400 ", etc.) so this digest proves
            # that the same "Inter sample" glyphs vary across wght values.
            if index >= 4:
                suffix_hasher.update(canonical_outline(cpp_outline))
        weight_outline_hashes.append(suffix_hasher.hexdigest())

    if len(set(weight_outline_hashes)) != len(weight_outline_hashes):
        raise SystemExit(
            f"variable outlines collapsed across weights: {weight_outline_hashes}"
        )
    if glyph_count != 64 or outline_command_count != 1507:
        raise SystemExit(
            f"comparison-count drift: glyphs={glyph_count} "
            f"outlines={outline_command_count}"
        )
    print(
        json.dumps(
            {
                "result": "exact-within-1e-6",
                "font_bytes": cpp["font_bytes"],
                "face_index": 0,
                "axis_tag": "wght",
                "weights": cpp_weights,
                "glyphs_compared": glyph_count,
                "outline_commands_compared": outline_command_count,
                "max_advance_delta": max_advance_delta,
                "max_outline_delta": max_outline_delta,
                "distinct_common_suffix_outline_hashes": weight_outline_hashes,
            },
            sort_keys=True,
        )
    )


def partition(lines):
    if not lines or lines[0] != "rive-golden-stream-v1":
        raise SystemExit("stream header differs")
    first_program = lines.index("save")
    prelude = lines[1:first_program]
    metadata = [
        line
        for line in prelude
        if line.startswith(("source ", "frameSize ", "sample ", "clearColor "))
    ]
    resources = [line for line in prelude if line.startswith("makeRenderPaint ")]
    unknown = [
        line
        for line in prelude
        if not line.startswith(
            ("source ", "frameSize ", "sample ", "clearColor ", "makeRenderPaint ")
        )
    ]
    if unknown:
        raise SystemExit(f"unknown stream prelude lines: {unknown}")
    kinds = [
        "resource" if line.startswith("makeRenderPaint ") else "metadata"
        for line in prelude
    ]
    # source/frame/sample are harness declarations, not renderer calls. The
    # C++ runner writes them after import-time paint creation while Rust writes
    # them before it. Permit only that whole-category placement difference;
    # order within each category and the complete renderer program stay exact.
    allowed_kinds = [
        ["resource"] * len(resources) + ["metadata"] * len(metadata),
        ["metadata"] * len(metadata) + ["resource"] * len(resources),
    ]
    if kinds not in allowed_kinds:
        raise SystemExit(f"interleaved stream prelude: {kinds}")
    return metadata, resources, lines[first_program:]


def path_parts(line):
    prefix, rest = line.split("points=[", 1)
    points, suffix = rest.split("]}} paint=", 1)
    numbers = [
        float(value)
        for value in re.findall(
            r"-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?", points
        )
    ]
    return prefix, numbers, suffix


def compare_streams(cpp_path, rust_path):
    cpp = pathlib.Path(cpp_path).read_text().splitlines()
    rust = pathlib.Path(rust_path).read_text().splitlines()
    cpp_metadata, cpp_resources, cpp_program = partition(cpp)
    rust_metadata, rust_resources, rust_program = partition(rust)
    if cpp_metadata != rust_metadata:
        raise SystemExit(f"metadata differs: cpp={cpp_metadata} rust={rust_metadata}")
    if cpp_resources != rust_resources:
        raise SystemExit("paint resources differ")
    if len(cpp_program) != len(rust_program):
        raise SystemExit(
            f"program length differs: cpp={len(cpp_program)} rust={len(rust_program)}"
        )

    different_draw_lines = 0
    different_coordinates = 0
    coordinates_compared = 0
    max_coordinate_delta = 0.0
    for index, (cpp_line, rust_line) in enumerate(
        zip(cpp_program, rust_program, strict=True)
    ):
        if cpp_line == rust_line:
            if cpp_line.startswith("drawPath "):
                _, values, _ = path_parts(cpp_line)
                coordinates_compared += len(values)
            continue
        if not cpp_line.startswith("drawPath ") or not rust_line.startswith(
            "drawPath "
        ):
            raise SystemExit(
                f"non-path command differs at {index}: "
                f"{cpp_line[:80]} vs {rust_line[:80]}"
            )
        cpp_prefix, cpp_values, cpp_suffix = path_parts(cpp_line)
        rust_prefix, rust_values, rust_suffix = path_parts(rust_line)
        if cpp_prefix != rust_prefix or cpp_suffix != rust_suffix:
            raise SystemExit(f"path structure differs at {index}")
        if len(cpp_values) != len(rust_values):
            raise SystemExit(f"path coordinate count differs at {index}")
        different_draw_lines += 1
        coordinates_compared += len(cpp_values)
        for cpp_value, rust_value in zip(cpp_values, rust_values, strict=True):
            delta = abs(cpp_value - rust_value)
            if delta:
                different_coordinates += 1
                max_coordinate_delta = max(max_coordinate_delta, delta)

    if max_coordinate_delta > 2e-6:
        raise SystemExit(f"max stream coordinate delta {max_coordinate_delta}")
    if (
        len(cpp_metadata) != 3
        or len(cpp_resources) != 8
        or len(cpp_program) != 38
        or coordinates_compared != 6894
    ):
        raise SystemExit(
            "stream-count drift: "
            f"metadata={len(cpp_metadata)} resources={len(cpp_resources)} "
            f"program={len(cpp_program)} coordinates={coordinates_compared}"
        )
    print(
        json.dumps(
            {
                "result": "same-commands-within-2e-6",
                "metadata_lines": len(cpp_metadata),
                "resource_lines": len(cpp_resources),
                "program_lines": len(cpp_program),
                "different_draw_lines": different_draw_lines,
                "coordinates_compared": coordinates_compared,
                "different_coordinates": different_coordinates,
                "max_coordinate_delta": max_coordinate_delta,
            },
            sort_keys=True,
        )
    )


def main():
    if len(sys.argv) != 4:
        raise SystemExit("usage: compare.py direct|stream CPP RUST")
    if sys.argv[1] == "direct":
        compare_direct(sys.argv[2], sys.argv[3])
    elif sys.argv[1] == "stream":
        compare_streams(sys.argv[2], sys.argv[3])
    else:
        raise SystemExit(f"unknown comparison {sys.argv[1]}")


if __name__ == "__main__":
    main()
