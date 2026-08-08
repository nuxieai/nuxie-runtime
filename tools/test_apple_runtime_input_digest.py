import copy
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.apple_runtime_input_digest import PACKAGING_INPUTS
from tools.apple_runtime_input_digest import InputDigestError
from tools.apple_runtime_input_digest import _fallback_apple_target_cfg
from tools.apple_runtime_input_digest import _build_environment
from tools.apple_runtime_input_digest import _tool_identities
from tools.apple_runtime_input_digest import _directory_content_hash
from tools.apple_runtime_input_digest import build_manifest


TARGET_A = "aarch64-apple-ios"
TARGET_B = "x86_64-apple-ios"


def package(repo: Path, name: str, *, source: str | None = None) -> dict:
    package_root = repo / "crates" / name
    return {
        "checksum": None if source is None else name[0] * 64,
        "id": name,
        "manifest_path": str(package_root / "Cargo.toml"),
        "name": name,
        "source": source,
        "version": "1.0.0",
    }


def metadata(repo: Path, *, target_specific: bool) -> dict:
    packages = [
        package(repo, "nux-apple-runtime"),
        package(repo, "direct"),
        package(repo, "transitive"),
        package(repo, "target-only"),
        package(repo, "unrelated"),
        package(repo, "registry", source="registry+https://example.invalid/index"),
    ]
    root_dependencies = [
        {"dep_kinds": [{"kind": None, "target": None}], "pkg": "direct"},
        {"dep_kinds": [{"kind": "build", "target": None}], "pkg": "registry"},
        {"dep_kinds": [{"kind": "dev", "target": None}], "pkg": "unrelated"},
    ]
    if target_specific:
        root_dependencies.append(
        {"dep_kinds": [{"kind": None, "target": 'cfg(target_arch = "x86_64")'}], "pkg": "target-only"}
        )
    return {
        "packages": packages,
        "resolve": {
            "nodes": [
                {"deps": root_dependencies, "features": ["apple-product"], "id": "nux-apple-runtime"},
                {
                    "deps": [{"dep_kinds": [{"kind": None, "target": None}], "pkg": "transitive"}],
                    "features": [],
                    "id": "direct",
                },
                {"deps": [], "features": ["backend"], "id": "transitive"},
                {"deps": [], "features": [], "id": "target-only"},
                {"deps": [], "features": [], "id": "unrelated"},
                {"deps": [], "features": [], "id": "registry"},
            ]
        },
    }


class InputDigestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.repo = Path(self.temporary.name)
        for relative_path in ("Cargo.toml", *PACKAGING_INPUTS):
            path = self.repo / relative_path
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(relative_path + "\n")
        (self.repo / "Cargo.lock").write_text(
            "version = 4\n\n"
            "[[package]]\n"
            "name = \"registry\"\n"
            "version = \"1.0.0\"\n"
            "source = \"registry+https://example.invalid/index\"\n"
            f"checksum = \"{'e' * 64}\"\n"
        )
        for name in (
            "nux-apple-runtime",
            "direct",
            "transitive",
            "target-only",
            "unrelated",
            "registry",
        ):
            package_root = self.repo / "crates" / name
            (package_root / "src").mkdir(parents=True)
            (package_root / "Cargo.toml").write_text(
                f"[package]\nname = {name!r}\n"
            )
            (package_root / "src" / "lib.rs").write_text(f"// {name}\n")
        self.metadata = {
            TARGET_A: metadata(self.repo, target_specific=False),
            TARGET_B: metadata(self.repo, target_specific=True),
        }
        self.configuration = {
            "buildProfile": "release-apple",
            "rustToolchain": "1.94.1",
        }

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def manifest(self) -> dict:
        return build_manifest(self.repo, self.metadata, self.configuration)

    def assert_changes_manifest(self, path: Path, replacement: str) -> None:
        baseline = self.manifest()
        path.write_text(replacement)
        self.assertNotEqual(baseline, self.manifest())

    def test_irrelevant_repository_and_dev_dependency_changes_do_not_invalidate(self) -> None:
        baseline = self.manifest()
        docs = self.repo / "docs"
        docs.mkdir()
        (docs / "unrelated.md").write_text("documentation only\n")
        (self.repo / "crates" / "unrelated" / "src" / "lib.rs").write_text("// changed dev-only crate\n")
        self.assertEqual(baseline, self.manifest())

    def test_reachable_package_tests_and_readme_do_not_invalidate(self) -> None:
        baseline = self.manifest()
        package_root = self.repo / "crates" / "direct"
        (package_root / "README.md").write_text("package documentation\n")
        tests = package_root / "tests"
        tests.mkdir()
        (tests / "integration.rs").write_text("compile_error!(\"not a library input\");\n")
        self.assertEqual(baseline, self.manifest())

    def test_direct_transitive_and_target_specific_source_changes_invalidate(self) -> None:
        for name in ("nux-apple-runtime", "direct", "transitive", "target-only"):
            with self.subTest(name=name):
                path = self.repo / "crates" / name / "src" / "lib.rs"
                original = path.read_text()
                self.assert_changes_manifest(path, original + "// changed\n")
                path.write_text(original)

    def test_manifest_lock_provider_patch_build_header_notice_and_tool_changes_invalidate(self) -> None:
        inputs = (
            self.repo / "Cargo.toml",
            self.repo / "crates" / "direct" / "Cargo.toml",
            self.repo / "crates" / "direct" / "build.rs",
            self.repo
            / "crates"
            / "nux-apple-runtime"
            / "include"
            / "nux_runtime.h",
            self.repo / "THIRD_PARTY_NOTICES.md",
            self.repo / "tools" / "build-apple-xcframework.sh",
        )
        inputs[2].write_text("fn main() {}\n")
        inputs[3].parent.mkdir()
        inputs[3].write_text("/* header */\n")
        (self.repo / ".cargo").mkdir()
        provider = self.repo / ".cargo" / "config.toml"
        provider.write_text("[source.crates-io]\nreplace-with = 'vendored'\n")
        for index, path in enumerate((*inputs, provider)):
            with self.subTest(path=path):
                original = path.read_text()
                self.assert_changes_manifest(path, original + f"# mutation {index}\n")
                path.write_text(original)

    def test_registry_checksum_and_resolved_features_invalidate(self) -> None:
        baseline = self.manifest()
        lockfile = self.repo / "Cargo.lock"
        original_lockfile = lockfile.read_text()
        lockfile.write_text(original_lockfile.replace("e" * 64, "f" * 64))
        self.assertNotEqual(baseline, self.manifest())
        lockfile.write_text(original_lockfile)

        changed_features = copy.deepcopy(self.metadata)
        direct = next(
            node
            for node in changed_features[TARGET_B]["resolve"]["nodes"]
            if node["id"] == "direct"
        )
        direct["features"] = ["target-provider"]
        self.assertNotEqual(
            baseline,
            build_manifest(self.repo, changed_features, self.configuration),
        )

    def test_unrelated_lock_entry_and_cargo_readme_do_not_invalidate(self) -> None:
        baseline = self.manifest()
        lockfile = self.repo / "Cargo.lock"
        lockfile.write_text(
            lockfile.read_text()
            + "\n[[package]]\nname = \"unrelated-lock\"\nversion = \"9.9.9\"\n"
        )
        cargo_readme = self.repo / ".cargo" / "README.md"
        cargo_readme.parent.mkdir(exist_ok=True)
        cargo_readme.write_text("not Cargo configuration\n")
        self.assertEqual(baseline, self.manifest())

    def test_repo_cargo_config_include_is_audited(self) -> None:
        cargo_dir = self.repo / ".cargo"
        cargo_dir.mkdir()
        config = cargo_dir / "config.toml"
        provider = cargo_dir / "provider.toml"
        config.write_text("include = ['provider.toml']\n")
        provider.write_text("[source.crates-io]\nregistry = 'https://example.invalid'\n")
        baseline = self.manifest()
        provider.write_text("[source.crates-io]\nregistry = 'https://changed.invalid'\n")
        self.assertNotEqual(baseline, self.manifest())

    def test_external_cargo_configuration_fails_closed(self) -> None:
        cargo_home = self.repo / "external-cargo-home"
        cargo_home.mkdir()
        (cargo_home / "config.toml").write_text("[build]\nrustflags = ['-Cfoo']\n")
        with patch.dict(os.environ, {"CARGO_HOME": str(cargo_home)}):
            with self.assertRaisesRegex(InputDigestError, "external Cargo configuration"):
                self.manifest()

    def test_symlinked_cargo_config_include_fails_closed(self) -> None:
        cargo_dir = self.repo / ".cargo"
        cargo_dir.mkdir()
        actual = cargo_dir / "actual.toml"
        actual.write_text("[build]\nrustflags = []\n")
        linked = cargo_dir / "linked.toml"
        linked.symlink_to(actual)
        (cargo_dir / "config.toml").write_text("include = ['linked.toml']\n")
        with self.assertRaisesRegex(InputDigestError, "symlink"):
            self.manifest()

    def test_host_specific_build_dependency_is_audited(self) -> None:
        host_metadata = metadata(self.repo, target_specific=False)
        host_package = package(self.repo, "host-build")
        host_metadata["packages"].append(host_package)
        host_metadata["resolve"]["nodes"].append(
            {"deps": [], "features": ["host-tool"], "id": "host-build"}
        )
        root = next(
            node
            for node in host_metadata["resolve"]["nodes"]
            if node["id"] == "nux-apple-runtime"
        )
        root["deps"].append(
            {
                "dep_kinds": [
                    {"kind": "build", "target": 'cfg(target_os = "macos")'}
                ],
                "pkg": "host-build",
            }
        )
        host_root = self.repo / "crates" / "host-build"
        (host_root / "src").mkdir(parents=True)
        (host_root / "Cargo.toml").write_text("[package]\nname = 'host-build'\n")
        (host_root / "src" / "lib.rs").write_text("// host tool\n")
        manifest = build_manifest(
            self.repo,
            self.metadata,
            self.configuration,
            host_metadata=host_metadata,
            host_target="aarch64-apple-darwin",
            host_cfg=_fallback_apple_target_cfg("aarch64-apple-darwin"),
        )
        record = next(
            package for package in manifest["packages"] if package["name"] == "host-build"
        )
        self.assertEqual(record["targets"], {"host": ["host-tool"]})

    def test_symlinked_package_directory_fails_closed(self) -> None:
        package_root = self.repo / "crates" / "direct"
        outside = self.repo / "outside"
        outside.mkdir()
        (outside / "compiled.rs").write_text("// outside\n")
        (package_root / "linked-source").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(InputDigestError, "symlink"):
            self.manifest()

    def test_toolchain_configuration_invalidate(self) -> None:
        baseline = self.manifest()
        changed = dict(self.configuration)
        changed["rustToolchain"] = "1.95.0"
        self.assertNotEqual(
            baseline,
            build_manifest(self.repo, self.metadata, changed),
        )

    def test_exact_root_resolution_excludes_workspace_feature_pollution(self) -> None:
        exact = {
            TARGET_A: {
                "nux-apple-runtime": ["apple-product"],
                "direct": [],
                "transitive": ["backend"],
                "registry": [],
            },
            TARGET_B: {
                "nux-apple-runtime": ["apple-product"],
                "direct": [],
                "transitive": ["backend"],
                "target-only": [],
                "registry": [],
            },
        }
        manifest = build_manifest(
            self.repo,
            self.metadata,
            self.configuration,
            resolutions_by_context=exact,
        )
        self.assertNotIn("unrelated", {package["name"] for package in manifest["packages"]})

    def test_actual_tool_binary_content_is_an_input(self) -> None:
        tool = self.repo / "tool"
        tool.write_bytes(b"version one")
        first = _tool_identities([f"compiler={tool}"])
        tool.write_bytes(b"version two")
        second = _tool_identities([f"compiler={tool}"])
        self.assertNotEqual(first, second)

    def test_resolved_provider_payload_content_is_an_input(self) -> None:
        provider = self.repo / "provider-payload"
        provider.mkdir()
        source = provider / "src.rs"
        source.write_text("version one\n")
        first = _directory_content_hash(provider)
        source.write_text("version two\n")
        self.assertNotEqual(first, _directory_content_hash(provider))

    def test_rust_library_component_content_is_an_input(self) -> None:
        component = self.repo / "synthetic-rustlib"
        component.mkdir()
        library = component / "libstd.rlib"
        library.write_bytes(b"first")
        first = _directory_content_hash(component)
        library.write_bytes(b"second")
        self.assertNotEqual(first, _directory_content_hash(component))

    def test_compiler_override_environment_fails_closed(self) -> None:
        for key in (
            "CC_aarch64_apple_ios",
            "CARGO_BUILD_RUSTC_WRAPPER",
            "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
        ):
            with self.subTest(key=key), patch.dict(os.environ, {key: "/tmp/compiler"}):
                with self.assertRaisesRegex(InputDigestError, key):
                    _build_environment()


if __name__ == "__main__":
    unittest.main()
