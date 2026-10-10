#!/usr/bin/env python3
"""Select direct Bazel Rust targets with the runtime's package/feature interface."""

from pathlib import Path
import filecmp
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import tomllib

from cargo_graph import collect_packages, resolve_features
from emit import render_package
from runtime_features import uses_native_tools

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from bazel_runtime_build import BazelRuntime, PROVENANCE_KEYS


def packages_from_workspace():
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())
    patches = {name: (ROOT / spec["path"]).resolve() for name, spec in workspace["patch"]["crates-io"].items()}
    manifests = [ROOT / member / "Cargo.toml" for member in workspace["workspace"]["members"]]
    manifests.extend(directory / "Cargo.toml" for directory in patches.values())
    return collect_packages(set(manifests), patches)


def command_options(arguments):
    args = list(arguments)
    command = args.pop(0)
    if command not in ("build", "check", "test", "run", "clippy"):
        raise ValueError("Expected build, check, test, run, or clippy")
    options = {"command": command, "packages": [], "features": [], "defaults": True,
               "all_features": False, "selection": None, "selector": None,
               "all_targets": False, "profile": "debug", "target": None,
               "runtime_args": [], "test_filter": None, "json": False, "jobs": os.environ.get("NUXIE_BAZEL_JOBS"), "no_run": False,
               "lib": False, "tests": [], "excludes": [], "target_dir": None}
    while args:
        arg = args.pop(0)
        if arg == "--":
            options["runtime_args"] = args
            break
        if arg in ("-p", "--package"):
            options["packages"].append(args.pop(0))
        elif arg in ("--bin", "--example", "--test"):
            options["selection"], options["selector"] = arg, args.pop(0)
            if arg == "--test":
                options["tests"].append(options["selector"])
        elif arg == "--lib":
            options["lib"] = True
            if options["selection"] is None:
                options["selection"] = arg
        elif arg in ("--features", "-F"):
            options["features"].extend(re.split(r"[, ]+", args.pop(0)))
        elif arg == "--no-default-features":
            options["defaults"] = False
        elif arg == "--all-features":
            options["all_features"] = True
        elif arg in ("--all-targets", "--tests"):
            options["all_targets"] = True
        elif arg == "--no-run":
            options["no_run"] = True
        elif arg == "--release":
            options["profile"] = "release"
        elif arg == "--profile":
            options["profile"] = args.pop(0)
        elif arg == "--target":
            options["target"] = args.pop(0)
        elif arg == "--target-dir":
            options["target_dir"] = args.pop(0)
        elif arg == "--exclude":
            options["excludes"].append(args.pop(0))
        elif arg == "--manifest-path":
            manifest = Path(args.pop(0)).resolve()
            if manifest != ROOT / "Cargo.toml":
                options["packages"].append(tomllib.loads(manifest.read_text())["package"]["name"])
        elif arg in ("--jobs", "-j"):
            options["jobs"] = args.pop(0)
        elif arg.startswith("--message-format"):
            options["json"] = "json" in (arg.split("=", 1)[1] if "=" in arg else args.pop(0))
        elif arg in ("--locked", "--offline", "--workspace", "--quiet", "-q"):
            pass
        elif not arg.startswith("-") and command == "test" and options["test_filter"] is None:
            options["test_filter"] = arg
        else:
            raise ValueError(f"Unsupported Rust selector {arg}; use Bazel for custom compiler configuration")
    options["features"] = sorted(set(filter(None, options["features"])))
    if options["profile"] == "dev":
        options["profile"] = "debug"
    return options


def materialize(options, packages):
    selected = options["packages"] or [name for name, package in packages.items() if package.directory.relative_to(ROOT).parts[0] != "vendor"]
    selected = [name for name in selected if name not in options["excludes"]]
    roots = {}
    for name in selected:
        if name not in packages:
            raise ValueError(f"Unknown runtime package {name}")
        features = ["default"] if options["defaults"] else []
        features.extend(options["features"])
        if options["all_features"]:
            features.extend(packages[name].manifest.get("features", {}))
        roots[name] = sorted(set(features))
    target = options["target"]
    platform = target_configuration(target) if target else "native"
    testing = options["command"] == "test" or options["all_targets"] or options["selection"] in ("--test", "--example")
    graph = resolve_features(packages, roots, include_dev=testing, platform=platform)
    registry_prefix = "bazel/native-tools-cargo" if uses_native_tools(graph) else "bazel/cargo"
    registry_repo = "@runtime_native_tools_crates" if uses_native_tools(graph) else "@runtime_crates"
    signature = hashlib.sha256(json.dumps({"roots": roots, "platform": platform, "testing": testing}, sort_keys=True).encode()).hexdigest()[:16]
    directory = ROOT / "build/bazel-config" / signature
    variant = "test" if testing else ""

    def label_for(name, suffix):
        package = packages[name]
        if suffix == "host":
            return f"//{package.directory.relative_to(ROOT)}:{name}__host"
        return f"//{directory.relative_to(ROOT)}/{name}:{name}" + ("__" + suffix if suffix else "")

    reachable = set(roots)
    pending = list(roots)
    while pending:
        for dep in graph.dependencies(pending.pop(), ("dependencies", "dev-dependencies") if testing else ("dependencies",)):
            if dep.local and dep.local not in reachable:
                reachable.add(dep.local)
                pending.append(dep.local)
    for name in sorted(reachable):
        package = packages[name]
        owner = "//" + str(package.directory.relative_to(ROOT))
        destination = directory / name
        destination.mkdir(parents=True, exist_ok=True)
        authored = package.directory.relative_to(ROOT).parts[0] != "vendor"
        (destination / "BUILD.bazel").write_text(render_package(package, {variant: graph}, label_for,
            registry_prefix=registry_prefix, registry_repo=registry_repo,
            fixture_labels=["//:fixtures"], source_owner=owner, examples=authored,
            unit_tests=authored, integration_tests=authored))
    labels = []
    for name in selected:
        package = packages[name]
        prefix = f"//{directory.relative_to(ROOT)}/{name}:"
        build = (directory / name / "BUILD.bazel").read_text()
        targets = re.findall(r'name = "([^"]+)"', build)
        product = name + ("__test" if testing else "")
        if options["selection"] == "--test":
            if options["lib"]:
                labels.append(prefix + name + "__unit_test")
            labels.extend(prefix + name + "__test_" + test for test in options["tests"])
        elif options["selection"] == "--bin":
            binary = options["selector"] + ("__test" if testing else "")
            if options["selector"] == name and (package.directory / "src/lib.rs").is_file():
                binary += "__bin"
            labels.append(prefix + binary)
        elif options["selection"] == "--example":
            labels.append(prefix + options["selector"] + ("__test" if testing else ""))
        elif options["command"] == "test":
            labels += [prefix + target for target in targets if target == name + "__unit_test" or
                       (options["selection"] != "--lib" and target.startswith(name + "__test_"))]
        else:
            if product in targets:
                labels.append(prefix + product)
            labels += [prefix + target for target in targets if target in (product + "__staticlib", product + "__cdylib")]
            if options["selection"] != "--lib":
                binaries = {binary["name"] for binary in package.manifest.get("bin", [])}
                binaries.update(path.stem for path in package.directory.glob("src/bin/*.rs"))
                labels += [prefix + target for target in targets if target == product + "__bin" or target in {binary + ("__test" if testing else "") for binary in binaries}]
            if options["all_targets"]:
                labels += [prefix + target for target in targets if target == name + "__unit_test" or target.startswith(name + "__test_")]
                examples = {example["name"] for example in package.manifest.get("example", [])}
                examples.update(path.stem for path in package.directory.glob("examples/*.rs"))
                labels += [prefix + target for target in targets if target in {example + "__test" for example in examples}]
    if not labels:
        raise ValueError("The Rust selector selected no direct Bazel targets")
    return list(dict.fromkeys(labels))


def target_configuration(triple):
    os_name = ("emscripten" if triple.endswith("emscripten") else "unknown" if triple.startswith("wasm32")
               else "android" if triple.endswith("linux-android") else "macos" if triple.endswith("apple-darwin")
               else "ios" if "apple-ios" in triple else "tvos" if "apple-tvos" in triple
               else "visionos" if "apple-visionos" in triple else "windows" if "windows" in triple else "linux")
    return {"target_arch": triple.split("-", 1)[0], "target_os": os_name,
            "target_vendor": "apple" if "apple" in triple else "pc" if "-pc-" in triple else "unknown",
            "target_env": "msvc" if triple.endswith("msvc") else "gnu" if triple.endswith("gnu") else "",
            "target_abi": "sim" if triple.endswith("-sim") else ""}


def bazel_command():
    configured = os.environ.get("NUXIE_BAZEL_BIN") or os.environ.get("BAZEL")
    candidates = [configured] if configured else [
        str(ROOT / "target/bazel-tools/node_modules/.bin/bazelisk"),
        str(ROOT / "node_modules/.bin/bazelisk"),
        str(ROOT.parent.parent / "node_modules/.bin/bazelisk"),
        shutil.which("bazelisk"), shutil.which("bazel"),
    ]
    executable = next((item for item in candidates if item and (Path(item).is_file() or shutil.which(item))), None)
    if executable is None:
        raise ValueError("Bazelisk is required; install it or set NUXIE_BAZEL_BIN")
    startup = ["--nosystem_rc", "--nohome_rc"]
    for variable, flag in (("NUXIE_BAZEL_OUTPUT_USER_ROOT", "--output_user_root"), ("NUXIE_BAZEL_OUTPUT_BASE", "--output_base")):
        value = os.environ.get(variable)
        if value:
            if not Path(value).is_absolute():
                raise ValueError(variable + " must be absolute")
            startup.append(flag + "=" + value)
    if os.environ.get("NUXIE_BAZEL_BATCH", os.environ.get("CI", "0")) == "1":
        startup.append("--batch")
    return [executable, *startup]


def publish_artifact(source, destination):
    """Replace complete products atomically and preserve warm-build mtimes."""
    if destination.is_file() and filecmp.cmp(source, destination, shallow=False):
        return
    descriptor, scratch = tempfile.mkstemp(prefix=destination.name + ".", dir=destination.parent)
    os.close(descriptor)
    try:
        shutil.copy2(source, scratch)
        os.replace(scratch, destination)
    finally:
        Path(scratch).unlink(missing_ok=True)


def main():
    try:
        options = command_options(sys.argv[1:])
        labels = materialize(options, packages_from_workspace())
        bazel = bazel_command()
        flags = ["--config=release"] if options["profile"] != "debug" else []
        flags += ["--@rules_rust//rust/settings:extra_rustc_flag=" + flag for flag in shlex.split(os.environ.get("RUSTFLAGS", ""))]
        flags += BazelRuntime.provenance_options()
        flags += ["--action_env=" + name for name in sorted(os.environ) if name not in PROVENANCE_KEYS and name.startswith(("NUX_RUNTIME_", "RIVE_", "EMSDK", "EM_CONFIG", "EMDAWN"))]
        if options["profile"] == "release-size":
            flags += ["--@rules_rust//rust/settings:extra_rustc_flag=-Copt-level=" + os.environ.get("CARGO_PROFILE_RELEASE_SIZE_OPT_LEVEL", "z"), "--@rules_rust//rust/settings:extra_rustc_flag=-Cstrip=symbols"]
        elif options["profile"] == "release-apple":
            flags += ["--@rules_rust//rust/settings:extra_rustc_flag=-Cstrip=debuginfo"]
        if options["jobs"]:
            flags.append("--jobs=" + options["jobs"])
        if options["target"]:
            if options["target"].startswith("wasm32"):
                flags += ["--platforms=@rules_rust//rust/platform:wasm", "--@rules_rust//rust/settings:extra_rustc_flag=-Cpanic=abort"]
            elif "apple" in options["target"] or options["target"].endswith("linux-android"):
                flags += BazelRuntime.platform_options(options["target"])
            else:
                flags.append("--platforms=//bazel/platforms:" + options["target"])
        runtime_args = list(options["runtime_args"])
        if options["command"] == "test" and not options["no_run"]:
            if options["test_filter"]:
                runtime_args.insert(0, options["test_filter"])
            flags += ["--test_output=all", *["--test_arg=" + arg for arg in runtime_args]]
            flags += ["--test_env=" + name for name in sorted(os.environ) if name.startswith(("NUXIE_", "RIVE_", "MTL_"))]
        if options["command"] == "clippy":
            flags += ["--aspects=@rules_rust//rust:defs.bzl%rust_clippy_aspect", "--output_groups=clippy_checks"]
        command = "test" if options["command"] == "test" and not options["no_run"] else "build"
        subprocess.run([*bazel, command, *flags, *labels], cwd=ROOT, check=True)
        if command == "test" or options["command"] == "clippy":
            return 0
        output = subprocess.check_output([*bazel, "cquery", *flags, "set(" + " ".join(labels) + ")", "--output=files"], cwd=ROOT, text=True)
        target_dir = Path(options["target_dir"] or os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
        if options["target"]:
            target_dir /= options["target"]
        target_dir /= options["profile"]
        target_dir.mkdir(parents=True, exist_ok=True)
        products = []
        for path in output.splitlines():
            source = ROOT / path
            if source.is_file():
                destination = target_dir / source.name
                publish_artifact(source, destination)
                products.append(destination)
                if options["json"]:
                    print(json.dumps({"reason": "compiler-artifact", "executable": str(destination) if os.access(destination, os.X_OK) else None, "filenames": [str(destination)], "target": {"name": options["selector"] or destination.name}, "profile": {"test": options["command"] == "test"}}))
        if options["command"] == "run":
            executables = [path for path in products if os.access(path, os.X_OK) and path.suffix not in (".a", ".so", ".dylib", ".rlib")]
            if len(executables) != 1:
                raise ValueError("run requires exactly one binary or example")
            return subprocess.run([str(executables[0]), *runtime_args], cwd=ROOT).returncode
        return 0
    except (ValueError, IndexError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
