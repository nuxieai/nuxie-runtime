import hashlib
import importlib.util
import json
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
    def test_inventory_names_match_pinned_upstream_registry_one_for_one(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        with tempfile.TemporaryDirectory() as directory:
            upstream = pathlib.Path(directory)
            for source in {case.source for case in inventory.cases}:
                target = upstream / source
                target.parent.mkdir(parents=True, exist_ok=True)
                names = [case.name for case in inventory.cases if case.source == source]
                target.write_text("\n".join(f"REGISTER_BENCH({name});" for name in names))
            self.assertEqual(
                tool.discover_upstream_cases(upstream, inventory),
                {case.name for case in inventory.cases},
            )

            first_source = upstream / inventory.cases[0].source
            first_source.write_text(first_source.read_text().replace("REGISTER_BENCH", "RENAMED"))
            with self.assertRaisesRegex(tool.ContractError, "upstream registry mismatch"):
                tool.check_upstream_case_contract(upstream, inventory)

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

    def test_report_only_emits_ratios_for_equivalent_cases(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        cpp = {case.name: index + 1.0 for index, case in enumerate(inventory.cases)}
        rust = {case.name: (index + 1.0) * 2.0 for index, case in enumerate(inventory.cases)}

        table = tool.render_report(inventory, cpp, rust)

        rows = [line for line in table.splitlines() if line.startswith("| `")]
        self.assertEqual(len(rows), 20)
        self.assertIn("| `BuildRawPath` |", rows[0])
        self.assertTrue(any("| `RawPathBounds` |" in row for row in rows))
        ratio_rows = [row for row in rows if "2.000x" in row]
        self.assertEqual(len(ratio_rows), 8)
        self.assertNotIn("2.000x", next(row for row in rows if "MapPointsAffine" in row))
        self.assertNotIn("2.000x", next(row for row in rows if "DrawRiveRenderPaths" in row))

    def test_criterion_uses_per_iteration_minimum_like_upstream_harness(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            criterion = pathlib.Path(directory)
            sample = criterion / "BuildRawPath" / "new" / "sample.json"
            sample.parent.mkdir(parents=True)
            sample.write_text(json.dumps({"iters": [1.0, 2.0, 4.0], "times": [9.0, 12.0, 40.0]}))

            self.assertEqual(tool.load_criterion_minimum(sample), 6.0)

    def test_run_manifest_rejects_mixed_or_stale_artifacts(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            artifact = root / "cpp.txt"
            artifact.write_text("artifact")
            run = {
                "status": "complete",
                "run_id": "test-run",
                "artifacts": {"cpp_output": {"path": str(artifact), "sha256": "wrong"}},
            }
            with self.assertRaisesRegex(tool.ContractError, "artifact hash mismatch"):
                tool.validate_run_artifacts(run)


if __name__ == "__main__":
    unittest.main()
