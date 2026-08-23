from __future__ import annotations

import importlib.util
import pathlib
import unittest


MODULE_PATH = pathlib.Path(__file__).with_name("check-native-metal-product-dependencies.py")
SPEC = importlib.util.spec_from_file_location("native_metal_product_dependencies", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class NativeMetalProductDependencyTests(unittest.TestCase):
    def test_accepts_native_only_normal_and_build_tree(self) -> None:
        lines = [
            "renderer-replay v0.1.0 (/repo/tools/renderer-replay)",
            "nuxie-renderer v0.1.0 (/repo/crates/nuxie-renderer)",
            "objc2-metal v0.3.2",
        ]
        self.assertEqual(CHECK.forbidden_packages(lines), [])

    def test_product_script_checks_normal_and_build_but_not_dev_edges(self) -> None:
        script = MODULE_PATH.with_name("check-native-metal-tracer-binary.sh").read_text()
        self.assertIn("-e normal,build", script)
        self.assertNotIn("-e normal,build,dev", script)

    def test_rejects_all_wgpu_naga_and_dawn_package_families(self) -> None:
        lines = [
            "wgpu v30.0.0 (/repo/vendor/wgpu)",
            "wgpu-core v30.0.0 (/repo/vendor/wgpu-core)",
            "wgpu-types v30.0.0",
            "naga v30.0.0",
            "naga-types v30.0.0",
            "dawn-native v1.0.0",
            "not-wgpu v1.0.0",
        ]
        self.assertEqual(
            CHECK.forbidden_packages(lines),
            ["dawn-native", "naga", "naga-types", "wgpu", "wgpu-core", "wgpu-types"],
        )

    def test_ignores_cargo_tree_annotations_and_non_package_text(self) -> None:
        lines = [
            "nuxie-renderer v0.1.0 (*)",
            "warning: a string mentions wgpu without being a package row",
            "overflow-gpu v1.0.0",
        ]
        self.assertEqual(CHECK.forbidden_packages(lines), [])


if __name__ == "__main__":
    unittest.main()
