import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
REMOVED_CRATES = (
    "nux-apple-runtime",
    "nuxie-apple-adapter",
    "nuxie-product",
    "nuxie-product-scripting",
    "nux-container",
)
REMOVED_MIGRATION_FILES = (
    "crates/nux-capi/exports-v3-legacy-migration.txt",
    "crates/nux-capi/include/module.migration.modulemap",
    "crates/nux-capi/smoke/distribution_legacy_consumer.c",
    "tools/build-apple-xcframework.sh",
    "tools/check-nux-capi-migration.sh",
    "tools/publish-apple-runtime-release.sh",
    "tools/test-apple-runtime-build-identity.sh",
    "tools/verify-apple-xcframework.sh",
    "tests/ExperienceRuntimeHostApp",
)


class SlimRuntimeSourceTests(unittest.TestCase):
    def test_legacy_product_crates_and_distribution_files_are_deleted(self) -> None:
        for crate in REMOVED_CRATES:
            self.assertFalse((REPO_ROOT / "crates" / crate).exists(), crate)
        for relative in REMOVED_MIGRATION_FILES:
            self.assertFalse((REPO_ROOT / relative).exists(), relative)

    def test_nux_capi_is_the_only_distribution_root(self) -> None:
        manifest = (REPO_ROOT / "crates/nux-capi/Cargo.toml").read_text()
        self.assertIn('version = "0.5.0"', manifest)
        self.assertNotIn("legacy-migration", manifest)
        for crate in REMOVED_CRATES:
            self.assertNotIn(crate, manifest)

        plan = subprocess.run(
            [str(REPO_ROOT / "tools/build-nux-capi-xcframeworks.sh"), "--plan"],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=True,
        ).stdout
        self.assertIn("root-package: nux-capi", plan)
        self.assertIn("feature-set: apple-metal,scripting", plan)
        self.assertNotIn("legacy", plan.lower())

    def test_distribution_exposes_one_module_and_two_symbol_partitions(self) -> None:
        module_map = (REPO_ROOT / "crates/nux-capi/include/module.modulemap").read_text()
        self.assertEqual(module_map.count("module "), 1)
        self.assertIn("module NuxieRuntimeC", module_map)
        self.assertNotIn("NuxieRuntimeFFI", module_map)
        self.assertNotIn("NuxieRuntimeInternal", module_map)
        for smoke in ("capi_lifetime.swift", "capi_metal_smoke.swift"):
            source = (REPO_ROOT / "crates/nux-capi/smoke" / smoke).read_text()
            self.assertIn("import NuxieRuntimeC", source)
            self.assertNotIn("NuxieRuntimeInternal", source)

        manifests = sorted(
            path.name
            for path in (REPO_ROOT / "crates/nux-capi").glob("exports-v3-*.txt")
        )
        self.assertEqual(
            manifests,
            ["exports-v3-apple-metal-extension.txt", "exports-v3-portable.txt"],
        )

    def test_release_size_evidence_pins_the_exact_v040_baseline(self) -> None:
        baseline = json.loads(
            (
                REPO_ROOT
                / "crates/nux-capi/size-baseline-apple-runtime-v0.4.0.json"
            ).read_text()
        )
        self.assertEqual(baseline["releaseTag"], "apple-runtime-v0.4.0")
        self.assertEqual(
            baseline["sourceRevision"],
            "e2c8ecff2cd80f47b07909888a5fb3699593348d",
        )
        self.assertEqual(
            baseline["artifacts"]["full-apple"]["compressedBytes"], 82_078_954
        )
        self.assertEqual(
            baseline["artifacts"]["ios-only"]["compressedBytes"], 49_176_625
        )


class ShippedSurfaceGuardTests(unittest.TestCase):
    def run_guard(self, header: str, exports: str = "nux_file_import\n"):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "include").mkdir()
            (root / "include/nux_capi.generated.h").write_text(header)
            (root / "include/nux_capi.h").write_text('#include "nux_capi.generated.h"\n')
            (root / "include/nux_capi_apple.h").write_text('#include "nux_capi.h"\n')
            (root / "include/module.modulemap").write_text(
                'module NuxieRuntimeC { header "nux_capi_apple.h" export * }\n'
            )
            (root / "exports-v3-portable.txt").write_text(exports)
            (root / "exports-v3-apple-metal-extension.txt").write_text("")
            return subprocess.run(
                [
                    "python3",
                    str(REPO_ROOT / "tools/check-nux-capi-surface.py"),
                    "--contract-root",
                    str(root),
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
            )

    def test_guard_accepts_generic_runtime_and_blend_mode_screen(self) -> None:
        result = self.run_guard(
            "typedef enum { NUX_BLEND_MODE_SCREEN = 14 } NuxBlendMode;\n"
            "void nux_file_import(void);\n"
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_guard_rejects_each_retired_semantic_family(self) -> None:
        identifiers = (
            "NuxExperienceContext",
            "NuxScreenSession",
            "NuxProductSession",
            "nux_product_session_create",
            "NuxRuntimePackage",
            "NuxAuthenticationPolicy",
            "NuxJourneyDescriptor",
            "NuxSDKSession",
            "NuxieHostCommand",
            "NuxPackageAuthentication",
            "NuxFlowSession",
            "NuxieScriptHost",
            "nux_response_set",
        )
        for identifier in identifiers:
            with self.subTest(identifier=identifier):
                result = self.run_guard(
                    f"typedef struct {identifier} {identifier};\n"
                    "void nux_file_import(void);\n"
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(identifier, result.stderr)


if __name__ == "__main__":
    unittest.main()
