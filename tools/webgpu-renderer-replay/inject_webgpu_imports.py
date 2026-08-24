#!/usr/bin/env python3
"""Connect wasm-bindgen's raw Dawn ABI imports to the browser WebGPU host."""

from pathlib import Path
import re
import sys


def main() -> None:
    if len(sys.argv) not in (2, 3):
        raise SystemExit(
            "usage: inject_webgpu_imports.py <wasm-bindgen-js> [host-import-path]"
        )

    path = Path(sys.argv[1])
    host_import_path = sys.argv[2] if len(sys.argv) == 3 else "../webgpu-host.js"
    source = path.read_text()
    env_imports = re.findall(r'^import \* as import\d+ from "env"\n', source, re.MULTILINE)
    env_entries = re.findall(r'^        "env": import\d+,\n', source, re.MULTILINE)
    if not env_imports or len(env_imports) != len(env_entries):
        raise SystemExit(
            "wasm-bindgen raw import shape changed: "
            f"imports={len(env_imports)} entries={len(env_entries)}"
        )

    source = re.sub(r'^import \* as import\d+ from "env"\n', "", source, flags=re.MULTILINE)
    source, marker_replacements = re.subn(
        r'(/\* @ts-self-types="[^"]+" \*/\n)',
        rf'\1import {{ createWebGpuImports }} from "{host_import_path}";\n',
        source,
        count=1,
    )
    if marker_replacements != 1:
        raise SystemExit(
            f"expected one wasm-bindgen self-types marker, replaced {marker_replacements}"
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
