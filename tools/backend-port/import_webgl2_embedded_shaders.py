#!/usr/bin/env python3
"""Import the exact generated GLSL strings compiled by pinned C++ WebGL2."""

from __future__ import annotations

import hashlib
import os
import subprocess
from pathlib import Path


PINNED_UPSTREAM = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
GENERATED_INPUTS = (
    "advanced_blend.minified.glsl",
    "atomic_draw.minified.glsl",
    "bezier_utils.minified.glsl",
    "blit_texture_as_draw.minified.glsl",
    "color_ramp.minified.glsl",
    "common.minified.glsl",
    "constants.minified.glsl",
    "draw_clockwise_clip.minified.frag",
    "draw_clockwise_path.minified.frag",
    "draw_image_mesh.minified.vert",
    "draw_mesh.minified.frag",
    "draw_msaa_object.minified.frag",
    "draw_path.minified.vert",
    "draw_path_common.minified.glsl",
    "draw_raster_order_path.minified.frag",
    "flush_uniforms.minified.glsl",
    "glsl.minified.glsl",
    "pls_load_store_ext.minified.glsl",
    "render_atlas.minified.glsl",
    "resolve_atlas.minified.glsl",
    "stencil_draw.minified.glsl",
    "tessellate.minified.glsl",
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def embedded_header_name(generated_input: str) -> str:
    return generated_input.replace(".minified", "") + ".hpp"


def extract_embedded_source(header: bytes) -> bytes:
    prefix = b'R"===('
    suffix = b')===";'
    start = header.find(prefix)
    if start < 0:
        raise RuntimeError("generated header has no embedded GLSL raw string")
    start += len(prefix)
    end = header.find(suffix, start)
    if end < 0:
        raise RuntimeError("generated header has no embedded GLSL terminator")
    return header[start:end]


def main() -> None:
    repo = Path(__file__).resolve().parents[2]
    upstream = Path(os.environ.get("RIVE_RUNTIME_DIR", "/Users/levi/dev/oss/rive-runtime"))
    revision = subprocess.check_output(
        ["git", "-C", str(upstream), "rev-parse", "HEAD"], text=True
    ).strip()
    if revision != PINNED_UPSTREAM:
        raise RuntimeError(f"expected pinned upstream {PINNED_UPSTREAM}, found {revision}")

    generated = upstream / "renderer/out/cpp-webgl2-oracle/include/generated/shaders"
    destination = (
        repo
        / "crates/nuxie-renderer/src/mechanical_port/webgl2/source/generated_glsl_embedded"
    )
    destination.mkdir(parents=True, exist_ok=True)
    manifest = ["generated_input\theader_sha256\tembedded_sha256\tembedded_bytes"]
    for generated_input in GENERATED_INPUTS:
        header_path = generated / embedded_header_name(generated_input)
        header = header_path.read_bytes()
        embedded = extract_embedded_source(header)
        (destination / generated_input).write_bytes(embedded)
        manifest.append(
            "\t".join(
                (
                    generated_input,
                    sha256(header),
                    sha256(embedded),
                    str(len(embedded)),
                )
            )
        )
    (destination / "MANIFEST.tsv").write_text("\n".join(manifest) + "\n")


if __name__ == "__main__":
    main()
