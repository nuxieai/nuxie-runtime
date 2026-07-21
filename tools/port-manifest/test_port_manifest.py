#!/usr/bin/env python3

import pathlib
import subprocess
import sys
import tempfile
import textwrap
import tomllib
import unittest


TOOL = pathlib.Path(__file__).with_name("port_manifest.py")


class PortManifestCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.repo = self.root / "repo"
        self.upstream = self.root / "rive-runtime"
        (self.repo / "crates/runtime/src").mkdir(parents=True)
        (self.repo / "crates/runtime/src/lib.rs").write_text("// runtime\n")
        (self.upstream / "src").mkdir(parents=True)

    def write_upstream(self, *paths: str) -> None:
        for path in paths:
            source = self.upstream / path
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text("// upstream\n")

    def write_manifest(self, body: str) -> pathlib.Path:
        manifest = self.repo / "port-manifest.toml"
        manifest.write_text(textwrap.dedent(body).lstrip())
        return manifest

    def run_check(
        self, manifest: pathlib.Path, upstream_ref: str | None = None
    ) -> subprocess.CompletedProcess[str]:
        command = [
            sys.executable,
            str(TOOL),
            "check",
            "--rive-runtime-dir",
            str(self.upstream),
            "--repo-root",
            str(self.repo),
            "--manifest",
            str(manifest),
        ]
        if upstream_ref is not None:
            command.extend(["--upstream-ref", upstream_ref])
        return subprocess.run(
            command,
            text=True,
            capture_output=True,
            check=False,
        )

    def run_generate(self, output: pathlib.Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(TOOL),
                "generate",
                "--rive-runtime-dir",
                str(self.upstream),
                "--upstream-ref",
                "test-ref",
                "--output",
                str(output),
            ],
            text=True,
            capture_output=True,
            check=False,
        )

    def test_check_fails_when_upstream_cpp_has_no_manifest_row(self) -> None:
        self.write_upstream("src/a.cpp", "src/nested/b.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            note = "Consolidated runtime port."
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing manifest rows: src/nested/b.cpp", result.stderr)

    def test_check_fails_when_declared_rust_module_does_not_exist(self) -> None:
        self.write_upstream("src/a.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/missing.rs"
            note = "Consolidated runtime port."
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "missing Rust module for src/a.cpp: crates/runtime/src/missing.rs",
            result.stderr,
        )

    def test_check_fails_when_upstream_path_is_declared_twice(self) -> None:
        self.write_upstream("src/a.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            note = "First row."

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            note = "Duplicate row."
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("duplicate manifest rows: src/a.cpp", result.stderr)

    def test_check_fails_when_manifest_row_is_not_in_upstream(self) -> None:
        self.write_upstream("src/a.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            note = "Current row."

            [[file]]
            upstream = "src/old.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            note = "Stale row."
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("stale manifest rows: src/old.cpp", result.stderr)

    def test_check_rejects_status_outside_the_declared_vocabulary(self) -> None:
        self.write_upstream("src/a.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/a.cpp"
            status = "todo"
            rust_module = ""
            note = "Not classified."
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid status for src/a.cpp: todo", result.stderr)

    def test_check_requires_absent_rows_to_cite_a_feature_gap(self) -> None:
        self.write_upstream("src/a.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/a.cpp"
            status = "absent"
            rust_module = ""
            note = "Not implemented."
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("absent row must cite an F-row id: src/a.cpp", result.stderr)

    def test_check_requires_ported_rows_to_declare_a_rust_module(self) -> None:
        self.write_upstream("src/a.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = ""
            note = "Consolidated runtime port."
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("ported row must declare a Rust module: src/a.cpp", result.stderr)

    def test_generate_sorts_rows_and_seeds_known_feature_gaps(self) -> None:
        self.write_upstream(
            "src/generated/node_base.cpp",
            "src/component.cpp",
            "src/audio/audio_engine.cpp",
        )
        output = self.repo / "generated.toml"

        result = self.run_generate(output)

        self.assertEqual(result.returncode, 0, result.stderr)
        document = tomllib.loads(output.read_text())
        rows = document["file"]
        self.assertEqual(
            [row["upstream"] for row in rows],
            [
                "src/audio/audio_engine.cpp",
                "src/component.cpp",
            ],
        )
        self.assertEqual(rows[0]["status"], "absent")
        self.assertIn("F1", rows[0]["note"])
        self.assertEqual(rows[0]["rust_module"], "")
        self.assertEqual(rows[1]["status"], "ported")
        self.assertEqual(rows[1]["rust_module"], "crates/nuxie-runtime/src/components.rs")
        self.assertEqual(document["upstream_ref"], "test-ref")

    def test_generate_seeds_the_register_feature_rows(self) -> None:
        expected = {
            "src/text/cursor.cpp": ("partial", "F2"),
            "src/command_queue.cpp": ("absent", "F3"),
            "src/constraints/scrolling/elastic_scroll_physics.cpp": ("absent", "F4"),
            "src/animation/keyboard_listener_group.cpp": ("absent", "F5"),
            "src/semantic/semantic_manager.cpp": ("absent", "F6"),
            "src/lua/lua_promise.cpp": ("absent", "F7"),
            "src/lua/renderer/lua_gpu.cpp": ("absent", "F8"),
            "src/joystick.cpp": ("partial", "F9"),
            "src/shapes/list_path.cpp": ("partial", "F10"),
            "src/async/work_pool.cpp": ("absent", "F12"),
            "src/listener_group.cpp": ("partial", "F13"),
            "src/core/binary_writer.cpp": ("not-applicable", "F14"),
        }
        self.write_upstream(*expected)
        output = self.repo / "generated.toml"

        result = self.run_generate(output)

        self.assertEqual(result.returncode, 0, result.stderr)
        rows = {
            row["upstream"]: row for row in tomllib.loads(output.read_text())["file"]
        }
        for upstream, (status, feature_id) in expected.items():
            with self.subTest(upstream=upstream):
                self.assertEqual(rows[upstream]["status"], status)
                self.assertIn(feature_id, rows[upstream]["note"])

    def test_check_reports_exact_inventory_and_status_counts(self) -> None:
        self.write_upstream("src/a.cpp", "src/b.cpp", "src/c.cpp", "src/d.cpp")
        manifest = self.write_manifest(
            """
            version = 1
            source_glob = "src/**/*.cpp"
            row_count = 4

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            note = "Ported."

            [[file]]
            upstream = "src/b.cpp"
            status = "partial"
            rust_module = "crates/runtime/src/lib.rs"
            note = "F2: partial."

            [[file]]
            upstream = "src/c.cpp"
            status = "absent"
            rust_module = ""
            note = "F1: absent."

            [[file]]
            upstream = "src/d.cpp"
            status = "not-applicable"
            rust_module = ""
            note = "F14: not applicable."
            """
        )

        result = self.run_check(manifest)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            result.stdout.strip(),
            "port-manifest: 4/4 rows (ported=1, partial=1, absent=1, "
            "not-applicable=1); Rust module paths verified",
        )

    def test_check_rejects_a_known_register_seed_reclassified_in_the_manifest(self) -> None:
        self.write_upstream("src/audio/audio_engine.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/audio/audio_engine.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            note = "Incorrectly claimed as ported."
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "register seed drift for src/audio/audio_engine.cpp: expected status=absent",
            result.stderr,
        )

    def test_check_rejects_the_wrong_feature_id_on_an_absent_seed(self) -> None:
        self.write_upstream("src/audio/audio_engine.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/audio/audio_engine.cpp"
            status = "absent"
            rust_module = ""
            note = "F2: wrong register row."
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "register seed drift for src/audio/audio_engine.cpp: expected feature ids F1",
            result.stderr,
        )

    def test_generate_routes_consolidated_subsystems_to_rust_modules(self) -> None:
        expected = {
            "src/animation/keyframe.cpp": "crates/nuxie-runtime/src/animation.rs",
            "src/constraints/constraint.cpp": "crates/nuxie-runtime/src/constraints.rs",
            "src/data_bind/data_bind.cpp": "crates/nuxie-runtime/src/artboard_data_bind.rs",
            "src/lua/lua_properties.cpp": "crates/nuxie-scripting/src/vm.rs",
            "src/shapes/path.cpp": "crates/nuxie-runtime/src/draw.rs",
            "src/text/text.cpp": "crates/nuxie-runtime/src/text.rs",
            "src/viewmodel/viewmodel.cpp": "crates/nuxie-runtime/src/view_model.rs",
        }
        self.write_upstream(*expected)
        output = self.repo / "generated.toml"

        result = self.run_generate(output)

        self.assertEqual(result.returncode, 0, result.stderr)
        rows = {
            row["upstream"]: row for row in tomllib.loads(output.read_text())["file"]
        }
        for upstream, rust_module in expected.items():
            with self.subTest(upstream=upstream):
                self.assertEqual(rows[upstream]["status"], "ported")
                self.assertEqual(rows[upstream]["rust_module"], rust_module)

    def test_generate_seeds_every_cpp_surface_named_by_the_feature_register(self) -> None:
        expected = {
            "src/assets/audio_asset.cpp": ("partial", "F1"),
            "src/audio/audio_engine.cpp": ("absent", "F1"),
            "src/audio/audio_reader.cpp": ("absent", "F1"),
            "src/audio/audio_sound.cpp": ("absent", "F1"),
            "src/audio/audio_source.cpp": ("absent", "F1"),
            "src/audio_event.cpp": ("absent", "F1"),
            "src/text/cursor.cpp": ("partial", "F2"),
            "src/text/raw_text_input.cpp": ("partial", "F2"),
            "src/text/text_input.cpp": ("partial", "F2"),
            "src/text/text_input_cursor.cpp": ("partial", "F2"),
            "src/text/text_input_drawable.cpp": ("partial", "F2"),
            "src/text/text_input_selected_text.cpp": ("partial", "F2"),
            "src/text/text_input_selection.cpp": ("partial", "F2"),
            "src/text/text_input_text.cpp": ("partial", "F2"),
            "src/text/text_selection_path.cpp": ("partial", "F2"),
            "src/command_queue.cpp": ("absent", "F3"),
            "src/command_server.cpp": ("absent", "F3"),
            "src/constraints/scrolling/clamped_scroll_physics.cpp": ("partial", "F4"),
            "src/constraints/scrolling/elastic_scroll_physics.cpp": ("absent", "F4"),
            "src/constraints/scrolling/scroll_bar_constraint.cpp": ("absent", "F4"),
            "src/constraints/scrolling/scroll_bar_constraint_proxy.cpp": ("absent", "F4"),
            "src/constraints/scrolling/scroll_constraint.cpp": ("partial", "F4"),
            "src/constraints/scrolling/scroll_constraint_proxy.cpp": ("partial", "F4"),
            "src/constraints/scrolling/scroll_physics.cpp": ("partial", "F4"),
            "src/animation/gamepad_listener_group.cpp": ("absent", "F5"),
            "src/animation/keyboard_listener_group.cpp": ("absent", "F5"),
            "src/animation/semantic_listener_group.cpp": ("absent", "F5"),
            "src/animation/text_input_listener_group.cpp": ("absent", "F5"),
            "src/animation/listener_types/listener_input_type_gamepad.cpp": ("absent", "F5"),
            "src/animation/listener_types/listener_input_type_keyboard.cpp": ("absent", "F5"),
            "src/animation/listener_types/listener_input_type_semantic.cpp": ("absent", "F5"),
            "src/input/gamepad_batch.cpp": ("absent", "F5"),
            "src/inputs/gamepad_input.cpp": ("absent", "F5"),
            "src/inputs/keyboard_input.cpp": ("absent", "F5"),
            "src/inputs/semantic_input.cpp": ("absent", "F5"),
            "src/semantic/semantic_data.cpp": ("absent", "F6"),
            "src/semantic/semantic_inference_registry.cpp": ("absent", "F6"),
            "src/semantic/semantic_manager.cpp": ("absent", "F6"),
            "src/semantic/semantic_provider.cpp": ("absent", "F6"),
            "src/lua/lua_audio.cpp": ("absent", "F7"),
            "src/lua/lua_buffer_ext.cpp": ("absent", "F7"),
            "src/lua/lua_data_context.cpp": ("absent", "F7"),
            "src/lua/lua_data_value.cpp": ("absent", "F7"),
            "src/lua/lua_image_decode.cpp": ("absent", "F7"),
            "src/lua/lua_promise.cpp": ("absent", "F7"),
            "src/lua/lua_scripted_context.cpp": ("absent", "F7"),
            "src/lua/lua_state.cpp": ("absent", "F7"),
            "src/lua/math/lua_color.cpp": ("absent", "F7"),
            "src/lua/math/lua_input.cpp": ("absent", "F7"),
            "src/lua/renderer/lua_blob.cpp": ("absent", "F7"),
            "src/lua/renderer/lua_gpu.cpp": ("absent", "F8"),
            "src/lua/renderer/lua_gradient.cpp": ("absent", "F7"),
            "src/lua/renderer/lua_image.cpp": ("absent", "F7"),
            "src/lua/renderer/lua_mesh.cpp": ("absent", "F7"),
            "src/joystick.cpp": ("partial", "F9"),
            "src/shapes/list_path.cpp": ("partial", "F10"),
            "src/async/work_pool.cpp": ("absent", "F12"),
            "src/profiler/profiler.cpp": ("absent", "F12"),
            "src/profiler/rive_profile.cpp": ("absent", "F12"),
            "src/listener_group.cpp": ("partial", "F13"),
            "src/nested_artboard.cpp": ("partial", "F13"),
            "src/data_bind/context/context_value_artboard.cpp": ("partial", "F13"),
            "src/text/text_modifier.cpp": ("partial", "F13"),
            "src/core/binary_writer.cpp": ("not-applicable", "F14"),
            "src/core/binary_data_reader.cpp": ("not-applicable", "F14"),
            "src/static_scene.cpp": ("not-applicable", "F14"),
            "src/hittest_command_path.cpp": ("not-applicable", "F14"),
            "src/intrinsically_sizeable.cpp": ("not-applicable", "F14"),
        }
        self.write_upstream(*expected)
        output = self.repo / "generated.toml"

        result = self.run_generate(output)

        self.assertEqual(result.returncode, 0, result.stderr)
        rows = {
            row["upstream"]: row for row in tomllib.loads(output.read_text())["file"]
        }
        for upstream, (status, feature_id) in expected.items():
            with self.subTest(upstream=upstream):
                self.assertEqual(rows[upstream]["status"], status)
                self.assertIn(feature_id, rows[upstream]["note"])

    def test_check_excludes_generated_cpp_from_the_provenance_surface(self) -> None:
        self.write_upstream("src/a.cpp", "src/generated/node_base.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            note = "Ported."
            """
        )

        result = self.run_check(manifest)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("port-manifest: 1/1 rows", result.stdout)

    def test_check_rejects_a_checkout_that_does_not_match_the_manifest_ref(self) -> None:
        self.write_upstream("src/a.cpp")
        manifest = self.write_manifest(
            """
            version = 1
            upstream_ref = "candidate-ref"

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            note = "Ported."
            """
        )

        result = self.run_check(manifest, upstream_ref="different-ref")

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "upstream ref mismatch: manifest candidate-ref, checkout different-ref",
            result.stderr,
        )

    def test_check_requires_every_row_field(self) -> None:
        self.write_upstream("src/a.cpp")
        manifest = self.write_manifest(
            """
            version = 1

            [[file]]
            upstream = "src/a.cpp"
            status = "ported"
            rust_module = "crates/runtime/src/lib.rs"
            """
        )

        result = self.run_check(manifest)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("manifest row missing field note: src/a.cpp", result.stderr)


if __name__ == "__main__":
    unittest.main()
