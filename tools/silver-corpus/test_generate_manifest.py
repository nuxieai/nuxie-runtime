import tempfile
import unittest
from pathlib import Path

import generate_manifest


class SilverManifestGeneratorTests(unittest.TestCase):
    def test_cpp_comment_stripping_ignores_dead_matches_and_preserves_lines(self):
        source = '''
TEST_CASE("live", "[silver]")
{
    auto file = ReadRiveFile("assets/live.riv", &silver); // matches in strings stay
    CHECK(silver.matches("live"));
}
// TEST_CASE("moved", "[silver]")
// {
//     CHECK(silver.matches("stale-line"));
// }
/* TEST_CASE("also moved", "[silver]")
{
    CHECK(silver.matches("stale-block"));
} */
'''
        stripped = generate_manifest.strip_cpp_comments(source)
        chunks = generate_manifest.test_chunks(stripped)

        self.assertEqual(len(chunks), 1)
        self.assertEqual(chunks[0][0], "live")
        self.assertEqual(chunks[0][1], 2)
        self.assertEqual(
            generate_manifest.LITERAL_MATCH.findall(chunks[0][2]), ["live"]
        )
        self.assertEqual(stripped.count("\n"), source.count("\n"))

    def test_gamepad_actions_replay_the_complete_pinned_sequence(self):
        actions = generate_manifest.p2e_gamepad_actions("gamepad_test")
        self.assertIsNotNone(actions)
        self.assertEqual(actions[0]["kind"], "bind-default-view-model")
        self.assertEqual(
            sum(action["kind"] == "gamepad-batch" for action in actions), 19
        )
        self.assertEqual(sum(action["kind"] == "frame" for action in actions), 19)
        self.assertEqual(actions[-1]["kind"], "draw")
        self.assertIsNone(generate_manifest.p2e_gamepad_actions("another_case"))

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

    def test_dynamic_helper_entries_are_hand_authored(self):
        producers = generate_manifest.dynamic_producers()
        self.assertEqual(len(producers), 12)
        self.assertTrue(
            {
                "layout_grid_stack_grid_with_layout_participants",
                "layout_grid_stack_grid_with_layouts",
                "layout_grid_stack_grid_with_layouts_size_changing",
                "layout_grid_stack_grid_with_layouts_size_span_changing",
                "layout_grid_stack_grid_with_layouts_span",
                "layout_grid_stack_stack_with_layouts",
            }.issubset({producer.id for producer in producers})
        )
        grid = next(
            producer
            for producer in producers
            if producer.id == "layout_grid_stack_grid_with_layouts"
        )
        self.assertEqual(grid.artboard, "GridWithLayouts")
        self.assertEqual(len(grid.actions), 362)
        self.assertEqual(sum(action["kind"] == "frame" for action in grid.actions), 120)

        snap = next(
            producer
            for producer in producers
            if producer.id == "layout_scroll_snap_padding_layouts"
        )
        drag = next(
            producer
            for producer in producers
            if producer.id == "layout_scroll_drag_multiplier_layouts"
        )
        self.assertEqual(
            sum(action["kind"] == "pointer-move" for action in snap.actions), 5
        )
        self.assertEqual(
            sum(action["kind"] == "pointer-move" for action in drag.actions), 9
        )
        self.assertEqual(
            snap.actions[-1],
            {
                "kind": "advance-draw-until-scroll-physics-stops",
                "max_frames": 56,
                "seconds": 0.016,
            },
        )
        self.assertEqual(drag.actions[-1], snap.actions[-1])
        scroll = [
            producer
            for producer in producers
            if producer.producer_class == "layout-scroll-dynamic"
        ]
        self.assertEqual(len(scroll), 6)
        self.assertTrue(all(producer.status == "diverges" for producer in scroll))
        self.assertTrue(
            all("first difference:" in producer.note for producer in scroll)
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

    def test_list_path_port_has_all_eight_phases_and_sixty_live_frames(self):
        actions = generate_manifest.fl_e8_list_path_actions("list_to_path")
        self.assertIsNotNone(actions)
        self.assertEqual(
            sum(action.get("kind") == "frame" for action in actions),
            67,
        )
        self.assertEqual(
            sum(
                action.get("kind") == "set-view-model-list-item-number"
                and action.get("property") == "inRotation"
                for action in actions
            ),
            60,
        )
        self.assertEqual(
            sum(
                action.get("kind") == "set-view-model-list-item-number"
                and action.get("property") == "rotation"
                for action in actions
            ),
            61,
        )
        self.assertEqual(
            [
                action.get("view_model")
                for action in actions
                if action.get("kind") == "append-view-model-list-item"
            ],
            [
                "vertex-x-y",
                "vertex-x-y",
                "vertex-x-y",
                "vertex-x-y",
                "vertex-rotation-distance",
                "vertex-detached",
                "vertex-in-out",
                "non-vertex",
                "vertex-incomplete",
            ],
        )

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

    def test_encodes_artboard_relative_pointer_expressions(self):
        actions, blocker = generate_manifest.executable_actions(
            """
            stateMachine->pointerDown(rive::Vec2D(artboard->width() / 2, 10));
            artboard->draw(renderer.get());
            """,
            "default",
            "none",
        )
        self.assertIsNone(blocker)
        self.assertEqual(
            actions,
            (
                {
                    "kind": "pointer-down",
                    "x": "artboard-width/2",
                    "y": 10.0,
                    "pointer_id": 0,
                },
                {"kind": "draw"},
            ),
        )

    def test_rejects_pointer_variables_until_the_body_is_expanded(self):
        actions, blocker = generate_manifest.executable_actions(
            """
            stateMachine->pointerDown(rive::Vec2D(xPos, 10));
            artboard->draw(renderer.get());
            """,
            "default",
            "none",
        )
        self.assertEqual(actions, ())
        self.assertEqual(blocker, "pointer-expression-encoding")

    def test_ports_typed_view_model_mutations_in_cpp_order(self):
        actions = generate_manifest.p1q_view_model_actions("stateful_nested")
        self.assertIsNotNone(actions)
        mutations = [
            item
            for item in actions
            if item["kind"].startswith("set-view-model-")
        ]
        self.assertEqual(
            mutations,
            [
                {
                    "kind": "set-view-model-string",
                    "property": "btn1Label",
                    "value": "One",
                },
                {
                    "kind": "set-view-model-color",
                    "property": "btn1Tint",
                    "value": 0xFFFF3344,
                },
                {
                    "kind": "set-view-model-string",
                    "property": "btn2Label",
                    "value": "Two",
                },
                {
                    "kind": "set-view-model-color",
                    "property": "btn2Tint",
                    "value": 0xFF33AAFF,
                },
            ],
        )

    def test_ports_nested_view_model_paths_and_live_font_bytes(self):
        car_actions = generate_manifest.p1q_view_model_actions("car_widgets_v01")
        self.assertIn(
            {
                "kind": "set-view-model-number",
                "property": "COMPASS/Rotation",
                "value": 20.0,
            },
            car_actions,
        )
        self.assertIn(
            {
                "kind": "fire-view-model-trigger",
                "property": "Button/Pressed",
            },
            generate_manifest.p1q_view_model_actions("rewards_demo"),
        )
        self.assertIn(
            {
                "kind": "set-view-model-font-bytes",
                "property": "fontProperty",
                "source": "kablammo.ttf",
            },
            generate_manifest.p1q_view_model_actions("data_bind_font_test"),
        )

    def test_ports_word_joiner_mutations_as_utf8_strings(self):
        actions = generate_manifest.p1q_view_model_actions("word_joiner_test")
        text_values = [
            item["value"]
            for item in actions
            if item["kind"] == "set-view-model-string"
            and item["property"] == "txt1"
        ]
        self.assertEqual(len(text_values), 9)
        self.assertEqual(text_values[0], "123456789012345678901234567890")
        self.assertEqual(text_values[1].count("\u2060"), 9)
        self.assertEqual(text_values[2].count("\u2060"), 19)
        self.assertEqual(text_values[-1].count("\u2060"), 90)

    def test_expands_pointer_loop_variables_with_cpp_update_order(self):
        actions = generate_manifest.p1q_pointer_actions(
            "scroll_threshold-vertical-scroll"
        )
        pointer_actions = [
            item for item in actions if item["kind"].startswith("pointer-")
        ]
        self.assertEqual(pointer_actions[0]["y"], 70.0)
        self.assertEqual(pointer_actions[1]["y"], 70.0)
        self.assertEqual(pointer_actions[4]["y"], 46.0)
        self.assertEqual(pointer_actions[5]["y"], 38.0)
        self.assertTrue(
            all(item["x"] == "artboard-width/2" for item in pointer_actions)
        )

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
