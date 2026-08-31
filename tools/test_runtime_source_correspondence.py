import tempfile
import unittest
from pathlib import Path

from tools.check_runtime_source_correspondence import OWNER_ROOT, missing_owners, upstream_owners


class RuntimeSourceCorrespondenceTests(unittest.TestCase):
    def test_pairs_and_header_only_owners_are_derived_from_upstream(self):
        self.assertEqual(upstream_owners([
            "src/animation/foo.cpp", "include/rive/animation/foo.hpp",
            "include/rive/header_only.hpp", "src/generated/base.cpp",
            "renderer/src/foo.cpp", "tests/unit_tests/foo.cpp", "src/README.md",
        ]), {"animation/foo", "header_only", "generated/base"})

    def test_missing_or_empty_rust_owner_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            root = repo / OWNER_ROOT
            root.mkdir(parents=True)
            (root / "present.rs").write_text("pub struct Present;\n")
            (root / "empty.rs").write_text("\n")
            missing = missing_owners(repo, {"present", "empty", "absent"})
            self.assertEqual([row.split(" -> ")[0] for row in missing], ["absent", "empty"])

    def test_upstream_naming_exception_uses_existing_rust_owner(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            owner = repo / OWNER_ROOT / "text/text_engine.rs"
            owner.parent.mkdir(parents=True)
            owner.write_text("pub struct TextEngine;\n")
            self.assertEqual(missing_owners(repo, {"text_engine"}), [])


if __name__ == "__main__":
    unittest.main()
