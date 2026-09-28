import tempfile
import unittest
from pathlib import Path

from tools.check_runtime_source_correspondence import (
    ADAPTED_OWNERS, EDITOR_ONLY_OWNERS, OWNER_ROOT,
    PREPROCESSOR_ADAPTED_OWNERS, missing_owners, upstream_owners,
)


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

    def test_adapted_owner_requires_real_nonempty_implementation_file(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            name = "lua/lua_asset_reference"
            self.assertEqual(len(missing_owners(repo, {name})), 1)
            target = repo / ADAPTED_OWNERS[name]
            target.parent.mkdir(parents=True)
            target.write_text("\n")
            self.assertEqual(len(missing_owners(repo, {name})), 1)
            target.write_text("pub struct ScopedAssetReference;\n")
            self.assertEqual(missing_owners(repo, {name}), [])

    def test_deferred_scope_is_exact_not_a_wasm_directory_exemption(self):
        with tempfile.TemporaryDirectory() as directory:
            missing = missing_owners(Path(directory), {
                "wasm/wasm_scripting_vm", "scripting_slots", "wasm/new_owner",
            })
            self.assertEqual([row.split(" -> ")[0] for row in missing], ["wasm/new_owner"])

    def test_editor_scope_is_exact_and_does_not_exempt_runtime_id_types(self):
        self.assertEqual(EDITOR_ONLY_OWNERS, {
            "core/fractional_index", "core/field_types/core_fractional_index_type",
            "editor/core_handle", "editor/object_arena",
        })
        with tempfile.TemporaryDirectory() as directory:
            missing = missing_owners(Path(directory), EDITOR_ONLY_OWNERS | {
                "editor/new_owner", "core/id", "core/field_types/core_id_type",
            })
            self.assertEqual([row.split(" -> ")[0] for row in missing], [
                "core/field_types/core_id_type", "core/id", "editor/new_owner",
            ])

    def test_preprocessor_adaptation_requires_existing_callback_owner(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            name = "core/editor_hooks"
            self.assertEqual(len(missing_owners(repo, {name})), 1)
            target = repo / PREPROCESSOR_ADAPTED_OWNERS[name]
            target.parent.mkdir(parents=True)
            target.write_text("\n")
            self.assertEqual(len(missing_owners(repo, {name})), 1)
            target.write_text("pub struct ComponentBase;\n")
            self.assertEqual(missing_owners(repo, {name}), [])


if __name__ == "__main__":
    unittest.main()
