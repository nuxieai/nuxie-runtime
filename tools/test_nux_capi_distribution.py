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

    def test_five_thin_builds_are_reused_by_both_artifacts(self) -> None:
        self.assertEqual(self.builder.count('"${rust_cargo}" build'), 1)
        self.assertIn('for target in "${targets[@]}"', self.builder)
        self.assertIn('full/NuxieRuntime.xcframework', self.builder)
        self.assertIn('ios/NuxieRuntime.xcframework', self.builder)
        self.assertIn('NuxieRuntime-iOS.xcframework.zip', self.builder)
        self.assertEqual(self.builder.count('--package nux-capi'), 1)
        self.assertNotIn('--package nux-apple-runtime', self.builder)

    def test_build_strips_bitcode_and_uses_the_three_symbol_manifests(self) -> None:
        self.assertIn('--remove-section=__LLVM,__bitcode', self.builder)
        self.assertIn('--remove-section=__LLVM,__cmdline', self.builder)
        for manifest in (
            'exports-v3-portable.txt',
            'exports-v3-apple-metal-extension.txt',
            'exports-v3-legacy-migration.txt',
        ):
            self.assertIn(manifest, self.builder)

    def test_publisher_is_guarded_and_uploads_both_assets_atomically(self) -> None:
        self.assertIn('status --porcelain', self.publisher)
        self.assertIn('merge-base --is-ancestor', self.publisher)
        self.assertIn('gh release view', self.publisher)
        self.assertIn('gh release create', self.publisher)
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


if __name__ == "__main__":
    unittest.main()
