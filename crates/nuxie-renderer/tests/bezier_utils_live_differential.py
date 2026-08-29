#!/usr/bin/env python3
"""Compile the repo-owned Bézier GLSL and run its pinned C++ test case."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


PIN = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
CASES = {
    "find_cubic_coeffs_tangents_glsl",
    "clamped_divide_glsl",
    "find_cubic_max_height_glsl",
    "measure_cubic_local_curvature_glsl",
}


def run(command: list[str], *, cwd: Path | None = None) -> str:
    result = subprocess.run(command, cwd=cwd, text=True, capture_output=True)
    if result.returncode:
        raise SystemExit(
            f"command failed ({result.returncode}): {' '.join(command)}\n"
            f"{result.stdout}{result.stderr}"
        )
    return result.stdout


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--case", required=True, choices=sorted(CASES))
    parser.add_argument(
        "--upstream-root",
        type=Path,
        default=Path(os.environ.get("RIVE_RUNTIME_DIR", "/Users/levi/dev/oss/rive-runtime")),
    )
    args = parser.parse_args()

    repo = Path(__file__).resolve().parents[3]
    upstream = args.upstream_root.resolve()
    actual_pin = run(["git", "rev-parse", "HEAD"], cwd=upstream).strip()
    if actual_pin != PIN:
        raise SystemExit(f"upstream checkout is {actual_pin}; expected {PIN}")

    rust_shader = repo / (
        "crates/nuxie-renderer/src/mechanical_port/shader-build-authority/source/"
        "renderer_src_shaders_bezier_utils_glsl__generated_input.source"
    )
    cpp_shader = upstream / "renderer/src/shaders/bezier_utils.glsl"
    if rust_shader.read_bytes() != cpp_shader.read_bytes():
        raise SystemExit("repo-owned production Bézier GLSL differs from the pinned source")

    compiler = shutil.which("clang++")
    if compiler is None:
        raise SystemExit("clang++ is required for the live shader differential")

    with tempfile.TemporaryDirectory(prefix="nuxie-bezier-differential-") as raw_temp:
        temp = Path(raw_temp)
        generated = temp / "include/generated/shaders"
        generated.mkdir(parents=True)
        shutil.copyfile(rust_shader, generated / "bezier_utils.minified.glsl")
        binary = temp / "bezier_utils_test"
        compile_command = [
            compiler,
            "-std=c++17",
            "-O0",
            "-g",
            "-ffp-contract=off",
            "-I",
            str(upstream / "include"),
            "-I",
            str(upstream / "tests/include"),
            "-I",
            str(upstream / "tests"),
            "-I",
            str(upstream / "tests/unit_tests"),
            "-I",
            str(temp / "include"),
            str(upstream / "tests/unit_tests/runtime/main_test.cpp"),
            str(upstream / "tests/unit_tests/runtime/bezier_utils_test.cpp"),
            str(upstream / "src/math/bezier_utils.cpp"),
            str(upstream / "src/math/vec2d.cpp"),
            "-o",
            str(binary),
        ]
        run(compile_command)
        output = run([str(binary), args.case, "-r", "compact"])
        if "Passed 1 test case" not in output:
            raise SystemExit(f"pinned case did not run exactly once:\n{output}")
        print(output, end="")


if __name__ == "__main__":
    main()
