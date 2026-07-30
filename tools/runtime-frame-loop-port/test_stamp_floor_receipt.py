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
        receipts = sorted(
            (
                repo_root / "docs/runtime-frame-loop-fl-c5-evidence"
            ).glob("floor*.log")
        )
        self.assertEqual(len(receipts), 13)
        for receipt in receipts:
            with self.subTest(receipt=receipt.name):
                first_line = receipt.read_bytes().splitlines()[0]
                self.assertRegex(
                    first_line.decode("ascii"),
                    r"\AFLOOR_RECEIPT_TREE_SHA=[0-9a-f]{40}\Z",
                )

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
