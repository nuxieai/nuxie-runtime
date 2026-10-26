#!/usr/bin/env python3
"""Build qualification hosts on the configured pool; never compile locally."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import sys
import uuid

ROOT = Path(__file__).resolve().parents[2]
PREFIX = "//tools/audio-device-validation"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("target", choices=["test", "mac", "ios", "android", "android-apk", "luau-test", "luau-mac", "luau-ios", "luau-android"])
    parser.add_argument("--library", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    native_path = Path.home() / ".local/share/nuxie-homelab/native.py"
    spec = importlib.util.spec_from_file_location("audio_pool_native", native_path)
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    sys.path.insert(0, str(ROOT / "tools"))
    from bazel_runtime_build import BazelRuntime
    os.environ["NUXIE_HOMELAB_RUNTIME_INVOCATION"] = str(uuid.uuid4())
    if args.target == "android-apk":
        if args.library is None or args.output is None:
            parser.error("android-apk requires --library and --output")
        library = str(args.library.resolve(strict=True))
        output = str(args.output.resolve())
        os.environ["NUXIE_HOMELAB_RUNTIME_IO"] = json.dumps({
            "schema": "nuxie-runtime-io.v1", "platform": "macos", "inputs": [library],
            "outputs": [{"path": output, "directory": False}],
        })
        result = native.run(ROOT, "run", [PREFIX + "/android:package"],
                            _required_platform="macos",
                            _run_arguments=["--library", library, "--output", output])
    else:
        platform, triple, label = {
            "test": ("linux", None, PREFIX + ":probe_test"),
            "luau-test": ("linux", None, PREFIX + ":luau_probe_test"),
            "luau-mac": ("macos", None, PREFIX + ":luau_macos_probe"),
            "luau-ios": ("macos", "aarch64-apple-ios", PREFIX + "/apple:ios_luau_host"),
            "luau-android": ("macos", "aarch64-linux-android", PREFIX + ":luau_probe__cdylib"),
            "mac": ("macos", None, PREFIX + ":macos_probe"),
            "ios": ("macos", "aarch64-apple-ios", PREFIX + "/apple:ios_host"),
            "android": ("macos", "aarch64-linux-android", PREFIX + ":audio-device-validation__cdylib"),
        }[args.target]
        flags = BazelRuntime.provenance_options()
        if triple:
            flags += BazelRuntime.platform_options(triple, ios="16.0", macos="12.0")
        test = args.target in ("test", "luau-test")
        if test:
            flags += ["--test_output=all"]
        pointer = ROOT / ".nuxie" / ("audio-validation-" + args.target + "-products.json")
        pointer.parent.mkdir(parents=True, exist_ok=True)
        result = native.run(ROOT, "test" if test else "build", flags + [label],
                            _required_platform=platform, _products=not test,
                            _pointer=None if test else pointer)
        if result == 0 and not test:
            products = json.loads(pointer.read_text())
            for output in products["outputs"]:
                print(ROOT / ".nuxie/homelab" / products["runId"] / output)
    if result is None:
        raise RuntimeError("pool unavailable; no local build fallback")
    return result


if __name__ == "__main__":
    raise SystemExit(main())
