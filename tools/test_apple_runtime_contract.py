import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from tools.apple_runtime_contract import ContractError
from tools.apple_runtime_contract import qualify_release_metadata
from tools.apple_runtime_contract import validate_build_inputs
from tools.apple_runtime_contract import validate_distribution_metadata
from tools.apple_runtime_contract import validate_header_symbol_partitions
from tools.apple_runtime_contract import validate_layout_oracle
from tools.apple_runtime_contract import validate_size_report
from tools.apple_runtime_contract import validate_slice_provenance
from tools.apple_runtime_contract import validate_symbol_partitions


APPLE_TARGETS = sorted(
    {
        "aarch64-apple-darwin",
        "aarch64-apple-ios",
        "aarch64-apple-ios-sim",
        "x86_64-apple-darwin",
        "x86_64-apple-ios",
    }
)
IOS_TARGETS = sorted(
    {"aarch64-apple-ios", "aarch64-apple-ios-sim", "x86_64-apple-ios"}
)


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
        "features": ["apple-metal", "scripting"],
        "files": [
            {"kind": "cargo-resolution", "path": "Cargo.lock", "sha256": "a" * 64}
        ],
        "packages": [
            {
                "checksum": None,
                "lockEntryHash": None,
                "manifestPath": "crates/nux-capi/Cargo.toml",
                "name": "nux-capi",
                "resolvedSourceHash": None,
                "source": None,
                "targets": {target: ["apple-metal", "scripting"] for target in APPLE_TARGETS},
                "version": "0.5.0",
            }
        ],
        "rootPackage": "nux-capi",
        "schemaVersion": 1,
        "targets": APPLE_TARGETS,
    }
    encoded = (json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n").encode()
    return document, encoded, hashlib.sha256(encoded).hexdigest()


def valid_distribution_metadata() -> dict[str, object]:
    revision = "a" * 40
    return {
        "schemaVersion": 6,
        "runtimeVersion": "0.5.0",
        "buildSourceRevision": revision,
        "releaseRevision": revision,
        "runtimeIdentity": f"0.5.0@{revision}",
        "contractFingerprint": "b" * 64,
        "buildInputsHash": "c" * 64,
        "artifacts": [
            {
                "kind": "full-apple",
                "archiveName": "NuxieRuntime.xcframework.zip",
                "bundleName": "NuxieRuntime.xcframework",
                "swiftPackageChecksum": "d" * 64,
                "targets": APPLE_TARGETS,
            },
            {
                "kind": "ios-only",
                "archiveName": "NuxieRuntime-iOS.xcframework.zip",
                "bundleName": "NuxieRuntime.xcframework",
                "swiftPackageChecksum": "e" * 64,
                "targets": IOS_TARGETS,
            },
        ],
    }


def delta_metrics(after: dict[str, object], before: dict[str, object]) -> dict[str, object]:
    return {
        "compressedBytes": after["compressedBytes"] - before["compressedBytes"],
        "expandedBytes": after["expandedBytes"] - before["expandedBytes"],
        "representativeLinkedBytes": {
            key: after["representativeLinkedBytes"][key]
            - before["representativeLinkedBytes"][key]
            for key in after["representativeLinkedBytes"]
        },
        "sliceBytes": {
            key: after["sliceBytes"][key] - before["sliceBytes"][key]
            for key in after["sliceBytes"]
        },
    }


def valid_size_report() -> dict[str, object]:
    baseline_document = json.loads(
        (
            Path(__file__).resolve().parents[1]
            / "crates/nux-capi/size-baseline-apple-runtime-v0.4.0.json"
        ).read_text()
    )
    baseline_artifacts = baseline_document["artifacts"]
    artifacts = json.loads(json.dumps(baseline_artifacts))
    for metrics in artifacts.values():
        metrics["compressedBytes"] -= 10
        metrics["expandedBytes"] -= 10
        for mapped_metric in ("representativeLinkedBytes", "sliceBytes"):
            for label in metrics[mapped_metric]:
                metrics[mapped_metric][label] -= 10
    return {
        "schemaVersion": 2,
        "baseline": {
            key: baseline_document[key]
            for key in (
                "releaseTag",
                "sourceRevision",
                "sizeReportSha256",
                "artifacts",
            )
        },
        "artifacts": artifacts,
        "deltasFromBaseline": {
            kind: delta_metrics(artifacts[kind], baseline_artifacts[kind])
            for kind in artifacts
        },
    }


class BuildInputManifestTests(unittest.TestCase):
    def test_exact_slim_closure_passes(self) -> None:
        document, encoded, digest = valid_build_inputs()
        validate_build_inputs(document, encoded, digest)

    def test_root_features_and_retired_packages_fail_closed(self) -> None:
        for mutation, message in (
            (("rootPackage", "nux-apple-runtime"), "root package"),
            (("features", ["legacy-migration"]), "feature set"),
        ):
            document, _, _ = valid_build_inputs()
            document[mutation[0]] = mutation[1]
            encoded = (json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n").encode()
            with self.assertRaisesRegex(ContractError, message):
                validate_build_inputs(document, encoded, hashlib.sha256(encoded).hexdigest())

        for retired in (
            "nux-apple-runtime",
            "nux-container",
            "nuxie-apple-adapter",
            "nuxie-product",
            "nuxie-product-scripting",
            "nuxie-project-data",
        ):
            with self.subTest(retired=retired):
                document, _, _ = valid_build_inputs()
                package = dict(document["packages"][0])
                package["name"] = retired
                package["manifestPath"] = f"crates/{retired}/Cargo.toml"
                document["packages"].append(package)
                encoded = (json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n").encode()
                with self.assertRaisesRegex(ContractError, "retired package"):
                    validate_build_inputs(document, encoded, hashlib.sha256(encoded).hexdigest())

    def test_noncanonical_or_incomplete_inputs_fail(self) -> None:
        document, encoded, digest = valid_build_inputs()
        with self.assertRaisesRegex(ContractError, "buildInputsHash"):
            validate_build_inputs(document, encoded, "f" * 64)
        pretty = json.dumps(document, indent=2).encode()
        with self.assertRaisesRegex(ContractError, "canonical JSON"):
            validate_build_inputs(document, pretty, hashlib.sha256(pretty).hexdigest())


class DistributionContractTests(unittest.TestCase):
    def test_two_symbol_partitions_are_exact_and_disjoint(self) -> None:
        validate_symbol_partitions(
            {
                "portable": "nux_file_free\nnux_player_step\n",
                "appleExtension": "nux_renderer_free\n",
            },
            "_nux_file_free\n_nux_player_step\n_nux_renderer_free\n_rust_eh_personality\n",
        )
        with self.assertRaisesRegex(ContractError, "overlap"):
            validate_symbol_partitions(
                {"portable": "nux_file_free\n", "appleExtension": "nux_file_free\n"},
                "_nux_file_free\n",
            )

    def test_header_equals_the_two_manifests(self) -> None:
        header = "NuxStatus nux_file_free(NuxFile *file);\nNuxStatus nux_renderer_free(NuxRenderer *renderer);\n"
        manifests = {
            "portable": "nux_file_free\n",
            "appleExtension": "nux_renderer_free\n",
        }
        validate_header_symbol_partitions(header, manifests)
        with self.assertRaisesRegex(ContractError, "missing=.*nux_player_step"):
            validate_header_symbol_partitions(
                header, {**manifests, "extra": "nux_player_step\n"}
            )

    def test_slice_provenance_uses_only_shipping_features(self) -> None:
        metadata = valid_distribution_metadata()
        target = "aarch64-apple-ios"
        build_inputs = {
            "configuration": {"buildProfile": "release-apple", "rustc": "rustc 1.94.1"},
            "packages": [
                {"name": "nux-capi", "targets": {target: ["apple-metal", "scripting"]}}
            ],
        }
        provenance = {
            "schemaVersion": 6,
            "rootPackage": "nux-capi",
            "runtimeVersion": "0.5.0",
            "buildSourceRevision": "a" * 40,
            "target": target,
            "profile": "release-apple",
            "features": "apple-metal,scripting",
            "rustc": "rustc 1.94.1",
            "buildInputsHash": "c" * 64,
            "contractFingerprint": "b" * 64,
        }
        encoded = json.dumps(provenance, separators=(",", ":"))
        validate_slice_provenance(encoded, metadata, build_inputs, target)
        with self.assertRaisesRegex(ContractError, "exactly one"):
            validate_slice_provenance(f"{encoded}\n{encoded}", metadata, build_inputs, target)

    def test_distribution_metadata_and_release_qualification(self) -> None:
        metadata = valid_distribution_metadata()
        validate_distribution_metadata(metadata)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "artifact-set.json"
            path.write_text(json.dumps(metadata))
            qualify_release_metadata(path, "f" * 40)
            self.assertEqual(json.loads(path.read_text())["releaseRevision"], "f" * 40)

    def test_layout_oracle_is_sorted_and_lp64(self) -> None:
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

    def test_size_report_proves_exact_before_after_deltas(self) -> None:
        report = valid_size_report()
        candidate = {"schemaVersion": 1, "mode": "candidate", "maximums": None}
        validate_size_report(report, candidate, release=False)
        with self.assertRaisesRegex(ContractError, "before/after delta"):
            report["deltasFromBaseline"]["full-apple"]["compressedBytes"] += 1
            validate_size_report(report, candidate, release=False)

    def test_size_report_rejects_any_mutation_of_the_immutable_v04_baseline(
        self,
    ) -> None:
        candidate = {"schemaVersion": 1, "mode": "candidate", "maximums": None}
        mutations = (
            ("sourceRevision", "f" * 40),
            ("sizeReportSha256", "9" * 64),
        )
        for key, value in mutations:
            with self.subTest(key=key):
                report = valid_size_report()
                report["baseline"][key] = value
                with self.assertRaisesRegex(ContractError, "immutable v0.4.0"):
                    validate_size_report(report, candidate, release=False)

        report = valid_size_report()
        report["baseline"]["artifacts"]["full-apple"]["expandedBytes"] += 1
        report["deltasFromBaseline"]["full-apple"]["expandedBytes"] -= 1
        with self.assertRaisesRegex(ContractError, "immutable v0.4.0"):
            validate_size_report(report, candidate, release=False)


if __name__ == "__main__":
    unittest.main()
