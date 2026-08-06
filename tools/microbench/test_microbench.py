import hashlib
import importlib.util
import pathlib
import tempfile
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
TOOL_PATH = REPO_ROOT / "tools" / "microbench" / "microbench.py"


def load_tool():
    spec = importlib.util.spec_from_file_location("microbench", TOOL_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class MicrobenchContractTests(unittest.TestCase):
    def test_inventory_names_match_upstream_registry_one_for_one(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")

        self.assertEqual(
            [case.name for case in inventory.cases],
            [
                "BuildRawPath",
                "DrawCustomFeathers",
                "DrawFeatheredPaths_paper",
                "DrawOneChopStrokes",
                "DrawOneCuspStrokes",
                "DrawRiveRenderPaths",
                "DrawRiveRenderPathsAsRoundJoinStrokes",
                "DrawRiveRenderPathsAsStrokes",
                "DrawTwoChopStrokes",
                "DrawTwoCuspStrokes",
                "DrawZeroChopStrokes",
                "IntersectionBoardBench_marty",
                "IntersectionBoardBench_paper",
                "IntersectionTileBench",
                "IntersectionTileBenchWithOverlap",
                "IterateRawPath",
                "MapPointsAffine",
                "MapPointsScaleTrans",
                "MeasurePath",
                "RawPathBounds",
            ],
        )
        self.assertEqual(len({case.name for case in inventory.cases}), 20)
        self.assertEqual(
            {case.crate for case in inventory.cases},
            {"nuxie-runtime", "nuxie-renderer"},
        )

    def test_checked_in_datasets_match_declared_hashes_and_shapes(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")

        tool.check_datasets(REPO_ROOT, inventory)
        bbox_datasets = [dataset for dataset in inventory.datasets if dataset.kind == "i32-ltrb"]
        self.assertEqual(
            [(dataset.name, dataset.count) for dataset in bbox_datasets],
            [("paper_bboxes_6_copies", 19_305), ("marty_bboxes_187_copies", 12_377)],
        )

    def test_dataset_check_rejects_modified_bytes(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        dataset = inventory.datasets[0]

        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            target = root / dataset.path
            target.parent.mkdir(parents=True)
            target.write_bytes(b"not the upstream dataset")
            with self.assertRaisesRegex(tool.ContractError, "sha256"):
                tool.check_dataset(root, dataset)

    def test_converted_bbox_bytes_are_the_declared_little_endian_rows(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        dataset = inventory.datasets[0]

        with tempfile.TemporaryDirectory() as directory:
            source = pathlib.Path(directory) / "boxes.hpp"
            source.write_text("{ -1, 2, 3, 4 },\n{ 5, 6, 7, 8 },\n")
            local = dataset._replace(
                source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(), count=2
            )

            content = tool.converted_dataset_content(source, local)

        self.assertEqual(len(content), 32)
        self.assertEqual(content[:16].hex(), "ffffffff020000000300000004000000")

    def test_ratio_table_preserves_inventory_order(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        cpp = {case.name: index + 1.0 for index, case in enumerate(inventory.cases)}
        rust = {case.name: (index + 1.0) * 2.0 for index, case in enumerate(inventory.cases)}

        table = tool.render_ratio_table(inventory, cpp, rust)

        rows = [line for line in table.splitlines() if line.startswith("| `")]
        self.assertEqual(len(rows), 20)
        self.assertIn("| `BuildRawPath` |", rows[0])
        self.assertIn("| `RawPathBounds` |", rows[-1])
        self.assertTrue(all("2.000x" in row for row in rows))


if __name__ == "__main__":
    unittest.main()
