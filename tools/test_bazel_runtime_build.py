import copy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.bazel_runtime_build import BazelRuntime, canonical, digest


TARGET = "aarch64-linux-android"
LABEL = "//crates/shipping:shipping__android__cdylib"


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.runtime = object.__new__(BazelRuntime)
        self.runtime.root = self.root
        self.runtime.execroot = self.root
        self.runtime.external_root = self.root / "downloaded"
        self.runtime.installbase = self.root / "installation"
        self.runtime.executable = self.write("bazelisk", "selected launcher")
        self.write("installation/A-server.jar", "selected server")
        for relative in (
            ".bazelrc", ".bazelversion", "MODULE.bazel", "MODULE.bazel.lock",
            "bazel/cargo/Cargo.toml", "bazel/cargo/Cargo.lock", "bazel/cargo/Cargo.Bazel.lock",
            "tools/bazel_runtime_build.py", "tools/bazel_runtime_query.bzl", "BUILD.bazel",
        ):
            self.write(relative, relative)
        self.write("Cargo.toml", '[workspace.package]\nversion = "1.2.3"\n')
        self.write("Cargo.lock", 'version = 4\n[[package]]\nname = "registry-dep"\nversion = "2.0.0"\n'
                   'source = "registry+https://example.invalid/index"\nchecksum = "' + "a" * 64 + '"\n')
        self.write("NOTICE", "shipping notice")
        self.write("crates/shipping/Cargo.toml", '[package]\nname = "shipping"\nversion.workspace = true\n')
        self.write("crates/shipping/src/lib.rs", "pub fn shipped() {}\n")
        self.write("crates/shipping/README.md", "not a selected action input")
        self.registry = "external/runtime_crates__registry-dep-2.0.0"
        self.registry_fixture = "downloaded/runtime_crates__registry-dep-2.0.0"
        self.write(f"{self.registry_fixture}/Cargo.toml", '[package]\nname = "registry-dep"\nversion = "2.0.0"\n')
        self.write(f"{self.registry_fixture}/src/lib.rs", "pub fn dependency() {}\n")
        self.compiler_repository = self.root / "downloaded/rust_compiler"
        self.compiler = self.write("downloaded/rust_compiler/bin/rustc", "selected compiler")
        self.write(f"downloaded/rust_compiler/lib/rustlib/{TARGET}/lib/libstd.rlib", "selected std")
        self.projections = [
            {"label": LABEL, "root": "crates/shipping/src/lib.rs", "cfgs": ['feature="shipping"'],
             "files": ["crates/shipping/src/lib.rs"]},
            {"label": "@registry//:dep", "root": f"{self.registry}/src/lib.rs", "cfgs": ['feature="fast"'],
             "files": [f"{self.registry}/src/lib.rs"]},
        ]
        self.runtime.toolchain = lambda *args, **kwargs: (
            self.compiler_repository, self.compiler, "rustc 1.94.1 fixture"
        )
        self.runtime.closure = lambda *args, **kwargs: copy.deepcopy(self.projections)

    def tearDown(self):
        self.temporary.cleanup()

    def write(self, relative, contents):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents)
        return path

    def evidence(self):
        return self.runtime.evidence(LABEL, [TARGET], ["NOTICE"])

    def test_local_inputs_and_resolved_external_payloads_are_audited(self):
        evidence = self.evidence()
        records = {record["path"]: record for record in evidence["files"]}
        self.assertEqual(records["crates/shipping/src/lib.rs"]["sha256"],
                         digest(self.root / "crates/shipping/src/lib.rs"))
        self.assertEqual(records["crates/shipping/Cargo.toml"]["sha256"],
                         digest(self.root / "crates/shipping/Cargo.toml"))
        self.assertIn("Cargo.toml", records)
        self.assertIn("Cargo.lock", records)
        packages = {package["name"]: package for package in evidence["packages"]}
        self.assertEqual(packages["shipping"]["version"], "1.2.3")
        self.assertEqual(packages["shipping"]["targets"], {TARGET: ["shipping"]})
        self.assertEqual(packages["registry-dep"]["checksum"], "a" * 64)
        self.assertEqual(packages["registry-dep"]["targets"], {TARGET: ["fast"]})
        self.assertIsNone(packages["registry-dep"]["manifestPath"])

    def test_source_payload_feature_library_and_tool_changes_each_invalidate(self):
        for relative in (
            "crates/shipping/src/lib.rs", f"{self.registry_fixture}/src/lib.rs",
            f"downloaded/rust_compiler/lib/rustlib/{TARGET}/lib/libstd.rlib",
            "downloaded/rust_compiler/bin/rustc", "installation/A-server.jar", "NOTICE",
        ):
            with self.subTest(relative=relative):
                baseline = canonical(self.evidence())
                path = self.root / relative
                original = path.read_bytes()
                path.write_bytes(original + b"changed")
                self.assertNotEqual(canonical(self.evidence()), baseline)
                path.write_bytes(original)
        for relative in ("Cargo.toml", "Cargo.lock", "crates/shipping/Cargo.toml"):
            with self.subTest(relative=relative):
                baseline = canonical(self.evidence())
                path = self.root / relative
                original = path.read_bytes()
                path.write_bytes(original + b"\n# changed manifest identity\n")
                self.assertNotEqual(canonical(self.evidence()), baseline)
                path.write_bytes(original)
        baseline = canonical(self.evidence())
        self.projections[0]["cfgs"].append('feature="extra"')
        self.assertNotEqual(canonical(self.evidence()), baseline)

    def test_unselected_local_file_does_not_change_the_configured_closure(self):
        baseline = canonical(self.evidence())
        self.write("crates/shipping/README.md", "edited unselected input")
        self.assertEqual(canonical(self.evidence()), baseline)

    def test_missing_lock_entry_and_missing_selected_file_fail(self):
        self.write("Cargo.lock", "version = 4\n")
        with self.assertRaisesRegex(ValueError, "absent from Cargo.lock"):
            self.evidence()
        self.projections = self.projections[:1]
        (self.root / "crates/shipping/src/lib.rs").unlink()
        with self.assertRaisesRegex(ValueError, "input is missing"):
            self.evidence()

    def test_selected_toolchain_is_available_before_execroot_links_are_created(self):
        compiler = self.write("downloaded/selected_toolchain/bin/rustc", "selected compiler")
        self.runtime.run = lambda *args: '{"actions":[{"arguments":["wrapper","--","external/selected_toolchain/rust_toolchain/bin/rustc"]}]}'
        with patch.object(BazelRuntime, "options", return_value=[]), patch(
            "tools.bazel_runtime_build.subprocess.check_output", return_value="rustc 1.94.1 selected\n"
        ) as invoked:
            repository, selected, _ = BazelRuntime.toolchain(self.runtime, LABEL, TARGET)
        self.assertEqual(repository, compiler.parent.parent)
        self.assertEqual(selected, compiler)
        invoked.assert_called_once_with([compiler, "-vV"], text=True)
        self.assertFalse((self.runtime.execroot / "external").exists())


class AndroidConfigurationTests(unittest.TestCase):
    def test_job_limits_must_be_positive_integers(self):
        for value in ("0", "-2", "two"):
            with self.subTest(value=value), patch.dict("os.environ", {"NUXIE_BAZEL_JOBS": value}, clear=True):
                with self.assertRaisesRegex(ValueError, "positive integer"):
                    BazelRuntime.options(TARGET)

    def test_ndk_fallback_and_api_sysroot_are_passed_to_selected_build_actions(self):
        with tempfile.TemporaryDirectory() as directory:
            sdk = Path(directory) / "sdk"
            ndk = (sdk / "ndk/29.0.14206865").resolve()
            prebuilt = ndk / "toolchains/llvm/prebuilt/test-host"
            (prebuilt / "sysroot").mkdir(parents=True)
            with patch.dict("os.environ", {"ANDROID_HOME": str(sdk)}, clear=True):
                options = BazelRuntime.options(TARGET)
            self.assertIn(f"--repo_env=ANDROID_NDK_HOME={ndk}", options)
            self.assertIn("--@rules_rust//rust/settings:extra_rustc_flag=-Cdebuginfo=0", options)
            self.assertIn(f"--action_env=BINDGEN_EXTRA_CLANG_ARGS=--target={TARGET}23 "
                          f"--sysroot={prebuilt}/sysroot -I{prebuilt}/sysroot/usr/include/{TARGET}", options)
            self.assertIn("--@rules_rust//rust/settings:extra_rustc_flag=-Clink-arg=-Wl,-z,max-page-size=16384", options)


class AppleConfigurationTests(unittest.TestCase):
    def test_device_simulator_and_macos_actions_use_matching_sdk_and_xcode(self):
        for target, sdk, triple in (
            ("aarch64-apple-ios", "iphoneos", "arm64-apple-ios15.0"),
            ("aarch64-apple-ios-sim", "iphonesimulator", "arm64-apple-ios15.0-simulator"),
            ("x86_64-apple-darwin", "macosx", "x86_64-apple-macos12.0"),
        ):
            with self.subTest(target=target), patch.dict("os.environ", {}, clear=True):
                def output(arguments, **kwargs):
                    if arguments == ["xcodebuild", "-version"]:
                        return "Xcode 26.2\nBuild version 17C52\n"
                    self.assertEqual(arguments, ["xcrun", "--sdk", sdk, "--show-sdk-path"])
                    return f"/selected/{sdk}.sdk\n"
                with patch("tools.bazel_runtime_build.subprocess.check_output", side_effect=output):
                    options = BazelRuntime.options(target)
                self.assertIn("--xcode_version=26.2", options)
                self.assertIn("--@rules_rust//rust/settings:extra_rustc_flag=-Cstrip=debuginfo", options)
                self.assertIn(f"--action_env=BINDGEN_EXTRA_CLANG_ARGS=--target={triple} "
                              f"--sysroot=/selected/{sdk}.sdk", options)


if __name__ == "__main__":
    unittest.main()
