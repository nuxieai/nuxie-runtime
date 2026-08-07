import unittest
from pathlib import Path

from tools.apple_runtime_contract import ContractError
from tools.apple_runtime_contract import validate_metadata
from tools.apple_runtime_contract import validate_symbols


REPO_ROOT = Path(__file__).resolve().parents[1]


def valid_metadata() -> dict[str, object]:
    revision = "a" * 40
    return {
        "schemaVersion": 3,
        "runtimeVersion": "0.3.0",
        "sourceRevision": revision,
        "buildInputsHash": None,
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
    def test_exact_schema_three_metadata_passes(self) -> None:
        validate_metadata(valid_metadata())

    def test_wrong_schema_or_hidden_abi_field_fails(self) -> None:
        wrong_schema = valid_metadata()
        wrong_schema["schemaVersion"] = 2
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

    def test_build_inputs_hash_is_null_or_a_digest(self) -> None:
        metadata = valid_metadata()
        metadata["buildInputsHash"] = "d" * 64
        validate_metadata(metadata)

        metadata["buildInputsHash"] = "not-a-digest"
        with self.assertRaisesRegex(ContractError, "buildInputsHash"):
            validate_metadata(metadata)

    def test_unverifiable_source_revision_fails(self) -> None:
        metadata = valid_metadata()
        metadata["sourceRevision"] = "unknown"
        metadata["runtimeIdentity"] = "0.3.0@unknown"
        with self.assertRaisesRegex(ContractError, "sourceRevision"):
            validate_metadata(metadata)


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
        self.assertIn("apple-runtime-v0.3.0", self.documentation)
        self.assertIn(
            "releases/download/apple-runtime-v<crate-version>/"
            "NuxieRuntime.xcframework.zip",
            self.documentation,
        )
        self.assertIn("swift package compute-checksum", self.documentation)

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
