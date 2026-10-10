#!/usr/bin/env python3
"""Direct Rust distribution builds and their configured Bazel input evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

BAZEL_VERSION = "9.3.0"
RUST_VERSION = "1.94.1"
ANDROID_LABEL = "//crates/nux-capi:nux-capi__android__cdylib"
APPLE_LABEL = "//crates/nux-apple-product-extension:nux-apple-product-extension__apple__staticlib"
PROVENANCE_KEYS = (
    "NUX_RUNTIME_SOURCE_REVISION", "NUX_RUNTIME_BUILD_INPUTS_HASH",
    "NUX_RUNTIME_CONTRACT_FINGERPRINT", "NUX_RUNTIME_BUILD_PROFILE",
    "NUX_RUNTIME_RUSTC_VERSION", "NUX_RUNTIME_DISTRIBUTION_ROOT_PACKAGE",
)


def canonical(document):
    return (json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n").encode()


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def tree_digest(root):
    root = Path(root)
    records = [{"path": path.relative_to(root).as_posix(), "sha256": digest(path)}
               for path in sorted(root.rglob("*")) if path.is_file()
               and path.name not in (".cargo-ok",) and "__pycache__" not in path.parts]
    if not records:
        raise ValueError(f"empty toolchain/source directory: {root}")
    return hashlib.sha256(canonical(records)).hexdigest()


class BazelRuntime:
    def __init__(self, repo_root):
        self.root = Path(repo_root).resolve()
        configured = os.environ.get("NUXIE_BAZEL_BIN")
        candidates = [configured] if configured else [
            str(self.root / "target/bazel-tools/node_modules/.bin/bazelisk"),
            str(self.root / "node_modules/.bin/bazelisk"),
            str(self.root.parent.parent / "node_modules/.bin/bazelisk"),
            shutil.which("bazelisk"), shutil.which("bazel"),
        ]
        self.executable = next((Path(item).resolve() for item in candidates
                                if item and Path(item).is_file()), None)
        if self.executable is None:
            raise ValueError("Bazelisk is required; it reads the pinned .bazelversion")
        self.startup = ["--nosystem_rc", "--nohome_rc"]
        output_root = os.environ.get("NUXIE_BAZEL_OUTPUT_USER_ROOT")
        if output_root:
            if not Path(output_root).is_absolute():
                raise ValueError("NUXIE_BAZEL_OUTPUT_USER_ROOT must be absolute")
            self.startup.append(f"--output_user_root={output_root}")
        output_base = os.environ.get("NUXIE_BAZEL_OUTPUT_BASE")
        if output_base:
            if not Path(output_base).is_absolute():
                raise ValueError("NUXIE_BAZEL_OUTPUT_BASE must be absolute")
            self.startup.append(f"--output_base={output_base}")
        if os.environ.get("NUXIE_BAZEL_BATCH", os.environ.get("CI", "0")) == "1":
            self.startup.append("--batch")
        if self.run("info", "release").strip() != f"release {BAZEL_VERSION}":
            raise ValueError(f"runtime distributions require Bazel {BAZEL_VERSION}")
        self.execroot = Path(self.run("info", "execution_root").strip())
        self.external_root = Path(self.run("info", "output_base").strip()) / "external"
        self.installbase = Path(self.run("info", "install_base").strip())

    def run(self, command, *arguments):
        result = subprocess.run([str(self.executable), *self.startup, command, *arguments],
                                cwd=self.root, capture_output=True, text=True, check=False)
        if result.returncode:
            raise ValueError(f"Bazel {command} failed:\n{result.stderr}")
        if command == "build":
            sys.stderr.write(result.stderr)
        return result.stdout

    @staticmethod
    def provenance_options():
        flags = []
        for key in PROVENANCE_KEYS:
            if key not in os.environ:
                continue
            value = os.environ[key]
            if (any(character in value for character in ("\0", "\n", "\r"))
                    or value.endswith("\\") or "${pwd}" in value):
                raise ValueError(f"{key} must be one env-file line without a trailing backslash or reserved ${{pwd}} marker")
            flags.append(f"--define={key}={value}")
        return flags

    @staticmethod
    def options(target, *, ios="15.0", macos="12.0"):
        flags = ["--config=release",
                 "--@rules_rust//rust/settings:lto=fat",
                 "--@rules_rust//rust/settings:codegen_units=1",
                 "--@rules_rust//rust/settings:extra_rustc_flag=-Cdebuginfo=0"]
        jobs = os.environ.get("NUXIE_BAZEL_JOBS")
        if jobs:
            if not jobs.isdecimal() or int(jobs) < 1:
                raise ValueError("NUXIE_BAZEL_JOBS must be a positive integer")
            flags.append(f"--jobs={jobs}")
        if not target.endswith("linux-android"):
            flags.append("--@rules_rust//rust/settings:extra_rustc_flag=-Cstrip=debuginfo")
        return flags + BazelRuntime.platform_options(target, ios=ios, macos=macos)

    @staticmethod
    def platform_options(target, *, ios="15.0", macos="12.0"):
        flags = [f"--platforms=//bazel/platforms:{target}"]
        if target.endswith("linux-android"):
            flags.extend(["--extra_toolchains=@androidndk//:all",
                          "--@rules_rust//rust/settings:extra_rustc_flag=-Clink-arg=-Wl,-z,max-page-size=16384",
                          "--@rules_rust//rust/settings:extra_rustc_flag=-Clink-arg=-lc++_shared"])
            ndk = os.environ.get("ANDROID_NDK_HOME") or os.environ.get("ANDROID_NDK_ROOT")
            if not ndk and os.environ.get("ANDROID_HOME"):
                ndk = str(Path(os.environ["ANDROID_HOME"]) / "ndk/29.0.14206865")
            if not ndk:
                raise ValueError("Android requires NDK 29.0.14206865 via ANDROID_NDK_HOME")
            ndk = Path(ndk).resolve()
            flags.append(f"--repo_env=ANDROID_NDK_HOME={ndk}")
            prebuilts = list((ndk / "toolchains/llvm/prebuilt").glob("*"))
            if len(prebuilts) != 1:
                raise ValueError("Android NDK must expose exactly one host prebuilt")
            sysroot = prebuilts[0] / "sysroot"
            bindgen = f"--target={target}23 --sysroot={sysroot} -I{sysroot}/usr/include/{target}"
        else:
            flags.extend([f"--ios_minimum_os={ios}", f"--macos_minimum_os={macos}"])
            xcode = subprocess.check_output(["xcodebuild", "-version"], text=True)
            xcode_version = re.search(r"^Xcode (\S+)$", xcode, re.MULTILINE)
            if xcode_version is None:
                raise ValueError("cannot determine the selected Xcode version")
            flags.append(f"--xcode_version={xcode_version.group(1)}")
            developer = os.environ.get("DEVELOPER_DIR")
            if developer:
                flags.append(f"--repo_env=DEVELOPER_DIR={Path(developer).resolve()}")
            sdk = "macosx" if target.endswith("darwin") else "iphoneos" if target == "aarch64-apple-ios" else "iphonesimulator"
            sdk_root = subprocess.check_output(["xcrun", "--sdk", sdk, "--show-sdk-path"], text=True).strip()
            arch = "arm64" if target.startswith("aarch64") else "x86_64"
            triple = f"{arch}-apple-macos{macos}" if sdk == "macosx" else f"{arch}-apple-ios{ios}" + ("-simulator" if sdk == "iphonesimulator" else "")
            bindgen = f"--target={triple} --sysroot={sdk_root}"
        flags.append(f"--action_env=BINDGEN_EXTRA_CLANG_ARGS={bindgen}")
        return flags

    def toolchain(self, label, target, **versions):
        graph = json.loads(self.run("aquery", f'mnemonic("Rustc", {label})',
                                   *self.options(target, **versions), "--output=jsonproto"))
        actions = graph.get("actions", [])
        if len(actions) != 1:
            raise ValueError(f"shipping target must have one Rustc action: {label}")
        action = actions[0]
        arguments = action["arguments"]
        rustc_argument = arguments[arguments.index("--") + 1]
        match = re.search(r"(?:^|/)external/([^/]+)/rust_toolchain/bin/rustc$", rustc_argument)
        if match is None:
            raise ValueError(f"cannot resolve selected Rust toolchain: {rustc_argument}")
        repository = self.external_root / match[1]
        rustc = repository / "bin/rustc"
        version = subprocess.check_output([rustc, "-vV"], text=True).strip().replace("\n", " ")
        if not version.startswith(f"rustc {RUST_VERSION} "):
            raise ValueError(f"selected Rust compiler must be {RUST_VERSION}: {version}")
        return repository, rustc, version

    def closure(self, label, target, **versions):
        encoded = self.run("cquery", f"deps({label})", *self.options(target, **versions),
                           "--@rules_rust//rust/settings:collect_cfgs",
                           "--output=starlark", "--starlark:file=tools/bazel_runtime_query.bzl")
        crates = [json.loads(line) for line in encoded.splitlines() if line.strip()]
        if not any(crate["label"].endswith(label) for crate in crates):
            raise ValueError(f"configured closure omits shipping root: {label}")
        return crates

    def evidence(self, label, targets, packaging_inputs, **versions):
        files = set(packaging_inputs)
        files.update(path.relative_to(self.root).as_posix() for path in self.root.rglob("BUILD.bazel")
                     if not any(part in ("target", ".git") or part.startswith("bazel-") for part in path.relative_to(self.root).parts))
        files.update(path.relative_to(self.root).as_posix() for path in self.root.rglob("*.bzl")
                     if not any(part in ("target", ".git") or part.startswith("bazel-") for part in path.relative_to(self.root).parts))
        files.update(["Cargo.toml", "Cargo.lock", ".bazelrc", ".bazelversion", "MODULE.bazel", "MODULE.bazel.lock",
                      "bazel/cargo/Cargo.toml", "bazel/cargo/Cargo.lock", "bazel/cargo/Cargo.Bazel.lock",
                      "tools/bazel_runtime_build.py", "tools/bazel_runtime_query.bzl"])
        packages = {}
        libraries = {}
        compiler = None
        rustc_version = None
        tools = {"bazel-launcher": digest(self.executable),
                 "bazel-server": digest(self.installbase / "A-server.jar")}
        workspace = tomllib.loads((self.root / "Cargo.toml").read_text())
        locked = {(item["name"], item["version"]): item for item in
                  tomllib.loads((self.root / "Cargo.lock").read_text()).get("package", [])}
        for target in targets:
            repository, compiler, rustc_version = self.toolchain(label, target, **versions)
            libraries[target] = tree_digest(repository / "lib/rustlib" / target / "lib")
            tools[f"rustc:{target}"] = digest(compiler)
            tools[f"rustc-libraries:{target}"] = tree_digest(repository / "lib")
            for crate in self.closure(label, target, **versions):
                if crate["root"].startswith("external/") and "runtime_crates__" not in crate["root"].split("/")[1]:
                    continue
                for relative in crate["files"]:
                    if not relative.startswith("external/"):
                        files.add(relative)
                source_path = (self.external_root / crate["root"].removeprefix("external/")
                               if crate["root"].startswith("external/")
                               else self.root / crate["root"])
                directory = source_path.parent
                while not (directory / "Cargo.toml").is_file():
                    if directory == directory.parent:
                        raise ValueError(f"crate has no Cargo manifest: {crate['label']}")
                    directory = directory.parent
                manifest = tomllib.loads((directory / "Cargo.toml").read_text())
                package = manifest["package"]
                name = package["name"]
                version = package.get("version", workspace.get("workspace", {}).get("package", {}).get("version"))
                if isinstance(version, dict):
                    version = workspace["workspace"]["package"]["version"]
                external = crate["root"].startswith("external/")
                if not external:
                    files.add(directory.relative_to(self.root).as_posix() + "/Cargo.toml")
                entry = locked.get((name, version)) if external else None
                source = entry.get("source") if entry else None
                if external and source is None:
                    raise ValueError(f"external crate is absent from Cargo.lock: {name} {version}")
                key = (name, version, source or "")
                if key not in packages:
                    packages[key] = {
                        "checksum": entry.get("checksum") if entry else None,
                        "lockEntryHash": hashlib.sha256(canonical(entry)).hexdigest() if entry else None,
                        "manifestPath": None if external else directory.relative_to(self.root).as_posix() + "/Cargo.toml",
                        "name": name, "resolvedSourceHash": tree_digest(directory) if external else None,
                        "source": source, "targets": {}, "version": version,
                    }
                record = packages[key]
                features = [cfg.removeprefix('feature="').removesuffix('"') for cfg in crate["cfgs"] if cfg.startswith('feature="')]
                record["targets"][target] = sorted(set(record["targets"].get(target, [])) | set(features))
        records = []
        for relative in sorted(files):
            path = self.root / relative
            if not path.is_file():
                raise ValueError(f"Bazel distribution input is missing: {relative}")
            records.append({"kind": "bazel-input", "path": relative, "sha256": digest(path)})
        return {"files": records, "packages": [packages[key] for key in sorted(packages)],
                "rustLibraries": libraries, "toolBinaries": tools,
                "rustc": rustc_version, "compiler": str(compiler),
                "bazel": f"bazel {BAZEL_VERSION}", "label": label,
                "platforms": {target: f"//bazel/platforms:{target}" for target in sorted(targets)}}

    def build(self, label, target, output, **versions):
        flags = self.options(target, **versions)
        flags.extend(self.provenance_options())
        self.run("build", label, *flags)
        files = self.run("cquery", label, *flags, "--output=files").strip().splitlines()
        extension = ".so" if target.endswith("linux-android") else ".a"
        matches = [self.execroot / item for item in files if item.endswith(extension)]
        if len(matches) != 1:
            raise ValueError(f"shipping target must produce exactly one {extension}: {files}")
        Path(output).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(matches[0], output)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("build", "toolchain"))
    parser.add_argument("--repo-root", default=Path(__file__).resolve().parents[1], type=Path)
    parser.add_argument("--label", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--minimum-ios-version", default="15.0")
    parser.add_argument("--minimum-macos-version", default="12.0")
    args = parser.parse_args()
    runtime = BazelRuntime(args.repo_root)
    versions = {"ios": args.minimum_ios_version, "macos": args.minimum_macos_version}
    if args.command == "build":
        if args.output is None:
            parser.error("build requires --output")
        runtime.build(args.label, args.target, args.output, **versions)
    else:
        _, compiler, version = runtime.toolchain(args.label, args.target, **versions)
        host = re.search(r" host: (\S+)", version).group(1)
        objcopy = compiler.parent.parent / "lib/rustlib" / host / "bin/llvm-objcopy"
        print(json.dumps({"rustc": str(compiler), "version": version, "objcopy": str(objcopy)}))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"bazel-runtime-build: {error}") from error
