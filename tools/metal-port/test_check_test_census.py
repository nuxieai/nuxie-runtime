from __future__ import annotations

import importlib.util
import pathlib
import tempfile
import unittest
from unittest import mock


MODULE_PATH = pathlib.Path(__file__).with_name("check_test_census.py")
SPEC = importlib.util.spec_from_file_location("check_test_census", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
CENSUS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CENSUS)


class TestCensusTests(unittest.TestCase):
    def test_name_hash_is_sorted_and_newline_terminated(self) -> None:
        self.assertEqual(
            CENSUS.names_sha256(["b", "a"]),
            CENSUS.hashlib.sha256(b"a\nb\n").hexdigest(),
        )

    def test_zero_selection_fails_closed(self) -> None:
        harness = {
            "id": "empty",
            "cargo_args": ["-p", "example", "--lib"],
            "expected_total": 1,
            "expected_active": 1,
            "expected_ignored": [],
            "names_sha256": "unused",
            "active_names_sha256": "unused",
        }
        with mock.patch.object(CENSUS, "run_list", return_value=[]):
            with self.assertRaisesRegex(CENSUS.CensusError, "selected zero tests"):
                CENSUS.check_harness(pathlib.Path("."), harness)

    def test_all_harnesses_report_each_red_lane(self) -> None:
        harnesses = [{"id": "first"}, {"id": "second"}, {"id": "green"}]
        with mock.patch.object(
            CENSUS,
            "check_harness",
            side_effect=[
                CENSUS.CensusError("first: count drift"),
                CENSUS.CensusError("second: selected zero tests"),
                ["canonical_test"],
            ],
        ):
            selected, errors = CENSUS.check_all_harnesses(pathlib.Path("."), harnesses)
        self.assertEqual(selected, {"green:canonical_test"})
        self.assertEqual(
            errors,
            ["first: count drift", "second: selected zero tests"],
        )

    def test_source_ignore_hash_covers_name_reason_and_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            path = root / "tests.rs"
            path.write_text(
                '#[test]\n#[ignore = "external oracle"]\nfn diagnostic_probe() {}\n',
                encoding="utf-8",
            )
            rows = CENSUS.source_ignores(root, ["tests.rs"])
            self.assertEqual(rows, [("tests.rs", "diagnostic_probe", "external oracle")])
            self.assertNotEqual(
                CENSUS.source_ignore_sha256(rows),
                CENSUS.source_ignore_sha256([("tests.rs", "diagnostic_probe", "changed")]),
            )

    def test_live_metal_guard_hash_covers_each_ordered_call(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            path = root / "tests.rs"
            path.write_text(
                'crate::live_metal_test_unavailable("device");\n'
                'crate::live_metal_test_unavailable("queue");\n',
                encoding="utf-8",
            )
            rows = CENSUS.live_metal_guards(root, ["tests.rs"])
            self.assertEqual(rows, [("tests.rs", 1, "device"), ("tests.rs", 2, "queue")])
            self.assertNotEqual(
                CENSUS.live_metal_guard_sha256(rows),
                CENSUS.live_metal_guard_sha256([("tests.rs", 1, "device")]),
            )

    def test_ore_census_rejects_compiler_inert_paths_and_names(self) -> None:
        with self.assertRaisesRegex(CENSUS.CensusError, "canonical mechanical owners"):
            CENSUS.validate_canonical_ore_census(
                ["crates/nuxie-ore-metal/src/metal/context.rs"],
                set(),
            )
        with self.assertRaisesRegex(CENSUS.CensusError, "canonical mechanical modules"):
            CENSUS.validate_canonical_ore_census(
                [],
                {"ore-tools:metal::context::tests::live_context"},
            )
        CENSUS.validate_canonical_ore_census(
            [
                "crates/nuxie-ore-metal/src/mechanical_port/source/renderer/src/ore/metal/ore_context_metal_mm.rs"
            ],
            {
                "ore-tools:mechanical_port::source::renderer::src::ore::metal::ore_context_metal_mm::tests::live_context"
            },
        )


if __name__ == "__main__":
    unittest.main()
