// language: metal3.2
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct VertexOutput {
    metal::float4 position;
};
struct type_3 {
    metal::float2 inner[3];
};

struct vertex_mainInput {
};
struct vertex_mainOutput {
    metal::float4 position [[position]];
};
vertex vertex_mainOutput vertex_main(
  uint vertex_index [[vertex_id]]
) {
    type_3 positions = type_3 {{metal::float2(-1.0, -1.0), metal::float2(3.0, -1.0), metal::float2(-1.0, 3.0)}};
    VertexOutput output = {};
    metal::float2 _e15 = positions.inner[metal::min(unsigned(vertex_index), 2u)];
    output.position = metal::float4(_e15, 0.0, 1.0);
    VertexOutput _e19 = output;
    const auto _tmp = _e19;
    return vertex_mainOutput { _tmp.position };
}
