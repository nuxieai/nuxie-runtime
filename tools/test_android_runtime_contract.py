import copy
import json
import pathlib
import stat
import struct
import subprocess
import tempfile
import unittest
import zipfile

from tools.android_runtime_contract import ABIS
from tools.android_runtime_contract import ANDROID_API_LEVEL
from tools.android_runtime_contract import ANDROID_NDK_VERSION
from tools.android_runtime_contract import ARTIFACT_NAME
from tools.android_runtime_contract import CARGO_NDK_VERSION
from tools.android_runtime_contract import CONTRACT_INPUTS
from tools.android_runtime_contract import ContractError
from tools.android_runtime_contract import DISTRIBUTION_INPUTS
from tools.android_runtime_contract import EXPECTED_FILES
from tools.android_runtime_contract import FEATURES
from tools.android_runtime_contract import RELEASE_TAG
from tools.android_runtime_contract import RUST_TOOLCHAIN
from tools.android_runtime_contract import TARGETS
from tools.android_runtime_contract import TOOL_ROLES
from tools.android_runtime_contract import ZIP_TIMESTAMP
from tools.android_runtime_contract import canonical_json
from tools.android_runtime_contract import contract_fingerprint
from tools.android_runtime_contract import create_deterministic_zip
from tools.android_runtime_contract import export_partitions
from tools.android_runtime_contract import file_records
from tools.android_runtime_contract import layout_assertions
from tools.android_runtime_contract import metadata_document
from tools.android_runtime_contract import parse_needed
from tools.android_runtime_contract import parse_nm_symbols
from tools.android_runtime_contract import sha256_bytes
from tools.android_runtime_contract import size_report
from tools.android_runtime_contract import validate_budget
from tools.android_runtime_contract import validate_build_inputs
from tools.android_runtime_contract import validate_elf_header
from tools.android_runtime_contract import validate_headers
from tools.android_runtime_contract import validate_metadata
from tools.android_runtime_contract import validate_provenance
from tools.android_runtime_contract import validate_size_report
from tools.android_runtime_contract import validate_zip


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]


def valid_inputs() -> dict[str, object]:
    return {
        "configuration": {
            "androidApiLevel": ANDROID_API_LEVEL,
            "androidNdk": ANDROID_NDK_VERSION,
            "androidNdkHostTag": "darwin-x86_64",
            "androidNdkSourcePropertiesSha256": "5" * 64,
            "buildEnvironment": {},
            "buildProfile": "release",
            "cargo": f"cargo {RUST_TOOLCHAIN} (a" + "1" * 40 + ")",
            "cargoNdk": f"cargo-ndk {CARGO_NDK_VERSION}",
            "python": "Python 3.14.0; zlib 1.3.1",
            "rustToolchain": RUST_TOOLCHAIN,
            "rustc": f"rustc {RUST_TOOLCHAIN} (b" + "2" * 39 + ")",
            "sourceDateEpoch": 1_800_000_000,
        },
        "features": FEATURES,
        "files": [
            {"path": path, "sha256": "1" * 64}
            for path in sorted(set(DISTRIBUTION_INPUTS))
        ],
        "ndkRuntimeLibraries": {abi: "2" * 64 for abi in ABIS},
        "rootPackage": "nux-capi",
        "runtimeVersion": "0.9.0",
        "rustLibraries": {target: "3" * 64 for target in TARGETS},
        "schemaVersion": 1,
        "sourceRevision": "a" * 40,
        "targets": TARGETS,
        "tools": [
            {
                "name": role,
                "role": role,
                "sha256": "4" * 64,
                "version": (
                    f"cargo {RUST_TOOLCHAIN} (a" + "1" * 40 + ")"
                    if role == "cargo"
                    else f"cargo-ndk {CARGO_NDK_VERSION}"
                    if role == "cargo-ndk"
                    else f"rustc {RUST_TOOLCHAIN} (b" + "2" * 39 + ")"
                    if role == "rustc"
                    else "Python 3.14.0; zlib 1.3.1"
                    if role == "python"
                    else "version"
                ),
            }
            for role in sorted(TOOL_ROLES)
        ],
    }


def write_prebuilt(root: pathlib.Path) -> None:
    for index, relative in enumerate(EXPECTED_FILES):
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes((f"{index}:{relative}\n" * (index + 1)).encode())


def fake_elf(machine: int) -> bytes:
    value = bytearray(64)
    value[:7] = b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HH", value, 16, 3, machine)
    return bytes(value)


class BuildInputContractTests(unittest.TestCase):
    def test_exact_pinned_cut_passes(self) -> None:
        document = valid_inputs()
        encoded = canonical_json(document)
        self.assertEqual(validate_build_inputs(document, encoded), document)

    def test_every_pinned_dimension_fails_closed(self) -> None:
        mutations = (
            ("configuration", "androidApiLevel", 24),
            ("configuration", "androidNdk", "27.0.0"),
            ("configuration", "buildProfile", "release-size"),
            ("configuration", "cargoNdk", "cargo-ndk 4.1.1"),
            ("configuration", "rustToolchain", "stable"),
        )
        for _, key, replacement in mutations:
            with self.subTest(key=key):
                document = valid_inputs()
                document["configuration"][key] = replacement
                with self.assertRaises(ContractError):
                    validate_build_inputs(document)
        for key, replacement in (
            ("features", ["android-vulkan", "scripting"]),
            ("targets", ["aarch64-linux-android"]),
            ("sourceRevision", "a" * 8),
        ):
            with self.subTest(key=key):
                document = valid_inputs()
                document[key] = replacement
                with self.assertRaises(ContractError):
                    validate_build_inputs(document)

    def test_manifest_requires_canonical_bytes_and_every_tool(self) -> None:
        document = valid_inputs()
        with self.assertRaisesRegex(ContractError, "canonical JSON"):
            validate_build_inputs(document, json.dumps(document, indent=2).encode())
        document = valid_inputs()
        document["tools"] = document["tools"][:-1]
        with self.assertRaisesRegex(ContractError, "exact tool set"):
            validate_build_inputs(document)


class DeterministicArchiveTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temporary.name)
        self.prebuilt = self.root / "prebuilt"
        write_prebuilt(self.prebuilt)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_same_tree_produces_identical_exact_five_file_zip(self) -> None:
        first = self.root / "first.zip"
        second = self.root / "second.zip"
        create_deterministic_zip(self.prebuilt, first)
        create_deterministic_zip(self.prebuilt, second)
        self.assertEqual(first.read_bytes(), second.read_bytes())
        contents = validate_zip(first)
        self.assertEqual(list(contents), list(EXPECTED_FILES))
        with zipfile.ZipFile(first) as archive:
            self.assertEqual(len(archive.infolist()), 5)
            for record in archive.infolist():
                self.assertEqual(record.date_time, ZIP_TIMESTAMP)
                self.assertEqual(stat.S_IMODE(record.external_attr >> 16), 0o644)

    def test_extra_tree_file_and_noncanonical_zip_fail(self) -> None:
        extra = self.prebuilt / "README"
        extra.write_text("not shipped")
        with self.assertRaisesRegex(ContractError, "artifact tree differs"):
            create_deterministic_zip(self.prebuilt, self.root / "extra.zip")

        malformed = self.root / "malformed.zip"
        with zipfile.ZipFile(malformed, "w") as archive:
            archive.writestr("../escape", b"x")
        with self.assertRaisesRegex(ContractError, "exact five-file tree"):
            validate_zip(malformed)


class AbiAndElfContractTests(unittest.TestCase):
    def test_contract_fingerprint_covers_header_layout_and_all_four_partitions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            for relative in CONTRACT_INPUTS:
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(relative + "\n")
            baseline = contract_fingerprint(root)
            header = root / "crates/nux-capi/include/nux_capi.generated.h"
            header.write_text(header.read_text() + "changed\n")
            self.assertNotEqual(baseline, contract_fingerprint(root))

    def test_partition_inventory_rejects_unsorted_and_invalid_overlap(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "crates/nux-capi"
            root.mkdir(parents=True)
            files = {
                "exports-v4-portable.txt": "nux_a\n",
                "exports-v4-apple-metal-extension.txt": "nux_shared\n",
                "exports-v4-android-vulkan-extension.txt": "nux_shared\n",
                "exports-v4-android-authored-wgsl-extension.txt": "nux_wgsl\n",
            }
            for name, contents in files.items():
                (root / name).write_text(contents)
            partitions = export_partitions(root.parents[1])
            self.assertEqual(partitions["portable"], ["nux_a"])
            (root / "exports-v4-portable.txt").write_text("nux_z\nnux_a\n")
            with self.assertRaisesRegex(ContractError, "unique and sorted"):
                export_partitions(root.parents[1])
            (root / "exports-v4-portable.txt").write_text("nux_shared\n")
            with self.assertRaisesRegex(ContractError, "overlap"):
                export_partitions(root.parents[1])

    def test_elf64_architecture_is_exact(self) -> None:
        validate_elf_header(fake_elf(183), 183, "arm64")
        validate_elf_header(fake_elf(62), 62, "x86_64")
        with self.assertRaises(ContractError):
            validate_elf_header(fake_elf(62), 183, "wrong")

    def test_dynamic_dependencies_and_exports_parse_exactly(self) -> None:
        needed = parse_needed(
            "0x1 (NEEDED) Shared library: [libc++_shared.so]\n"
            "0x1 (NEEDED) Shared library: [libc.so]\n"
        )
        self.assertEqual(needed, {"libc++_shared.so", "libc.so"})
        symbols = parse_nm_symbols(
            "0000 T nux_file_import\n0001 T unrelated\n0002 T nux_player_step\n"
        )
        self.assertEqual(symbols, {"nux_file_import", "nux_player_step"})

    def test_full_and_android_selected_header_unions_are_distinct_and_exact(self) -> None:
        partitions = {
            "portable": ["nux_portable"],
            "androidVulkan": ["nux_android"],
            "androidAuthoredWgsl": ["nux_wgsl"],
            "appleMetal": ["nux_apple"],
        }
        raw = (
            "#define NUX_CAPI_ABI_VERSION 4\n"
            "void nux_portable(void); void nux_android(void);\n"
            "void nux_wgsl(void); void nux_apple(void);\n"
        )
        selected = "void nux_portable(void); void nux_android(void); void nux_wgsl(void);\n"
        self.assertEqual(
            validate_headers(raw, selected, partitions),
            {"nux_portable", "nux_android", "nux_wgsl"},
        )
        with self.assertRaisesRegex(ContractError, "ABI version 4"):
            validate_headers(raw.replace("VERSION 4", "VERSION 3"), selected, partitions)

    def test_layout_oracle_covers_full_header_but_emits_android_selected_types(self) -> None:
        raw = (
            "typedef struct NuxCommon { unsigned value; } NuxCommon;\n"
            "typedef struct NuxAppleOnly { void *device; } NuxAppleOnly;\n"
        )
        selected = "typedef struct NuxCommon { unsigned value; } NuxCommon;\n"
        oracle = {
            "schemaVersion": 1,
            "dataModel": "apple-lp64",
            "types": [
                {
                    "name": "NuxAppleOnly",
                    "size": 8,
                    "alignment": 8,
                    "fields": [{"name": "device", "offset": 0}],
                },
                {
                    "name": "NuxCommon",
                    "size": 4,
                    "alignment": 4,
                    "fields": [{"name": "value", "offset": 0}],
                },
            ],
        }
        source = layout_assertions(raw, oracle, selected)
        self.assertIn("sizeof(NuxCommon)", source)
        self.assertNotIn("sizeof(NuxAppleOnly)", source)
        oracle["types"][1]["fields"][0]["name"] = "wrong"
        with self.assertRaisesRegex(ContractError, "fields differ"):
            layout_assertions(raw, oracle, selected)


class EvidenceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temporary.name)
        prebuilt = self.root / "prebuilt"
        write_prebuilt(prebuilt)
        self.archive = self.root / ARTIFACT_NAME
        create_deterministic_zip(prebuilt, self.archive)
        self.contents = validate_zip(self.archive)
        self.records = file_records(self.contents)
        self.inputs = valid_inputs()
        self.input_bytes = canonical_json(self.inputs)
        self.fingerprint = "b" * 64
        self.metadata = metadata_document(
            runtime_version="0.9.0",
            source_revision="a" * 40,
            fingerprint=self.fingerprint,
            build_inputs_hash=sha256_bytes(self.input_bytes),
            archive=self.archive,
            records=self.records,
        )
        self.budget = {
            "schemaVersion": 1,
            "releaseTag": RELEASE_TAG,
            "maximums": {
                "archiveBytes": self.archive.stat().st_size + 100,
                "expandedBytes": sum(record["sizeBytes"] for record in self.records) + 100,
                "fileBytes": {
                    record["path"]: record["sizeBytes"] + 100
                    for record in self.records
                },
            },
        }

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_metadata_binds_every_byte_and_full_source_revision(self) -> None:
        validate_metadata(
            self.metadata,
            archive=self.archive,
            records=self.records,
            build_inputs=self.inputs,
            build_inputs_hash=sha256_bytes(self.input_bytes),
            fingerprint=self.fingerprint,
            release_revision="a" * 40,
        )
        changed = copy.deepcopy(self.metadata)
        changed["buildSourceRevision"] = "a" * 8
        with self.assertRaisesRegex(ContractError, "full lowercase"):
            validate_metadata(
                changed,
                archive=self.archive,
                records=self.records,
                build_inputs=self.inputs,
                build_inputs_hash=sha256_bytes(self.input_bytes),
                fingerprint=self.fingerprint,
                release_revision=None,
            )
        changed = copy.deepcopy(self.metadata)
        changed["artifact"]["files"][0]["sha256"] = "c" * 64
        with self.assertRaisesRegex(ContractError, "exact bytes"):
            validate_metadata(
                changed,
                archive=self.archive,
                records=self.records,
                build_inputs=self.inputs,
                build_inputs_hash=sha256_bytes(self.input_bytes),
                fingerprint=self.fingerprint,
                release_revision=None,
            )

    def test_size_budget_reports_headroom_and_fails_overage(self) -> None:
        budget = validate_budget(self.budget)
        encoded_budget = (json.dumps(budget) + "\n").encode()
        report = size_report(
            self.archive, self.records, budget, sha256_bytes(encoded_budget)
        )
        validate_size_report(
            report,
            budget,
            sha256_bytes(encoded_budget),
            self.archive,
            self.records,
        )
        smaller_budget = copy.deepcopy(budget)
        smaller_budget["maximums"]["archiveBytes"] = 1
        oversized = size_report(
            self.archive,
            self.records,
            smaller_budget,
            sha256_bytes(encoded_budget),
        )
        with self.assertRaisesRegex(ContractError, "aggregate"):
            validate_size_report(
                oversized,
                smaller_budget,
                sha256_bytes(encoded_budget),
                self.archive,
                self.records,
            )

    def test_embedded_provenance_is_exact(self) -> None:
        expected = {
            "schemaVersion": 6,
            "rootPackage": "nux-capi",
            "runtimeVersion": "0.9.0",
            "buildSourceRevision": "a" * 40,
            "target": "aarch64-linux-android",
            "profile": "release",
            "features": "android-vulkan,android-authored-wgsl,scripting",
            "rustc": self.inputs["configuration"]["rustc"],
            "buildInputsHash": self.metadata["buildInputsHash"],
            "contractFingerprint": self.metadata["contractFingerprint"],
        }
        strings = "prefix\n" + json.dumps(expected, separators=(",", ":")) + "\nsuffix\n"
        validate_provenance(
            strings,
            metadata=self.metadata,
            build_inputs=self.inputs,
            target="aarch64-linux-android",
        )
        with self.assertRaisesRegex(ContractError, "exactly one"):
            validate_provenance(
                strings + strings,
                metadata=self.metadata,
                build_inputs=self.inputs,
                target="aarch64-linux-android",
            )


class PipelineContractTests(unittest.TestCase):
    def test_committed_budget_is_a_frozen_exact_tree_budget(self) -> None:
        budget = validate_budget(
            json.loads((REPO_ROOT / "tools/android-runtime-size-budget-v4.json").read_text())
        )
        self.assertEqual(budget["releaseTag"], "android-runtime-v0.3.7")
        self.assertEqual(list(budget["maximums"]["fileBytes"]), list(EXPECTED_FILES))

    def test_builder_plan_exposes_every_pinned_dimension(self) -> None:
        plan = subprocess.run(
            [str(REPO_ROOT / "tools/build-nux-capi-android.sh"), "--plan"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        for expected in (
            "NuxieRuntimeAndroid.zip",
            "android-runtime-v0.3.7",
            "Rust".lower(),
            "1.94.1",
            "4.1.2",
            "26.1.10909125",
            "android-api: 23",
            "arm64-v8a x86_64",
            "android-vulkan,scripting,android-authored-wgsl",
            "ABI4",
            "DT_NEEDED",
        ):
            self.assertIn(expected, plan.lower() if expected == "rust" else plan)

    def test_builder_invokes_cargo_ndk_as_a_cargo_subcommand(self) -> None:
        builder = (REPO_ROOT / "tools/build-nux-capi-android.sh").read_text()
        self.assertIn('"${rust_cargo}" ndk --version', builder)
        self.assertIn('"${rust_cargo}" ndk \\', builder)
        self.assertNotIn('"${cargo_ndk}" --version', builder)

    def test_publisher_orders_draft_download_verify_before_publish(self) -> None:
        publisher = (REPO_ROOT / "tools/publish-nux-capi-android-release.sh").read_text()
        self.assertIn('expected_tag="android-runtime-v0.3.7"', publisher)
        self.assertIn("rev-parse refs/remotes/origin/main", publisher)
        self.assertIn("ls-remote --exit-code origin", publisher)
        self.assertIn("gh release create", publisher)
        self.assertIn("--draft", publisher)
        self.assertIn("gh release download", publisher)
        self.assertIn("cmp ", publisher)
        self.assertIn("--draft=false", publisher)
        self.assertLess(publisher.index("gh release create"), publisher.index("gh release download"))
        self.assertLess(publisher.index("gh release download"), publisher.rindex(" verify "))
        self.assertLess(publisher.rindex(" verify "), publisher.index("--draft=false"))


if __name__ == "__main__":
    unittest.main()
