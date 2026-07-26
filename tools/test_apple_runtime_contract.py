import unittest

from tools.apple_runtime_contract import ContractError
from tools.apple_runtime_contract import validate_metadata
from tools.apple_runtime_contract import validate_symbols


def valid_metadata() -> dict[str, object]:
    revision = "a" * 40
    return {
        "schemaVersion": 2,
        "runtimeVersion": "0.2.0",
        "sourceRevision": revision,
        "runtimeIdentity": f"0.2.0@{revision}",
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
        "minimumIOSVersion": "15.0",
        "thirdPartyNoticesPath": (
            "NuxieRuntime.xcframework/THIRD_PARTY_NOTICES.md"
        ),
        "swiftPackageChecksum": "c" * 64,
    }


class MetadataTests(unittest.TestCase):
    def test_exact_schema_two_metadata_passes(self) -> None:
        validate_metadata(valid_metadata())

    def test_wrong_schema_or_hidden_abi_field_fails(self) -> None:
        wrong_schema = valid_metadata()
        wrong_schema["schemaVersion"] = 1
        with self.assertRaisesRegex(ContractError, "schemaVersion"):
            validate_metadata(wrong_schema)

        hidden_abi = valid_metadata()
        hidden_abi["abiMajor"] = 1
        with self.assertRaisesRegex(ContractError, "extra=.*abiMajor"):
            validate_metadata(hidden_abi)

    def test_identity_fingerprint_and_checksum_disagreement_fails(self) -> None:
        wrong_identity = valid_metadata()
        wrong_identity["runtimeIdentity"] = "0.2.0@" + "d" * 40
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

    def test_unverifiable_source_revision_fails(self) -> None:
        metadata = valid_metadata()
        metadata["sourceRevision"] = "unknown"
        metadata["runtimeIdentity"] = "0.2.0@unknown"
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


if __name__ == "__main__":
    unittest.main()
