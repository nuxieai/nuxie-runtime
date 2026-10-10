import pathlib
import tempfile
import unittest
import json
import subprocess
import tomllib
import os
import shutil
from unittest.mock import patch

from cargo_graph import collect_packages, resolve_features, resolve_native_features, cfg_matches, write_registry_workspace
from emit import dependency_expression, feature_expression, render_package


class CargoGraphTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)

    def package(self, name, body):
        directory = self.root / name
        directory.mkdir()
        manifest = directory / "Cargo.toml"
        manifest.write_text(f'[package]\nname = "{name}"\nversion = "0.1.0"\n' + body)
        return manifest

    def test_optional_and_weak_dependency_features(self):
        manifests = [
            self.package("root", '[features]\nweak = ["dep?/extra"]\nstrong = ["dep:dep", "dep?/extra"]\n[dependencies]\ndep = { path = "../dep", optional = true, default-features = false }\n'),
            self.package("dep", '[features]\ndefault = ["forbidden"]\nforbidden = []\nextra = []\n'),
        ]
        packages = collect_packages(manifests)
        weak = resolve_features(packages, {"root": ["weak"]})
        self.assertEqual(weak.features["dep"], set())
        strong = resolve_features(packages, {"root": ["strong"]})
        self.assertEqual(strong.features["dep"], {"extra"})

    def test_dependency_dev_features_are_only_enabled_for_test_roots(self):
        manifests = [
            self.package("root", '[dependencies]\ndep = { path = "../dep" }\n[dev-dependencies]\nprobe = { path = "../probe", features = ["root-tests"], default-features = false }\n'),
            self.package("dep", '[dev-dependencies]\nprobe = { path = "../probe", features = ["dependency-tests"], default-features = false }\n'),
            self.package("probe", '[features]\nroot-tests = []\ndependency-tests = []\n'),
        ]
        packages = collect_packages(manifests)
        graph = resolve_features(packages, {"root": []}, include_dev=True)
        self.assertEqual(graph.features["probe"], {"root-tests"})
        self.assertEqual(graph.dependencies("dep", ("dev-dependencies",)), [])
        graph = resolve_features(packages, {"root": [], "dep": []}, include_dev=True)
        self.assertEqual(graph.features["probe"], {"root-tests", "dependency-tests"})

    def test_optional_build_dependencies_follow_selected_features(self):
        manifest = self.package("root", '[features]\ngenerate = ["dep:bindgen"]\n[build-dependencies]\ncc = "1"\nbindgen = { version = "1", optional = true }\n')
        (manifest.parent / "build.rs").write_text("fn main() {}")
        packages = collect_packages([manifest])
        for features, expected in (([], ["cc"]), (["generate"], ["bindgen", "cc"])):
            graph = resolve_features(packages, {"root": features})
            rendered = render_package(packages["root"], {"": graph}, lambda name, _: "//root:" + name)
            expression = next(line.strip()[len("deps = "):-1] for line in rendered.splitlines() if line.strip().startswith("deps = "))
            actual = eval(expression, {"__builtins__": {}, "crate_deps": lambda aliases, **_: aliases})
            self.assertEqual(actual, expected)

    def test_provenance_inputs_are_scoped_to_distribution_build_scripts(self):
        provenance_input = object()
        for name in ("nux-capi", "nux-apple-product-extension", "ordinary"):
            manifest = self.package(name, "")
            (manifest.parent / "build.rs").write_text("fn main() {}")
            package = collect_packages([manifest])[name]
            graph = resolve_features({name: package}, {name: []})
            for owner in (None, "@source//crates/" + name):
                rendered = render_package(package, {"": graph}, lambda name, _: "//root:" + name,
                                          source_owner=owner)
                scripts = []
                def ignore(*args, **kwargs):
                    pass
                scope = {"load": ignore, "package": ignore, "exports_files": ignore,
                         "filegroup": ignore, "glob": lambda *args, **kwargs: [],
                         "source_path": lambda *args: "crates/" + name,
                         "provenance_env_file": lambda: provenance_input,
                         "all_crate_deps": lambda **kwargs: [],
                         "cargo_build_script": lambda **kwargs: scripts.append(kwargs)}
                exec(rendered, scope)
                self.assertEqual(len(scripts), 1)
                expected = [] if name == "ordinary" else [provenance_input]
                self.assertEqual(scripts[0].get("build_script_env_files", []), expected)
                self.assertTrue(rendered.endswith("\n"))
                self.assertFalse(rendered.endswith("\n\n"))

    def test_target_features_do_not_cross_host_and_wasm(self):
        manifests = [
            self.package("root", '[target.\'cfg(target_arch = "wasm32")\'.dependencies]\ndep = { path = "../dep", features = ["browser"], default-features = false }\n[target.\'cfg(not(target_arch = "wasm32"))\'.dependencies]\ndep = { path = "../dep", features = ["native"], default-features = false }\n'),
            self.package("dep", '[features]\nbrowser = []\nnative = []\n'),
        ]
        packages = collect_packages(manifests)
        self.assertEqual(resolve_features(packages, {"root": []}, platform="wasm").features["dep"], {"browser"})
        self.assertEqual(resolve_features(packages, {"root": []}, platform="native").features["dep"], {"native"})

    def test_native_windows_predicates_match_the_msvc_host_triple(self):
        with patch("cargo_graph.sys.platform", "win32"), patch("cargo_graph.host_platform.machine", return_value="AMD64"):
            self.assertTrue(cfg_matches('cfg(all(target_arch = "x86_64", target_vendor = "pc", target_os = "windows", target_env = "msvc"))', "native"))
            self.assertFalse(cfg_matches("cfg(unix)", "native"))

    def test_strong_dependency_feature_enables_implicit_local_feature(self):
        manifests = [
            self.package("root", '[features]\nextra = ["dep/extra"]\n[dependencies]\ndep = { path = "../dep", optional = true, default-features = false }\n'),
            self.package("dep", '[features]\nextra = []\n'),
        ]
        packages = collect_packages(manifests)
        graph = resolve_features(packages, {"root": ["extra"]})
        self.assertEqual(graph.features["root"], {"extra", "dep"})
        self.assertEqual(graph.features["dep"], {"extra"})

    def test_inherited_dependencies_and_registry_path_rewrite(self):
        (self.root / "Cargo.toml").write_text('[workspace]\n[workspace.package]\nedition = "2024"\n[workspace.dependencies]\nserde = { version = "1", features = ["derive"] }\n')
        manifests = [self.package("root", 'edition.workspace = true\n[dependencies]\nserde = { workspace = true, features = ["alloc"] }\ndep = { path = "../dep", default-features = false }\n'), self.package("dep", "")]
        packages = collect_packages(manifests)
        self.assertEqual(packages["root"].edition, "2024")
        self.assertEqual(packages["root"].dependencies[0].spec["features"], ["alloc", "derive"])
        source_lock = self.root / "Cargo.lock"
        source_lock.write_text("version = 4\n")
        destination = self.root / "registry"
        write_registry_workspace(packages, destination, source_lock)
        import tomllib
        emitted = tomllib.loads((destination / "root/Cargo.toml").read_text())
        self.assertEqual(emitted["dependencies"]["dep"]["path"], "../dep")
        self.assertEqual(emitted["dependencies"]["serde"]["features"], ["alloc", "derive"])

    def test_native_platform_features_and_optional_edges_stay_isolated(self):
        manifests = [
            self.package("root", '[target.\'cfg(target_os = "linux")\'.dependencies]\ndep = { path = "../dep", features = ["linux"], default-features = false }\n[target.\'cfg(target_os = "macos")\'.dependencies]\ndep = { path = "../dep", features = ["mac"], default-features = false }\n'),
            self.package("dep", '[features]\nlinux = []\nmac = ["dep:probe"]\n[dependencies]\nprobe = { path = "../probe", optional = true }\n'),
            self.package("probe", ""),
        ]
        packages = collect_packages(manifests)
        graph = resolve_native_features(packages, {"root": []})
        self.assertEqual(graph.platforms["linux"].features["dep"], {"linux"})
        self.assertEqual(graph.platforms["macos"].features["dep"], {"mac"})
        self.assertEqual(graph.platforms["linux"].dependencies("dep"), [])
        self.assertEqual([dep.local for dep in graph.platforms["macos"].dependencies("dep")], ["probe"])
        self.assertIn('"//bazel:linux": ["linux"]', feature_expression(graph, "dep"))
        self.assertIn('"//bazel:macos": ["mac"]', feature_expression(graph, "dep"))

    def test_portable_generation_is_identical_on_linux_and_macos(self):
        manifests = [self.package("root", '[target.\'cfg(target_os = "macos")\'.dependencies]\ndep = { path = "../dep", features = ["mac"], default-features = false }\n'), self.package("dep", '[features]\nmac = []\n')]
        packages = collect_packages(manifests)
        label_for = lambda name, variant: "//" + name + ":" + name
        outputs = []
        for host in ("linux", "darwin"):
            with patch("sys.platform", host):
                graph = resolve_native_features(packages, {"root": []})
                outputs.append(render_package(packages["root"], {"": graph}, label_for))
        self.assertEqual(outputs[0], outputs[1])

    def test_native_dependency_select_branches_are_concrete_lists(self):
        manifests = [self.package("root", '[target.\'cfg(target_os = "macos")\'.dependencies]\ndep = { path = "../dep" }\n'), self.package("dep", "")]
        packages = collect_packages(manifests)
        graph = resolve_native_features(packages, {"root": []})
        expression = dependency_expression(graph.dependencies("root"), "", lambda name, _: "//" + name + ":" + name,
                                           "bazel/cargo", "root", "@runtime_crates")

        def strict_select(branches):
            # Bazel rejects a select value nested inside another select. A
            # wrapper instead of a list makes this independent oracle catch it.
            for value in branches.values():
                self.assertIs(type(value), list)
            return ("selection", branches)

        _, branches = eval(expression, {"__builtins__": {}, "select": strict_select})
        self.assertEqual(branches["//bazel:macos"], ["//dep:dep"])
        self.assertEqual(branches["//bazel:linux"], [])

    def test_registry_vendor_defaults_and_unused_features_do_not_pollute_product(self):
        manifests = [
            self.package("root", '[features]\nruntime = []\nunused = ["vendor/linked"]\n[dependencies]\nvendor = { path = "../vendor", default-features = false, features = ["loaded"] }\n'),
            self.package("vendor", '[features]\ndefault = ["linked"]\nloaded = []\nlinked = ["dep:probe"]\ntest-mode = ["testhelper/test-only"]\n[dependencies]\nprobe = { path = "../probe", optional = true }\n[dev-dependencies]\ntesthelper = { path = "../testhelper" }\n'),
            self.package("probe", ""),
            self.package("testhelper", '[features]\ntest-only = []\n'),
        ]
        packages = collect_packages(manifests)
        source_lock = self.root / "Cargo.lock"
        source_lock.write_text("version = 4\n")
        destination = self.root / "registry"
        write_registry_workspace(packages, destination, source_lock,
                                 workspace_members={"root"}, enabled_features={"root": {"runtime"}})
        metadata = json.loads(subprocess.check_output([
            "cargo", "metadata", "--manifest-path", str(destination / "Cargo.toml"),
            "--offline", "--format-version", "1",
        ], text=True, stderr=subprocess.DEVNULL))
        names = {package["id"]: package["name"] for package in metadata["packages"]}
        self.assertEqual({names[name] for name in metadata["workspace_members"]}, set(packages))
        resolved = {names[node["id"]]: set(node["features"]) for node in metadata["resolve"]["nodes"]}
        self.assertEqual(resolved["vendor"], {"default", "loaded"})
        vendor_node = next(node for node in metadata["resolve"]["nodes"] if names[node["id"]] == "vendor")
        self.assertEqual(vendor_node["deps"], [])
        self.assertNotIn("unused", resolved["root"])
        vendor = tomllib.loads((destination / "vendor/Cargo.toml").read_text())
        self.assertEqual(vendor["features"]["default"], [])
        self.assertEqual(vendor["features"]["__bazel_dependency_default"], ["linked"])
        self.assertEqual(vendor["features"]["test-mode"], [])
        self.assertNotIn("dev-dependencies", vendor)

        # An incoming edge requesting defaults must still get the dependency's
        # original default closure, even though its metadata root is inert.
        packages["root"].dependencies[0].spec["default-features"] = True
        write_registry_workspace(packages, destination, source_lock,
                                 workspace_members={"root"}, enabled_features={"root": {"runtime"}})
        metadata = json.loads(subprocess.check_output([
            "cargo", "metadata", "--manifest-path", str(destination / "Cargo.toml"),
            "--offline", "--format-version", "1",
        ], text=True, stderr=subprocess.DEVNULL))
        vendor_node = next(node for node in metadata["resolve"]["nodes"] if names[node["id"]] == "vendor")
        self.assertIn("linked", vendor_node["features"])
        self.assertEqual([names[dep["pkg"]] for dep in vendor_node["deps"]], ["probe"])

    def test_compile_time_includes_and_runtime_fixture_paths_survive_removed_sandbox(self):
        manifest = self.package("reader", "")
        package_dir = manifest.parent
        (package_dir / "src").mkdir()
        (package_dir / "fixture.txt").write_text("fixture authority")
        source = package_dir / "src/lib.rs"
        source.write_text('''
const INCLUDED: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixture.txt"));
#[test]
fn runtime_fixture_matches_compiled_authority() {
    let root = option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(std::fs::read_to_string(std::path::Path::new(root).join("fixture.txt")).unwrap(), INCLUDED);
}
''')
        package = collect_packages([manifest])["reader"]
        graph = resolve_features({"reader": package}, {"reader": []})
        for owner in (None, "//reader"):
            rendered = render_package(package, {"test": graph}, lambda name, _: "//reader:" + name,
                                      source_owner=owner)
            self.assertIn('BAZEL_CARGO_MANIFEST_DIR = _source_dir', rendered)
        executable = self.root / "reader-test"
        subprocess.run(["rustc", "--test", str(source), "-o", str(executable)], check=True,
                       env={**os.environ, "CARGO_MANIFEST_DIR": str(package_dir), "BAZEL_CARGO_MANIFEST_DIR": "reader"})
        runfiles = self.root / "runfiles"
        (runfiles / "reader").mkdir(parents=True)
        shutil.copyfile(package_dir / "fixture.txt", runfiles / "reader/fixture.txt")
        shutil.rmtree(package_dir)
        subprocess.run([str(executable)], cwd=runfiles, check=True, stdout=subprocess.DEVNULL)

    def test_registry_workspace_removes_obsolete_generated_stubs(self):
        packages = collect_packages([self.package("root", ""), self.package("obsolete", "")])
        source_lock = self.root / "Cargo.lock"
        source_lock.write_text("version = 4\n")
        destination = self.root / "registry"
        write_registry_workspace(packages, destination, source_lock)
        authored = destination / "unrelated"
        authored.mkdir()
        (authored / "Cargo.toml").write_text('[package]\nname = "unrelated"\nversion = "0.1.0"\n')
        write_registry_workspace({"root": packages["root"]}, destination, source_lock)
        self.assertFalse((destination / "obsolete").exists())
        self.assertTrue((authored / "Cargo.toml").exists())
        self.assertEqual(tomllib.loads((destination / "Cargo.toml").read_text())["workspace"]["members"], ["root"])


if __name__ == "__main__":
    unittest.main()
