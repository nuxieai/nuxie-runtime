import importlib.util
import json
import subprocess
import unittest
from pathlib import Path
from unittest import mock


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
        self.pipeline = (REPO_ROOT / ".buildkite/pipeline.yml").read_text()
        self.makefile = (REPO_ROOT / "Makefile").read_text()
        layout_checker_path = REPO_ROOT / "tools/check-nux-capi-layout.py"
        layout_spec = importlib.util.spec_from_file_location(
            "check_nux_capi_layout", layout_checker_path
        )
        assert layout_spec is not None and layout_spec.loader is not None
        self.layout_checker = importlib.util.module_from_spec(layout_spec)
        layout_spec.loader.exec_module(self.layout_checker)
        self.size_budgets = json.loads(
            (REPO_ROOT / "crates/nux-capi/size-budgets-v3.json").read_text()
        )
        self.portable_exports = (
            REPO_ROOT / "crates/nux-capi/exports-v3-portable.txt"
        ).read_text().splitlines()
        self.apple_exports = (
            REPO_ROOT / "crates/nux-capi/exports-v3-apple-metal-extension.txt"
        ).read_text().splitlines()
        self.abi_layout = json.loads(
            (REPO_ROOT / "crates/nux-capi/abi-layout-v3.json").read_text()
        )
        self.size_baseline = json.loads(
            (
                REPO_ROOT
                / "crates/nux-capi/size-baseline-apple-runtime-v0.4.0.json"
            ).read_text()
        )

    def test_five_thin_builds_are_reused_by_both_artifacts(self) -> None:
        self.assertEqual(self.builder.count('"${rust_cargo}" build'), 1)
        self.assertIn('for target in "${targets[@]}"', self.builder)
        self.assertIn('full/NuxieRuntime.xcframework', self.builder)
        self.assertIn('ios/NuxieRuntime.xcframework', self.builder)
        self.assertIn('NuxieRuntime-iOS.xcframework.zip', self.builder)
        self.assertEqual(self.builder.count('--package nux-capi'), 1)
        self.assertNotIn('--package nux-apple-runtime', self.builder)

    def test_build_strips_bitcode_and_uses_the_two_symbol_manifests(self) -> None:
        self.assertIn('--remove-section=__LLVM,__bitcode', self.builder)
        self.assertIn('--remove-section=__LLVM,__cmdline', self.builder)
        for manifest in (
            'exports-v3-portable.txt',
            'exports-v3-apple-metal-extension.txt',
        ):
            self.assertIn(manifest, self.builder)
        self.assertNotIn('exports-v3-legacy-migration.txt', self.builder)

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
        self.assertIn('Slim-runtime size comparison', self.publisher)
        self.assertIn('deltasFromBaseline', self.publisher)
        self.assertIn('--release', self.publisher)
        self.assertIn('expected_tag="apple-runtime-v${runtime_version}"', self.publisher)

    def test_packaged_consumers_cover_composed_behavior(self) -> None:
        self.assertIn("composed_script_asset.riv.base64", self.verifier)
        self.assertGreaterEqual(self.verifier.count("capi_metal_smoke.c"), 2)
        self.assertGreaterEqual(self.verifier.count("capi_metal_smoke.swift"), 2)
        self.assertIn('"${composed_fixture}" --composed', self.verifier)
        self.assertNotIn("distribution_legacy_consumer.c", self.verifier)
        self.assertNotIn("NuxieRuntimeFFI", self.verifier)
        self.assertIn(
            'single_library "${full_framework}/macos-arm64_x86_64"', self.verifier
        )
        self.assertIn('= "arm64 x86_64"', self.verifier)
        self.assertIn("root_entries", self.verifier)
        self.assertIn('test "${entry_count}" = 2', self.verifier)
        self.assertIn("header-symbols", self.verifier)
        self.assertIn("slice-provenance", self.verifier)
        self.assertIn("size-baseline-apple-runtime-v0.4.0.json", self.verifier)
        self.assertIn('"schemaVersion": 2', self.verifier)
        self.assertIn('"deltasFromBaseline"', self.verifier)
        self.assertIn("check-nux-capi-surface.py", self.verifier)
        self.assertNotIn("-lnux_capi", self.verifier)

    def test_packaged_consumers_cover_portable_scheduling_and_presentation_ack(self) -> None:
        symbols = {
            "nux_player_acknowledge_presented",
            "nux_player_step_result_scheduling",
        }
        self.assertTrue(symbols.issubset(self.portable_exports))
        self.assertTrue(symbols.isdisjoint(self.apple_exports))
        self.assertIn(
            "NuxPlayerSchedulingInfo",
            {record["name"] for record in self.abi_layout["types"]},
        )
        for consumer in (
            REPO_ROOT / "crates/nux-capi/smoke/distribution_consumer.c",
            REPO_ROOT / "crates/nux-capi/smoke/distribution_consumer.swift",
        ):
            source = consumer.read_text()
            for symbol in symbols:
                self.assertIn(symbol, source)

    def test_release_size_budgets_are_frozen_for_both_artifacts(self) -> None:
        self.assertEqual(self.size_budgets["mode"], "release")
        self.assertEqual(
            set(self.size_budgets["maximums"]), {"full-apple", "ios-only"}
        )
        self.assertEqual(self.size_baseline["releaseTag"], "apple-runtime-v0.4.0")
        self.assertEqual(
            self.size_baseline["sourceRevision"],
            "e2c8ecff2cd80f47b07909888a5fb3699593348d",
        )

    def test_pr_ci_runs_distribution_contract_tests_when_tooling_changes(self) -> None:
        c_abi_lane = self.pipeline.split(':linux: C ABI smoke"', 1)[1].split(
            ':mac: Apple distribution compile"', 1
        )[0]
        self.assertIn("make nux-capi-pr-gate", c_abi_lane)
        for required_input in (
            "build-nux-capi-xcframeworks.sh",
            "publish-nux-capi-release.sh",
            "verify-nux-capi-xcframeworks.sh",
            "report-all.sh",
            "check-nux-capi-exports.sh",
            "test_nux_capi_distribution.py",
            "test_apple_runtime_contract.py",
            "test_apple_runtime_input_digest.py",
            "test_slim_runtime_distribution.py",
        ):
            self.assertIn(required_input, c_abi_lane)

    def test_layout_checker_uses_a_cross_platform_clang_driver(self) -> None:
        with mock.patch.object(self.layout_checker.sys, "platform", "darwin"):
            self.assertEqual(
                self.layout_checker.clang_command(), ["xcrun", "clang"]
            )
        with mock.patch.object(self.layout_checker.sys, "platform", "linux"):
            self.assertEqual(
                self.layout_checker.clang_command(), ["clang", "-D__APPLE__"]
            )

    def test_distribution_gates_report_every_independent_verdict(self) -> None:
        contract_gate = self.makefile.split(
            "nux-capi-distribution-contract-gate:", 1
        )[1].split("\n\n", 1)[0]
        self.assertIn('tools/report-all.sh "nux-capi-distribution-contract"', contract_gate)
        for target in (
            "nux-capi-layout-contract",
            "nux-capi-surface-contract",
            "nux-capi-distribution-contract-test",
        ):
            self.assertIn(target, contract_gate)

        pr_gate = self.makefile.split("nux-capi-pr-gate:", 1)[1].split(
            "\n\n", 1
        )[0]
        self.assertIn('tools/report-all.sh "nux-capi-pr"', pr_gate)
        self.assertIn("capi-smoke", pr_gate)
        self.assertIn("nux-capi-distribution-contract-gate", pr_gate)

        result = subprocess.run(
            [
                str(REPO_ROOT / "tools/report-all.sh"),
                "distribution-probe",
                "first verdict",
                "printf first; exit 7",
                "second verdict",
                "printf second; exit 9",
            ],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("first", result.stdout)
        self.assertIn("second", result.stdout)
        self.assertIn("2 of 2 checks failed", result.stderr)


if __name__ == "__main__":
    unittest.main()
