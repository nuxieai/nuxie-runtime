from __future__ import annotations

import pathlib
import subprocess
import tempfile
import tomllib
import unittest


SCRIPT = pathlib.Path(__file__).with_name("refresh_receipts.py")
CHECK_SUFFIXES = {
    "source-review": "source-review.toml",
    "ownership-review": "ownership-review.toml",
    "fix": "fix.toml",
    "compile": "compile.toml",
    "verification": "verification.toml",
}


class RefreshReceiptsTests(unittest.TestCase):
    def test_refresh_binds_full_current_files_and_detects_later_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "repo"
            upstream = pathlib.Path(directory) / "upstream"
            receipt_dir = root / "docs" / "metal-port-receipts"
            artifact = root / "out.rs"
            source = upstream / "source.hpp"
            receipt_dir.mkdir(parents=True)
            artifact.write_text("line one\nline two\n")
            source.parent.mkdir(parents=True)
            source.write_text("source one\nsource two\nsource three\n")
            manifest = root / "manifest.toml"
            manifest.write_text(
                "\n".join(
                    [
                        '[[translation_unit]]',
                        'id = "unit"',
                        'sources = ["source.hpp"]',
                        'rust_targets = ["out.rs"]',
                        'artifact_targets = []',
                        '',
                    ]
                )
            )
            receipt = receipt_dir / "unit.translation.toml"
            receipt.write_text(
                "\n".join(
                    [
                        "schema_version = 1",
                        'unit = "unit"',
                        'receipt_kind = "translation"',
                        f'upstream_ref = "{"a" * 40}"',
                        f'workspace_base_ref = "{"b" * 40}"',
                        'role = "luna-extra-high"',
                        'open_findings = 0',
                        'omitted_lines = 0',
                        'omitted_declarations = 0',
                        'omitted_conditionals = 0',
                        'omitted_include_owners = 0',
                        'commands = ["stale :: exit=0 :: count=1"]',
                        'evidence = ["stale"]',
                        f'artifact_digests = {{ "out.rs" = "{"0" * 64}" }}',
                        f'source_digests = {{ "source.hpp" = "{"0" * 64}" }}',
                        '',
                    ]
                )
            )
            command = [
                "python3",
                str(SCRIPT),
                "--repo-root",
                str(root),
                "--upstream-root",
                str(upstream),
                "--manifest",
                str(manifest),
                "--kind",
                "translation",
            ]
            subprocess.run([*command, "--write"], check=True, capture_output=True, text=True)
            with receipt.open("rb") as handle:
                refreshed = tomllib.load(handle)
            self.assertEqual(
                refreshed["evidence"],
                ["cpp:source.hpp:1-3", "rust:out.rs:1-2"],
            )
            self.assertTrue(refreshed["commands"][0].endswith("count=2"))
            subprocess.run([*command, "--check"], check=True, capture_output=True, text=True)

            artifact.write_text("line one\nline two\nappended\n")
            result = subprocess.run(
                [*command, "--check"], capture_output=True, text=True
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("stale receipt", result.stderr)

    def test_create_missing_builds_every_final_receipt_kind(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "repo"
            upstream = pathlib.Path(directory) / "upstream"
            receipt_dir = root / "docs" / "metal-port-receipts"
            report_dir = root / "docs" / "metal-port-reports"
            receipt_dir.mkdir(parents=True)
            report_dir.mkdir(parents=True)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.email", "test@example.com"], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.name", "Test"], check=True)
            artifact = root / "out.rs"
            source = upstream / "source.hpp"
            artifact.write_text("translated\n")
            upstream.mkdir(parents=True)
            source.write_text("source\n")
            manifest = root / "manifest.toml"
            manifest.write_text(
                "\n".join(
                    [
                        "[[translation_unit]]",
                        'id = "unit"',
                        f'base_ref = "{"a" * 40}"',
                        'sources = ["source.hpp"]',
                        'rust_targets = ["out.rs"]',
                        "artifact_targets = []",
                        "",
                    ]
                )
            )
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            subprocess.run(["git", "-C", str(root), "commit", "-qm", "base"], check=True)
            kinds = ["source-review", "ownership-review", "fix", "compile", "verification"]
            command = [
                "python3",
                str(SCRIPT),
                "--repo-root",
                str(root),
                "--upstream-root",
                str(upstream),
                "--manifest",
                str(manifest),
                "--create-missing",
            ]
            for kind in kinds:
                command.extend(["--kind", kind])
            subprocess.run([*command, "--write"], check=True, capture_output=True, text=True)
            for kind in kinds:
                path = receipt_dir / f"unit.{CHECK_SUFFIXES[kind]}"
                self.assertTrue(path.is_file())
            with (receipt_dir / "unit.verification.toml").open("rb") as handle:
                verification = tomllib.load(handle)
            self.assertEqual(set(verification["suite_reports"]), {f"V{i}" for i in range(10)})


if __name__ == "__main__":
    unittest.main()
