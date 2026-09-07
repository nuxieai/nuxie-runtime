// Exact target-2 MSL and target-10 maps extracted from upstream d7fff883
// tests/gm/ore_gm_shaders.rstb.hpp (triangle index 0, binding_witness index 6).
pub const TRIANGLE_MSL: &[u8] = br#"// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct VertexOut {
    metal::float4 pos;
    metal::float4 color;
};

struct vs_mainInput {
    metal::float2 pos [[attribute(0)]];
    metal::float4 color [[attribute(1)]];
};
struct vs_mainOutput {
    metal::float4 pos [[position]];
    metal::float4 color [[user(loc0), center_perspective]];
};
vertex vs_mainOutput vs_main(
  vs_mainInput varyings [[stage_in]]
) {
    const auto pos = varyings.pos;
    const auto color = varyings.color;
    const auto _tmp = VertexOut {metal::float4(pos, 0.0, 1.0), color};
    return vs_mainOutput { _tmp.pos, _tmp.color };
}


struct fs_mainInput {
    metal::float4 color_1 [[user(loc0), center_perspective]];
};
struct fs_mainOutput {
    metal::float4 member_1 [[color(0)]];
};
fragment fs_mainOutput fs_main(
  fs_mainInput varyings_1 [[stage_in]]
, metal::float4 pos_1 [[position]]
) {
    const VertexOut in = { pos_1, varyings_1.color_1 };
    return fs_mainOutput { in.color };
}
"#;
pub const TRIANGLE_MAP: &[u8] = &[3, 2, 14, 0, 0, 0, 0, 0, 9, 0, 0, 0];
pub const WITNESS_MSL: &[u8] = br#"// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct Uniforms {
    metal::float4 color;
};
struct type_3 {
    metal::float2 inner[3];
};

struct vs_mainInput {
};
struct vs_mainOutput {
    metal::float4 member [[position]];
};
vertex vs_mainOutput vs_main(
  uint vid [[vertex_id]]
) {
    type_3 positions = type_3 {{metal::float2(-1.0, -1.0), metal::float2(3.0, -1.0), metal::float2(-1.0, 3.0)}};
    metal::float2 _e13 = positions.inner[vid];
    return vs_mainOutput { metal::float4(_e13, 0.0, 1.0) };
}


struct fs_mainOutput {
    metal::float4 member_1 [[color(0)]];
};
fragment fs_mainOutput fs_main(
  constant Uniforms& u_low [[buffer(0)]]
, constant Uniforms& u_high [[buffer(1)]]
) {
    metal::float4 _e2 = u_low.color;
    metal::float4 _e6 = u_high.color;
    return fs_mainOutput { metal::float4(_e2.xyz + _e6.xyz, 1.0) };
}
"#;
pub const WITNESS_MAP: &[u8] = &[
    3, 2, 14, 0, 2, 0, 0, 0, 9, 0, 1, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 7, 0,
    1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 163, 244, 48, 8, 105, 107, 173, 150,
];
