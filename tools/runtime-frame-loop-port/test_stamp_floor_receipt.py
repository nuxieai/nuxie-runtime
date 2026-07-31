from __future__ import annotations

import importlib.util
import pathlib
import tempfile
import unittest


MODULE_PATH = pathlib.Path(__file__).with_name("stamp_floor_receipt.py")
SPEC = importlib.util.spec_from_file_location("stamp_floor_receipt", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
STAMP = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(STAMP)


class StampFloorReceiptTest(unittest.TestCase):
    def test_tracked_floor_receipts_carry_an_internal_tree_sha(self) -> None:
        repo_root = pathlib.Path(__file__).resolve().parents[2]
        receipts = STAMP.tracked_floor_receipts(repo_root)
        self.assertTrue(receipts)
        self.assertEqual(STAMP.validate_tracked_floor_receipts(repo_root), [])
        self.assertTrue(
            any("superseded" in receipt.parts for receipt in receipts),
            "the tracked set must recursively include retained superseded receipts",
        )

    def test_corrupting_any_tracked_receipt_fails_validation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            evidence = root / "docs/runtime-frame-loop-fl-c5-evidence"
            superseded = evidence / "superseded"
            superseded.mkdir(parents=True)
            (evidence / "floor-current.log").write_text("current\n")
            (superseded / "floor-old.log").write_text("old\n")
            import subprocess

            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            subprocess.run(
                ["git", "config", "user.email", "test@example.com"],
                cwd=root,
                check=True,
            )
            subprocess.run(
                ["git", "config", "user.name", "Test"],
                cwd=root,
                check=True,
            )
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(["git", "commit", "-qm", "fixture"], cwd=root, check=True)
            tree_sha = STAMP.resolve_tree_sha(root, None)
            receipts = STAMP.tracked_floor_receipts(root)
            for receipt in receipts:
                STAMP.stamp_receipt(receipt, receipt, tree_sha)

            self.assertEqual(STAMP.validate_tracked_floor_receipts(root), [])
            for corrupted in receipts:
                with self.subTest(receipt=corrupted.relative_to(root)):
                    for receipt in receipts:
                        STAMP.stamp_receipt(receipt, receipt, tree_sha)
                    corrupted.write_text(
                        "FLOOR_RECEIPT_TREE_SHA=not-a-commit\ncorrupt\n"
                    )
                    errors = STAMP.validate_tracked_floor_receipts(root)
                    self.assertEqual(len(errors), 1)
                    self.assertIn(corrupted.relative_to(root).as_posix(), errors[0])

    def test_stamp_is_inside_copy_and_replaces_an_existing_stamp(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            source = root / "raw.log"
            destination = root / "evidence.log"
            source.write_bytes(b"floor passed\n")

            first_sha = "1" * 40
            STAMP.stamp_receipt(source, destination, first_sha)
            self.assertEqual(
                destination.read_bytes(),
                f"FLOOR_RECEIPT_TREE_SHA={first_sha}\nfloor passed\n".encode(),
            )

            second_sha = "2" * 40
            STAMP.stamp_receipt(destination, destination, second_sha)
            self.assertEqual(
                destination.read_bytes(),
                f"FLOOR_RECEIPT_TREE_SHA={second_sha}\nfloor passed\n".encode(),
            )

    def test_invalid_sha_does_not_create_a_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            source = root / "raw.log"
            destination = root / "evidence.log"
            source.write_text("floor passed\n")

            with self.assertRaisesRegex(ValueError, "40 lowercase"):
                STAMP.stamp_receipt(source, destination, "not-a-tree")

            self.assertFalse(destination.exists())


if __name__ == "__main__":
    unittest.main()
