// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct VertexOutput {
    metal::float4 position;
    metal::float2 uv;
    char _pad2[8];
};

struct fragment_mainInput {
    metal::float2 uv [[user(loc0), center_perspective]];
};
struct fragment_mainOutput {
    metal::float4 member [[color(0)]];
};
fragment fragment_mainOutput fragment_main(
  fragment_mainInput varyings [[stage_in]]
, metal::float4 position [[position]]
, metal::texture2d<float, metal::access::sample> source_texture [[texture(0)]]
, metal::sampler source_sampler [[sampler(0)]]
) {
    const VertexOutput input = { position, varyings.uv };
    metal::float4 _e4 = source_texture.sample(source_sampler, input.uv);
    return fragment_mainOutput { _e4 };
}
