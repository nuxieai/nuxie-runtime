import json
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class DistributionToolTests(unittest.TestCase):
    def setUp(self) -> None:
        self.builder = (
            REPO_ROOT / "tools/build-nux-capi-xcframeworks.sh"
        ).read_text()
        self.publisher = (
            REPO_ROOT / "tools/publish-nux-capi-release.sh"
        ).read_text()
        self.verifier = (
            REPO_ROOT / "tools/verify-nux-capi-xcframeworks.sh"
        ).read_text()
        self.size_budgets = json.loads(
            (REPO_ROOT / "crates/nux-capi/size-budgets-v3.json").read_text()
        )
        self.capi_manifest = tomllib.loads(
            (REPO_ROOT / "crates/nux-capi/Cargo.toml").read_text()
        )
        self.apple_manifest = tomllib.loads(
            (REPO_ROOT / "crates/nux-apple-runtime/Cargo.toml").read_text()
        )

    def test_migration_distribution_composes_from_the_upper_apple_leaf(self) -> None:
        self.assertNotIn("legacy-migration", self.capi_manifest["features"])
        self.assertNotIn("nux-apple-runtime", self.capi_manifest["dependencies"])

        migration = self.apple_manifest["features"]["migration-distribution"]
        self.assertEqual(
            migration,
            [
                "apple-product",
                "product-configured-import",
            ],
        )
        self.assertEqual(
            self.apple_manifest["features"]["product-configured-import"],
            [
                "dep:nux-capi",
                "dep:nuxie-project-data",
                "nux-capi/apple-metal",
                "nux-capi/scripting",
            ],
        )
        capi = self.apple_manifest["dependencies"]["nux-capi"]
        self.assertEqual(capi["path"], "../nux-capi")
        self.assertFalse(capi["default-features"])
        self.assertTrue(capi["optional"])
        project_data = self.apple_manifest["dependencies"]["nuxie-project-data"]
        self.assertEqual(project_data["path"], "../nuxie-project-data")
        self.assertTrue(project_data["optional"])

        self.assertIn('--package nux-apple-runtime', self.builder)
        self.assertIn('--features migration-distribution', self.builder)
        self.assertIn('libnux_apple_runtime.a', self.builder)

    def test_five_thin_builds_are_reused_by_both_artifacts(self) -> None:
        self.assertEqual(self.builder.count('"${rust_cargo}" build'), 1)
        self.assertIn('for target in "${targets[@]}"', self.builder)
        self.assertIn('full/NuxieRuntime.xcframework', self.builder)
        self.assertIn('ios/NuxieRuntime.xcframework', self.builder)
        self.assertIn('NuxieRuntime-iOS.xcframework.zip', self.builder)
        self.assertEqual(self.builder.count('--package nux-apple-runtime'), 1)
        self.assertNotIn('--package nux-capi', self.builder)

    def test_build_strips_bitcode_and_uses_the_four_symbol_manifests(self) -> None:
        self.assertIn('--remove-section=__LLVM,__bitcode', self.builder)
        self.assertIn('--remove-section=__LLVM,__cmdline', self.builder)
        for manifest in (
            'exports-v3-portable.txt',
            'exports-v3-apple-metal-extension.txt',
            'exports-v3-product-extension.txt',
            'exports-v3-legacy-migration.txt',
        ):
            self.assertIn(manifest, self.builder)

    def test_publisher_is_guarded_and_uploads_both_assets_atomically(self) -> None:
        self.assertIn('status --porcelain', self.publisher)
        self.assertIn('rev-parse refs/remotes/origin/main', self.publisher)
        self.assertIn('ls-remote --exit-code origin', self.publisher)
        self.assertIn('test "${remote_tag_revision}" = "${source_revision}"', self.publisher)
        self.assertIn('gh release view', self.publisher)
        self.assertIn('gh release create', self.publisher)
        self.assertIn('--draft', self.publisher)
        self.assertIn('gh release edit "${release_tag}"', self.publisher)
        self.assertIn('--draft=false', self.publisher)
        self.assertLess(
            self.publisher.index('gh release download "${release_tag}"'),
            self.publisher.index('gh release edit "${release_tag}"'),
        )
        self.assertIn('NuxieRuntime.xcframework.zip', self.publisher)
        self.assertIn('NuxieRuntime-iOS.xcframework.zip', self.publisher)
        self.assertIn('SIZE_REPORT.json', self.publisher)
        self.assertIn('--release', self.publisher)
        self.assertIn('expected_tag="apple-runtime-v${runtime_version}"', self.publisher)

    def test_packaged_consumers_cover_composed_behavior_and_legacy_lane(self) -> None:
        self.assertIn("composed_script_asset.riv.base64", self.verifier)
        self.assertGreaterEqual(self.verifier.count("capi_metal_smoke.c"), 2)
        self.assertGreaterEqual(self.verifier.count("capi_metal_smoke.swift"), 2)
        self.assertIn('"${composed_fixture}" --composed', self.verifier)
        self.assertGreaterEqual(
            self.verifier.count("distribution_legacy_consumer.c"), 2
        )
        self.assertIn(
            'single_library "${full_framework}/macos-arm64_x86_64"', self.verifier
        )
        self.assertIn('= "arm64 x86_64"', self.verifier)
        self.assertIn("root_entries", self.verifier)
        self.assertIn('test "${entry_count}" = 2', self.verifier)
        self.assertIn("header-symbols", self.verifier)
        self.assertIn("slice-provenance", self.verifier)
        self.assertNotIn("-lnux_capi", self.verifier)

    def test_product_consumers_are_owned_by_the_upper_apple_leaf(self) -> None:
        apple_smoke = REPO_ROOT / "crates/nux-apple-runtime/smoke"
        capi_smoke = REPO_ROOT / "crates/nux-capi/smoke"
        self.assertTrue((apple_smoke / "distribution_legacy_consumer.c").is_file())
        self.assertTrue((apple_smoke / "distribution_migration_consumer.swift").is_file())
        self.assertFalse((capi_smoke / "distribution_legacy_consumer.c").exists())
        portable_swift = (capi_smoke / "distribution_consumer.swift").read_text()
        self.assertNotIn("NuxieRuntimeFFI", portable_swift)
        self.assertNotIn("nux_experience_", portable_swift)
        self.assertNotIn("nux_screen_session_", portable_swift)

    def test_release_size_budgets_are_frozen_for_both_artifacts(self) -> None:
        self.assertEqual(self.size_budgets["mode"], "release")
        self.assertEqual(
            set(self.size_budgets["maximums"]), {"full-apple", "ios-only"}
        )


if __name__ == "__main__":
    unittest.main()
