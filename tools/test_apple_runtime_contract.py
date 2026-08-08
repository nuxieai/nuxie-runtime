import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from tools.apple_runtime_contract import ContractError
from tools.apple_runtime_contract import validate_build_inputs
from tools.apple_runtime_contract import validate_metadata
from tools.apple_runtime_contract import qualify_release_metadata
from tools.apple_runtime_contract import validate_distribution_metadata
from tools.apple_runtime_contract import validate_layout_oracle
from tools.apple_runtime_contract import validate_size_report
from tools.apple_runtime_contract import validate_symbol_partitions
from tools.apple_runtime_contract import validate_symbols


REPO_ROOT = Path(__file__).resolve().parents[1]


def valid_build_inputs() -> tuple[dict[str, object], bytes, str]:
    document = {
        "configuration": {
            "buildEnvironment": {},
            "buildProfile": "release-apple",
            "cargo": "cargo 1.94.1",
            "hostTarget": "aarch64-apple-darwin",
            "minimumIOSVersion": "15.0",
            "minimumMacOSVersion": "12.0",
            "rustToolchain": "1.94.1",
            "rustc": "rustc 1.94.1",
            "rustLibraries": {"aarch64-apple-ios": "1" * 64},
            "toolBinaries": {"cargo": "e" * 64, "rustc": "f" * 64},
            "sdk": {
                "iphoneOS": "26.2 (23C53)",
                "iphoneSimulator": "26.2 (23C53)",
                "macOS": "26.2 (25C56)",
            },
            "xcode": {"build": "17C52", "version": "26.2"},
        },
        "features": ["apple-product"],
        "files": [
            {"kind": "cargo-resolution", "path": "Cargo.lock", "sha256": "a" * 64}
        ],
        "packages": [
            {
                "checksum": None,
                "lockEntryHash": None,
                "manifestPath": "crates/nux-apple-runtime/Cargo.toml",
                "name": "nux-apple-runtime",
                "resolvedSourceHash": None,
                "source": None,
                "targets": {"aarch64-apple-ios": ["apple-product"]},
                "version": "0.3.0",
            }
        ],
        "rootPackage": "nux-apple-runtime",
        "schemaVersion": 1,
        "targets": sorted(
            (
                "aarch64-apple-darwin",
                "aarch64-apple-ios",
                "aarch64-apple-ios-sim",
                "x86_64-apple-darwin",
                "x86_64-apple-ios",
            )
        ),
    }
    encoded = (json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n").encode()
    return document, encoded, hashlib.sha256(encoded).hexdigest()


def valid_metadata() -> dict[str, object]:
    revision = "a" * 40
    return {
        "schemaVersion": 5,
        "runtimeVersion": "0.3.0",
        "buildSourceRevision": revision,
        "releaseRevision": revision,
        "buildInputsHash": "d" * 64,
        "buildInputsManifestPath": "NuxieRuntime.xcframework/BUILD_INPUTS.json",
        "runtimeIdentity": f"0.3.0@{revision}",
        "contractFingerprint": "b" * 64,
        "luaurVersion": "0.1.8",
        "buildProfile": "release-apple",
        "rustToolchain": "1.94.1",
        "xcodeVersion": "26.2",
        "xcodeBuild": "17C52",
        "iphoneOSSDKVersion": "26.2",
        "iphoneOSSDKBuild": "23C53",
        "iphoneSimulatorSDKVersion": "26.2",
        "iphoneSimulatorSDKBuild": "23C53",
        "macOSSDKVersion": "26.2",
        "macOSSDKBuild": "25C56",
        "minimumIOSVersion": "15.0",
        "minimumMacOSVersion": "12.0",
        "thirdPartyNoticesPath": (
            "NuxieRuntime.xcframework/THIRD_PARTY_NOTICES.md"
        ),
        "swiftPackageChecksum": "c" * 64,
    }


class MetadataTests(unittest.TestCase):
    def test_exact_schema_five_metadata_passes(self) -> None:
        validate_metadata(valid_metadata())

    def test_wrong_schema_or_hidden_abi_field_fails(self) -> None:
        wrong_schema = valid_metadata()
        wrong_schema["schemaVersion"] = 3
        with self.assertRaisesRegex(ContractError, "schemaVersion"):
            validate_metadata(wrong_schema)

        hidden_abi = valid_metadata()
        hidden_abi["abiMajor"] = 1
        with self.assertRaisesRegex(ContractError, "extra=.*abiMajor"):
            validate_metadata(hidden_abi)

    def test_identity_fingerprint_and_checksum_disagreement_fails(self) -> None:
        wrong_identity = valid_metadata()
        wrong_identity["runtimeIdentity"] = "0.3.0@" + "d" * 40
        with self.assertRaisesRegex(ContractError, "runtimeIdentity"):
            validate_metadata(wrong_identity)

        malformed_fingerprint = valid_metadata()
        malformed_fingerprint["contractFingerprint"] = "not-a-digest"
        with self.assertRaisesRegex(ContractError, "contractFingerprint"):
            validate_metadata(malformed_fingerprint)

        malformed_checksum = valid_metadata()
        malformed_checksum["swiftPackageChecksum"] = "not-a-digest"
        with self.assertRaisesRegex(ContractError, "swiftPackageChecksum"):
            validate_metadata(malformed_checksum)

    def test_build_inputs_hash_and_manifest_path_fail_closed(self) -> None:
        metadata = valid_metadata()
        metadata["buildInputsHash"] = "not-a-digest"
        with self.assertRaisesRegex(ContractError, "buildInputsHash"):
            validate_metadata(metadata)

        missing_hash = valid_metadata()
        missing_hash["buildInputsHash"] = None
        with self.assertRaisesRegex(ContractError, "buildInputsHash"):
            validate_metadata(missing_hash)

        wrong_path = valid_metadata()
        wrong_path["buildInputsManifestPath"] = "BUILD_INPUTS.json"
        with self.assertRaisesRegex(ContractError, "buildInputsManifestPath"):
            validate_metadata(wrong_path)

    def test_unverifiable_source_revision_fails(self) -> None:
        metadata = valid_metadata()
        metadata["buildSourceRevision"] = "unknown"
        metadata["runtimeIdentity"] = "0.3.0@unknown"
        with self.assertRaisesRegex(ContractError, "buildSourceRevision"):
            validate_metadata(metadata)

    def test_release_revision_can_advance_without_changing_build_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "artifact.json"
            metadata = valid_metadata()
            path.write_text(json.dumps(metadata))
            release_revision = "f" * 40
            qualify_release_metadata(path, release_revision)
            qualified = json.loads(path.read_text())
            self.assertEqual(qualified["releaseRevision"], release_revision)
            self.assertEqual(
                qualified["buildSourceRevision"], metadata["buildSourceRevision"]
            )
            self.assertEqual(qualified["runtimeIdentity"], metadata["runtimeIdentity"])
            self.assertEqual(
                qualified["swiftPackageChecksum"], metadata["swiftPackageChecksum"]
            )

    def test_failed_release_candidate_preserves_original_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            original = Path(directory) / "artifact.json"
            candidate = Path(directory) / ".artifact-qualified"
            original.write_text(json.dumps(valid_metadata()))
            original_bytes = original.read_bytes()
            shutil.copyfile(original, candidate)
            qualify_release_metadata(candidate, "f" * 40)

            # A verifier failure stops before the publisher's final atomic move.
            self.assertNotEqual(candidate.read_bytes(), original_bytes)
            self.assertEqual(original.read_bytes(), original_bytes)


class BuildInputManifestTests(unittest.TestCase):
    def test_exact_canonical_manifest_and_digest_pass(self) -> None:
        document, encoded, digest = valid_build_inputs()
        validate_build_inputs(document, encoded, digest)

    def test_missing_closure_or_stale_digest_fails_closed(self) -> None:
        document, encoded, digest = valid_build_inputs()
        incomplete = dict(document)
        incomplete["packages"] = []
        with self.assertRaisesRegex(ContractError, "dependency closure"):
            validate_build_inputs(incomplete, encoded, digest)

        with self.assertRaisesRegex(ContractError, "buildInputsHash"):
            validate_build_inputs(document, encoded, "f" * 64)

    def test_noncanonical_or_incomplete_target_manifest_fails(self) -> None:
        document, encoded, digest = valid_build_inputs()
        pretty = json.dumps(document, indent=2).encode()
        with self.assertRaisesRegex(ContractError, "canonical JSON"):
            validate_build_inputs(document, pretty, hashlib.sha256(pretty).hexdigest())

        missing_target = dict(document)
        missing_target["targets"] = document["targets"][:-1]
        with self.assertRaisesRegex(ContractError, "exact Apple target set"):
            validate_build_inputs(missing_target, encoded, digest)

    def test_environment_override_manifest_fails_closed(self) -> None:
        document, _, _ = valid_build_inputs()
        document["configuration"]["buildEnvironment"] = {"CC": "/tmp/compiler"}
        encoded = (json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n").encode()
        with self.assertRaisesRegex(ContractError, "forbidden environment overrides"):
            validate_build_inputs(document, encoded, hashlib.sha256(encoded).hexdigest())


class SymbolTests(unittest.TestCase):
    HEADER = """
NuxStatus nux_runtime_bind(const uint8_t *version,
                           uint64_t version_len);
void nux_operation_result_free(struct NuxOperationResult *result);
"""
    SYMBOLS = "_nux_operation_result_free\n_nux_runtime_bind\n_rust_eh_personality\n"

    def test_exact_public_symbol_set_passes(self) -> None:
        validate_symbols(self.HEADER, self.SYMBOLS)

    def test_missing_or_extra_public_symbol_fails(self) -> None:
        with self.assertRaisesRegex(ContractError, "missing=.*nux_runtime_bind"):
            validate_symbols(self.HEADER, "_nux_operation_result_free\n")
        with self.assertRaisesRegex(ContractError, "extra=.*nux_unpublished"):
            validate_symbols(
                self.HEADER,
                self.SYMBOLS + "_nux_unpublished\n",
            )

    def test_removed_client_abi_identifier_fails_even_when_exported(self) -> None:
        header = self.HEADER + (
            "\nuint16_t nux_runtime_abi_minor(void);\n"
            "#define NUX_RUNTIME_ABI_MINOR 6\n"
        )
        symbols = self.SYMBOLS + "_nux_runtime_abi_minor\n"
        with self.assertRaisesRegex(ContractError, "removed client ABI identifiers"):
            validate_symbols(header, symbols)


class MigrationDistributionTests(unittest.TestCase):
    def test_three_independent_symbol_partitions_are_exact_and_disjoint(self) -> None:
        validate_symbol_partitions(
            {
                "portable": "nux_file_free\nnux_player_step\n",
                "appleExtension": "nux_renderer_free\n",
                "legacyMigration": "nux_screen_session_free\n",
            },
            "_nux_file_free\n_nux_player_step\n_nux_renderer_free\n"
            "_nux_screen_session_free\n_rust_eh_personality\n",
        )

    def test_symbol_partitions_reject_overlap_unsorted_and_unlisted_exports(self) -> None:
        with self.assertRaisesRegex(ContractError, "overlap"):
            validate_symbol_partitions(
                {"portable": "nux_file_free\n", "appleExtension": "nux_file_free\n"},
                "_nux_file_free\n",
            )
        with self.assertRaisesRegex(ContractError, "sorted"):
            validate_symbol_partitions(
                {"portable": "nux_player_step\nnux_file_free\n"},
                "_nux_file_free\n_nux_player_step\n",
            )
        with self.assertRaisesRegex(ContractError, "extra=.*nux_unlisted"):
            validate_symbol_partitions(
                {"portable": "nux_file_free\n"},
                "_nux_file_free\n_nux_unlisted\n",
            )

    def test_schema_six_describes_both_artifacts_from_one_input_identity(self) -> None:
        revision = "a" * 40
        document = {
            "schemaVersion": 6,
            "runtimeVersion": "0.4.0",
            "buildSourceRevision": revision,
            "releaseRevision": revision,
            "runtimeIdentity": f"0.4.0@{revision}",
            "contractFingerprint": "b" * 64,
            "buildInputsHash": "c" * 64,
            "artifacts": [
                {
                    "kind": "full-apple",
                    "archiveName": "NuxieRuntime.xcframework.zip",
                    "bundleName": "NuxieRuntime.xcframework",
                    "swiftPackageChecksum": "d" * 64,
                    "targets": sorted(
                        [
                            "aarch64-apple-darwin",
                            "aarch64-apple-ios",
                            "aarch64-apple-ios-sim",
                            "x86_64-apple-darwin",
                            "x86_64-apple-ios",
                        ]
                    ),
                },
                {
                    "kind": "ios-only",
                    "archiveName": "NuxieRuntime-iOS.xcframework.zip",
                    "bundleName": "NuxieRuntime.xcframework",
                    "swiftPackageChecksum": "e" * 64,
                    "targets": sorted(
                        [
                            "aarch64-apple-ios",
                            "aarch64-apple-ios-sim",
                            "x86_64-apple-ios",
                        ]
                    ),
                },
            ],
        }
        validate_distribution_metadata(document)

        document["artifacts"][1]["swiftPackageChecksum"] = "d" * 64
        with self.assertRaisesRegex(ContractError, "distinct archive checksums"):
            validate_distribution_metadata(document)

    def test_layout_oracle_is_complete_sorted_and_lp64(self) -> None:
        validate_layout_oracle(
            {
                "schemaVersion": 1,
                "dataModel": "apple-lp64",
                "types": [
                    {
                        "name": "NuxStringView",
                        "size": 16,
                        "alignment": 8,
                        "fields": [
                            {"name": "data", "offset": 0},
                            {"name": "len", "offset": 8},
                        ],
                    }
                ],
            }
        )
        with self.assertRaisesRegex(ContractError, "field offsets"):
            validate_layout_oracle(
                {
                    "schemaVersion": 1,
                    "dataModel": "apple-lp64",
                    "types": [
                        {
                            "name": "NuxStringView",
                            "size": 16,
                            "alignment": 8,
                            "fields": [
                                {"name": "len", "offset": 8},
                                {"name": "data", "offset": 0},
                            ],
                        }
                    ],
                }
            )

    def test_size_report_supports_candidate_and_release_budget_modes(self) -> None:
        full_slices = {
            "aarch64-apple-darwin": 7,
            "aarch64-apple-ios": 7,
            "aarch64-apple-ios-sim": 7,
            "x86_64-apple-darwin": 7,
            "x86_64-apple-ios": 7,
        }
        ios_slices = {
            "aarch64-apple-ios": 7,
            "aarch64-apple-ios-sim": 7,
            "x86_64-apple-ios": 7,
        }
        report = {
            "schemaVersion": 1,
            "artifacts": {
                "full-apple": {
                    "compressedBytes": 10,
                    "expandedBytes": 20,
                    "representativeLinkedBytes": 4,
                    "sliceBytes": full_slices,
                },
                "ios-only": {
                    "compressedBytes": 8,
                    "expandedBytes": 12,
                    "representativeLinkedBytes": 4,
                    "sliceBytes": ios_slices,
                },
            },
        }
        candidate = {
            "schemaVersion": 1,
            "mode": "candidate",
            "maximums": None,
        }
        validate_size_report(report, candidate, release=False)
        with self.assertRaisesRegex(ContractError, "release size budgets"):
            validate_size_report(report, candidate, release=True)

        release = {
            "schemaVersion": 1,
            "mode": "release",
            "maximums": {
                "full-apple": {
                    "compressedBytes": 9,
                    "expandedBytes": 20,
                    "representativeLinkedBytes": 4,
                    "sliceBytes": full_slices,
                },
                "ios-only": {
                    "compressedBytes": 8,
                    "expandedBytes": 12,
                    "representativeLinkedBytes": 4,
                    "sliceBytes": ios_slices,
                },
            },
        }
        with self.assertRaisesRegex(ContractError, "full-apple.compressedBytes"):
            validate_size_report(report, release, release=True)


class ReleaseToolSourcePolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.publisher = (
            REPO_ROOT / "tools/publish-apple-runtime-release.sh"
        ).read_text()
        self.documentation = (
            REPO_ROOT / "docs/apple-runtime-release.md"
        ).read_text()
        self.module_map = (
            REPO_ROOT / "crates/nux-apple-runtime/include/module.modulemap"
        ).read_text()
        self.verifier = (
            REPO_ROOT / "tools/verify-apple-xcframework.sh"
        ).read_text()

    def test_publisher_requires_exact_tag_and_main_ancestry(self) -> None:
        self.assertIn('expected_tag="apple-runtime-v${runtime_version}"', self.publisher)
        self.assertIn('tagged_revision="$(git -C "${repo_root}" rev-list -n 1', self.publisher)
        self.assertIn('"refs/tags/${release_tag}")"', self.publisher)
        self.assertIn('test "${source_revision}" = "${tagged_revision}"', self.publisher)
        self.assertIn('merge-base --is-ancestor', self.publisher)
        self.assertIn('"${source_revision}" refs/remotes/origin/main', self.publisher)

    def test_publisher_verifies_before_and_after_upload(self) -> None:
        verifier = '"${script_dir}/verify-apple-xcframework.sh"'
        self.assertEqual(self.publisher.count(verifier), 2)
        self.assertIn('release "${qualified_metadata}" "${source_revision}"', self.publisher)
        self.assertIn('cp "${metadata}" "${qualified_metadata}"', self.publisher)
        self.assertIn('mv "${qualified_metadata}" "${metadata}"', self.publisher)
        qualify = self.publisher.index('release "${qualified_metadata}"')
        verify = self.publisher.index(verifier, qualify)
        replace = self.publisher.index('mv "${qualified_metadata}" "${metadata}"')
        self.assertLess(qualify, verify)
        self.assertLess(verify, replace)
        self.assertIn('releaseRevision string)" = "${source_revision}"', self.publisher)
        self.assertNotIn(
            'buildSourceRevision string)" = "${source_revision}"', self.publisher
        )
        self.assertIn('[[ ! "${build_source_revision}" =~ ^[0-9a-f]{40}$ ]]', self.publisher)
        self.assertIn('"${build_source_revision}" "${source_revision}"', self.publisher)
        self.assertIn('"${release_revision}" != "${checkout_revision}"', self.verifier)
        self.assertIn('"${clean_build_source_revision}" "${release_revision}"', self.verifier)
        self.assertIn('gh release create "${release_tag}"', self.publisher)
        self.assertIn('gh release download "${release_tag}"', self.publisher)
        self.assertIn('cmp "${archive}" "${download_root}/NuxieRuntime.xcframework.zip"', self.publisher)

    def test_publisher_requires_release_qualified_build_inputs(self) -> None:
        for key, expected in (
            ("buildProfile", "release-apple"),
            ("rustToolchain", "1.94.1"),
            ("minimumIOSVersion", "15.0"),
            ("minimumMacOSVersion", "12.0"),
        ):
            self.assertIn(
                f'"${{metadata}}" {key} string)" = "{expected}"',
                self.publisher,
            )

    def test_documentation_names_the_stable_consumer_coordinates(self) -> None:
        self.assertIn("apple-runtime-v0.3.1", self.documentation)
        self.assertIn(
            "releases/download/apple-runtime-v<crate-version>/"
            "NuxieRuntime.xcframework.zip",
            self.documentation,
        )
        self.assertIn("swift package compute-checksum", self.documentation)

    def test_documentation_builds_from_the_exact_landed_commit(self) -> None:
        checkout = self.documentation.index(
            "git checkout --detach <exact-landed-origin-main-sha>"
        )
        build = self.documentation.index("make apple-runtime-xcframework", checkout)
        tag = self.documentation.index("git tag -a apple-runtime-v0.3.1", build)
        publish = self.documentation.index(
            "tools/publish-apple-runtime-release.sh apple-runtime-v0.3.1", tag
        )
        self.assertLess(checkout, build)
        self.assertLess(build, tag)
        self.assertLess(tag, publish)
        self.assertIn("Do not reuse a pre-merge artifact", self.documentation)

    def test_module_map_autolinks_every_apple_system_dependency(self) -> None:
        for framework in (
            "Foundation",
            "QuartzCore",
            "Metal",
            "CoreGraphics",
            "ImageIO",
            "Security",
        ):
            self.assertIn(f'link framework "{framework}"', self.module_map)
            self.assertNotIn(f'-framework {framework}', self.verifier)


if __name__ == "__main__":
    unittest.main()
