import copy
import importlib.util
import sys
import unittest
from pathlib import Path


TOOL = Path(__file__).with_name("perf_gate.py")
REPO_ROOT = TOOL.parents[2]
SPEC = importlib.util.spec_from_file_location("perf_gate", TOOL)
assert SPEC is not None and SPEC.loader is not None
PERF_GATE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = PERF_GATE
SPEC.loader.exec_module(PERF_GATE)


class PerfCorpusTests(unittest.TestCase):
    def setUp(self):
        self.manifest = PERF_GATE.load_manifest(REPO_ROOT / "perf-corpus.toml")
        self.corpus = PERF_GATE.load_toml(REPO_ROOT / "corpus.toml")

    def test_checked_in_manifest_is_broad_and_diverse(self):
        PERF_GATE.validate_manifest(
            self.manifest,
            self.corpus,
            corpus_path=REPO_ROOT / "corpus.toml",
            rive_runtime_dir=None,
        )
        self.assertGreaterEqual(len(self.manifest.files), 20)

    def test_manifest_rejects_a_parked_source_entry(self):
        changed = copy.deepcopy(self.corpus)
        selected = {file.id for file in self.manifest.files}
        entry = next(file for file in changed["file"] if file["id"] in selected)
        entry["status"] = "diverges"

        with self.assertRaisesRegex(ValueError, "must remain exact"):
            PERF_GATE.validate_manifest(
                self.manifest,
                changed,
                corpus_path=REPO_ROOT / "corpus.toml",
                rive_runtime_dir=None,
            )

    def test_manifest_rejects_missing_feature_diversity(self):
        files = tuple(
            PERF_GATE.PerfFile(file.id, file.file_bytes, ("largest",), file.note)
            for file in self.manifest.files
        )
        manifest = PERF_GATE.PerfManifest(
            self.manifest.source, self.manifest.minimum_files, files
        )

        with self.assertRaisesRegex(ValueError, "missing required diversity"):
            PERF_GATE.validate_manifest(
                manifest,
                self.corpus,
                corpus_path=REPO_ROOT / "corpus.toml",
                rive_runtime_dir=None,
            )


if __name__ == "__main__":
    unittest.main()
