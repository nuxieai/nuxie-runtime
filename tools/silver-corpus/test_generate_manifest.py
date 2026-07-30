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
