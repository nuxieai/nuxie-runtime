#!/usr/bin/env python3
"""Cross-language contract tests for the atlas oracle formats."""

import os
import pathlib
import shutil
import struct
import subprocess
import tempfile
import unittest


HEADER_BYTES = 20
MAGIC = b"RIVEMSK\0"
INPUT_HEADER_BYTES = 40
INPUT_MAGIC = b"RIVEATI\0"
BLIT_MAGIC = b"RIVEABL\0"
DIRECT_GRID_HEADER_BYTES = 64
DIRECT_GRID_MAGIC = b"RIVEDGI\0"
ROOT = pathlib.Path(__file__).resolve().parents[2]
EXPORTER = pathlib.Path(__file__).with_name("runtime-src") / "main.cpp"
BUILD_SCRIPT = pathlib.Path(__file__).with_name("build.sh")
README = pathlib.Path(__file__).with_name("README.md")
RUNTIME_PATCH = pathlib.Path(__file__).with_name("runtime.patch")
RUST_RENDERER = ROOT / "crates" / "nuxie-renderer" / "src" / "lib.rs"
POLYSHARK_STREAM = ROOT / "fixtures" / "renderer" / "streams" / "gm" / "feather_polyshapes.rive-stream"
POLYSHARK_GENERATOR = pathlib.Path(__file__).with_name("generate_polyshark_stream_path.py")
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


def make_inputs() -> bytes:
    header = INPUT_MAGIC + struct.pack("<8I", 1, 1, 2, 1, 2, 1, 16, 16)
    return header + bytes(16) + bytes(2 * 16)


def parse_direct_grid_inputs(data: bytes) -> dict:
    if len(data) < DIRECT_GRID_HEADER_BYTES:
        raise ValueError("file is shorter than the 64-byte RIVEDGI header")
    if data[:8] != DIRECT_GRID_MAGIC:
        raise ValueError("bad RIVEDGI magic")
    fields = struct.unpack_from("<14I", data, 8)
    (version, header_bytes, flags, interlock_mode, draw_batch_count,
     tess_width, tess_height, contour_count, triangle_vertex_count,
     draw_batch_stride, contour_stride, triangle_vertex_stride,
     tess_texel_stride, reserved) = fields
    if version != 1 or header_bytes != DIRECT_GRID_HEADER_BYTES:
        raise ValueError("unsupported RIVEDGI header")
    if flags != 1 or interlock_mode == 0:
        raise ValueError("RIVEDGI requires clockwise-fill atomic facts")
    if reserved != 0:
        raise ValueError("RIVEDGI reserved header bytes must be zero")
    if not tess_width or not tess_height:
        raise ValueError("RIVEDGI tessellation dimensions must be nonzero")
    if contour_count != 100:
        raise ValueError("RIVEDGI must contain exactly 100 contours")
    if draw_batch_count < 3 or not triangle_vertex_count or triangle_vertex_count % 3:
        raise ValueError("RIVEDGI draw or triangle record count is invalid")
    if (draw_batch_stride, contour_stride, triangle_vertex_stride, tess_texel_stride) != (20, 16, 12, 16):
        raise ValueError("RIVEDGI stride mismatch")
    expected = (DIRECT_GRID_HEADER_BYTES + draw_batch_count * draw_batch_stride
                + contour_count * contour_stride
                + triangle_vertex_count * triangle_vertex_stride
                + tess_width * tess_height * tess_texel_stride)
    if len(data) != expected:
        raise ValueError("RIVEDGI length mismatch")
    offset = DIRECT_GRID_HEADER_BYTES
    batches = [struct.unpack_from("<5I", data, offset + i * draw_batch_stride)
               for i in range(draw_batch_count)]
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


def encode_direct_grid_inputs(parsed: dict) -> bytes:
    batches = parsed["batches"]
    contours = parsed["contours"]
    triangles = parsed["triangles"]
    header = DIRECT_GRID_MAGIC + struct.pack(
        "<14I", 1, DIRECT_GRID_HEADER_BYTES, 1, parsed["interlock_mode"],
        len(batches), parsed["tess_width"], parsed["tess_height"],
        len(contours), len(triangles), 20, 16, 12, 16, 0,
    )
    return (header + b"".join(struct.pack("<5I", *batch) for batch in batches)
            + b"".join(struct.pack("<4I", *contour) for contour in contours)
            + b"".join(struct.pack("<3I", *triangle) for triangle in triangles)
            + parsed["tessellation"])


def make_direct_grid_inputs() -> bytes:
    parsed = {
        "interlock_mode": 3,
        "batches": [(1, 0, 0, 0, 1)] * 4,
        "contours": [(0, 0, 1, index) for index in range(100)],
        "triangles": [(0, 0, 0x00010001)] * 6,
        "tess_width": 2,
        "tess_height": 1,
        "tessellation": bytes(range(32)),
    }
    return encode_direct_grid_inputs(parsed)


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

    def test_direct_grid_format_round_trips_and_rejects_malformed_counts(self):
        data = make_direct_grid_inputs()
        parsed = parse_direct_grid_inputs(data)
        self.assertEqual(len(parsed["batches"]), 4)
        self.assertEqual(len(parsed["contours"]), 100)
        self.assertEqual(len(parsed["triangles"]), 6)
        self.assertEqual(parsed["tessellation"], bytes(range(32)))
        self.assertEqual(encode_direct_grid_inputs(parsed), data)

        bad_header = bytearray(data)
        struct.pack_into("<I", bad_header, 12, 60)
        with self.assertRaisesRegex(ValueError, "header"):
            parse_direct_grid_inputs(bad_header)
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
            "directCuspCase || directPolySharkCase || directGridCase;",
            "const bool fillCase = circleCase || cuspCase || directCase;",
            "path->fillRule(rive::FillRule::clockwise);",
            "path->cubicTo(51.2f, 16, 12.8f, 16, 48, 48);",
            "path->cubicTo(133.635864f, 0, -33.6358566f, 0, 100, 100);",
            '#include "generated_polyshark_path.inc"',
            "rive::Mat2D(kPolySharkStreamScale,",
            "addFeatherPolyShapesShark(path.get());",
            "void addClockwiseNestedGrid(rive::RenderPath* path)",
            "largeclippedpath_clockwise_nested.rive-stream:10",
            "addClockwiseNestedGrid(path.get());",
            "paint->style(fillCase ? rive::RenderPaintStyle::fill",
            ": rive::RenderPaintStyle::stroke);",
            "path->cubicTo(kSquareMax,",
            "paint->thickness(kStrokeThickness);",
            "paint->join(rive::StrokeJoin::miter);",
            "paint->cap(rive::StrokeCap::butt);",
            "paint->feather(directGridCase ? 0.f : (directCase ? 1.f : kFeather));",
            ".msaaSampleCount = directCase ? 0u : 4u",
            "void onMap(WGPUMapAsyncStatus status,",
            "status == WGPUMapAsyncStatus_Success",
            "context->static_impl_cast<rive::gpu::RenderContextWebGPUImpl>();",
            "webgpuContext->makeRenderTarget(",
            "webgpuContext->atlasMaskTextureForOracle();",
            "webgpuContext->tessellationTextureForOracle();",
            "writeInputs(inputsOutput,",
            "writeDirectGridInputs(inputsOutput,",
            "constexpr char kMagic[8] = {'R', 'I', 'V', 'E', 'D', 'G', 'I', '\\0'};",
            "writeBlit(blitOutput,",
            "readTexture(instance, device, queue, targetTexture, 4);",
            "atlasWidth = atlas.GetWidth();",
            "atlasHeight = atlas.GetHeight();",
            "if (atlasWidth != kExpectedPhysicalAtlasSize ||",
            "readTexture(instance, device, queue, atlas, kBytesPerTexel);",
            "readTexture(instance, device, queue, tessellation, kTessBytesPerTexel);",
            "writeU32(header, 8, 1);",
            "writeU32(header, 12, width);",
            "writeU32(header, 16, height);",
        ):
            self.assertIn(fragment, source)
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
            '"$direct_cusp_inputs_output" "$direct_cusp_blit_output" direct-cusp',
            'direct_polyshark_inputs_output="${RIVE_DIRECT_POLYSHARK_INPUT_OUTPUT:-$script_dir/out/direct-polyshark-inputs.bin}"',
            'direct_grid_inputs_output="${RIVE_DIRECT_GRID_INPUT_OUTPUT:-$script_dir/out/direct-grid-inputs.bin}"',
            'polyshark_generator="$script_dir/generate_polyshark_stream_path.py"',
            'python3 "$polyshark_generator" --stream "$polyshark_stream" --check',
            '--output "$injected_dir/generated_polyshark_path.inc"',
            '"$direct_polyshark_inputs_output" /dev/null direct-polyshark',
            '"$direct_grid_inputs_output" /dev/null direct-grid',
            'if [[ "$output_bytes" != "4628" ]]',
            'if [[ "$blit_bytes" != "16404" ]]',
            'if [[ "$fill_output_bytes" != "4628" ]]',
            'if [[ "$fill_blit_bytes" != "16404" ]]',
            'if [[ "$cusp_output_bytes" != "4628" ]]',
            'if [[ "$cusp_blit_bytes" != "16404" ]]',
            'if (( softened_cusp_bytes <= 20 ))',
            'if (( direct_cusp_inputs_bytes <= 56 ))',
            'if [[ "$direct_polyshark_inputs_bytes" != "163896" ]]',
            'if (( direct_grid_inputs_bytes <= 64 + 20 * 3 + 16 * 100 + 12 ))',
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

            gpu = (temp / "renderer/include/rive/renderer/gpu.hpp").read_text()
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
            ):
                self.assertIn(fragment, gpu)
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
                "tessellationTextureForOracle() const",
                "m_atlasMaskOracleFacts.contours.assign(",
                "m_atlasMaskOracleFacts.triangles.assign(",
            ):
                self.assertIn(fragment, webgpu_header + cpp)
            self.assertIn("m_atlasInputContoursForOracle.push_back(", logical_flush)
            self.assertIn("m_atlasInputTrianglesForOracle.push_back(", logical_flush)
            self.assertIn("copy_range_for_oracle", gpu + logical_flush)
            self.assertIn("pointForOracle", gpu + logical_flush)

            premake = (temp / "renderer/premake5.lua").read_text()
            self.assertGreater(premake.index("project('rive_atlas_mask_oracle')"),
                               premake.index("if RIVE_WAGYU_PORT then"))
            self.assertTrue(premake.rstrip().endswith("end"))


if __name__ == "__main__":
    unittest.main()
