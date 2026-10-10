#!/usr/bin/env python3
"""Publish Bazel's pinned wasm-bindgen into this checkout's private tools."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from runtime import ROOT, bazel_command


def publish(source, destination):
    if destination.is_symlink():
        raise ValueError("Refusing a symlinked browser tool output: " + str(destination))
    if destination.is_file() and destination.read_bytes() == source.read_bytes():
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    descriptor, scratch = tempfile.mkstemp(prefix=destination.name + ".", dir=destination.parent)
    os.close(descriptor)
    try:
        shutil.copyfile(source, scratch)
        Path(scratch).chmod(0o755)
        os.replace(scratch, destination)
    finally:
        Path(scratch).unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "target/browser-tools")
    args = parser.parse_args()
    bazel = bazel_command()
    label = "//bazel:wasm_bindgen"
    flags = ["--jobs=" + os.environ.get("NUXIE_BAZEL_JOBS", "2")]
    subprocess.run([*bazel, "build", *flags, label], cwd=ROOT, check=True)
    products = subprocess.check_output([*bazel, "cquery", label, "--output=files"], cwd=ROOT, text=True).splitlines()
    if len(products) != 1:
        raise ValueError("Expected exactly one wasm-bindgen product")
    execution_root = Path(subprocess.check_output([*bazel, "info", "execution_root"], cwd=ROOT, text=True).strip())
    product = Path(products[0])
    if not product.is_absolute():
        product = execution_root / product
    if subprocess.check_output([str(product), "--version"], text=True).strip() != "wasm-bindgen 0.2.126":
        raise ValueError("Unexpected pinned wasm-bindgen version")
    destination = args.output.resolve() / "bin/wasm-bindgen"
    publish(product, destination)
    print(destination)


if __name__ == "__main__":
    main()
