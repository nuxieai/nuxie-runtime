from __future__ import annotations

import importlib.util
import pathlib
import subprocess
import tempfile
import unittest


MODULE_PATH = pathlib.Path(__file__).with_name("check.py")
SPEC = importlib.util.spec_from_file_location("metal_port_check", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class MetalPortCheckTests(unittest.TestCase):
    def test_reference_provenance_is_bound_to_manifest_and_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            stream = root / "fixtures/scene.rive-stream"
            reference = root / "fixtures/scene.png"
            provenance = root / "fixtures/scene.provenance"
            stream.parent.mkdir(parents=True)
            stream.write_bytes(b"stream")
            reference.write_bytes(b"png")
            runtime_revision = "a" * 40
            input_manifest_sha256 = "b" * 64
            replay_sha256 = "c" * 64
            provenance.write_text(
                "\n".join(
                    [
                        "provenance_schema=1",
                        "renderer_implementation=cpp-native-metal",
                        "capture_tool=renderer-replay-ffi-metal",
                        "backend=metal",
                        "adapter_device=Test Metal Device",
                        "case_id=scene",
                        f"stream_sha256={CHECK.sha256_file(stream)}",
                        f"runtime_revision={runtime_revision}",
                        f"reference_input_manifest_sha256={input_manifest_sha256}",
                        f"replay_sha256={replay_sha256}",
                        f"png_sha256={CHECK.sha256_file(reference)}",
                        "frame=0",
                        "frame_width=64",
                        "frame_height=64",
                        "mode=clockwise-atomic",
                        "sample_count=1",
                    ]
                )
                + "\n"
            )
            subprocess.run(["git", "-C", str(root), "add", "fixtures"], check=True)
            manifest = {
                "upstream_ref": runtime_revision,
                "reference_provenance": [
                    {
                        "id": "scene",
                        "path": "fixtures/scene.provenance",
                        "stream": "fixtures/scene.rive-stream",
                        "reference": "fixtures/scene.png",
                        "renderer_implementation": "cpp-native-metal",
                        "capture_tool": "renderer-replay-ffi-metal",
                        "backend": "metal",
                        "adapter_device": "Test Metal Device",
                        "replay_sha256": replay_sha256,
                        "reference_input_manifest_sha256": input_manifest_sha256,
                        "frame": 0,
                        "frame_width": 64,
                        "frame_height": 64,
                        "mode": "clockwise-atomic",
                        "sample_count": 1,
                    }
                ],
            }

            errors: list[str] = []
            CHECK.validate_reference_provenance(manifest, root, errors)
            self.assertEqual(errors, [])

            stream.write_bytes(b"drifted stream")
            errors.clear()
            CHECK.validate_reference_provenance(manifest, root, errors)
            self.assertIn("stream_sha256", "\n".join(errors))

            stream.write_bytes(b"stream")
            provenance.write_text(
                provenance.read_text().replace(
                    f"replay_sha256={replay_sha256}", f"replay_sha256={'0' * 64}"
                )
            )
            errors.clear()
            CHECK.validate_reference_provenance(manifest, root, errors)
            self.assertIn("replay_sha256", "\n".join(errors))

    def test_scope_expansion_is_exhaustive_and_honors_exclusions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "renderer/src/metal"
            source.mkdir(parents=True)
            (source / "a.mm").write_text("a")
            (source / "b.h").write_text("b")
            self.assertEqual(
                CHECK.expand_source_scope(
                    root,
                    ["renderer/src/metal/*"],
                    ["renderer/src/metal/b.h"],
                ),
                {"renderer/src/metal/a.mm"},
            )

    def test_missing_upstream_source_and_unproved_port_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            upstream = root / "upstream"
            repo = root / "repo"
            (upstream / "renderer/src/metal").mkdir(parents=True)
            repo.mkdir()
            (upstream / "renderer/src/metal/a.mm").write_text("a")
            manifest = {
                "source_globs": ["renderer/src/metal/*"],
                "source_excludes": [],
                "source": [
                    {
                        "upstream": "renderer/src/metal/extra.mm",
                        "status": "ported",
                        "issue": "UNIV-2086",
                        "lane": "renderer-platform",
                        "rust_modules": [],
                        "evidence": [],
                    }
                ],
            }
            errors: list[str] = []
            CHECK.validate_source_rows(manifest, repo, upstream, errors)
            joined = "\n".join(errors)
            self.assertIn("untracked upstream Metal sources", joined)
            self.assertIn("out-of-scope source rows", joined)
            self.assertIn("without a Rust module", joined)
            self.assertIn("without verification evidence", joined)

    def test_citations_are_line_checked(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "source.mm"
            source.write_text("one\ntwo\n")
            errors: list[str] = []
            CHECK.validate_citation("cpp:source.mm:1-2", root, root, errors)
            self.assertEqual(errors, [])
            CHECK.validate_citation("cpp:source.mm:3", root, root, errors)
            self.assertIn("citation line is outside", errors[-1])

    def test_ownership_promotion_requires_existing_evidence_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "source.mm"
            source.write_text("source\n")
            ownership = {
                "owner": [
                    {
                        "id": "renderer.device",
                        "issue": "UNIV-2086",
                        "status": "ported",
                        "required_tests": ["device lifetime"],
                        "citations": ["cpp:source.mm:1"],
                        "evidence_paths": ["tests/missing.rs"],
                    }
                ]
            }
            errors: list[str] = []
            CHECK.validate_owner_rows(ownership, root, root, errors)
            self.assertIn("names missing evidence path", "\n".join(errors))

            ownership["owner"][0]["evidence_paths"] = []
            errors.clear()
            CHECK.validate_owner_rows(ownership, root, root, errors)
            self.assertIn("without concrete evidence paths", "\n".join(errors))

    def test_ownership_promotion_rejects_untracked_evidence_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            source = root / "source.mm"
            source.write_text("source\n")
            evidence = root / "tests/evidence.rs"
            evidence.parent.mkdir()
            evidence.write_text("evidence\n")
            ownership = {
                "owner": [
                    {
                        "id": "renderer.device",
                        "issue": "UNIV-2086",
                        "status": "verified",
                        "required_tests": ["device lifetime"],
                        "citations": ["cpp:source.mm:1"],
                        "evidence_paths": ["tests/evidence.rs"],
                    }
                ]
            }
            errors: list[str] = []
            CHECK.validate_owner_rows(ownership, root, root, errors)
            self.assertIn("names untracked evidence path", "\n".join(errors))

            subprocess.run(
                ["git", "-C", str(root), "add", "tests/evidence.rs"], check=True
            )
            errors.clear()
            CHECK.validate_owner_rows(ownership, root, root, errors)
            self.assertNotIn("evidence path", "\n".join(errors))

    def test_source_promotion_requires_tracked_modules_and_distinct_parity_evidence(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            upstream = root / "upstream"
            repo = root / "repo"
            source = upstream / "renderer/src/metal/a.mm"
            source.parent.mkdir(parents=True)
            source.write_text("source\n")
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            module = repo / "src/metal.rs"
            module.parent.mkdir(parents=True)
            module.write_text("module\n")
            evidence = repo / "tests/metal.rs"
            evidence.parent.mkdir()
            evidence.write_text("evidence\n")
            parity = repo / "docs/evidence/UNIV-2086.md"
            parity.parent.mkdir(parents=True)
            parity.write_text("parity\n")
            manifest = {
                "source_globs": ["renderer/src/metal/*"],
                "source_excludes": [],
                "source": [
                    {
                        "upstream": "renderer/src/metal/a.mm",
                        "status": "verified",
                        "issue": "UNIV-2086",
                        "lane": "renderer-platform",
                        "rust_modules": ["src/metal.rs"],
                        "evidence": ["tests/metal.rs"],
                        "parity_evidence": [],
                    }
                ],
            }
            errors: list[str] = []
            CHECK.validate_source_rows(manifest, repo, upstream, errors)
            joined = "\n".join(errors)
            self.assertIn("names untracked Rust module", joined)
            self.assertIn("names untracked evidence path", joined)
            self.assertIn("verified without parity evidence", joined)

            subprocess.run(
                [
                    "git",
                    "-C",
                    str(repo),
                    "add",
                    "src/metal.rs",
                    "tests/metal.rs",
                    "docs/evidence/UNIV-2086.md",
                ],
                check=True,
            )
            manifest["source"][0]["parity_evidence"] = [
                "docs/evidence/UNIV-2086.md"
            ]
            errors.clear()
            CHECK.validate_source_rows(manifest, repo, upstream, errors)
            self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
