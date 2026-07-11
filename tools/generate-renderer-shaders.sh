#!/bin/bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
shader_dir="$rive_runtime/renderer/src/shaders"
output_dir="$root/crates/nuxie-renderer/src/generated"
venv="$root/target/renderer-shader-venv"
export PATH="$HOME/.cargo/bin:$PATH"

for tool in glslc spirv-opt naga; do
    if ! command -v "$tool" >/dev/null; then
        echo "missing shader tool: $tool" >&2
        exit 1
    fi
done

if [[ ! -x "$venv/bin/python3" ]]; then
    python3 -m venv "$venv"
    "$venv/bin/pip" install ply
fi

mkdir -p "$output_dir"
rm -f "$output_dir"/*.wgsl

PATH="$venv/bin:$HOME/.cargo/bin:$PATH" make -C "$shader_dir" wgsl
while IFS= read -r header; do
    source="${header#"$shader_dir/"}"
    source="${source%.hpp}.wgsl"
    PATH="$venv/bin:$HOME/.cargo/bin:$PATH" make -C "$shader_dir" "$source"
    cp "$shader_dir/$source" "$output_dir/$(basename "$source")"
done < <(find "$shader_dir/out/generated/wgsl" -maxdepth 1 -name '*.hpp' | sort)

echo "generated $(find "$output_dir" -name '*.wgsl' | wc -l | tr -d ' ') WGSL modules"
