import copy
import hashlib
import tempfile
import unittest
from pathlib import Path

from tools.apple_runtime_input_digest import DEFAULT_TARGETS, PACKAGING_INPUTS
from tools.apple_runtime_input_digest import InputDigestError, _tool_identities, build_manifest
from tools.bazel_runtime_build import APPLE_LABEL, canonical


class ConfiguredGraph:
    def __init__(self):
        self.calls = []
        self.evidence_document = {
            "bazel": "bazel 9.3.0",
            "rustc": "rustc 1.94.1 fixture",
            "platforms": {target: f"//bazel/platforms:{target}" for target in DEFAULT_TARGETS},
            "rustLibraries": {target: "a" * 64 for target in DEFAULT_TARGETS},
            "toolBinaries": {"bazel-server": "b" * 64, "rustc": "c" * 64},
            "files": [{"kind": "bazel-input", "path": "src/lib.rs", "sha256": "d" * 64}],
            "packages": [{"name": "nux-apple-product-extension", "targets": {
                target: ["apple-runtime"] for target in DEFAULT_TARGETS
            }}],
        }

    def evidence(self, *arguments, **options):
        self.calls.append((arguments, options))
        return copy.deepcopy(self.evidence_document)


class InputDigestTests(unittest.TestCase):
    def setUp(self):
        self.graph = ConfiguredGraph()
        self.configuration = {
            "buildProfile": "release-apple", "rustToolchain": "1.94.1",
            "minimumIOSVersion": "15.0", "minimumMacOSVersion": "12.0",
            "packagingTools": {"lipo": "e" * 64},
        }

    def manifest(self):
        return build_manifest(Path("/runtime"), self.configuration, bazel=self.graph)

    def test_queries_exact_shipping_root_platforms_and_deployment_versions(self):
        manifest = self.manifest()
        self.assertEqual(self.graph.calls, [
            ((APPLE_LABEL, DEFAULT_TARGETS, PACKAGING_INPUTS), {"ios": "15.0", "macos": "12.0"})
        ])
        self.assertEqual(manifest["schemaVersion"], 2)
        self.assertEqual(manifest["features"], ["apple-runtime"])
        self.assertEqual(manifest["rootPackage"], "nux-apple-product-extension")
        self.assertEqual(manifest["targets"], sorted(DEFAULT_TARGETS))

    def test_selected_toolchain_identity_replaces_caller_estimates(self):
        self.configuration["rustc"] = "an ambient compiler"
        manifest = self.manifest()
        self.assertEqual(manifest["configuration"]["rustc"], "rustc 1.94.1 fixture")
        self.assertEqual(manifest["configuration"]["toolBinaries"], {
            "bazel-server": "b" * 64, "rustc": "c" * 64, "lipo": "e" * 64
        })
        self.assertIn("packagingTools", self.configuration)
        self.assertNotIn("packagingTools", manifest["configuration"])

    def test_input_features_tools_and_target_libraries_each_bind_manifest_bytes(self):
        baseline = canonical(self.manifest())
        mutations = [
            ("files", [{"kind": "bazel-input", "path": "src/lib.rs", "sha256": "f" * 64}]),
            ("packages", [{"name": "nux-apple-product-extension", "targets": {
                target: ["apple-runtime", "extra-feature"] for target in DEFAULT_TARGETS
            }}]),
            ("toolBinaries", {"bazel-server": "f" * 64}),
            ("rustLibraries", {target: "f" * 64 for target in DEFAULT_TARGETS}),
        ]
        for key, value in mutations:
            with self.subTest(key=key):
                graph = ConfiguredGraph()
                graph.evidence_document[key] = value
                changed = build_manifest(Path("/runtime"), self.configuration, bazel=graph)
                self.assertNotEqual(canonical(changed), baseline)

    def test_same_configured_graph_produces_identical_canonical_bytes(self):
        self.assertEqual(canonical(self.manifest()), canonical(self.manifest()))

    def test_tool_content_not_its_machine_path_is_the_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            first = Path(directory) / "first"
            second = Path(directory) / "second"
            first.write_bytes(b"selected packaging tool")
            second.write_bytes(first.read_bytes())
            expected = hashlib.sha256(first.read_bytes()).hexdigest()
            self.assertEqual(_tool_identities([f"lipo={first}"]), {"lipo": expected})
            self.assertEqual(_tool_identities([f"lipo={first}"]), _tool_identities([f"lipo={second}"]))
            first.write_bytes(b"replacement packaging tool")
            self.assertNotEqual(_tool_identities([f"lipo={first}"]), {"lipo": expected})

    def test_duplicate_roles_relative_paths_and_missing_files_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            tool = Path(directory) / "tool"
            tool.write_bytes(b"tool")
            for arguments in ([f"lipo={tool}", f"lipo={tool}"], ["lipo=relative"], ["broken"]):
                with self.subTest(arguments=arguments), self.assertRaises(InputDigestError):
                    _tool_identities(arguments)
            with self.assertRaises(OSError):
                _tool_identities([f"lipo={tool.with_name('missing')}"])


if __name__ == "__main__":
    unittest.main()
