import tempfile
import unittest
from pathlib import Path

import generate_manifest


class SilverManifestGeneratorTests(unittest.TestCase):
    def test_discovers_literal_producer_metadata(self):
        source = """
TEST_CASE("renders selected board", "[silver]")
{
    rive::File::deterministicMode = true;
    auto file = ReadRiveFile("assets/example.riv", &silver);
    auto board = file->artboardNamed("Board");
    auto sm = board->stateMachineNamed("Machine");
    sm->advanceAndApply(0.125f);
    CHECK(silver.matches("example-Board"));
}
"""
        chunks = generate_manifest.test_chunks(source)
        self.assertEqual(len(chunks), 1)
        name, line, chunk = chunks[0]
        self.assertEqual(name, "renders selected board")
        self.assertEqual(line, 2)
        self.assertEqual(generate_manifest.LITERAL_MATCH.search(chunk).group(1), "example-Board")
        self.assertEqual(generate_manifest.RIV_STRING.findall(chunk), ["assets/example.riv"])
        self.assertEqual(generate_manifest.ARTBOARD_NAME.findall(chunk), ["Board"])
        self.assertEqual(generate_manifest.STATE_MACHINE_NAME.findall(chunk), ["Machine"])
        self.assertEqual(generate_manifest.SAMPLE_TIME.findall(chunk), ["0.125"])

    def test_dynamic_layout_entries_are_hand_authored(self):
        producers = generate_manifest.dynamic_producers()
        self.assertEqual(len(producers), 6)
        self.assertEqual(
            {producer.id for producer in producers},
            {
                "layout_scroll_snap_padding_layouts",
                "layout_scroll_snap_padding_list",
                "layout_scroll_snap_padding_virtualized",
                "layout_scroll_drag_multiplier_layouts",
                "layout_scroll_drag_multiplier_list",
                "layout_scroll_drag_multiplier_virtualized",
            },
        )
        self.assertTrue(
            all(producer.producer_class == "layout-scroll-dynamic" for producer in producers)
        )

    def test_expands_constant_cpp_frame_loops_into_ordered_actions(self):
        chunk = """
        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.0f);
        artboard->draw(renderer.get());
        int frames = (int)(0.1f / 0.05f);
        for (int i = 0; i < frames; i++)
        {
            silver.addFrame();
            stateMachine->advanceAndApply(0.05f);
            artboard->draw(renderer.get());
        }
        """
        actions, blocker = generate_manifest.executable_actions(
            chunk, "default", "none"
        )
        self.assertIsNone(blocker)
        self.assertEqual(
            actions,
            (
                {"kind": "bind-default-view-model"},
                {"kind": "advance", "target": "state-machine", "seconds": 0.0},
                {"kind": "draw"},
                {"kind": "frame"},
                {"kind": "advance", "target": "state-machine", "seconds": 0.05},
                {"kind": "draw"},
                {"kind": "frame"},
                {"kind": "advance", "target": "state-machine", "seconds": 0.05},
                {"kind": "draw"},
            ),
        )

    def test_expands_literal_cpp_frame_count(self):
        actions, blocker = generate_manifest.executable_actions(
            """
            int frames = 2;
            for (int i = 0; i < frames; i++)
            {
                silver.addFrame();
                artboard->advance(0.016f);
                artboard->draw(renderer.get());
            }
            """,
            "none",
            "none",
        )
        self.assertIsNone(blocker)
        self.assertEqual(
            actions,
            (
                {"kind": "frame"},
                {"kind": "advance", "target": "artboard", "seconds": 0.016},
                {"kind": "draw"},
                {"kind": "frame"},
                {"kind": "advance", "target": "artboard", "seconds": 0.016},
                {"kind": "draw"},
            ),
        )

    def test_names_blocking_subsystem_instead_of_thinning_mutations(self):
        actions, blocker = generate_manifest.executable_actions(
            'vmi->propertyValue("score")->as<Number>()->propertyValue(42);',
            "default",
            "none",
        )
        self.assertEqual(actions, ())
        self.assertEqual(blocker, "view-model-mutation")

    def test_rejects_unencoded_focus_actions_instead_of_silently_dropping_them(self):
        actions, blocker = generate_manifest.executable_actions(
            """
            stateMachine->focusManager()->focusNext();
            stateMachine->advanceAndApply(0.016f);
            artboard->draw(renderer.get());
            """,
            "default",
            "none",
        )
        self.assertEqual(actions, ())
        self.assertEqual(blocker, "focus-keyboard-dispatch")

    def test_rejects_named_view_model_binding_as_non_default(self):
        actions, blocker = generate_manifest.executable_actions(
            """
            auto vm = file->viewModel("ViewModel1");
            auto vmi = file->createDefaultViewModelInstance(vm);
            stateMachine->bindViewModelInstance(vmi);
            artboard->draw(renderer.get());
            """,
            "default",
            "none",
        )
        self.assertEqual(actions, ())
        self.assertEqual(blocker, "named-view-model-instance")

    def test_encodes_literal_pointer_events_in_source_order(self):
        chunk = """
        stateMachine->pointerMove(rive::Vec2D(10.5f, -2), 0.25f, 7);
        stateMachine->pointerDown(Vec2D(10.5f, -2), 7);
        stateMachine->advanceAndApply(0.016f);
        stateMachine->pointerUp(rive::Vec2D(12, 3));
        stateMachine->pointerExit(rive::Vec2D(12, 3), 7);
        artboard->draw(renderer.get());
        """
        actions, blocker = generate_manifest.executable_actions(
            chunk, "default", "none"
        )
        self.assertIsNone(blocker)
        self.assertEqual(
            actions,
            (
                {
                    "kind": "pointer-move",
                    "x": 10.5,
                    "y": -2.0,
                    "seconds": 0.25,
                    "pointer_id": 7,
                },
                {"kind": "pointer-down", "x": 10.5, "y": -2.0, "pointer_id": 7},
                {"kind": "advance", "target": "state-machine", "seconds": 0.016},
                {"kind": "pointer-up", "x": 12.0, "y": 3.0, "pointer_id": 0},
                {"kind": "pointer-exit", "x": 12.0, "y": 3.0, "pointer_id": 7},
                {"kind": "draw"},
            ),
        )

    def test_rejects_pointer_expressions_that_cannot_be_evaluated(self):
        actions, blocker = generate_manifest.executable_actions(
            """
            stateMachine->pointerDown(rive::Vec2D(artboard->width() / 2, 10));
            artboard->draw(renderer.get());
            """,
            "default",
            "none",
        )
        self.assertEqual(actions, ())
        self.assertEqual(blocker, "pointer-expression-encoding")

    def test_render_pins_lane_and_provenance_ratchets(self):
        producer = generate_manifest.Producer(
            id="placeholder",
            source="placeholder.riv",
            dependencies=(),
            artboard="default",
            animation="none",
            state_machine="default",
            lane="runtime",
            deterministic="cpp-test-defined",
            random="cpp-test-defined",
            view_model="none",
            sample_times=(),
            actions="cpp-test-body",
            status="pending",
            producer_class="runtime-literal",
            provenance_file="tests/unit_tests/runtime/example.cpp",
            provenance_test="example",
            producer_line=1,
            note="pending",
        )
        with self.assertRaisesRegex(ValueError, "ratchet mismatch"):
            generate_manifest.render([producer])

    def test_checked_in_manifest_is_generated_from_pinned_upstream(self):
        runtime_dir = Path("/Users/levi/dev/oss/rive-runtime")
        manifest = Path(__file__).resolve().parents[2] / "silver-corpus.toml"
        if not runtime_dir.is_dir() or not manifest.is_file():
            self.skipTest("pinned upstream or checked-in manifest is unavailable")
        expected = generate_manifest.render(generate_manifest.discover(runtime_dir))
        self.assertEqual(manifest.read_text(encoding="utf-8"), expected)


if __name__ == "__main__":
    unittest.main()
