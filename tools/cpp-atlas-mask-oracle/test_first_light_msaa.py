#!/usr/bin/env python3
"""Focused tests for strict First Light MSAA capture support."""

from __future__ import annotations

import hashlib
import pathlib
import struct
import sys
import tempfile
import tomllib
import unittest


TOOLS = pathlib.Path(__file__).resolve().parent
ROOT = TOOLS.parents[1]
sys.path.insert(0, str(TOOLS))

import generate_path_stream_replay as replay
import generate_msaa_reference_registry as registry
import capture_msaa_references as capture
import inventory_msaa_references as inventory


class FirstLightMsaaTest(unittest.TestCase):
    def test_header_only_stream_compiles_for_strict_dawn_replay(self) -> None:
        stream = ROOT / "fixtures/renderer/streams/first-light-rectangle.rive-stream"
        lines = stream.read_text(encoding="utf-8").splitlines()

        selection = replay.select_first_light_frame(lines)
        generated = replay.generate_include(
            stream=stream,
            profile="first-light",
            expected_sha256=hashlib.sha256(stream.read_bytes()).hexdigest(),
            expected_source=None,
            expected_source_suffix=None,
            expected_artboard="",
            expected_scene="",
            expected_width=64,
            expected_height=64,
            expected_clear_color="0x00000000",
            expected_sample_seconds=None,
            expected_counts={"drawPath": 1},
            function_name="replayFirstLight",
            blend_mode_override=None,
        )

        self.assertEqual((selection.width, selection.height), (64, 64))
        self.assertEqual(selection.command_lines, (lines[2],))
        self.assertIn("context->makeEmptyRenderPath()", generated)
        self.assertIn("context->makeRenderPaint()", generated)
        self.assertIn("renderer->drawPath", generated)

    def test_legacy_and_case_local_provenance_validate_together(self) -> None:
        manifest = tomllib.loads(
            (TOOLS / "msaa-reference-corpus.toml").read_text(encoding="utf-8")
        )["case"]
        legacy_case = next(case for case in manifest if case["id"] == "gm-rect-msaa")
        legacy_png = (
            ROOT
            / "fixtures/renderer/reference/dawn-webgpu-metal/gm/gm-rect-msaa.png"
        )
        first_light_case = {
            "id": "first-light-rectangle-msaa",
            "stream": "fixtures/renderer/streams/first-light-rectangle.rive-stream",
            "sha256": "bcbd5f7dd60748ddefe57186b8460a6d31673c8ef23c9eb80784fcf5f35116a7",
            "profile": "first-light",
            "width": 64,
            "height": 64,
            "clear_color": "0x00000000",
            "counts": {"drawPath": 1},
        }
        first_light_png = (
            ROOT
            / "fixtures/renderer/reference/dawn-webgpu-metal/first-light"
            / "first-light-rectangle-msaa.png"
        )

        inventory.validate_strict_reference(legacy_png, legacy_case, capture)
        inventory.validate_strict_reference(
            first_light_png, first_light_case, capture
        )

        with tempfile.TemporaryDirectory() as directory:
            tampered_png = pathlib.Path(directory) / first_light_png.name
            tampered_png.write_bytes(first_light_png.read_bytes())
            _, fields = capture.parse_provenance(first_light_png.with_suffix(".provenance"))
            fields["case_identity_sha256"] = "0" * 64
            tampered_png.with_suffix(".provenance").write_text(
                "".join(f"{key}={fields[key]}\n" for key in sorted(fields)),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "per-case identity mismatch"):
                inventory.validate_strict_reference(
                    tampered_png, first_light_case, capture
                )

    def test_capture_manifest_accepts_first_light_profile(self) -> None:
        stream = ROOT / "fixtures/renderer/streams/first-light-rectangle.rive-stream"
        stream_sha256 = hashlib.sha256(stream.read_bytes()).hexdigest()
        manifest_text = f'''version = 1

[[case]]
id = "first-light-rectangle-msaa"
stream = "fixtures/renderer/streams/first-light-rectangle.rive-stream"
sha256 = "{stream_sha256}"
profile = "first-light"
width = 64
height = 64
clear_color = "0x00000000"
counts = {{ drawPath = 1 }}
'''
        with tempfile.TemporaryDirectory() as directory:
            manifest = pathlib.Path(directory) / "manifest.toml"
            manifest.write_text(manifest_text, encoding="utf-8")
            cases = capture.load_cases(manifest, ROOT)

        self.assertEqual(len(cases), 1)
        self.assertEqual(cases[0].case_id, "first-light-rectangle-msaa")
        self.assertEqual(cases[0].identity_case["profile"], "first-light")

    def test_capture_publishes_case_local_provenance(self) -> None:
        source_png = (
            ROOT
            / "fixtures/renderer/reference/dawn-webgpu-metal/first-light"
            / "first-light-rectangle-msaa.png"
        )
        source_provenance = source_png.with_suffix(".provenance")
        _, source_fields = capture.parse_provenance(source_provenance)
        identity_case = {
            "id": "first-light-rectangle-msaa",
            "stream": "fixtures/renderer/streams/first-light-rectangle.rive-stream",
            "sha256": source_fields["stream_sha256"],
            "profile": "first-light",
            "width": 64,
            "height": 64,
            "clear_color": "0x00000000",
            "counts": {"drawPath": 1},
        }
        stream = ROOT / identity_case["stream"]
        case = capture.Case(
            identity_case["id"],
            stream,
            identity_case["sha256"],
            64,
            64,
            identity_case,
        )
        pixels = capture.decode_png_rgba8(source_png, 64, 64)
        upstream = {
            key: value
            for key, value in source_fields.items()
            if key in capture.UPSTREAM_PROVENANCE_KEYS
        }
        registry_sha256 = "1" * 64
        upstream["registry_sha256"] = registry_sha256

        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            png = root / "reference.png"
            png.write_bytes(source_png.read_bytes())
            artifact = root / "reference.rgba"
            artifact.write_bytes(
                capture.RIVEABL_MAGIC + struct.pack("<III", 1, 64, 64) + pixels
            )
            provenance = root / "reference.provenance"
            provenance.write_text(
                "".join(f"{key}={upstream[key]}\n" for key in sorted(upstream)),
                encoding="utf-8",
            )

            capture.validate_and_append_provenance(
                provenance,
                case,
                artifact,
                png,
                64,
                64,
                source_fields["runtime_revision"],
                source_fields["dawn_revision"],
                registry_sha256,
            )
            _, final = capture.parse_provenance(provenance)

        self.assertNotIn("registry_sha256", final)
        self.assertEqual(final["provenance_version"], "2")
        self.assertEqual(len(final["case_identity_sha256"]), 64)

    def test_registry_includes_first_light_case(self) -> None:
        stream = ROOT / "fixtures/renderer/streams/first-light-rectangle.rive-stream"
        manifest_text = f'''version = 1

[[case]]
id = "first-light-rectangle-msaa"
stream = "fixtures/renderer/streams/first-light-rectangle.rive-stream"
sha256 = "{hashlib.sha256(stream.read_bytes()).hexdigest()}"
profile = "first-light"
width = 64
height = 64
clear_color = "0x00000000"
counts = {{ drawPath = 1 }}
'''
        with tempfile.TemporaryDirectory() as directory:
            manifest = pathlib.Path(directory) / "manifest.toml"
            manifest.write_text(manifest_text, encoding="utf-8")
            generated, digest = registry.generate_registry(
                manifest, ROOT, "1" * 40, "2" * 40
            )

        self.assertIn("std::array<MsaaReferenceCase, 1>", generated)
        self.assertIn('"first-light-rectangle-msaa"', generated)
        self.assertEqual(len(digest), 64)

    def test_inventory_discovers_first_light_profile(self) -> None:
        entry = {
            "id": "first-light-rectangle-msaa",
            "kind": "gm-stream",
            "stream": "fixtures/renderer/streams/first-light-rectangle.rive-stream",
        }

        case, rejection = inventory.inspect_first_light(entry, replay)

        self.assertIsNone(rejection)
        self.assertEqual(case["profile"], "first-light")
        self.assertEqual(case["counts"], {"drawPath": 1})


if __name__ == "__main__":
    unittest.main()
