#!/usr/bin/env python3
"""Cross-language contract tests for the RIVEMSK v1 interchange format."""

import os
import pathlib
import shutil
import struct
import subprocess
import tempfile
import unittest


HEADER_BYTES = 20
MAGIC = b"RIVEMSK\0"
ROOT = pathlib.Path(__file__).resolve().parents[2]
EXPORTER = pathlib.Path(__file__).with_name("runtime-src") / "main.cpp"
BUILD_SCRIPT = pathlib.Path(__file__).with_name("build.sh")
RUNTIME_PATCH = pathlib.Path(__file__).with_name("runtime.patch")
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

    def test_exporter_configuration_matches_coordinated_stroke_fixture(self):
        source = EXPORTER.read_text()
        for fragment in (
            "constexpr uint32_t kFrameWidth = 64;",
            "constexpr uint32_t kFrameHeight = 64;",
            "constexpr uint32_t kHeaderBytes = 20;",
            "path->moveTo(16, 16);",
            "path->lineTo(48, 16);",
            "path->lineTo(48, 48);",
            "path->lineTo(16, 48);",
            "path->close();",
            "paint->style(rive::RenderPaintStyle::stroke);",
            "paint->thickness(8);",
            "paint->join(rive::StrokeJoin::miter);",
            "paint->cap(rive::StrokeCap::butt);",
            "paint->feather(20);",
            "void onMap(WGPUMapAsyncStatus status,",
            "status == WGPUMapAsyncStatus_Success",
            "context->static_impl_cast<rive::gpu::RenderContextWebGPUImpl>();",
            "webgpuContext->makeRenderTarget(",
            "webgpuContext->atlasMaskTextureForOracle();",
            "const uint32_t atlasWidth = atlas.GetWidth();",
            "const uint32_t atlasHeight = atlas.GetHeight();",
            "if (atlasWidth != kFrameWidth || atlasHeight != kFrameHeight)",
            '"cpp-atlas-mask-oracle: expected a 64x64 physical atlas, got %ux%u\\n"',
            "const uint32_t width = kFrameWidth;",
            "const uint32_t height = kFrameHeight;",
            ".width = width,",
            ".height = height,",
            "writeU32(header, 8, 1);",
            "writeU32(header, 12, width);",
            "writeU32(header, 16, height);",
        ):
            self.assertIn(fragment, source)
        self.assertNotIn("RenderPaintStyle::fill", source)
        self.assertNotIn("path->cubicTo(", source)
        self.assertNotIn("WGPUBufferMapAsyncStatus", source)
        self.assertNotIn("context->makeRenderTarget(", source)
        self.assertNotIn("context->atlasMaskTextureForOracle()", source)
        self.assertNotIn("std::min", source)
        self.assertNotIn("copyWidth", source)
        self.assertNotIn("copyHeight", source)

    def test_build_pins_and_discovers_naga(self):
        source = BUILD_SCRIPT.read_text()
        for fragment in (
            'naga_bin="${RIVE_ATLAS_MASK_NAGA:-$HOME/.cargo/bin/naga}"',
            'expected_naga_version="30.0.0"',
            '"$naga_bin" --version',
            'export PATH="$(dirname "$naga_bin"):$PATH"',
            'dawn_args_snapshot_candidate="$(mktemp ',
            'dawn_args_snapshot="$dawn_args_snapshot_candidate"',
            'cp "$dawn_args_snapshot" "$dawn_args"',
            'cmp -s "$dawn_args_snapshot" "$dawn_args"',
            'rm -f "$output"',
        ):
            self.assertIn(fragment, source)

    def test_runtime_patch_applies_and_only_makes_atlas_copyable(self):
        files = (
            "renderer/include/rive/renderer/webgpu/render_context_webgpu_impl.hpp",
            "renderer/premake5.lua",
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

            cpp = (temp / files[2]).read_text()
            atlas = cpp[cpp.index("void RenderContextWebGPUImpl::resizeAtlasTexture"):
                        cpp.index("void RenderContextWebGPUImpl::resizeAtomicCoverageBacking")]
            gradient = cpp[cpp.index("void RenderContextWebGPUImpl::resizeGradientTexture"):
                           cpp.index("void RenderContextWebGPUImpl::resizeTessellationTexture")]
            self.assertIn("wgpu::TextureUsage::CopySrc", atlas)
            self.assertIn("wgpu::TextureFormat::R16Float", atlas)
            self.assertNotIn("wgpu::TextureUsage::CopySrc", gradient)

            premake = (temp / files[1]).read_text()
            self.assertGreater(premake.index("project('rive_atlas_mask_oracle')"),
                               premake.index("if RIVE_WAGYU_PORT then"))
            self.assertTrue(premake.rstrip().endswith("end"))


if __name__ == "__main__":
    unittest.main()
