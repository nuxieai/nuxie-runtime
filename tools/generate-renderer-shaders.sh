#!/bin/bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
shader_dir="$rive_runtime/renderer/src/shaders"
output_dir="${RENDERER_SHADER_OUTPUT_DIR:-$root/crates/nuxie-renderer/src/generated}"
upstream_out="${RENDERER_SHADER_UPSTREAM_OUT:-$shader_dir/out/generated}"
venv="$root/target/renderer-shader-venv"
export PATH="$HOME/.cargo/bin:$PATH"
# Upstream's WGSL header minifier assigns short identifiers while iterating
# Python sets. Fix the hash seed so its compiler-input headers are byte-stable.
export PYTHONHASHSEED=0

expected_runtime_revision="7c778d13c5d903b3b74eec1dd6bb68a811dea5f2"
expected_naga_version="30.0.0"
expected_glslang_version="Glslang Version: 11:16.2.0"
expected_spirv_tools_version="SPIRV-Tools v2026.1 unknown hash, 2026-01-22T19:45:19+00:00"
expected_ply_version="3.11"

for tool in glslangValidator spirv-opt naga; do
    if ! command -v "$tool" >/dev/null; then
        echo "missing shader tool: $tool" >&2
        exit 1
    fi
done

runtime_revision="$(git -C "$rive_runtime" rev-parse HEAD)"
if [[ "$runtime_revision" != "$expected_runtime_revision" ]]; then
    echo "wrong rive-runtime revision: expected $expected_runtime_revision, got $runtime_revision" >&2
    exit 1
fi
if ! git -C "$rive_runtime" diff --quiet HEAD -- renderer/src/shaders; then
    echo "rive-runtime shader sources have tracked or staged changes" >&2
    exit 1
fi
untracked_runtime_sources="$(
    git -C "$rive_runtime" ls-files --others --exclude-standard -- renderer/src/shaders \
        | grep -v '^renderer/src/shaders/out/' || true
)"
if [[ -n "$untracked_runtime_sources" ]]; then
    echo "rive-runtime shader sources have untracked inputs:" >&2
    printf '%s\n' "$untracked_runtime_sources" >&2
    exit 1
fi
if ! git -C "$root" diff --quiet HEAD -- tools/renderer-shaders; then
    echo "local clockwise-atomic shader sources have tracked or staged changes" >&2
    exit 1
fi
untracked_local_sources="$(
    git -C "$root" ls-files --others --exclude-standard -- tools/renderer-shaders || true
)"
if [[ -n "$untracked_local_sources" ]]; then
    echo "local clockwise-atomic shader sources have untracked inputs:" >&2
    printf '%s\n' "$untracked_local_sources" >&2
    exit 1
fi

naga_version="$(naga --version)"
glslang_version="$(glslangValidator --version | head -n 1)"
spirv_tools_version="$(spirv-opt --version 2>&1 | head -n 1)"
if [[ "$naga_version" != "$expected_naga_version" ]]; then
    echo "wrong naga version: expected $expected_naga_version, got $naga_version" >&2
    exit 1
fi
if [[ "$glslang_version" != "$expected_glslang_version" ]]; then
    echo "wrong glslangValidator version: expected '$expected_glslang_version', got '$glslang_version'" >&2
    exit 1
fi
if [[ "$spirv_tools_version" != "$expected_spirv_tools_version" ]]; then
    echo "wrong spirv-opt version: expected '$expected_spirv_tools_version', got '$spirv_tools_version'" >&2
    exit 1
fi

generate_clockwise_atomic_shader() {
    local source="$1"
    local stage="$2"
    local output="$3"
    local pls_impl="$4"
    shift 4

    local stem="$output_dir/${output%.wgsl}"
    local unoptimized="$stem.unoptimized.spv"
    local optimized="$stem.spv"
    local source_path
    if [[ -f "$root/$source" ]]; then
        source_path="$root/$source"
    else
        source_path="$shader_dir/$source"
    fi
    local stage_define
    if [[ "$stage" == "vert" ]]; then
        stage_define="-DVERTEX"
    else
        stage_define="-DFRAGMENT"
    fi

    local pls_define="-DPLS_IMPL_$pls_impl"

    glslangValidator \
        -S "$stage" \
        "$stage_define" \
        -DTARGET_SPIRV \
        -DTARGET_WGSL \
        -DUSE_WEBGPU_SAMPLERS \
        -DFIXED_FUNCTION_COLOR_OUTPUT \
        "$pls_define" \
        -I"$upstream_out" \
        -V \
        "$@" \
        -o "$unoptimized" \
        "$source_path"
    spirv-opt --preserve-bindings --preserve-interface -O \
        "$unoptimized" -o "$optimized"
    TERM=dumb naga --keep-coordinate-space "$optimized" "$output_dir/$output" \
        2> >(grep -v "Unknown decoration RelaxedPrecision" >&2 || true)
    sed -E 's/[[:space:]]+$//' "$output_dir/$output" > "$output_dir/$output.tmp"
    mv "$output_dir/$output.tmp" "$output_dir/$output"
    rm -f "$unoptimized" "$optimized"
}

if [[ ! -x "$venv/bin/python3" ]]; then
    python3 -m venv "$venv"
    "$venv/bin/pip" install "ply==$expected_ply_version"
fi
ply_version="$("$venv/bin/python3" -c 'import importlib.metadata; print(importlib.metadata.version("ply"))')"
if [[ "$ply_version" != "$expected_ply_version" ]]; then
    echo "wrong ply version: expected $expected_ply_version, got $ply_version" >&2
    exit 1
fi

mkdir -p "$output_dir"
rm -f "$output_dir"/*.wgsl

PATH="$venv/bin:$HOME/.cargo/bin:$PATH" make -C "$shader_dir" OUT="$upstream_out" wgsl
while IFS= read -r header; do
    source="${header%.hpp}.wgsl"
    PATH="$venv/bin:$HOME/.cargo/bin:$PATH" make -C "$shader_dir" OUT="$upstream_out" "$source"
    cp "$source" "$output_dir/$(basename "$source")"
done < <(find "$upstream_out/wgsl" -maxdepth 1 -name '*.hpp' | sort)

# Upstream does not currently emit WebGPU-flavored clockwiseAtomic modules.
# Compile its path/interior and borrowed-coverage sources with the same
# GLSL -> SPIR-V -> naga pipeline used by the regular WebGPU shader set. These
# modules intentionally remain separate from atomic_draw_*: their coverage
# buffer encoding and pass schedule are incompatible.
generate_clockwise_atomic_shader \
    spirv/draw_clockwise_atomic_path.main vert \
    clockwise_atomic_draw_path.webgpu_vert.wgsl STORAGE_BUFFER
generate_clockwise_atomic_shader \
    spirv/draw_clockwise_atomic_path.main frag \
    clockwise_atomic_draw_path.webgpu_fixedcolor_frag.wgsl STORAGE_BUFFER
generate_clockwise_atomic_shader \
    spirv/draw_clockwise_atomic_borrowed_coverage.frag frag \
    clockwise_atomic_draw_path_borrowed.webgpu_frag.wgsl STORAGE_BUFFER
generate_clockwise_atomic_shader \
    spirv/draw_clockwise_atomic_interior_triangles.main vert \
    clockwise_atomic_draw_interior_triangles.webgpu_vert.wgsl STORAGE_BUFFER
generate_clockwise_atomic_shader \
    spirv/draw_clockwise_atomic_interior_triangles.main frag \
    clockwise_atomic_draw_interior_triangles.webgpu_fixedcolor_frag.wgsl STORAGE_BUFFER
generate_clockwise_atomic_shader \
    spirv/draw_clockwise_atomic_borrowed_coverage_interior_triangles.frag frag \
    clockwise_atomic_draw_interior_triangles_borrowed.webgpu_frag.wgsl STORAGE_BUFFER
generate_clockwise_atomic_shader \
    tools/renderer-shaders/clockwise_atomic_path_webgpu.main frag \
    clockwise_atomic_draw_path_sampled_clip.webgpu_fixedcolor_frag.wgsl NONE
generate_clockwise_atomic_shader \
    tools/renderer-shaders/clockwise_atomic_path_webgpu.main frag \
    clockwise_atomic_draw_interior_triangles_sampled_clip.webgpu_fixedcolor_frag.wgsl NONE \
    -DCWA_INTERIOR_TRIANGLES
generate_clockwise_atomic_shader \
    spirv/draw_clockwise_atomic_clip.frag frag \
    clockwise_atomic_draw_clip.webgpu_fixedcolor_frag.wgsl SUBPASS_LOAD
generate_clockwise_atomic_shader \
    spirv/draw_clockwise_atomic_clip_interior_triangles.frag frag \
    clockwise_atomic_draw_clip_interior_triangles.webgpu_fixedcolor_frag.wgsl SUBPASS_LOAD

echo "generated $(find "$output_dir" -name '*.wgsl' | wc -l | tr -d ' ') WGSL modules"
