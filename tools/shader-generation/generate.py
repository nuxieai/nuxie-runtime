#!/usr/bin/env python3
"""Bazel action: run the existing pinned recipes with declared compiler inputs."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from verify_inputs import verify_inputs


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    for flag in ("manifest", "generator", "overlay", "ply-init", "naga", "glslang", "spirv-opt", "output"):
        parser.add_argument("--" + flag, required=True, type=Path)
    args = parser.parse_args()
    # The manifest is the root-level export of the checksummed source archive.
    source_root = args.manifest.resolve().parent
    revision = json.loads(args.manifest.read_text())["revision"]
    manifest = verify_inputs(args.manifest, source_root, revision)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="renderer-shaders-", dir=output.parent) as temporary:
        work = Path(temporary)
        runtime = work / "upstream"
        shaders = runtime / "renderer/src/shaders"
        shutil.copytree(source_root / "renderer/src/shaders", shaders)
        ply_root = runtime / "renderer/dependencies/dabeaz_ply_3.11"
        shutil.copytree(args.ply_init.resolve().parent, ply_root / "ply")
        repo = work / "repo"
        (repo / "tools/shader-generation").mkdir(parents=True)
        shutil.copyfile(args.generator, repo / "tools/generate-renderer-shaders.sh")
        shutil.copyfile(Path(__file__).with_name("verify_inputs.py"), repo / "tools/shader-generation/verify_inputs.py")
        (repo / "tools/renderer-shaders").mkdir()
        shutil.copyfile(args.overlay, repo / "tools/renderer-shaders/clockwise_atomic_path_webgpu.main")
        bin_dir = work / "bin"
        bin_dir.mkdir()
        compilers = {"naga": args.naga, "glslangValidator": args.glslang, "spirv-opt": args.spirv_opt}
        for name, path in compilers.items():
            path = path.resolve(strict=True)
            if not os.access(path, os.X_OK):
                raise ValueError(f"declared compiler is not executable: {name}")
            (bin_dir / name).symlink_to(path)
        # Do not inherit PATH, PYTHONPATH, MAKEFLAGS, HOME, or user compiler flags.
        # Standard shell/Python/make utilities belong to the execution platform;
        # all shader compilers and Python PLY files are declared action inputs.
        env = {
            "PATH": f"{bin_dir}:/usr/bin:/bin",
            "HOME": str(work),
            "TMPDIR": str(work),
            "LANG": "C", "LC_ALL": "C", "TERM": "dumb",
            "PYTHONHASHSEED": "0", "PYTHONDONTWRITEBYTECODE": "1",
            "PYTHONPATH": str(ply_root),
            "RIVE_RUNTIME_DIR": str(runtime),
            "RENDERER_SHADER_OUTPUT_DIR": str(output / "wgsl"),
            "RENDERER_SHADER_UPSTREAM_OUT": str(output / "upstream"),
        }
        subprocess.run(["/bin/bash", str(repo / "tools/generate-renderer-shaders.sh"),
                        "--declared-inputs", str(args.manifest.resolve())], env=env, check=True)
        subprocess.run(["make", "-C", str(shaders), f"OUT={output / 'upstream'}", "minify", "spirv"], env=env, check=True)
        # Capture the declared upstream generator itself. Feature objects use
        # identity hashing, so preserve this emission's bytes and provenance
        # rather than claiming that PYTHONHASHSEED fixes their iteration order.
        metal_generator = shaders / "metal/generate_draw_combinations.py"
        metal_capture = output / "upstream/draw_combinations.metal"
        subprocess.run(["/usr/bin/python3", str(metal_generator), str(metal_capture)],
                       env=env, check=True)
        provenance = {
            "revision": revision,
            "source_manifest_sha256": sha256(args.manifest),
            "source_files": manifest["files"],
            "generator_sha256": sha256(args.generator),
            "overlay_sha256": sha256(args.overlay),
            "compiler_sha256": {name: sha256(path) for name, path in compilers.items()},
            "metal_draw_combinations": {
                "revision": revision,
                "generator_path": "renderer/src/shaders/metal/generate_draw_combinations.py",
                "generator_sha256": sha256(metal_generator),
                "output_path": "upstream/draw_combinations.metal",
                "output_sha256": sha256(metal_capture),
            },
        }
        (output / "provenance.json").write_text(json.dumps(provenance, sort_keys=True, indent=2) + "\n")


if __name__ == "__main__":
    main()
