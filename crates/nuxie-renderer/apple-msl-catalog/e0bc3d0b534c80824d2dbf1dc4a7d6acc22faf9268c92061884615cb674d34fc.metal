// language: metal3.1
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct VertexOutput {
    metal::float4 position;
    metal::float4 color;
};

struct fragment_mainInput {
    metal::float4 color [[user(loc0), center_perspective]];
};
struct fragment_mainOutput {
    metal::float4 member [[color(0)]];
};
fragment fragment_mainOutput fragment_main(
  fragment_mainInput varyings [[stage_in]]
, metal::float4 position [[position]]
) {
    const VertexOutput input = { position, varyings.color };
    return fragment_mainOutput { input.color };
}
