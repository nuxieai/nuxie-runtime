#!/usr/bin/env python3
"""Cross-language contract tests for the atlas oracle formats."""

import hashlib
import os
import pathlib
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest


HEADER_BYTES = 20
MAGIC = b"RIVEMSK\0"
INPUT_HEADER_BYTES = 40
INPUT_MAGIC = b"RIVEATI\0"
BLIT_MAGIC = b"RIVEABL\0"
ATOMIC_COVERAGE_HEADER_BYTES = 24
ATOMIC_COVERAGE_MAGIC = b"RIVEAPC\0"
ATOMIC_COLOR_HEADER_BYTES = 24
ATOMIC_COLOR_MAGIC = b"RIVEACO\0"
ATOMIC_COLORBURN_PAIR_FRAME_SIZE = 1024
TESS_SPAN_HEADER_BYTES = 28
TESS_SPAN_MAGIC = b"RIVEATS\0"
TESS_SPAN_RECORD_BYTES = 64
DIRECT_INPUT_HEADER_BYTES = 64
DIRECT_GRID_MAGIC = b"RIVEDGI\0"
DIRECT_FLOWER_MAGIC = b"RIVEDFI\0"
DIRECT_BAD_SKIN_MAGIC = b"RIVEDBI\0"
# Production enum and patch-layout values from
# renderer/include/rive/renderer/gpu.hpp. The generated direct-grid artifact
# records this same four-draw schedule.
DIRECT_INTERLOCK_ATOMICS = 1
DRAW_TYPE_OUTER_CURVE_PATCHES = 2
DRAW_TYPE_INTERIOR_TRIANGULATION = 3
DRAW_TYPE_RENDER_PASS_INITIALIZE = 15
DRAW_TYPE_RENDER_PASS_RESOLVE = 16
OUTER_CURVE_PATCH_SEGMENT_SPAN = 17
DIRECT_DRAW_TYPES = (
    DRAW_TYPE_RENDER_PASS_INITIALIZE,
    DRAW_TYPE_OUTER_CURVE_PATCHES,
    DRAW_TYPE_INTERIOR_TRIANGULATION,
    DRAW_TYPE_RENDER_PASS_RESOLVE,
)
ROOT = pathlib.Path(__file__).resolve().parents[2]
EXPORTER = pathlib.Path(__file__).with_name("runtime-src") / "main.cpp"
BUILD_SCRIPT = pathlib.Path(__file__).with_name("build.sh")
README = pathlib.Path(__file__).with_name("README.md")
RUNTIME_PATCH = pathlib.Path(__file__).with_name("runtime.patch")
RUST_RENDERER = ROOT / "crates" / "nuxie-renderer" / "src" / "lib.rs"
STROKES_ROUND_STREAM = ROOT / "fixtures" / "renderer" / "streams" / "gm" / "strokes_round.rive-stream"
RAWTEXT_STREAM = ROOT / "fixtures" / "renderer" / "streams" / "gm" / "rawtext.rive-stream"
POLYSHARK_STREAM = ROOT / "fixtures" / "renderer" / "streams" / "gm" / "feather_polyshapes.rive-stream"
FLOWER_STREAM = ROOT / "fixtures" / "renderer" / "streams" / "gm" / "largeclippedpath_clockwise_nested.rive-stream"
BAD_SKIN_STREAM = ROOT / "fixtures" / "renderer" / "streams" / "riv" / "bad_skin.rive-stream"
INTERLEAVED_FEATHER_STREAM = ROOT / "fixtures" / "renderer" / "streams" / "gm" / "interleavedfeather.rive-stream"
INTERLEAVED_FEATHER_SHA256 = "8868c228229b6708e4e46c947177bfd982c6e7a60ee9b1c3a7da43a7ec0ee17a"
DSTREADSHUFFLE_STREAM = ROOT / "fixtures" / "renderer" / "streams" / "gm" / "dstreadshuffle.rive-stream"
DSTREADSHUFFLE_SHA256 = "0e08ecd19e6a9e1f89f3ae2291181cea3513edf5bbe8cadcd3e1e10a0c33f195"
SPOTIFY_KIDS_APP_ICON_STREAM = ROOT / "fixtures" / "renderer" / "streams" / "riv" / "spotify_kids_app_icon.rive-stream"
SPOTIFY_KIDS_APP_ICON_SHA256 = "1c230de80579ddfc9953541ec3311c981e8f53d94c4d023c5429635186ebbd88"
SPOTIFY_KIDS_APP_ICON_SOURCE_SUFFIX = "tests/unit_tests/assets/spotify_kids_app_icon.riv"
POLYSHARK_GENERATOR = pathlib.Path(__file__).with_name("generate_polyshark_stream_path.py")
RAWTEXT_GENERATOR = pathlib.Path(__file__).with_name("generate_rawtext_stream_path.py")
COLORBURN_PAIR_GENERATOR = pathlib.Path(__file__).with_name(
    "generate_interleaved_colorburn_pair_path.py"
)
PATH_STREAM_GENERATOR = pathlib.Path(__file__).with_name(
    "generate_path_stream_replay.py"
)
RUNTIME = pathlib.Path(os.environ.get("RIVE_RUNTIME_DIR", "/Users/levi/dev/oss/rive-runtime"))

# Exact bytes asserted by AtlasMask::serialize() in commit 10a64ec.
RUST_SERIALIZER_FIXTURE = bytes([
    *b"RIVEMSK\0",
    1, 0, 0, 0,
    2, 0, 0, 0,
    2, 0, 0, 0,
    0, 0, 0, 60, 0, 56, 0, 188,
])


def parse_mask(data: bytes) -> dict:
    if len(data) < HEADER_BYTES:
        raise ValueError("file is shorter than the 20-byte RIVEMSK header")
    if data[:8] != MAGIC:
        raise ValueError("bad RIVEMSK magic")
    version, width, height = struct.unpack_from("<3I", data, 8)
    if version != 1:
        raise ValueError("unsupported RIVEMSK version")
    if width == 0 or height == 0:
        raise ValueError("RIVEMSK dimensions must be nonzero")
    payload_bytes = width * height * 2
    if len(data) != HEADER_BYTES + payload_bytes:
        raise ValueError("file length disagrees with row-packed R16Float payload")
    return {"width": width, "height": height, "payload_bytes": payload_bytes}


def make_mask(width=3, height=2) -> bytes:
    return MAGIC + struct.pack("<3I", 1, width, height) + bytes(range(width * height * 2))


def parse_inputs(data: bytes) -> dict:
    if len(data) < INPUT_HEADER_BYTES:
        raise ValueError("file is shorter than the 40-byte RIVEATI header")
    if data[:8] != INPUT_MAGIC:
        raise ValueError("bad RIVEATI magic")
    fields = struct.unpack_from("<8I", data, 8)
    version, base_patch, patch_count, contour_count, width, height, contour_stride, texel_stride = fields
    if version != 1:
        raise ValueError("unsupported RIVEATI version")
    if not width or not height:
        raise ValueError("RIVEATI dimensions must be nonzero")
    if contour_stride != 16 or texel_stride != 16:
        raise ValueError("RIVEATI stride mismatch")
    expected = INPUT_HEADER_BYTES + contour_count * contour_stride + width * height * texel_stride
    if len(data) != expected:
        raise ValueError("RIVEATI length mismatch")
    return {
        "base_patch": base_patch,
        "patch_count": patch_count,
        "contour_count": contour_count,
        "width": width,
        "height": height,
    }


def make_inputs(base_patch=1, patch_count=2, contour_count=1,
                width=2, height=1) -> bytes:
    header = INPUT_MAGIC + struct.pack(
        "<8I", 1, base_patch, patch_count, contour_count,
        width, height, 16, 16
    )
    return header + bytes(contour_count * 16) + bytes(width * height * 16)


def parse_atomic_coverage(data: bytes) -> dict:
    if len(data) < ATOMIC_COVERAGE_HEADER_BYTES:
        raise ValueError("file is shorter than the 24-byte RIVEAPC header")
    if data[:8] != ATOMIC_COVERAGE_MAGIC:
        raise ValueError("bad RIVEAPC magic")
    version, width, height, word_count = struct.unpack_from("<4I", data, 8)
    if version != 1:
        raise ValueError("unsupported RIVEAPC version")
    if not width or not height:
        raise ValueError("RIVEAPC dimensions must be nonzero")
    if word_count != width * height:
        raise ValueError("RIVEAPC word count must equal width times height")
    expected = ATOMIC_COVERAGE_HEADER_BYTES + word_count * 4
    if len(data) != expected:
        raise ValueError("RIVEAPC length mismatch")
    words = struct.unpack_from(f"<{word_count}I", data, ATOMIC_COVERAGE_HEADER_BYTES)
    return {"width": width, "height": height, "word_count": word_count,
            "words": words}


def make_atomic_coverage(width=2, height=2) -> bytes:
    words = tuple(range(width * height))
    return ATOMIC_COVERAGE_MAGIC + struct.pack(
        "<4I", 1, width, height, len(words)
    ) + struct.pack(f"<{len(words)}I", *words)


def parse_atomic_color(data: bytes) -> dict:
    if len(data) < ATOMIC_COLOR_HEADER_BYTES:
        raise ValueError("file is shorter than the 24-byte RIVEACO header")
    if data[:8] != ATOMIC_COLOR_MAGIC:
        raise ValueError("bad RIVEACO magic")
    version, width, height, word_count = struct.unpack_from("<4I", data, 8)
    if version != 1:
        raise ValueError("unsupported RIVEACO version")
    if not width or not height:
        raise ValueError("RIVEACO dimensions must be nonzero")
    if word_count != width * height:
        raise ValueError("RIVEACO word count must equal width times height")
    expected = ATOMIC_COLOR_HEADER_BYTES + word_count * 4
    if len(data) != expected:
        raise ValueError("RIVEACO length mismatch")
    # The payload preserves the native backing-buffer order, including tile swizzle.
    words = struct.unpack_from(f"<{word_count}I", data, ATOMIC_COLOR_HEADER_BYTES)
    return {"width": width, "height": height, "word_count": word_count,
            "words": words}


def make_atomic_color(width=2, height=2) -> bytes:
    words = tuple(range(width * height))
    return ATOMIC_COLOR_MAGIC + struct.pack(
        "<4I", 1, width, height, len(words)
    ) + struct.pack(f"<{len(words)}I", *words)


def validate_atomic_colorburn_pair(data: bytes) -> dict:
    validated = parse_atomic_color(data)
    if (validated["width"], validated["height"], validated["word_count"]) != (
        ATOMIC_COLORBURN_PAIR_FRAME_SIZE,
        ATOMIC_COLORBURN_PAIR_FRAME_SIZE,
        ATOMIC_COLORBURN_PAIR_FRAME_SIZE ** 2,
    ):
        raise ValueError("atomic-colorburn-pair color must be exactly 1024x1024 u32 words")
    return validated


def validate_atomic_colorburn_pair_coverage(data: bytes) -> dict:
    validated = parse_atomic_coverage(data)
    if (validated["width"], validated["height"], validated["word_count"]) != (
        ATOMIC_COLORBURN_PAIR_FRAME_SIZE,
        ATOMIC_COLORBURN_PAIR_FRAME_SIZE,
        ATOMIC_COLORBURN_PAIR_FRAME_SIZE ** 2,
    ):
        raise ValueError("atomic-colorburn-pair coverage must be exactly 1024x1024 u32 words")
    return validated


def validate_direct_cusp_coverage(data: bytes) -> dict:
    validated = parse_atomic_coverage(data)
    if (validated["width"], validated["height"], validated["word_count"]) != (
        64, 64, 4096
    ):
        raise ValueError("direct-cusp coverage must be exactly 64x64 u32 words")
    return validated


def _parse_direct_inputs(data: bytes, magic: bytes, name: str,
                         expected_contour_count: int) -> dict:
    if len(data) < DIRECT_INPUT_HEADER_BYTES:
        raise ValueError(f"file is shorter than the 64-byte {name} header")
    if data[:8] != magic:
        raise ValueError(f"bad {name} magic")
    fields = struct.unpack_from("<14I", data, 8)
    (version, header_bytes, flags, interlock_mode, draw_batch_count,
     tess_width, tess_height, contour_count, triangle_vertex_count,
     draw_batch_stride, contour_stride, triangle_vertex_stride,
     tess_texel_stride, reserved) = fields
    if version != 1 or header_bytes != DIRECT_INPUT_HEADER_BYTES:
        raise ValueError(f"unsupported {name} header")
    if flags != 1 or interlock_mode != DIRECT_INTERLOCK_ATOMICS:
        raise ValueError(f"{name} requires clockwise-fill atomic facts")
    if reserved != 0:
        raise ValueError(f"{name} reserved header bytes must be zero")
    if not tess_width or not tess_height:
        raise ValueError(f"{name} tessellation dimensions must be nonzero")
    if contour_count != expected_contour_count:
        raise ValueError(
            f"{name} must contain exactly {expected_contour_count} contours"
        )
    if draw_batch_count != len(DIRECT_DRAW_TYPES):
        raise ValueError(f"{name} must contain exactly four draw batches")
    if not triangle_vertex_count or triangle_vertex_count % 3:
        raise ValueError(f"{name} draw or triangle record count is invalid")
    if (draw_batch_stride, contour_stride, triangle_vertex_stride, tess_texel_stride) != (20, 16, 12, 16):
        raise ValueError(f"{name} stride mismatch")
    expected = (DIRECT_INPUT_HEADER_BYTES + draw_batch_count * draw_batch_stride
                + contour_count * contour_stride
                + triangle_vertex_count * triangle_vertex_stride
                + tess_width * tess_height * tess_texel_stride)
    if len(data) != expected:
        raise ValueError(f"{name} length mismatch")
    offset = DIRECT_INPUT_HEADER_BYTES
    batches = [struct.unpack_from("<5I", data, offset + i * draw_batch_stride)
               for i in range(draw_batch_count)]
    if tuple(batch[0] for batch in batches) != DIRECT_DRAW_TYPES:
        raise ValueError(f"{name} draw schedule shape mismatch")
    _, _, _, outer_base_element, outer_element_count = batches[1]
    outer_end_element = outer_base_element + outer_element_count
    if (outer_base_element == 0 or outer_element_count == 0 or
            outer_end_element > 0xFFFFFFFF or
            outer_end_element * OUTER_CURVE_PATCH_SEGMENT_SPAN >
            tess_width * tess_height):
        raise ValueError(f"{name} outer-cubic draw range is invalid")
    _, _, _, _, interior_element_count = batches[2]
    if interior_element_count != triangle_vertex_count:
        raise ValueError(
            f"{name} interior draw count must equal triangle vertex count"
        )
    offset += draw_batch_count * draw_batch_stride
    contours = [struct.unpack_from("<4I", data, offset + i * contour_stride)
                for i in range(contour_count)]
    offset += contour_count * contour_stride
    triangles = [struct.unpack_from("<3I", data, offset + i * triangle_vertex_stride)
                 for i in range(triangle_vertex_count)]
    offset += triangle_vertex_count * triangle_vertex_stride
    return {
        "interlock_mode": interlock_mode,
        "batches": batches,
        "contours": contours,
        "triangles": triangles,
        "tess_width": tess_width,
        "tess_height": tess_height,
        "tessellation": data[offset:],
    }


def parse_direct_grid_inputs(data: bytes) -> dict:
    return _parse_direct_inputs(data, DIRECT_GRID_MAGIC, "RIVEDGI", 100)


def parse_direct_flower_inputs(data: bytes) -> dict:
    return _parse_direct_inputs(data, DIRECT_FLOWER_MAGIC, "RIVEDFI", 2)


def parse_direct_bad_skin_inputs(data: bytes) -> dict:
    return _parse_direct_inputs(data, DIRECT_BAD_SKIN_MAGIC, "RIVEDBI", 1)


def _encode_direct_inputs(parsed: dict, magic: bytes) -> bytes:
    batches = parsed["batches"]
    contours = parsed["contours"]
    triangles = parsed["triangles"]
    header = magic + struct.pack(
        "<14I", 1, DIRECT_INPUT_HEADER_BYTES, 1, parsed["interlock_mode"],
        len(batches), parsed["tess_width"], parsed["tess_height"],
        len(contours), len(triangles), 20, 16, 12, 16, 0,
    )
    return (header + b"".join(struct.pack("<5I", *batch) for batch in batches)
            + b"".join(struct.pack("<4I", *contour) for contour in contours)
            + b"".join(struct.pack("<3I", *triangle) for triangle in triangles)
            + parsed["tessellation"])


def encode_direct_grid_inputs(parsed: dict) -> bytes:
    return _encode_direct_inputs(parsed, DIRECT_GRID_MAGIC)


def encode_direct_flower_inputs(parsed: dict) -> bytes:
    return _encode_direct_inputs(parsed, DIRECT_FLOWER_MAGIC)


def encode_direct_bad_skin_inputs(parsed: dict) -> bytes:
    return _encode_direct_inputs(parsed, DIRECT_BAD_SKIN_MAGIC)


def make_direct_grid_inputs() -> bytes:
    parsed = {
        "interlock_mode": DIRECT_INTERLOCK_ATOMICS,
        "batches": [
            (DRAW_TYPE_RENDER_PASS_INITIALIZE, 0, 1, 0, 1),
            (DRAW_TYPE_OUTER_CURVE_PATCHES, 0x80, 1, 1, 20),
            (DRAW_TYPE_INTERIOR_TRIANGULATION, 0x80, 1, 0, 6),
            (DRAW_TYPE_RENDER_PASS_RESOLVE, 0, 1, 0, 1),
        ],
        "contours": [(0, 0, 1, index) for index in range(100)],
        "triangles": [(0, 0, 0x00010001)] * 6,
        "tess_width": 32,
        "tess_height": 16,
        "tessellation": bytes(32 * 16 * 16),
    }
    return encode_direct_grid_inputs(parsed)


def make_direct_flower_inputs() -> bytes:
    parsed = {
        "interlock_mode": DIRECT_INTERLOCK_ATOMICS,
        "batches": [
            (DRAW_TYPE_RENDER_PASS_INITIALIZE, 0, 1, 0, 1),
            (DRAW_TYPE_OUTER_CURVE_PATCHES, 0x80, 1, 1, 20),
            (DRAW_TYPE_INTERIOR_TRIANGULATION, 0x80, 1, 0, 6),
            (DRAW_TYPE_RENDER_PASS_RESOLVE, 0, 1, 0, 1),
        ],
        "contours": [(0, 0, 1, index) for index in range(2)],
        "triangles": [(0, 0, 0x00010001)] * 6,
        "tess_width": 32,
        "tess_height": 16,
        "tessellation": bytes(32 * 16 * 16),
    }
    return encode_direct_flower_inputs(parsed)


def make_direct_bad_skin_inputs() -> bytes:
    parsed = {
        "interlock_mode": DIRECT_INTERLOCK_ATOMICS,
        "batches": [
            (DRAW_TYPE_RENDER_PASS_INITIALIZE, 0, 1, 0, 1),
            (DRAW_TYPE_OUTER_CURVE_PATCHES, 0x80, 1, 1, 20),
            (DRAW_TYPE_INTERIOR_TRIANGULATION, 0x80, 1, 0, 6),
            (DRAW_TYPE_RENDER_PASS_RESOLVE, 0, 1, 0, 1),
        ],
        "contours": [(0, 0, 1, 0)],
        "triangles": [(0, 0, 0x00010001)] * 6,
        "tess_width": 32,
        "tess_height": 16,
        "tessellation": bytes(32 * 16 * 16),
    }
    return encode_direct_bad_skin_inputs(parsed)


def parse_blit(data: bytes) -> dict:
    if len(data) < HEADER_BYTES:
        raise ValueError("file is shorter than the 20-byte RIVEABL header")
    if data[:8] != BLIT_MAGIC:
        raise ValueError("bad RIVEABL magic")
    version, width, height = struct.unpack_from("<3I", data, 8)
    if version != 1:
        raise ValueError("unsupported RIVEABL version")
    if not width or not height:
        raise ValueError("RIVEABL dimensions must be nonzero")
    if len(data) != HEADER_BYTES + width * height * 4:
        raise ValueError("RIVEABL length mismatch")
    return {"width": width, "height": height}


def validate_direct_strokes_round_inputs(data: bytes) -> dict:
    validated = parse_inputs(data)
    if validated["contour_count"] != 1:
        raise ValueError("direct-strokes-round must contain exactly one contour")
    if (validated["base_patch"], validated["patch_count"]) != (1, 10):
        raise ValueError("direct-strokes-round patch range must be exactly 1+10")
    return validated


def parse_tess_spans(data: bytes) -> dict:
    if len(data) < TESS_SPAN_HEADER_BYTES:
        raise ValueError("file is shorter than the 28-byte RIVEATS header")
    if data[:8] != TESS_SPAN_MAGIC:
        raise ValueError("bad RIVEATS magic")
    version, header_bytes, first_span, span_count, record_bytes = struct.unpack_from(
        "<5I", data, 8
    )
    if version != 1 or header_bytes != TESS_SPAN_HEADER_BYTES:
        raise ValueError("unsupported RIVEATS header")
    if record_bytes != TESS_SPAN_RECORD_BYTES:
        raise ValueError("RIVEATS record stride mismatch")
    if not span_count:
        raise ValueError("RIVEATS must contain at least one span")
    expected = TESS_SPAN_HEADER_BYTES + span_count * record_bytes
    if len(data) != expected:
        raise ValueError("RIVEATS length mismatch")
    records = [
        struct.unpack_from("<16I", data, TESS_SPAN_HEADER_BYTES + i * record_bytes)
        for i in range(span_count)
    ]
    return {"first_span": first_span, "span_count": span_count, "records": records}


def make_tess_spans(first_span=0, span_count=1) -> bytes:
    header = TESS_SPAN_MAGIC + struct.pack(
        "<5I", 1, TESS_SPAN_HEADER_BYTES, first_span, span_count,
        TESS_SPAN_RECORD_BYTES
    )
    return header + bytes(span_count * TESS_SPAN_RECORD_BYTES)


def validate_direct_strokes_round_spans(data: bytes) -> dict:
    validated = parse_tess_spans(data)
    if (validated["first_span"], validated["span_count"]) != (0, 11):
        raise ValueError(
            "direct-strokes-round span range must be exactly 0+11"
        )
    return validated


def validate_direct_rawtext_inputs(data: bytes) -> dict:
    validated = parse_inputs(data)
    actual = tuple(validated[field] for field in (
        "base_patch", "patch_count", "contour_count", "width", "height"
    ))
    if actual != (1, 318, 36, 2048, 2):
        raise ValueError(
            "direct-rawtext must be exactly patches=1+318, contours=36, "
            "tessellation=2048x2"
        )
    return validated


def validate_direct_rawtext_spans(data: bytes) -> dict:
    validated = parse_tess_spans(data)
    if (validated["first_span"], validated["span_count"]) != (0, 438):
        raise ValueError("direct-rawtext span range must be exactly 0+438")
    return validated


class FormatTests(unittest.TestCase):
    def test_accepts_rust_serializer_fixture_byte_for_byte(self):
        self.assertEqual(parse_mask(RUST_SERIALIZER_FIXTURE), {
            "width": 2,
            "height": 2,
            "payload_bytes": 8,
        })
        self.assertEqual(RUST_SERIALIZER_FIXTURE[:HEADER_BYTES],
                         MAGIC + struct.pack("<3I", 1, 2, 2))

    def test_accepts_canonical_row_packed_r16float(self):
        self.assertEqual(parse_mask(make_mask()), {"width": 3, "height": 2, "payload_bytes": 12})

    def test_rejects_extended_or_padded_payload(self):
        with self.assertRaisesRegex(ValueError, "length"):
            parse_mask(make_mask() + b"\0" * 256)

    def test_rejects_bad_magic_and_dimensions(self):
        data = bytearray(make_mask())
        data[0] = 0
        with self.assertRaisesRegex(ValueError, "magic"):
            parse_mask(data)
        with self.assertRaisesRegex(ValueError, "nonzero"):
            parse_mask(MAGIC + struct.pack("<3I", 1, 0, 2))

    def test_accepts_and_rejects_canonical_atlas_inputs(self):
        self.assertEqual(parse_inputs(make_inputs()), {
            "base_patch": 1,
            "patch_count": 2,
            "contour_count": 1,
            "width": 2,
            "height": 1,
        })
        with self.assertRaisesRegex(ValueError, "length"):
            parse_inputs(make_inputs() + b"\0")
        bad_stride = bytearray(make_inputs())
        struct.pack_into("<I", bad_stride, 32, 12)
        with self.assertRaisesRegex(ValueError, "stride"):
            parse_inputs(bad_stride)

    def test_direct_cusp_atomic_coverage_is_little_endian_and_strict(self):
        coverage = make_atomic_coverage()
        self.assertEqual(parse_atomic_coverage(coverage), {
            "width": 2,
            "height": 2,
            "word_count": 4,
            "words": (0, 1, 2, 3),
        })
        self.assertEqual(coverage[:ATOMIC_COVERAGE_HEADER_BYTES],
                         ATOMIC_COVERAGE_MAGIC + struct.pack("<4I", 1, 2, 2, 4))
        bad_word_count = bytearray(coverage)
        struct.pack_into("<I", bad_word_count, 20, 3)
        with self.assertRaisesRegex(ValueError, "word count"):
            parse_atomic_coverage(bad_word_count)
        with self.assertRaisesRegex(ValueError, "length"):
            parse_atomic_coverage(coverage + b"\0")
        self.assertEqual(
            validate_direct_cusp_coverage(make_atomic_coverage(64, 64))["word_count"],
            4096,
        )
        with self.assertRaisesRegex(ValueError, "exactly 64x64"):
            validate_direct_cusp_coverage(make_atomic_coverage(63, 64))

    def test_direct_cusp_atomic_coverage_cli_validates_contract(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            artifact_path = pathlib.Path(temp_dir) / "coverage.bin"
            artifact_path.write_bytes(make_atomic_coverage(64, 64))
            result = subprocess.run(
                [sys.executable, __file__, "--validate-direct-cusp-coverage",
                 str(artifact_path)],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("RIVEAPC valid: words=4096 coverage=64x64", result.stdout)

            artifact_path.write_bytes(make_atomic_coverage(63, 64))
            invalid = subprocess.run(
                [sys.executable, __file__, "--validate-direct-cusp-coverage",
                 str(artifact_path)],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(invalid.returncode, 0)
            self.assertIn("direct-cusp-coverage artifact validation failed", invalid.stderr)

    def test_atomic_colorburn_pair_color_is_native_order_and_strict(self):
        color = make_atomic_color()
        self.assertEqual(parse_atomic_color(color), {
            "width": 2,
            "height": 2,
            "word_count": 4,
            "words": (0, 1, 2, 3),
        })
        self.assertEqual(color[:ATOMIC_COLOR_HEADER_BYTES],
                         ATOMIC_COLOR_MAGIC + struct.pack("<4I", 1, 2, 2, 4))
        bad_word_count = bytearray(color)
        struct.pack_into("<I", bad_word_count, 20, 3)
        with self.assertRaisesRegex(ValueError, "word count"):
            parse_atomic_color(bad_word_count)
        with self.assertRaisesRegex(ValueError, "length"):
            parse_atomic_color(color + b"\0")

    def test_atomic_colorburn_pair_color_cli_validates_contract(self):
        width = ATOMIC_COLORBURN_PAIR_FRAME_SIZE
        artifact = ATOMIC_COLOR_MAGIC + struct.pack("<4I", 1, width, width, width * width)
        artifact += bytes(width * width * 4)
        with tempfile.TemporaryDirectory() as temp_dir:
            artifact_path = pathlib.Path(temp_dir) / "color.bin"
            artifact_path.write_bytes(artifact)
            result = subprocess.run(
                [sys.executable, __file__, "--validate-atomic-colorburn-pair",
                 str(artifact_path)],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("RIVEACO valid: words=1048576 color=1024x1024", result.stdout)

            invalid_path = pathlib.Path(temp_dir) / "invalid-color.bin"
            invalid_path.write_bytes(make_atomic_color(2, 2))
            invalid = subprocess.run(
                [sys.executable, __file__, "--validate-atomic-colorburn-pair",
                 str(invalid_path)],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(invalid.returncode, 0)
            self.assertIn("atomic-colorburn-pair artifact validation failed", invalid.stderr)

    def test_atomic_colorburn_pair_coverage_contract(self):
        width = ATOMIC_COLORBURN_PAIR_FRAME_SIZE
        artifact = ATOMIC_COVERAGE_MAGIC + struct.pack(
            "<4I", 1, width, width, width * width
        ) + bytes(width * width * 4)
        validated = validate_atomic_colorburn_pair_coverage(artifact)
        self.assertEqual(validated["word_count"], width * width)
        with self.assertRaisesRegex(ValueError, "exactly 1024x1024"):
            validate_atomic_colorburn_pair_coverage(make_atomic_coverage())

    def test_direct_strokes_round_inputs_require_one_contour_and_patches(self):
        self.assertEqual(validate_direct_strokes_round_inputs(make_inputs(patch_count=10)), {
            "base_patch": 1,
            "patch_count": 10,
            "contour_count": 1,
            "width": 2,
            "height": 1,
        })
        wrong_range = make_inputs(patch_count=9)
        with self.assertRaisesRegex(ValueError, r"exactly 1\+10"):
            validate_direct_strokes_round_inputs(wrong_range)
        two_contours = make_inputs(patch_count=10, contour_count=2)
        with self.assertRaisesRegex(ValueError, "exactly one contour"):
            validate_direct_strokes_round_inputs(two_contours)

    def test_direct_rawtext_contract_pins_full_preparation_shape(self):
        inputs = make_inputs(
            patch_count=318, contour_count=36, width=2048, height=2
        )
        self.assertEqual(validate_direct_rawtext_inputs(inputs)["patch_count"], 318)
        with self.assertRaisesRegex(ValueError, r"patches=1\+318"):
            validate_direct_rawtext_inputs(
                make_inputs(
                    patch_count=317, contour_count=36, width=2048, height=2
                )
            )
        self.assertEqual(
            validate_direct_rawtext_spans(
                make_tess_spans(first_span=0, span_count=438)
            )["span_count"],
            438,
        )
        with self.assertRaisesRegex(ValueError, r"exactly 0\+438"):
            validate_direct_rawtext_spans(
                make_tess_spans(first_span=0, span_count=437)
            )

    def test_direct_strokes_round_cli_validates_arguments_and_contract(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            artifact_path = pathlib.Path(temp_dir) / "inputs.bin"
            artifact_path.write_bytes(make_inputs(patch_count=10))
            result = subprocess.run(
                [sys.executable, __file__, "--validate-direct-strokes-round",
                 str(artifact_path)],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("RIVEATI valid: patchCount=10 contours=1", result.stdout)

            missing_path = subprocess.run(
                [sys.executable, __file__, "--validate-direct-strokes-round"],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(missing_path.returncode, 0)
            self.assertIn("usage: format_test.py --validate-direct-strokes-round PATH",
                          missing_path.stderr)

            artifact_path.write_bytes(make_inputs(patch_count=9))
            invalid = subprocess.run(
                [sys.executable, __file__, "--validate-direct-strokes-round",
                 str(artifact_path)],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(invalid.returncode, 0)
            self.assertIn("direct-strokes-round artifact validation failed: "
                          "direct-strokes-round patch range must be exactly 1+10",
                          invalid.stderr)

    def test_tess_span_format_pins_header_stride_and_exact_length(self):
        self.assertEqual(parse_tess_spans(make_tess_spans(first_span=4)), {
            "first_span": 4,
            "span_count": 1,
            "records": [(0,) * 16],
        })
        wrong_stride = bytearray(make_tess_spans())
        struct.pack_into("<I", wrong_stride, 24, 60)
        with self.assertRaisesRegex(ValueError, "stride"):
            parse_tess_spans(wrong_stride)
        with self.assertRaisesRegex(ValueError, "length"):
            parse_tess_spans(make_tess_spans() + b"\0")
        self.assertEqual(
            validate_direct_strokes_round_spans(
                make_tess_spans(first_span=0, span_count=11)
            )["span_count"],
            11,
        )
        with self.assertRaisesRegex(ValueError, r"exactly 0\+11"):
            validate_direct_strokes_round_spans(
                make_tess_spans(first_span=1, span_count=11)
            )

    def test_strokes_round_oracle_literal_matches_stream_draw_38(self):
        stream_draw = STROKES_ROUND_STREAM.read_text().splitlines()[123]
        self.assertIn(
            "points=[(25.5016327,70.300293),(67.7646637,70.300293),"
            "(79.4274673,70.300293),(88.8961792,80.9101868),"
            "(88.8961792,89.5240784)",
            stream_draw,
        )
        self.assertIn(
            "paint={id=1,style=stroke,color=0x7a52bdb0,thickness=4.5,"
            "join=0,cap=0,feather=0",
            stream_draw,
        )
        exporter = EXPORTER.read_text()
        for fragment in (
            "path->moveTo(25.5016327f, 70.300293f);",
            "path->lineTo(67.7646637f, 70.300293f);",
            "79.4274673f,",
            "88.8961792f,",
            "4.37011719f,",
            "16.0329189f,",
            "paint->thickness(directStrokesRoundCase ? kDirectStrokesRoundThickness",
            "? 0x7a52bdb0",
        ):
            self.assertIn(fragment, exporter)

    def test_direct_grid_format_round_trips_and_rejects_malformed_counts(self):
        data = make_direct_grid_inputs()
        parsed = parse_direct_grid_inputs(data)
        self.assertEqual(parsed["interlock_mode"], DIRECT_INTERLOCK_ATOMICS)
        self.assertEqual(len(parsed["batches"]), 4)
        self.assertEqual(len(parsed["contours"]), 100)
        self.assertEqual(len(parsed["triangles"]), 6)
        self.assertEqual(parsed["tessellation"], bytes(32 * 16 * 16))
        self.assertEqual(encode_direct_grid_inputs(parsed), data)

        bad_header = bytearray(data)
        struct.pack_into("<I", bad_header, 12, 60)
        with self.assertRaisesRegex(ValueError, "header"):
            parse_direct_grid_inputs(bad_header)
        bad_interlock = bytearray(data)
        struct.pack_into("<I", bad_interlock, 20, 3)
        with self.assertRaisesRegex(ValueError, "atomic"):
            parse_direct_grid_inputs(bad_interlock)
        bad_batch_count = bytearray(data)
        struct.pack_into("<I", bad_batch_count, 24, 3)
        with self.assertRaisesRegex(ValueError, "four draw batches"):
            parse_direct_grid_inputs(bad_batch_count)
        bad_contours = bytearray(data)
        struct.pack_into("<I", bad_contours, 36, 99)
        with self.assertRaisesRegex(ValueError, "100 contours"):
            parse_direct_grid_inputs(bad_contours)
        bad_triangles = bytearray(data)
        struct.pack_into("<I", bad_triangles, 40, 5)
        with self.assertRaisesRegex(ValueError, "triangle"):
            parse_direct_grid_inputs(bad_triangles)
        with self.assertRaisesRegex(ValueError, "length"):
            parse_direct_grid_inputs(data[:-1])

        for batch_index, draw_type in (
            (0, DRAW_TYPE_OUTER_CURVE_PATCHES),
            (1, DRAW_TYPE_INTERIOR_TRIANGULATION),
            (3, DRAW_TYPE_RENDER_PASS_INITIALIZE),
        ):
            with self.subTest(batch_index=batch_index, draw_type=draw_type):
                bad_schedule = bytearray(data)
                struct.pack_into("<I", bad_schedule,
                                 DIRECT_INPUT_HEADER_BYTES + batch_index * 20,
                                 draw_type)
                with self.assertRaisesRegex(ValueError, "schedule shape"):
                    parse_direct_grid_inputs(bad_schedule)

        for outer_base, outer_count in (
            (0, 20),
            (1, 0),
            (31, 1),
            (0xFFFFFFFF, 20),
        ):
            with self.subTest(outer_base=outer_base, outer_count=outer_count):
                bad_outer_range = bytearray(data)
                outer_offset = DIRECT_INPUT_HEADER_BYTES + 20
                struct.pack_into("<2I", bad_outer_range, outer_offset + 12,
                                 outer_base, outer_count)
                with self.assertRaisesRegex(ValueError, "outer-cubic.*range"):
                    parse_direct_grid_inputs(bad_outer_range)

        bad_interior_count = bytearray(data)
        interior_offset = DIRECT_INPUT_HEADER_BYTES + 2 * 20
        struct.pack_into("<I", bad_interior_count, interior_offset + 16, 3)
        with self.assertRaisesRegex(ValueError, "interior draw count"):
            parse_direct_grid_inputs(bad_interior_count)

    def test_direct_flower_format_round_trips_and_rejects_malformed_facts(self):
        data = make_direct_flower_inputs()
        parsed = parse_direct_flower_inputs(data)
        self.assertEqual(parsed["interlock_mode"], DIRECT_INTERLOCK_ATOMICS)
        self.assertEqual(len(parsed["batches"]), 4)
        self.assertEqual(len(parsed["contours"]), 2)
        self.assertEqual(len(parsed["triangles"]), 6)
        self.assertEqual(encode_direct_flower_inputs(parsed), data)

        bad_magic = bytearray(data)
        bad_magic[6] = ord("G")
        with self.assertRaisesRegex(ValueError, "RIVEDFI magic"):
            parse_direct_flower_inputs(bad_magic)
        bad_interlock = bytearray(data)
        struct.pack_into("<I", bad_interlock, 20, 3)
        with self.assertRaisesRegex(ValueError, "atomic"):
            parse_direct_flower_inputs(bad_interlock)
        bad_contours = bytearray(data)
        struct.pack_into("<I", bad_contours, 36, 100)
        with self.assertRaisesRegex(ValueError, "2 contours"):
            parse_direct_flower_inputs(bad_contours)
        bad_schedule = bytearray(data)
        struct.pack_into("<I", bad_schedule, DIRECT_INPUT_HEADER_BYTES + 20,
                         DRAW_TYPE_INTERIOR_TRIANGULATION)
        with self.assertRaisesRegex(ValueError, "schedule shape"):
            parse_direct_flower_inputs(bad_schedule)
        bad_outer_range = bytearray(data)
        struct.pack_into("<I", bad_outer_range,
                         DIRECT_INPUT_HEADER_BYTES + 20 + 12, 0)
        with self.assertRaisesRegex(ValueError, "outer-cubic.*range"):
            parse_direct_flower_inputs(bad_outer_range)
        bad_interior_count = bytearray(data)
        struct.pack_into("<I", bad_interior_count,
                         DIRECT_INPUT_HEADER_BYTES + 2 * 20 + 16, 3)
        with self.assertRaisesRegex(ValueError, "interior draw count"):
            parse_direct_flower_inputs(bad_interior_count)

    def test_direct_bad_skin_format_round_trips_and_rejects_malformed_facts(self):
        data = make_direct_bad_skin_inputs()
        parsed = parse_direct_bad_skin_inputs(data)
        self.assertEqual(parsed["interlock_mode"], DIRECT_INTERLOCK_ATOMICS)
        self.assertEqual(len(parsed["batches"]), 4)
        self.assertEqual(len(parsed["contours"]), 1)
        self.assertEqual(len(parsed["triangles"]), 6)
        self.assertEqual(encode_direct_bad_skin_inputs(parsed), data)

        bad_magic = bytearray(data)
        bad_magic[6] = ord("G")
        with self.assertRaisesRegex(ValueError, "RIVEDBI magic"):
            parse_direct_bad_skin_inputs(bad_magic)
        bad_header = bytearray(data)
        struct.pack_into("<I", bad_header, 12, 60)
        with self.assertRaisesRegex(ValueError, "header"):
            parse_direct_bad_skin_inputs(bad_header)
        bad_contours = bytearray(data)
        struct.pack_into("<I", bad_contours, 36, 2)
        with self.assertRaisesRegex(ValueError, "1 contours"):
            parse_direct_bad_skin_inputs(bad_contours)
        bad_schedule = bytearray(data)
        struct.pack_into("<I", bad_schedule, DIRECT_INPUT_HEADER_BYTES + 20,
                         DRAW_TYPE_INTERIOR_TRIANGULATION)
        with self.assertRaisesRegex(ValueError, "schedule shape"):
            parse_direct_bad_skin_inputs(bad_schedule)
        bad_interior_count = bytearray(data)
        struct.pack_into("<I", bad_interior_count,
                         DIRECT_INPUT_HEADER_BYTES + 2 * 20 + 16, 3)
        with self.assertRaisesRegex(ValueError, "interior draw count"):
            parse_direct_bad_skin_inputs(bad_interior_count)

    def test_accepts_and_rejects_canonical_atlas_blit(self):
        data = BLIT_MAGIC + struct.pack("<3I", 1, 2, 1) + bytes(8)
        self.assertEqual(parse_blit(data), {"width": 2, "height": 1})
        with self.assertRaisesRegex(ValueError, "length"):
            parse_blit(data + b"\0")

    def test_exporter_configuration_matches_coordinated_fixtures(self):
        source = EXPORTER.read_text()
        for fragment in (
            "constexpr uint32_t kFrameWidth = 64;",
            "constexpr uint32_t kFrameHeight = 64;",
            "constexpr uint32_t kExpectedLogicalAtlasSize = 39;",
            "constexpr uint32_t kExpectedPhysicalAtlasSize = 48;",
            "constexpr uint32_t kMaskHeaderBytes = 20;",
            "constexpr uint32_t kInputsHeaderBytes = 40;",
            "constexpr uint32_t kDirectGridHeaderBytes = 64;",
            "constexpr uint32_t kBlitHeaderBytes = 20;",
            "constexpr uint32_t kExpectedPolySharkPatchCount = 786;",
            "constexpr uint32_t kExpectedPolySharkContourCount = 1;",
            "constexpr uint32_t kExpectedPolySharkTessHeight = 5;",
            "constexpr uint32_t kDirectGridFrameSize = 1000;",
            "constexpr uint32_t kDirectGridContourCount = 100;",
            "constexpr uint32_t kDirectFlowerContourCount = 2;",
            "constexpr uint32_t kDirectBadSkinFrameWidth = 999;",
            "constexpr uint32_t kDirectBadSkinFrameHeight = 720;",
            "constexpr uint32_t kDirectBadSkinContourCount = 1;",
            "constexpr uint32_t kAtomicInterleavedFeatherFullFrameSize = 1000;",
            "constexpr uint32_t kAtomicDstReadShuffleFullFrameWidth = 530;",
            "constexpr uint32_t kAtomicDstReadShuffleFullFrameHeight = 690;",
            '#include "generated_interleavedfeather_full.inc"',
            '#include "generated_dstreadshuffle_full.inc"',
            '#include "generated_dstreadshuffle_srcover_control.inc"',
            'std::strcmp(argv[4], "atomic-interleavedfeather-full") == 0;',
            'std::strcmp(argv[4], "atomic-dstreadshuffle-full") == 0;',
            'std::strcmp(argv[4], "atomic-dstreadshuffle-srcover-full") == 0;',
            "const bool fullStreamCase =",
            "atomicDstReadShuffleSrcOverCase",
            "atomicDstReadShuffleCase =",
            "adapterOptions.backendType = WGPUBackendType_Metal;",
            "fullStreamCase\n                                         ? &adapterOptions",
            "writeAdapterProvenance(auxiliaryOutput, adapter.Get());",
            "info.backendType != WGPUBackendType_Metal",
            'file << "backend=metal\\n";',
            "replayInterleavedFeatherFull(&renderer, context.get());",
            "replayDstReadShuffleFull(&renderer, context.get());",
            "replayDstReadShuffleSrcOverControl(&renderer, context.get());",
            "facts.drawBatches.size() < 3",
            "if (!fullStreamCase)",
            "const auto& facts = webgpuContext->atlasMaskFactsForOracle();",
            'std::printf("draw schedule: interlock=%u fixedFunctionColorOutput=%d batches=%zu',
            "facts.contentWidth != kExpectedLogicalAtlasSize",
            "facts.contentHeight != kExpectedLogicalAtlasSize",
            "facts.pathTranslateX != kAtlasPadding",
            "facts.pathTranslateY != kAtlasPadding",
            "facts.strokeBatchCount != 1",
            "facts.strokeScissor.right != kExpectedLogicalAtlasSize",
            "facts.strokeScissor.bottom != kExpectedLogicalAtlasSize",
            "path->moveTo(kSquareMin, kSquareMin);",
            "path->lineTo(kSquareMax, kSquareMin);",
            "path->lineTo(kSquareMax, kSquareMax);",
            "path->lineTo(kSquareMin, kSquareMax);",
            "path->close();",
            'const bool circleCase = argc > 4 && std::strcmp(argv[4], "fill") == 0;',
            'const bool cuspCase = argc > 4 && std::strcmp(argv[4], "cusp") == 0;',
            'argc > 4 && std::strcmp(argv[4], "direct-polyshark") == 0;',
            'argc > 4 && std::strcmp(argv[4], "direct-grid") == 0;',
            'argc > 4 && std::strcmp(argv[4], "direct-flower") == 0;',
            'argc > 4 && std::strcmp(argv[4], "direct-bad-skin") == 0;',
            'argc > 4 && std::strcmp(argv[4], "nested-evenodd-path-clipped") == 0;',
            'argc > 4 && std::strcmp(argv[4], "nested-clockwise-path-clipped") == 0;',
            'argc > 4 && std::strcmp(argv[4], "advanced-blend") == 0;',
            'argc > 4 && std::strcmp(argv[4], "atomic-advanced-blend") == 0;',
            'argc > 4 && std::strcmp(argv[4], "msaa-intersection-groups") == 0;',
            "directGridCase || directFlowerCase || directBadSkinCase;",
            "nestedEvenOddPathClippedCase ||",
            "nestedClockwisePathClippedCase;",
            "constexpr uint32_t kIntersectionGroupBatchCount = 9;",
            "void addIntersectionGroupRect(rive::RenderPath* path,",
            "draw0Path->fillRule(rive::FillRule::clockwise);",
            "draw1Path->fillRule(rive::FillRule::nonZero);",
            "draw2Path->fillRule(rive::FillRule::clockwise);",
            "draw0Paint->color(0xffffffff);",
            "draw1Paint->color(0x80ffffff);",
            "draw2Paint->color(0x80ffffff);",
            "facts.drawBatches.size() == kIntersectionGroupBatchCount;",
            "expectedTypes = {",
            "expectedContents = {",
            "expectedBases = {",
            "batch.elementCount == 1 &&",
            "Draws 1 and 2 are both positive-key three-pass fills.",
            "group-3 type 10 sorts ahead of draw 1's lower type 8",
            "draw 1 above all three layers reserved by draw 0.",
            "msaa intersection-group oracle must schedule tagged draws 0 and 2 ahead of overlapping draw 1",
            "This mode is schedule-only.",
            "path->fillRule(rive::FillRule::clockwise);",
            "path->cubicTo(51.2f, 16, 12.8f, 16, 48, 48);",
            "path->cubicTo(133.635864f, 0, -33.6358566f, 0, 100, 100);",
            '#include "generated_polyshark_path.inc"',
            "rive::Mat2D(kPolySharkStreamScale,",
            "addFeatherPolyShapesShark(path.get());",
            "void addClockwiseNestedGrid(rive::RenderPath* path)",
            "largeclippedpath_clockwise_nested.rive-stream:10",
            "addClockwiseNestedGrid(path.get());",
            "void addClockwiseNestedFlower(rive::RenderPath* path)",
            "largeclippedpath_clockwise_nested.rive-stream:7",
            "addClockwiseNestedFlower(path.get());",
            "paint->style(fillCase ? rive::RenderPaintStyle::fill",
            ": rive::RenderPaintStyle::stroke);",
            "path->cubicTo(kSquareMax,",
            "paint->thickness(directStrokesRoundCase ? kDirectStrokesRoundThickness",
            "paint->join(rive::StrokeJoin::miter);",
            "paint->cap(rive::StrokeCap::butt);",
            "paint->feather(directTriangulatedCase || directStrokesRoundCase ||",
            ".msaaSampleCount = directOutputCase ? 0u : 4u",
            "void onMap(WGPUMapAsyncStatus status,",
            "status == WGPUMapAsyncStatus_Success",
            "context->static_impl_cast<rive::gpu::RenderContextWebGPUImpl>();",
            "webgpuContext->makeRenderTarget(",
            "webgpuContext->atlasMaskTextureForOracle();",
            "webgpuContext->tessellationTextureForOracle();",
            "writeInputs(inputsOutput,",
            "writeDirectGridInputs(inputsOutput,",
            "writeDirectFlowerInputs(inputsOutput,",
            "writeDirectBadSkinInputs(inputsOutput,",
            "constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'D', 'G', 'I', '\\0'};",
            "constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'D', 'F', 'I', '\\0'};",
            "constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'D', 'B', 'I', '\\0'};",
            "writeBlit(blitOutput,",
            "readTexture(instance, device, queue, targetTexture, 4);",
            "atlasWidth = atlas.GetWidth();",
            "atlasHeight = atlas.GetHeight();",
            "if (atlasWidth != expectedAtlasWidth ||",
            "readTexture(instance, device, queue, atlas, kBytesPerTexel);",
            "readTexture(instance, device, queue, tessellation, kTessBytesPerTexel);",
            "writeU32(header, 8, 1);",
            "writeU32(header, 12, width);",
            "writeU32(header, 16, height);",
        ):
            self.assertIn(fragment, source)
        self.assertNotIn("kIntersectionGroupDrawCount", source)

        full_stream_case = source[
            source.index("const bool fullStreamCase ="):
            source.index("const bool intersectionGroupsCase =")
        ]
        self.assertIn("atomicDstReadShuffleCase", full_stream_case)

        frame_selection = source[
            source.index("const uint32_t frameWidth"):
            source.index("targetDesc.size", source.index("const uint32_t frameWidth"))
        ]
        self.assertIn("atomicDstReadShuffleCase", frame_selection)
        self.assertIn("kAtomicDstReadShuffleFullFrameWidth", frame_selection)
        self.assertIn("kAtomicDstReadShuffleFullFrameHeight", frame_selection)

        begin_frame = source[
            source.index("context->beginFrame("):
            source.index("path->moveTo", source.index("context->beginFrame("))
        ]
        self.assertIn("atomicDstReadShuffleCase", begin_frame)
        self.assertIn("0xffffffff", begin_frame)

        full_stream_schedule = source[
            source.index("else if (fullStreamCase)"):
            source.index("else if (atomicAdvancedBlendCase)")
        ]
        self.assertIn(
            "const bool expectedFixedFunctionColorOutput =",
            full_stream_schedule,
        )
        self.assertIn(
            "atomicDstReadShuffleSrcOverCase || atomicSpotifyKidsAppIconFullCase;",
            full_stream_schedule,
        )
        self.assertIn(
            "facts.fixedFunctionColorOutput != expectedFixedFunctionColorOutput",
            full_stream_schedule,
        )
        self.assertIn(
            "full path-stream oracle must execute the expected atomic color-output mode, "
            "initialize, draws, and resolve",
            full_stream_schedule,
        )

        assertion_start = source.index(
            "    if (intersectionGroupsCase)\n"
            "    {\n"
            "        const uint32_t draw0Contents"
        )
        assertion_end = source.index(
            "    else if (advancedBlendCase)", assertion_start
        )
        assertion = source[assertion_start:assertion_end]
        type_block = assertion[
            assertion.index("expectedTypes = {"):
            assertion.index("expectedContents = {")
        ]
        self.assertEqual(
            re.findall(r"DrawType::(\w+)", type_block),
            [
                "msaaMidpointFanBorrowedCoverage",
                "msaaMidpointFans",
                "msaaMidpointFanStencilReset",
                "msaaMidpointFanBorrowedCoverage",
                "msaaMidpointFans",
                "msaaMidpointFanStencilReset",
                "msaaMidpointFanBorrowedCoverage",
                "msaaMidpointFans",
                "msaaMidpointFanStencilReset",
            ],
        )
        contents_block = assertion[
            assertion.index("expectedContents = {"):
            assertion.index("expectedBases = {")
        ]
        self.assertEqual(
            re.findall(r"draw[012]Contents", contents_block),
            [
                "draw0Contents",
                "draw0Contents",
                "draw0Contents",
                "draw2Contents",
                "draw2Contents",
                "draw2Contents",
                "draw1Contents",
                "draw1Contents",
                "draw1Contents",
            ],
        )
        bases_block = assertion[
            assertion.index("expectedBases = {"):
            assertion.index("expectedFeatures =")
        ]
        self.assertIn("1, 1, 1, 2, 2, 2, 3, 3, 3", bases_block)
        self.assertNotIn("WGPUBufferMapAsyncStatus", source)
        self.assertNotIn("context->makeRenderTarget(", source)
        self.assertNotIn("context->atlasMaskTextureForOracle()", source)
        self.assertNotIn("copyWidth", source)
        self.assertNotIn("copyHeight", source)

    def test_direct_polyshark_provenance_matches_stream_line_28(self):
        stream_lines = POLYSHARK_STREAM.read_text().splitlines()
        self.assertEqual(
            stream_lines[13],
            "transform matrix=[1.46300006,0,0,1.46300006,0,0]",
        )
        stream_line = stream_lines[27]
        self.assertIn("drawPath path={id=4,fillRule=2", stream_line)
        self.assertIn("points=[(1.35999978,118.419998)", stream_line)
        self.assertIn("feather=1", stream_line)

        source = EXPORTER.read_text()
        for fragment in (
            "rive::Mat2D(kPolySharkStreamScale,",
            '#include "generated_polyshark_path.inc"',
        ):
            self.assertIn(fragment, source)
        self.assertNotIn("wangs_formula", source)

        with tempfile.TemporaryDirectory() as temp_dir:
            first = pathlib.Path(temp_dir) / "first.inc"
            second = pathlib.Path(temp_dir) / "second.inc"
            command = ["python3", str(POLYSHARK_GENERATOR), "--stream",
                       str(POLYSHARK_STREAM), "--output"]
            subprocess.run(command + [str(first)], check=True)
            subprocess.run(command + [str(second)], check=True)
            self.assertEqual(first.read_bytes(), second.read_bytes())
            generated = first.read_text().splitlines()
            self.assertEqual(len(generated), 321)
            self.assertEqual(
                generated[2], "constexpr float kPolySharkStreamScale = 1.46300006f;"
            )
            self.assertEqual(
                generated[5], "    path->moveTo(1.35999978f, 118.419998f);"
            )
            self.assertEqual(
                generated[-2], "    path->lineTo(1.35999978f, 118.419998f);"
            )

            changed_stream = pathlib.Path(temp_dir) / "changed.rive-stream"
            changed_output = pathlib.Path(temp_dir) / "changed.inc"
            changed_lines = stream_lines.copy()
            changed_lines[13] = "transform matrix=[2,0,0,2,0,0]"
            changed_stream.write_text("\n".join(changed_lines) + "\n")
            subprocess.run(
                [
                    "python3",
                    str(POLYSHARK_GENERATOR),
                    "--stream",
                    str(changed_stream),
                    "--output",
                    str(changed_output),
                ],
                check=True,
            )
            self.assertIn(
                "constexpr float kPolySharkStreamScale = 2.f;",
                changed_output.read_text(),
            )

    def test_direct_rawtext_provenance_matches_stream_line_9(self):
        stream_lines = RAWTEXT_STREAM.read_text().splitlines()
        self.assertTrue(stream_lines[8].startswith(
            "drawPath path={id=1,fillRule=2,path={verbs=[move,line"
        ))
        self.assertIn("points=[(5.9765625,66.796875)", stream_lines[8])
        self.assertTrue(stream_lines[8].endswith(
            "paint={id=1,style=fill,color=0xff000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}"
        ))

        source = EXPORTER.read_text()
        self.assertIn('#include "generated_rawtext_path.inc"', source)
        self.assertIn("addRawTextDrawOne(path.get());", source)
        self.assertIn('std::strcmp(argv[4], "direct-rawtext")', source)
        self.assertIn("constexpr uint32_t kDirectRawTextPatchCount = 318;", source)
        self.assertIn("facts.drawBatches[1].elementCount == kDirectRawTextPatchCount", source)

        with tempfile.TemporaryDirectory() as temp_dir:
            first = pathlib.Path(temp_dir) / "first.inc"
            second = pathlib.Path(temp_dir) / "second.inc"
            command = ["python3", str(RAWTEXT_GENERATOR), "--stream",
                       str(RAWTEXT_STREAM), "--output"]
            subprocess.run(command + [str(first)], check=True)
            subprocess.run(command + [str(second)], check=True)
            self.assertEqual(first.read_bytes(), second.read_bytes())
            generated = first.read_text().splitlines()
            self.assertEqual(len(generated), 511)
            self.assertEqual(
                generated[4], "    path->moveTo(5.9765625f, 66.796875f);"
            )
            self.assertEqual(generated[-2], "    path->close();")

    def test_direct_flower_provenance_matches_stream_line_7(self):
        stream_line = FLOWER_STREAM.read_text().splitlines()[6]
        self.assertIn("clipPath path={id=1,fillRule=2", stream_line)
        fixture_path = re.search(
            r"verbs=\[([^]]+)\],points=\[(.*)\]\}\}$", stream_line
        )
        self.assertIsNotNone(fixture_path)
        expected_verbs = fixture_path.group(1).split(",")
        expected_points = re.findall(
            r"\(([^,]+),([^)]+)\)", fixture_path.group(2)
        )

        def f32_bits(literal: str) -> int:
            if literal.endswith("f"):
                literal = literal[:-1]
            return struct.unpack("<I", struct.pack("<f", float(literal)))[0]

        expected_point_bits = [
            (f32_bits(x), f32_bits(y)) for x, y in expected_points
        ]
        source = EXPORTER.read_text()
        function_start = source.index(
            "void addClockwiseNestedFlower(rive::RenderPath* path)"
        )
        function_end = source.index(
            "\nvoid addBadSkinHair", function_start
        )
        function_source = source[function_start:function_end]
        commands = re.findall(
            r"path->(moveTo|cubicTo|close)\((.*?)\);",
            function_source,
            re.DOTALL,
        )
        actual_verbs = {
            "moveTo": "move",
            "cubicTo": "cubic",
            "close": "close",
        }
        self.assertEqual(
            [actual_verbs[command] for command, _ in commands], expected_verbs
        )

        actual_point_bits = []
        for command, argument_list in commands:
            if command == "close":
                self.assertEqual(argument_list.strip(), "")
                continue
            arguments = [part.strip() for part in argument_list.split(",")]
            expected_argument_count = 2 if command == "moveTo" else 6
            self.assertEqual(len(arguments), expected_argument_count)
            actual_point_bits.extend(
                (f32_bits(arguments[index]), f32_bits(arguments[index + 1]))
                for index in range(0, len(arguments), 2)
            )

        self.assertEqual(len(expected_verbs), 17)
        self.assertEqual(len(expected_point_bits), 41)
        self.assertEqual(actual_point_bits, expected_point_bits)

    def test_direct_bad_skin_provenance_matches_stream_lines_328_to_330(self):
        stream_lines = BAD_SKIN_STREAM.read_text().splitlines()
        self.assertEqual(
            stream_lines[327],
            "transform matrix=[1.00501573,0.116219193,-0.11621917,1.00501561,550.433167,361.510925]",
        )
        self.assertIn("makeEmptyRenderPath {id=4,fillRule=0", stream_lines[328])
        stream_line = stream_lines[329]
        self.assertIn("drawPath path={id=4,fillRule=0", stream_line)
        fixture_path = re.search(
            r"verbs=\[([^]]+)\],points=\[(.*)\]\}\}", stream_line
        )
        self.assertIsNotNone(fixture_path)
        expected_verbs = fixture_path.group(1).split(",")
        expected_points = re.findall(
            r"\(([^,]+),([^)]+)\)", fixture_path.group(2)
        )

        def f32_bits(literal: str) -> int:
            return struct.unpack("<I", struct.pack("<f", float(literal.rstrip("f"))))[0]

        expected_point_bits = [
            (f32_bits(x), f32_bits(y)) for x, y in expected_points
        ]
        source = EXPORTER.read_text()
        function_start = source.index("void addBadSkinHair(rive::RenderPath* path)")
        function_end = source.index("\nstd::vector<uint8_t> readTexture", function_start)
        function_source = source[function_start:function_end]
        commands = re.findall(
            r"path->(moveTo|cubicTo|close)\((.*?)\);",
            function_source,
            re.DOTALL,
        )
        actual_verbs = {"moveTo": "move", "cubicTo": "cubic", "close": "close"}
        self.assertEqual(
            [actual_verbs[command] for command, _ in commands], expected_verbs
        )
        actual_point_bits = []
        for command, argument_list in commands:
            if command == "close":
                self.assertEqual(argument_list.strip(), "")
                continue
            arguments = [part.strip() for part in argument_list.split(",")]
            self.assertEqual(len(arguments), 2 if command == "moveTo" else 6)
            actual_point_bits.extend(
                (f32_bits(arguments[index]), f32_bits(arguments[index + 1]))
                for index in range(0, len(arguments), 2)
            )
        self.assertEqual(len(expected_verbs), 15)
        self.assertEqual(len(expected_point_bits), 40)
        self.assertEqual(actual_point_bits, expected_point_bits)
        self.assertIn("constexpr uint32_t kDirectBadSkinFrameWidth = 999;", source)
        self.assertIn("constexpr uint32_t kDirectBadSkinFrameHeight = 720;", source)
        self.assertIn("constexpr uint32_t kDirectBadSkinContourCount = 1;", source)
        self.assertIn("constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'D', 'B', 'I', '\\0'};", source)
        self.assertIn('std::strcmp(argv[4], "direct-bad-skin")', source)
        self.assertIn("addBadSkinHair(path.get());", source)
        self.assertIn("renderer.transform(rive::Mat2D(1.00501573f,", source)
        self.assertIn(
            "if (fillCase && !directBadSkinCase)\n    {\n        path->fillRule(rive::FillRule::clockwise);",
            source,
        )

    def test_atomic_colorburn_pair_provenance_and_storage_contract(self):
        stream_lines = INTERLEAVED_FEATHER_STREAM.read_text().splitlines()
        self.assertEqual(
            stream_lines[55],
            "transform matrix=[1,0,0,1,485.557434,246.052628]",
        )
        self.assertEqual(
            stream_lines[56],
            "transform matrix=[0.490357965,0,0,0.490357965,0,0]",
        )
        self.assertEqual(
            stream_lines[57],
            "transform matrix=[-0.530765116,-0.847518981,0.847518981,-0.530765116,0,0]",
        )
        self.assertIn(
            "paint={id=1,style=fill,color=0x4affafc5,thickness=6.35442448,join=0,cap=0,feather=9.56621265,blendMode=19,shader=0}",
            stream_lines[58],
        )
        self.assertIn(
            "paint={id=1,style=stroke,color=0xe0000000,thickness=5.00454855,join=1,cap=0,feather=9.56621265,blendMode=19,shader=0}",
            stream_lines[59],
        )
        source = EXPORTER.read_text()
        for fragment in (
            "constexpr uint32_t kAtomicColorBurnPairFrameSize = 1024;",
            '#include "generated_interleaved_colorburn_pair_path.inc"',
            "addInterleavedFeatherColorBurnPairPath(path.get());",
            "path->fillRule(rive::FillRule::clockwise);",
            "renderer.translate(485.557434f, 246.052628f);",
            "renderer.scale(0.490357965f, 0.490357965f);",
            "rive::Mat2D(-0.530765116f,",
            "paint->color(0x4affafc5);",
            "paint->color(0xe0000000);",
            "paint->thickness(5.00454855f);",
            "paint->join(rive::StrokeJoin::round);",
            "paint->cap(rive::StrokeCap::butt);",
            "constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'A', 'C', 'O', '\\0'};",
            "writeAtomicColor(auxiliaryOutput,",
        ):
            self.assertIn(fragment, source)
        patch = RUNTIME_PATCH.read_text()
        self.assertIn("atomicPLSColorBufferForOracle() const", patch)
        self.assertIn("atomicPLSColorBufferSizeForOracle() const", patch)

        with tempfile.TemporaryDirectory() as temp_dir:
            first = pathlib.Path(temp_dir) / "first.inc"
            second = pathlib.Path(temp_dir) / "second.inc"
            command = ["python3", str(COLORBURN_PAIR_GENERATOR), "--stream",
                       str(INTERLEAVED_FEATHER_STREAM), "--output"]
            subprocess.run(command + [str(first)], check=True)
            subprocess.run(command + [str(second)], check=True)
            self.assertEqual(first.read_bytes(), second.read_bytes())
            generated = first.read_text()
            self.assertIn("path->moveTo(100.f, 0.f);", generated)
            self.assertIn("path->moveTo(60.0000038f, 0.f);", generated)
            self.assertIn(
                "path->cubicTo(60.0000038f, -33.1149063f, 33.1149063f, "
                "-60.0000038f, 0.f, -60.0000038f);",
                generated,
            )

            source_points = re.findall(
                r"\(([-+0-9.eE]+),([-+0-9.eE]+)\)",
                stream_lines[58].split("points=[", 1)[1].split("]}} paint=", 1)[0],
            )
            generated_values = re.findall(
                r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?(?=f)",
                generated,
            )
            source_bits = [
                struct.pack("<f", float(value))
                for point in source_points
                for value in point
            ]
            generated_bits = [
                struct.pack("<f", float(value)) for value in generated_values
            ]
            self.assertEqual(generated_bits, source_bits)

    def test_interleavedfeather_full_replay_is_deterministic_and_strict(self):
        expected_counts = [
            "save=301",
            "restore=301",
            "transform=900",
            "makeEmptyRenderPath=1",
            "makeRenderPaint=1",
            "drawPath=451",
        ]

        def command(stream, output=None, check=False):
            result = [
                "python3",
                str(PATH_STREAM_GENERATOR),
                "--stream",
                str(stream),
                "--expected-sha256",
                INTERLEAVED_FEATHER_SHA256,
                "--expected-source",
                "gm:interleavedfeather",
                "--expected-width",
                "1000",
                "--expected-height",
                "1000",
            ]
            for count in expected_counts:
                result.extend(["--expected-count", count])
            result.extend(["--function", "replayInterleavedFeatherFull"])
            result.extend(["--check"] if check else ["--output", str(output)])
            return result

        subprocess.run(command(INTERLEAVED_FEATHER_STREAM, check=True), check=True)
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_dir = pathlib.Path(temp_dir)
            first = temp_dir / "first.inc"
            second = temp_dir / "second.inc"
            subprocess.run(command(INTERLEAVED_FEATHER_STREAM, first), check=True)
            subprocess.run(command(INTERLEAVED_FEATHER_STREAM, second), check=True)
            self.assertEqual(first.read_bytes(), second.read_bytes())

            generated = first.read_text()
            digest = hashlib.sha256(INTERLEAVED_FEATHER_STREAM.read_bytes()).hexdigest()
            self.assertEqual(digest, INTERLEAVED_FEATHER_SHA256)
            self.assertIn(f"interleavedfeather.rive-stream sha256={digest}", generated)
            self.assertIn(
                "void replayInterleavedFeatherFull(rive::RiveRenderer* renderer, "
                "rive::gpu::RenderContext* context)",
                generated,
            )
            self.assertEqual(generated.count("renderer->save();"), 301)
            self.assertEqual(generated.count("renderer->restore();"), 301)
            self.assertEqual(generated.count("renderer->transform("), 900)
            self.assertEqual(generated.count("renderer->drawPath("), 451)

            drifted = temp_dir / "drifted.rive-stream"
            drifted.write_text(
                INTERLEAVED_FEATHER_STREAM.read_text().replace(
                    "clearColor value=0x00000000",
                    "clearColor value=0xffffffff",
                    1,
                )
            )
            wrong_digest = subprocess.run(
                command(drifted, check=True),
                text=True,
                capture_output=True,
            )
            self.assertNotEqual(wrong_digest.returncode, 0)
            self.assertIn("path-only stream sha256 drifted", wrong_digest.stderr)

            drifted_digest = hashlib.sha256(drifted.read_bytes()).hexdigest()
            drifted_command = command(drifted, check=True)
            digest_index = drifted_command.index("--expected-sha256") + 1
            drifted_command[digest_index] = drifted_digest
            wrong_header = subprocess.run(
                drifted_command,
                text=True,
                capture_output=True,
            )
            self.assertNotEqual(wrong_header.returncode, 0)
            self.assertIn(
                "header or clear-color contract drifted", wrong_header.stderr
            )

    def test_dstreadshuffle_replay_is_deterministic_and_enforces_opaque_clear_color(self):
        expected_counts = [
            "save=97",
            "restore=97",
            "transform=96",
            "makeEmptyRenderPath=193",
            "makeRenderPaint=97",
            "drawPath=97",
        ]

        def command(output=None, check=False, clear_color="0xffffffff"):
            result = [
                sys.executable,
                str(PATH_STREAM_GENERATOR),
                "--stream",
                str(DSTREADSHUFFLE_STREAM),
                "--expected-sha256",
                DSTREADSHUFFLE_SHA256,
                "--expected-source",
                "gm:dstreadshuffle",
                "--expected-width",
                "530",
                "--expected-height",
                "690",
                "--expected-clear-color",
                clear_color,
            ]
            for count in expected_counts:
                result.extend(["--expected-count", count])
            result.extend(["--function", "replayDstReadShuffleFull"])
            result.extend(["--check"] if check else ["--output", str(output)])
            return result

        subprocess.run(command(check=True), check=True)
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_dir = pathlib.Path(temp_dir)
            first = temp_dir / "first.inc"
            second = temp_dir / "second.inc"
            subprocess.run(command(first), check=True)
            subprocess.run(command(second), check=True)
            self.assertEqual(first.read_bytes(), second.read_bytes())

            generated = first.read_text()
            digest = hashlib.sha256(DSTREADSHUFFLE_STREAM.read_bytes()).hexdigest()
            self.assertEqual(digest, DSTREADSHUFFLE_SHA256)
            self.assertIn(f"dstreadshuffle.rive-stream sha256={digest}", generated)
            self.assertIn(
                "void replayDstReadShuffleFull(rive::RiveRenderer* renderer, "
                "rive::gpu::RenderContext* context)",
                generated,
            )
            self.assertEqual(generated.count("renderer->save();"), 97)
            self.assertEqual(generated.count("renderer->restore();"), 97)
            self.assertEqual(generated.count("renderer->transform("), 96)
            self.assertEqual(generated.count("context->makeEmptyRenderPath()"), 193)
            self.assertEqual(generated.count("context->makeRenderPaint()"), 97)
            self.assertEqual(generated.count("renderer->drawPath("), 97)

            wrong_clear_color = subprocess.run(
                command(check=True, clear_color="0x00000000"),
                text=True,
                capture_output=True,
            )
            self.assertNotEqual(wrong_clear_color.returncode, 0)
            self.assertIn(
                "header or clear-color contract drifted",
                wrong_clear_color.stderr,
            )

    def test_dstreadshuffle_srcover_control_replay_is_deterministic_and_overrides_only_blend_modes(self):
        expected_counts = [
            "save=97",
            "restore=97",
            "transform=96",
            "makeEmptyRenderPath=193",
            "makeRenderPaint=97",
            "drawPath=97",
        ]

        def command(output=None, check=False, override_blend_mode=None):
            result = [
                sys.executable,
                str(PATH_STREAM_GENERATOR),
                "--stream",
                str(DSTREADSHUFFLE_STREAM),
                "--expected-sha256",
                DSTREADSHUFFLE_SHA256,
                "--expected-source",
                "gm:dstreadshuffle",
                "--expected-width",
                "530",
                "--expected-height",
                "690",
                "--expected-clear-color",
                "0xffffffff",
            ]
            for count in expected_counts:
                result.extend(["--expected-count", count])
            if override_blend_mode is not None:
                result.extend(["--override-blend-mode", str(override_blend_mode)])
            result.extend(["--function", "replayDstReadShuffleSrcOverControl"])
            result.extend(["--check"] if check else ["--output", str(output)])
            return result

        subprocess.run(command(check=True, override_blend_mode=3), check=True)
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_dir = pathlib.Path(temp_dir)
            baseline = temp_dir / "baseline.inc"
            first = temp_dir / "first.inc"
            second = temp_dir / "second.inc"
            subprocess.run(command(baseline), check=True)
            subprocess.run(command(first, override_blend_mode=3), check=True)
            subprocess.run(command(second, override_blend_mode=3), check=True)
            self.assertEqual(first.read_bytes(), second.read_bytes())

            baseline_generated = baseline.read_text()
            generated = first.read_text()
            baseline_lines = baseline_generated.splitlines()
            control_lines = generated.splitlines()
            override_comment = "// Diagnostic paint blend-mode override=3."
            self.assertEqual(control_lines.count(override_comment), 1)
            self.assertEqual(len(control_lines), len(baseline_lines) + 1)
            control_lines.remove(override_comment)

            blend_mode_pattern = re.compile(
                r"    paint(\d+)->blendMode\(static_cast<rive::BlendMode>\((\d+)\)\);"
            )
            baseline_setters = [
                blend_mode_pattern.fullmatch(line) for line in baseline_lines
            ]
            self.assertEqual(sum(match is not None for match in baseline_setters), 97)

            control_setter_count = 0
            for baseline_line, control_line, baseline_setter in zip(
                baseline_lines, control_lines, baseline_setters
            ):
                if baseline_setter is None:
                    self.assertEqual(control_line, baseline_line)
                    continue
                paint_id, _ = baseline_setter.groups()
                self.assertEqual(
                    control_line,
                    f"    paint{paint_id}->blendMode(static_cast<rive::BlendMode>(3));",
                )
                control_setter_count += 1
            self.assertEqual(control_setter_count, 97)

    def test_spotify_kids_app_icon_riv_replay_is_deterministic_and_clips_paths(self):
        expected_counts = [
            "drawPath=14",
            "clipPath=6",
            "transform=15",
            "save=18",
            "restore=18",
            "makeEmptyRenderPath=20",
            "makeRenderPaint=48",
        ]

        def command(stream, output=None, check=False):
            result = [
                sys.executable,
                str(PATH_STREAM_GENERATOR),
                "--profile",
                "riv",
                "--stream",
                str(stream),
                "--expected-sha256",
                hashlib.sha256(stream.read_bytes()).hexdigest(),
                "--expected-source-suffix",
                SPOTIFY_KIDS_APP_ICON_SOURCE_SUFFIX,
                "--expected-artboard",
                "artboard",
                "--expected-scene",
                "State Machine 1",
                "--expected-width",
                "1024",
                "--expected-height",
                "1436",
                "--expected-sample-seconds",
                "0",
            ]
            for count in expected_counts:
                result.extend(["--expected-count", count])
            result.extend(["--function", "replaySpotifyKidsAppIconFull"])
            result.extend(["--check"] if check else ["--output", str(output)])
            return result

        self.assertEqual(
            hashlib.sha256(SPOTIFY_KIDS_APP_ICON_STREAM.read_bytes()).hexdigest(),
            SPOTIFY_KIDS_APP_ICON_SHA256,
        )
        subprocess.run(command(SPOTIFY_KIDS_APP_ICON_STREAM, check=True), check=True)
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_dir = pathlib.Path(temp_dir)
            first = temp_dir / "first.inc"
            second = temp_dir / "second.inc"
            subprocess.run(command(SPOTIFY_KIDS_APP_ICON_STREAM, first), check=True)
            subprocess.run(command(SPOTIFY_KIDS_APP_ICON_STREAM, second), check=True)
            self.assertEqual(first.read_bytes(), second.read_bytes())

            generated = first.read_text()
            self.assertIn(
                "void replaySpotifyKidsAppIconFull(rive::RiveRenderer* renderer, "
                "rive::gpu::RenderContext* context)",
                generated,
            )
            self.assertEqual(generated.count("renderer->drawPath("), 14)
            self.assertEqual(generated.count("renderer->clipPath("), 6)
            self.assertEqual(generated.count("renderer->save();"), 18)
            self.assertEqual(generated.count("renderer->restore();"), 18)
            self.assertIn("path1->lineTo(1024.f, 1436.f);", generated)
            self.assertIn("renderer->clipPath(path1.get());", generated)

            def assert_rejected(name, content, message):
                drifted = temp_dir / f"{name}.rive-stream"
                drifted.write_text(content)
                result = subprocess.run(
                    command(drifted, check=True),
                    text=True,
                    capture_output=True,
                )
                self.assertNotEqual(result.returncode, 0, name)
                self.assertIn(message, result.stderr, name)

            original = SPOTIFY_KIDS_APP_ICON_STREAM.read_text()
            assert_rejected(
                "clear-color",
                original.replace(
                    "sample seconds=0\n",
                    "sample seconds=0\nclearColor value=0x00000000\n",
                    1,
                ),
                "RIV profile header contract drifted",
            )
            assert_rejected(
                "undeclared-clip",
                original.replace("clipPath path={id=1", "clipPath path={id=21", 1),
                "path snapshot references an undeclared path",
            )
            first_draw = next(
                line for line in original.splitlines() if line.startswith("drawPath path={id=2,")
            )
            assert_rejected(
                "mutated-path",
                original.replace(
                    "\nframe\n",
                    "\n" + first_draw.replace("(-512,-312)", "(-511,-312)", 1) + "\nframe\n",
                    1,
                ),
                "mutates after its first snapshot",
            )
            assert_rejected(
                "shader",
                original.replace("shader=0", "shader=1", 1),
                "does not support paint shaders",
            )
            assert_rejected(
                "image",
                original.replace("sample seconds=0\n", "sample seconds=0\nmakeRenderImage id=1\n", 1),
                "unsupported path-only stream command",
            )
            assert_rejected(
                "extra-frame",
                original + "frame\n",
                "exactly one terminal frame marker",
            )

    def test_build_pins_and_discovers_naga(self):
        source = BUILD_SCRIPT.read_text()
        for fragment in (
            'naga_bin="${RIVE_ATLAS_MASK_NAGA:-$HOME/.cargo/bin/naga}"',
            'ninja_bin="${RIVE_ATLAS_MASK_NINJA:-$dawn_dir/third_party/ninja/ninja}"',
            'Darwin) default_gn="$dawn_dir/buildtools/mac/gn"',
            'expected_naga_version="30.0.0"',
            '"$naga_bin" --version',
            'export PATH="$(dirname "$naga_bin"):$PATH"',
            'dawn_args_snapshot_candidate="$(mktemp ',
            'dawn_args_snapshot="$dawn_args_snapshot_candidate"',
            'cp "$dawn_args_snapshot" "$dawn_args"',
            'cmp -s "$dawn_args_snapshot" "$dawn_args"',
            'rm -f "$output"',
            'rm -f "$output" "$inputs_output"',
            '"$output" "$inputs_output"',
            '"$fill_output" "$fill_inputs_output" "$fill_blit_output" fill',
            '"$cusp_output" "$cusp_inputs_output" "$cusp_blit_output" cusp "$softened_cusp_output"',
            'direct_cusp_coverage_output="${RIVE_DIRECT_CUSP_COVERAGE_OUTPUT:-$script_dir/out/direct-cusp-coverage.bin}"',
            '"$direct_cusp_inputs_output" "$direct_cusp_blit_output" direct-cusp "$direct_cusp_coverage_output"',
            'direct_polyshark_inputs_output="${RIVE_DIRECT_POLYSHARK_INPUT_OUTPUT:-$script_dir/out/direct-polyshark-inputs.bin}"',
            'direct_grid_inputs_output="${RIVE_DIRECT_GRID_INPUT_OUTPUT:-$script_dir/out/direct-grid-inputs.bin}"',
            'direct_flower_inputs_output="${RIVE_DIRECT_FLOWER_INPUT_OUTPUT:-$script_dir/out/direct-flower-inputs.bin}"',
            'direct_bad_skin_inputs_output="${RIVE_DIRECT_BAD_SKIN_INPUT_OUTPUT:-$script_dir/out/direct-bad-skin-inputs.bin}"',
            'direct_strokes_round_inputs_output="${RIVE_DIRECT_STROKES_ROUND_INPUT_OUTPUT:-$script_dir/out/direct-strokes-round-inputs.bin}"',
            'direct_strokes_round_blit_output="${RIVE_DIRECT_STROKES_ROUND_BLIT_OUTPUT:-$script_dir/out/direct-strokes-round-blit.rgba}"',
            'direct_strokes_round_spans_output="${RIVE_DIRECT_STROKES_ROUND_SPANS_OUTPUT:-$script_dir/out/direct-strokes-round-spans.bin}"',
            'direct_rawtext_inputs_output="${RIVE_DIRECT_RAWTEXT_INPUT_OUTPUT:-$script_dir/out/direct-rawtext-inputs.bin}"',
            'direct_rawtext_blit_output="${RIVE_DIRECT_RAWTEXT_BLIT_OUTPUT:-$script_dir/out/direct-rawtext-blit.rgba}"',
            'direct_rawtext_spans_output="${RIVE_DIRECT_RAWTEXT_SPANS_OUTPUT:-$script_dir/out/direct-rawtext-spans.bin}"',
            'polyshark_generator="$script_dir/generate_polyshark_stream_path.py"',
            'python3 "$polyshark_generator" --stream "$polyshark_stream" --check',
            '--output "$injected_dir/generated_polyshark_path.inc"',
            'rawtext_generator="$script_dir/generate_rawtext_stream_path.py"',
            'python3 "$rawtext_generator" --stream "$rawtext_stream" --check',
            '--output "$injected_dir/generated_rawtext_path.inc"',
            'colorburn_pair_generator="$script_dir/generate_interleaved_colorburn_pair_path.py"',
            'python3 "$colorburn_pair_generator" --stream "$colorburn_pair_stream" --check',
            '--output "$injected_dir/generated_interleaved_colorburn_pair_path.inc"',
            '"$direct_polyshark_inputs_output" /dev/null direct-polyshark',
            '"$direct_grid_inputs_output" /dev/null direct-grid',
            '"$direct_flower_inputs_output" /dev/null direct-flower',
            '"$direct_bad_skin_inputs_output" /dev/null direct-bad-skin',
            '"$direct_strokes_round_inputs_output" "$direct_strokes_round_blit_output" direct-strokes-round "$direct_strokes_round_spans_output"',
            '"$direct_rawtext_inputs_output" "$direct_rawtext_blit_output" direct-rawtext "$direct_rawtext_spans_output"',
            'nested_evenodd_path_clipped_blit_output="${RIVE_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT_OUTPUT:-$script_dir/out/atlas-nested-evenodd-path-clipped-blit.rgba}"',
            'nested_clockwise_path_clipped_blit_output="${RIVE_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT_OUTPUT:-$script_dir/out/atlas-nested-clockwise-path-clipped-blit.rgba}"',
            'advanced_blend_blit_output="${RIVE_ATLAS_ADVANCED_BLEND_BLIT_OUTPUT:-$script_dir/out/atlas-advanced-blend-blit.rgba}"',
            'atomic_advanced_blend_output="${RIVE_ATOMIC_ADVANCED_BLEND_OUTPUT:-$script_dir/out/atomic-advanced-blend.rgba}"',
            'atomic_colorburn_pair_output="${RIVE_ATOMIC_COLORBURN_PAIR_OUTPUT:-$script_dir/out/atomic-colorburn-pair.rgba}"',
            'atomic_colorburn_pair_color_output="${RIVE_ATOMIC_COLORBURN_PAIR_COLOR_OUTPUT:-$script_dir/out/atomic-colorburn-pair.color}"',
            'atomic_colorburn_pair_coverage_output="${RIVE_ATOMIC_COLORBURN_PAIR_COVERAGE_OUTPUT:-$script_dir/out/atomic-colorburn-pair.coverage}"',
            'atomic_interleavedfeather_full_output="${RIVE_ATOMIC_INTERLEAVEDFEATHER_FULL_OUTPUT:-$script_dir/out/atomic-interleavedfeather-full.rgba}"',
            'atomic_interleavedfeather_full_provenance="${RIVE_ATOMIC_INTERLEAVEDFEATHER_FULL_PROVENANCE:-$script_dir/out/atomic-interleavedfeather-full.provenance}"',
            'atomic_dstreadshuffle_full_output="${RIVE_ATOMIC_DSTREADSHUFFLE_FULL_OUTPUT:-$script_dir/out/atomic-dstreadshuffle-full.rgba}"',
            'atomic_dstreadshuffle_full_provenance="${RIVE_ATOMIC_DSTREADSHUFFLE_FULL_PROVENANCE:-$script_dir/out/atomic-dstreadshuffle-full.provenance}"',
            'atomic_dstreadshuffle_srcover_output="${RIVE_ATOMIC_DSTREADSHUFFLE_SRCOVER_OUTPUT:-$script_dir/out/atomic-dstreadshuffle-srcover.rgba}"',
            'atomic_dstreadshuffle_srcover_provenance="${RIVE_ATOMIC_DSTREADSHUFFLE_SRCOVER_PROVENANCE:-$script_dir/out/atomic-dstreadshuffle-srcover.provenance}"',
            'expected_interleavedfeather_sha256="8868c228229b6708e4e46c947177bfd982c6e7a60ee9b1c3a7da43a7ec0ee17a"',
            'expected_dstreadshuffle_sha256="0e08ecd19e6a9e1f89f3ae2291181cea3513edf5bbe8cadcd3e1e10a0c33f195"',
            'expected_runtime_revision="7c778d13c5d903b3b74eec1dd6bb68a811dea5f2"',
            'expected_dawn_revision="211333b2e3e429c3508f25c81c547f602adf448c"',
            "sha256_file()",
            'git -C "$runtime" diff --quiet',
            'git -C "$runtime" diff --cached --quiet',
            'git -C "$dawn_dir" diff --quiet',
            'git -C "$dawn_dir" diff --cached --quiet',
            'path_stream_generator="$script_dir/generate_path_stream_replay.py"',
            'python3 "$path_stream_generator"',
            '--expected-sha256 "$expected_interleavedfeather_sha256"',
            '--function replayInterleavedFeatherFull --check',
            '--output "$injected_dir/generated_interleavedfeather_full.inc"',
            '--stream "$dstreadshuffle_stream"',
            '--expected-sha256 "$expected_dstreadshuffle_sha256"',
            '--expected-source gm:dstreadshuffle',
            '--expected-width 530 --expected-height 690',
            '--expected-clear-color 0xffffffff',
            '--expected-count save=97 --expected-count restore=97',
            '--expected-count transform=96 --expected-count makeEmptyRenderPath=193',
            '--expected-count makeRenderPaint=97 --expected-count drawPath=97',
            '--function replayDstReadShuffleFull',
            '--output "$injected_dir/generated_dstreadshuffle_full.inc"',
            '--override-blend-mode 3',
            '--function replayDstReadShuffleSrcOverControl',
            '--output "$injected_dir/generated_dstreadshuffle_srcover_control.inc"',
            '"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$nested_evenodd_path_clipped_blit_output" nested-evenodd-path-clipped',
            '"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$nested_clockwise_path_clipped_blit_output" nested-clockwise-path-clipped',
            '"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$advanced_blend_blit_output" advanced-blend',
            '"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_advanced_blend_output" atomic-advanced-blend',
            '"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_colorburn_pair_output" atomic-colorburn-pair "$atomic_colorburn_pair_color_output" "$atomic_colorburn_pair_coverage_output"',
            '"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_interleavedfeather_full_output" atomic-interleavedfeather-full "$atomic_interleavedfeather_full_provenance"',
            'atomic_interleavedfeather_full_sha256="$(sha256_file "$atomic_interleavedfeather_full_output")"',
            '"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_dstreadshuffle_full_output" atomic-dstreadshuffle-full "$atomic_dstreadshuffle_full_provenance"',
            'atomic_dstreadshuffle_full_sha256="$(sha256_file "$atomic_dstreadshuffle_full_output")"',
            '"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_dstreadshuffle_srcover_output" atomic-dstreadshuffle-srcover-full "$atomic_dstreadshuffle_srcover_provenance"',
            'atomic_dstreadshuffle_srcover_sha256="$(sha256_file "$atomic_dstreadshuffle_srcover_output")"',
            "artifact_sha256=$atomic_interleavedfeather_full_sha256",
            "artifact_sha256=$atomic_dstreadshuffle_full_sha256",
            "stream_sha256=$expected_dstreadshuffle_sha256",
            "artifact_sha256=$atomic_dstreadshuffle_srcover_sha256",
            "blend_mode_override=srcOver",
            "runtime_revision=$expected_runtime_revision",
            "dawn_revision=$expected_dawn_revision",
            '"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null /dev/null msaa-intersection-groups',
            'python3 "$script_dir/format_test.py" --validate-direct-grid "$direct_grid_inputs_output"',
            'python3 "$script_dir/format_test.py" --validate-direct-flower "$direct_flower_inputs_output"',
            'python3 "$script_dir/format_test.py" --validate-direct-bad-skin "$direct_bad_skin_inputs_output"',
            'python3 "$script_dir/format_test.py" --validate-direct-cusp-coverage "$direct_cusp_coverage_output"',
            'python3 "$script_dir/format_test.py" --validate-atomic-colorburn-pair "$atomic_colorburn_pair_color_output"',
            'python3 "$script_dir/format_test.py" --validate-atomic-colorburn-pair-coverage "$atomic_colorburn_pair_coverage_output"',
            'python3 "$script_dir/format_test.py" --validate-direct-strokes-round "$direct_strokes_round_inputs_output"',
            'python3 "$script_dir/format_test.py" --validate-direct-strokes-round-spans "$direct_strokes_round_spans_output"',
            'python3 "$script_dir/format_test.py" --validate-direct-rawtext "$direct_rawtext_inputs_output"',
            'python3 "$script_dir/format_test.py" --validate-direct-rawtext-spans "$direct_rawtext_spans_output"',
            'if [[ "$output_bytes" != "4628" ]]',
            'if [[ "$blit_bytes" != "16404" ]]',
            'if [[ "$atomic_advanced_blend_bytes" != "16404" ]]',
            'if [[ "$atomic_colorburn_pair_bytes" != "4194324" ]]',
            'if [[ "$atomic_colorburn_pair_color_bytes" != "4194328" ]]',
            'if [[ "$atomic_colorburn_pair_coverage_bytes" != "4194328" ]]',
            'if [[ "$atomic_interleavedfeather_full_bytes" != "4000020" ]]',
            'if [[ "$atomic_dstreadshuffle_full_bytes" != "1462820" ]]',
            'atomic dstreadshuffle provenance is missing $provenance: $atomic_dstreadshuffle_full_provenance',
            'if [[ "$atomic_dstreadshuffle_srcover_bytes" != "1462820" ]]',
            'atomic dstreadshuffle SrcOver provenance is missing $provenance: $atomic_dstreadshuffle_srcover_provenance',
            'if [[ "$fill_output_bytes" != "4628" ]]',
            'if [[ "$fill_blit_bytes" != "16404" ]]',
            'if [[ "$cusp_output_bytes" != "4628" ]]',
            'if [[ "$cusp_blit_bytes" != "16404" ]]',
            'if (( softened_cusp_bytes <= 20 ))',
            'if (( direct_cusp_inputs_bytes <= 56 ))',
            'if [[ "$direct_cusp_coverage_bytes" != "16408" ]]',
            'if [[ "$direct_polyshark_inputs_bytes" != "163896" ]]',
            'if [[ "$direct_strokes_round_blit_bytes" != "640020" ]]',
            'if [[ "$direct_rawtext_inputs_bytes" != "66152" ]]',
            'if [[ "$direct_rawtext_spans_bytes" != "28060" ]]',
            'if [[ "$direct_rawtext_blit_bytes" != "536020" ]]',
            'echo "atomic dstreadshuffle full output: $atomic_dstreadshuffle_full_output"',
            'echo "atomic dstreadshuffle full provenance: $atomic_dstreadshuffle_full_provenance"',
            'echo "atomic dstreadshuffle SrcOver output: $atomic_dstreadshuffle_srcover_output"',
            'echo "atomic dstreadshuffle SrcOver provenance: $atomic_dstreadshuffle_srcover_provenance"',
        ):
            self.assertIn(fragment, source)

    def test_rust_fixture_uses_physical_placement_contract_and_required_env(self):
        source = RUST_RENDERER.read_text()
        for fragment in (
            "const ATLAS_ORACLE_FRAME_SIZE: u32 = 64;",
            "const ATLAS_ORACLE_PHYSICAL_SIZE: u32 = 48;",
            "const ATLAS_ORACLE_LOGICAL_SIZE: u32 = 39;",
            "const ATLAS_ORACLE_PLACEMENT: [f32; 2] = [2.0, 2.0];",
            "let mut placement = feather_atlas_placement(",
            "assert_eq!([placement.width, placement.height], [39, 39]);",
            "uniforms.atlas_texture_inverse_size =",
            "2.0 / placement.width as f32",
            "placement.width,",
            '#[ignore = "requires RIVE_CPP_ATLAS_MASK from the C++ WebGPU oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_FILL_MASK from the C++ WebGPU oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_FILL_INPUTS from the C++ WebGPU oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_CUSP_MASK from the C++ WebGPU oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_CUSP_INPUTS from the C++ WebGPU oracle"]',
            '#[ignore = "requires RIVE_CPP_SOFTENED_CUSP from the C++ oracle"]',
            '#[ignore = "requires RIVE_CPP_DIRECT_CUSP_INPUTS from the C++ WebGPU oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_BLIT from the C++ WebGPU MSAA oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT from the C++ WebGPU MSAA oracle"]',
            '#[ignore = "requires RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT from the C++ WebGPU MSAA oracle"]',
            '#[ignore = "requires RIVE_CPP_ATOMIC_ADVANCED_BLEND from the C++ WebGPU atomic oracle"]',
            '#[ignore = "requires RIVE_CPP_ATOMIC_INTERLEAVEDFEATHER_FULL and its provenance from the C++ WebGPU-on-Metal oracle"]',
            '#[ignore = "requires RIVE_CPP_DIRECT_STROKES_ROUND_SPANS from the C++ WebGPU oracle"]',
            '#[ignore = "requires RIVE_CPP_DIRECT_RAWTEXT_INPUTS from the C++ WebGPU oracle"]',
            '#[ignore = "requires RIVE_CPP_DIRECT_RAWTEXT_SPANS from the C++ WebGPU oracle"]',
            '.expect("RIVE_CPP_ATLAS_MASK is required for the ignored C++ atlas-mask oracle test")',
            'path.is_absolute()',
            "fn documented_cpp_atlas_mask_path_is_absolute_from_repo_root()",
        ):
            self.assertIn(fragment, source)
        readme = README.read_text()
        self.assertIn('RIVE_CPP_ATLAS_MASK="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-mask.r16f"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-blit.rgba"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-clipped-blit.rgba"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-path-clipped-blit.rgba"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_CHANGING_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-changing-path-clipped-blit.rgba"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_NESTED_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-nested-path-clipped-blit.rgba"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-nested-evenodd-path-clipped-blit.rgba"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-nested-clockwise-path-clipped-blit.rgba"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_ADVANCED_BLEND_BLIT="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-advanced-blend-blit.rgba"',
                      readme)
        self.assertIn('RIVE_CPP_ATOMIC_ADVANCED_BLEND="$PWD/tools/cpp-atlas-mask-oracle/out/atomic-advanced-blend.rgba"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_FILL_MASK="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-fill-mask.r16f"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_FILL_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-fill-inputs.bin"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_CUSP_MASK="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-cusp-mask.r16f"',
                      readme)
        self.assertIn('RIVE_CPP_ATLAS_CUSP_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/atlas-cusp-inputs.bin"',
                      readme)
        self.assertIn('RIVE_CPP_SOFTENED_CUSP="$PWD/tools/cpp-atlas-mask-oracle/out/softened-cusp.bin"',
                      readme)
        self.assertIn('RIVE_CPP_DIRECT_CUSP_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-cusp-inputs.bin"',
                      readme)
        self.assertIn('RIVE_CPP_DIRECT_POLYSHARK_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-polyshark-inputs.bin"',
                      readme)
        self.assertIn('RIVE_CPP_DIRECT_STROKES_ROUND_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-strokes-round-inputs.bin"',
                      readme)
        self.assertIn('RIVE_CPP_DIRECT_STROKES_ROUND_SPANS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-strokes-round-spans.bin"',
                      readme)
        self.assertIn('RIVE_CPP_DIRECT_RAWTEXT_INPUTS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-rawtext-inputs.bin"',
                      readme)
        self.assertIn('RIVE_CPP_DIRECT_RAWTEXT_SPANS="$PWD/tools/cpp-atlas-mask-oracle/out/direct-rawtext-spans.bin"',
                      readme)
        self.assertIn("-- --exact --ignored --nocapture", readme)

    def test_runtime_patch_applies_and_observes_production_atlas_state(self):
        files = (
            "renderer/include/rive/renderer/gpu.hpp",
            "renderer/include/rive/renderer/render_context.hpp",
            "renderer/include/rive/renderer/webgpu/render_context_webgpu_impl.hpp",
            "renderer/premake5.lua",
            "renderer/src/render_context.cpp",
            "renderer/src/webgpu/render_context_webgpu_impl.cpp",
        )
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = pathlib.Path(temp_dir)
            for relative in files:
                source = RUNTIME / relative
                destination = temp / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, destination)
            subprocess.run(["git", "apply", "--check", str(RUNTIME_PATCH)],
                           cwd=temp, check=True)
            subprocess.run(["git", "apply", str(RUNTIME_PATCH)], cwd=temp, check=True)

            cpp = (temp / "renderer/src/webgpu/render_context_webgpu_impl.cpp").read_text()
            atlas = cpp[cpp.index("void RenderContextWebGPUImpl::resizeAtlasTexture"):
                        cpp.index("void RenderContextWebGPUImpl::resizeAtomicCoverageBacking")]
            gradient = cpp[cpp.index("void RenderContextWebGPUImpl::resizeGradientTexture"):
                           cpp.index("void RenderContextWebGPUImpl::resizeTessellationTexture")]
            tessellation = cpp[cpp.index("void RenderContextWebGPUImpl::resizeTessellationTexture"):
                               cpp.index("void RenderContextWebGPUImpl::resizeAtlasTexture")]
            self.assertIn("wgpu::TextureUsage::CopySrc", atlas)
            self.assertIn("wgpu::TextureUsage::CopySrc", tessellation)
            self.assertIn("wgpu::TextureFormat::R16Float", atlas)
            self.assertIn("wgpu::TextureFormat::RGBA32Uint", tessellation)
            self.assertNotIn("wgpu::TextureUsage::CopySrc", gradient)
            coverage = cpp[cpp.index("wgpu::Buffer RenderContextWebGPUImpl::atomicPLSCoverageBuffer()"):
                           cpp.index("wgpu::RenderPipeline RenderContextWebGPUImpl::makeDrawPipeline")]
            self.assertIn("desc.usage |= wgpu::BufferUsage::CopySrc", coverage)
            color = cpp[cpp.index("wgpu::Buffer RenderContextWebGPUImpl::atomicPLSColorBuffer()"):
                        cpp.index("wgpu::Buffer RenderContextWebGPUImpl::atomicPLSClipBuffer()")]
            self.assertIn("desc.usage |= wgpu::BufferUsage::CopySrc", color)

            gpu = (temp / "renderer/include/rive/renderer/gpu.hpp").read_text()
            render_context_header = (
                temp / "renderer/include/rive/renderer/render_context.hpp"
            ).read_text()
            logical_flush = (temp / "renderer/src/render_context.cpp").read_text()
            webgpu_header = (
                temp / "renderer/include/rive/renderer/webgpu/render_context_webgpu_impl.hpp"
            ).read_text()
            for fragment in (
                "float atlasPathTranslateXForOracle = 0;",
                "float atlasPathTranslateYForOracle = 0;",
                "bool atlasPathTransformForOracleValid = false;",
                "AtlasInputContourForOracle",
                "AtlasInputTriangleForOracle",
                "atlasInputTrianglesForOracle",
                "const TessVertexSpan* tessVertexSpansForOracle = nullptr;",
            ):
                self.assertIn(fragment, gpu)
            self.assertIn(
                "std::vector<gpu::TessVertexSpan> m_tessVertexSpansForOracle;",
                render_context_header,
            )
            self.assertIn(
                "m_pendingAtlasDraws.front()->atlasTransform().translateX;", logical_flush
            )
            self.assertIn(
                "m_pendingAtlasDraws.front()->atlasTransform().translateY;", logical_flush
            )
            for fragment in (
                "const AtlasMaskOracleFacts& atlasMaskFactsForOracle() const",
                "m_atlasMaskOracleFacts.contentWidth = desc.atlasContentWidth;",
                "desc.atlasPathTranslateXForOracle;",
                "desc.atlasPathTranslateYForOracle;",
                "desc.atlasStrokeBatchCount + desc.atlasFillBatchCount",
                "const AtlasDrawBatch* oracleBatch =",
                "oracleBatch != nullptr ? oracleBatch->scissor",
                "oracleBatch != nullptr ? oracleBatch->basePatch",
                "oracleBatch != nullptr ? oracleBatch->patchCount",
                "batch.drawType == DrawType::midpointFanPatches ||",
                "batch.drawType == DrawType::midpointFanCenterAAPatches",
                "m_atlasMaskOracleFacts.tessVertexSpans.assign(",
                "tessellationTextureForOracle() const",
                "atomicPLSCoverageBufferForOracle() const",
                "atomicPLSCoverageBufferSizeForOracle() const",
                "atomicPLSColorBufferForOracle() const",
                "atomicPLSColorBufferSizeForOracle() const",
                "m_atlasMaskOracleFacts.contours.assign(",
                "m_atlasMaskOracleFacts.triangles.assign(",
            ):
                self.assertIn(fragment, webgpu_header + cpp)
            self.assertIn("m_atlasInputContoursForOracle.push_back(", logical_flush)
            self.assertIn("m_atlasInputTrianglesForOracle.push_back(", logical_flush)
            self.assertIn("m_tessVertexSpansForOracle.resize(", logical_flush)
            self.assertIn("copy_range_for_oracle", gpu + logical_flush)
            self.assertIn("pointForOracle", gpu + logical_flush)

            premake = (temp / "renderer/premake5.lua").read_text()
            self.assertGreater(premake.index("project('rive_atlas_mask_oracle')"),
                               premake.index("if RIVE_WAGYU_PORT then"))
            self.assertTrue(premake.rstrip().endswith("end"))


if __name__ == "__main__":
    validators = {
        "--validate-direct-grid":
            ("direct-grid", "RIVEDGI", parse_direct_grid_inputs),
        "--validate-direct-flower":
            ("direct-flower", "RIVEDFI", parse_direct_flower_inputs),
        "--validate-direct-bad-skin":
            ("direct-bad-skin", "RIVEDBI", parse_direct_bad_skin_inputs),
        "--validate-direct-cusp-coverage":
            ("direct-cusp-coverage", "RIVEAPC", validate_direct_cusp_coverage),
        "--validate-atomic-colorburn-pair":
            ("atomic-colorburn-pair", "RIVEACO", validate_atomic_colorburn_pair),
        "--validate-atomic-colorburn-pair-coverage":
            ("atomic-colorburn-pair-coverage", "RIVEAPC", validate_atomic_colorburn_pair_coverage),
        "--validate-direct-strokes-round":
            ("direct-strokes-round", "RIVEATI", validate_direct_strokes_round_inputs),
        "--validate-direct-strokes-round-spans":
            ("direct-strokes-round-spans", "RIVEATS",
             validate_direct_strokes_round_spans),
        "--validate-direct-rawtext":
            ("direct-rawtext", "RIVEATI", validate_direct_rawtext_inputs),
        "--validate-direct-rawtext-spans":
            ("direct-rawtext-spans", "RIVEATS", validate_direct_rawtext_spans),
    }
    if sys.argv[1:2] and sys.argv[1] in validators:
        if len(sys.argv) != 3:
            raise SystemExit(f"usage: format_test.py {sys.argv[1]} PATH")
        artifact_name, format_name, parser = validators[sys.argv[1]]
        try:
            artifact = pathlib.Path(sys.argv[2]).read_bytes()
            validated = parser(artifact)
        except (OSError, ValueError) as error:
            raise SystemExit(f"{artifact_name} artifact validation failed: {error}")
        if format_name == "RIVEATI":
            print(
                f"{format_name} valid: "
                f"patchCount={validated['patch_count']} "
                f"contours={validated['contour_count']} "
                f"tessellation={validated['width']}x{validated['height']}"
            )
        elif format_name == "RIVEATS":
            print(
                f"{format_name} valid: "
                f"firstSpan={validated['first_span']} "
                f"spans={validated['span_count']}"
            )
        elif format_name in ("RIVEAPC", "RIVEACO"):
            print(
                f"{format_name} valid: words={validated['word_count']} "
                f"{'coverage' if format_name == 'RIVEAPC' else 'color'}="
                f"{validated['width']}x{validated['height']}"
            )
        else:
            print(
                f"{format_name} valid: "
                f"batches={len(validated['batches'])} "
                f"contours={len(validated['contours'])} "
                f"triangleVertices={len(validated['triangles'])} "
                f"tessellation={validated['tess_width']}x{validated['tess_height']}"
            )
    else:
        unittest.main()
