// Exact target-2 MSL, target-10 binding map, and pair section extracted from
// upstream 50ba2f5a581089e93592393d00b61da4ba46473e tests/gm/ore_gm_shaders.rstb.hpp,
// index 13 (depth_sample_witness). Header SHA256:
// 9e575fc2d477f07195f6016446a13f2d830aed0ff12c4f4a9d3a8f2eefef45cf.
// No compilation or rebake: RSTB v4 directory and whole-module entry table decoded.
pub const MSL: &[u8] = br#"// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct Uniforms {
    metal::float4 tint;
};
struct type_5 {
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
    type_5 positions = type_5 {{metal::float2(-1.0, -1.0), metal::float2(3.0, -1.0), metal::float2(-1.0, 3.0)}};
    metal::float2 _e13 = positions.inner[vid];
    return vs_mainOutput { metal::float4(_e13, 0.0, 1.0) };
}


struct fs_mainOutput {
    metal::float4 member_1 [[color(0)]];
};
fragment fs_mainOutput fs_main(
  constant Uniforms& u [[buffer(0)]]
, metal::sampler pointSampler [[sampler(0)]]
, metal::depth2d<float, metal::access::sample> depthTex [[texture(0)]]
) {
    float d0_ = depthTex.sample(pointSampler, metal::float2(0.5), metal::level(0));
    float d1_ = depthTex.sample(pointSampler, metal::float2(0.5));
    metal::float4 _e13 = u.tint;
    return fs_mainOutput { metal::float4((_e13.xyz * d0_) * d1_, 1.0) };
}
"#;
pub const MAP: &[u8] = &[
    3, 2, 18, 0, 3, 0, 0, 0, 9, 0, 2, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 1,
    0, 5, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 3, 7, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 0,
    0, 0, 0, 0, 32, 155, 217, 129, 203, 6, 53, 68, 1, 250, 208, 67, 65, 253, 59, 98, 233,
];
pub const PAIRS: &[u8] = &[1, 1, 1, 0];
pub const MSL_SHA256: &str = "3e7d036f706a29b1cb71b2a4dbb6424bb7d42ce3f6ae5e4c664bdc921360126b";
pub const MAP_SHA256: &str = "519a54a4170d5e6f48bd0c591db432008931737a273b56df294c5aa898484998";
