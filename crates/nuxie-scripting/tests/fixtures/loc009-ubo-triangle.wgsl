// Resourced GPU-canvas shader: full-screen triangle whose color comes from a
// uniform buffer (UBO). Exercises the assembler's binding-map + GL-fixup path
// (UBO at @group(0) @binding(0)) — the part untested by the resourceless shader.
// Entry points must be vs_main / fs_main.
struct Uniforms {
    color: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(idx) - 1);
    let y = f32(i32(idx & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return u.color;
}
