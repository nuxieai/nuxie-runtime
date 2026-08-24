#!/usr/bin/env python3
"""Connect wasm-bindgen's raw Dawn ABI imports to the browser WebGPU host."""

from pathlib import Path
import re
import sys


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: inject_webgpu_imports.py <wasm-bindgen-js>")

    path = Path(sys.argv[1])
    source = path.read_text()
    env_imports = re.findall(r'^import \* as import\d+ from "env"\n', source, re.MULTILINE)
    env_entries = re.findall(r'^        "env": import\d+,\n', source, re.MULTILINE)
    if not env_imports or len(env_imports) != len(env_entries):
        raise SystemExit(
            "wasm-bindgen raw import shape changed: "
            f"imports={len(env_imports)} entries={len(env_entries)}"
        )

    source = re.sub(r'^import \* as import\d+ from "env"\n', "", source, flags=re.MULTILINE)
    source = source.replace(
        '/* @ts-self-types="./webgpu_renderer_replay.d.ts" */\n',
        '/* @ts-self-types="./webgpu_renderer_replay.d.ts" */\n'
        'import { createWebGpuImports } from "../webgpu-host.js";\n',
        1,
    )
    source, replacements = re.subn(
        r'(?:^        "env": import\d+,\n)+',
        '        "env": createWebGpuImports(() => wasm),\n',
        source,
        flags=re.MULTILINE,
    )
    if replacements != 1:
        raise SystemExit(f"expected one raw env import block, replaced {replacements}")
    path.write_text(source)


if __name__ == "__main__":
    main()
