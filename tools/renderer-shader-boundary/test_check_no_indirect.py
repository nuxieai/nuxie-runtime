from __future__ import annotations

import importlib.util
import pathlib
import tempfile
import unittest


CHECK_PATH = pathlib.Path(__file__).with_name("check_no_indirect.py")
SPEC = importlib.util.spec_from_file_location("check_no_indirect", CHECK_PATH)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class NoIndirectRendererTest(unittest.TestCase):
    def check(self, source: str):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            (root / "renderer.rs").write_text(source, encoding="utf-8")
            return CHECK.findings(root)

    def test_direct_draw_and_dispatch_are_allowed(self) -> None:
        self.assertEqual(
            self.check("pass.draw(0..3, 0..1); pass.dispatch_workgroups(1, 1, 1);"),
            [],
        )

    def test_every_pinned_wgpu_indirect_entry_point_is_rejected(self) -> None:
        for identifier in sorted(CHECK.FORBIDDEN_IDENTIFIERS):
            with self.subTest(identifier=identifier):
                found = self.check(f"pass.{identifier}(buffer, 0);")
                self.assertEqual(len(found), 1)
                self.assertEqual(found[0][2], identifier)

    def test_ufcs_and_split_calls_cannot_bypass_the_ratchet(self) -> None:
        found = self.check(
            "let call = wgpu::RenderPass::draw_indirect;\n"
            "pass.dispatch_workgroups_indirect\n    (buffer, 0);\n"
        )
        self.assertEqual(
            [finding[2] for finding in found],
            ["draw_indirect", "dispatch_workgroups_indirect"],
        )

    def test_comments_and_literals_are_not_execution(self) -> None:
        self.assertEqual(
            self.check(
                "// pass.draw_indirect(buffer, 0)\n"
                "/* nested /* dispatch_workgroups_indirect */ comment */\n"
                'let ordinary = "multi_draw_indirect";\n'
                'let raw = r#"draw_indexed_indirect"#;\n'
                "let byte = b\"draw_mesh_tasks_indirect\";\n"
            ),
            [],
        )

    def test_longer_unrelated_identifiers_are_allowed(self) -> None:
        self.assertEqual(self.check("let draw_indirectly = false;"), [])

    def test_shader_module_creation_sites_are_counted_after_lexing(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            (root / "shader_catalog.rs").write_text(
                'device.create_shader_module(desc); // create_shader_module\n'
                'let text = "create_shader_module";\n',
                encoding="utf-8",
            )
            (root / "nested").mkdir()
            (root / "nested" / "new_pipeline.rs").write_text(
                "device.create_shader_module(desc);\n",
                encoding="utf-8",
            )

            self.assertEqual(
                CHECK.shader_module_creation_sites(root),
                {
                    pathlib.PurePosixPath("nested/new_pipeline.rs"): 1,
                    pathlib.PurePosixPath("shader_catalog.rs"): 1,
                },
            )


if __name__ == "__main__":
    unittest.main()
