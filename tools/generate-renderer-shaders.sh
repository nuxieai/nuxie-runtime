#!/bin/bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
shader_dir="$rive_runtime/renderer/src/shaders"
output_dir="$root/crates/nuxie-renderer/src/generated"
venv="$root/target/renderer-shader-venv"
export PATH="$HOME/.cargo/bin:$PATH"

for tool in glslangValidator spirv-opt naga; do
    if ! command -v "$tool" >/dev/null; then
        echo "missing shader tool: $tool" >&2
        exit 1
    fi
done

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
        -I"$shader_dir/out/generated" \
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
