// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct VertexOutput {
    metal::float4 position;
    metal::float2 uv;
    char _pad2[8];
};

struct fragment_straight_alphaInput {
    metal::float2 uv [[user(loc0), center_perspective]];
};
struct fragment_straight_alphaOutput {
    metal::float4 member [[color(0)]];
};
fragment fragment_straight_alphaOutput fragment_straight_alpha(
  fragment_straight_alphaInput varyings [[stage_in]]
, metal::float4 position [[position]]
, metal::texture2d<float, metal::access::sample> source_texture [[texture(0)]]
, metal::sampler source_sampler [[sampler(0)]]
) {
    const VertexOutput input = { position, varyings.uv };
    metal::float4 premultiplied = source_texture.sample(source_sampler, input.uv);
    if (premultiplied.w <= 0.0) {
        return fragment_straight_alphaOutput { metal::float4(0.0) };
    }
    return fragment_straight_alphaOutput { metal::float4(premultiplied.xyz / metal::float3(premultiplied.w), premultiplied.w) };
}
