// language: metal4.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct VertexOutput {
    metal::float4 position;
    metal::float2 uv;
    char _pad2[8];
};
struct type_3 {
    metal::float2 inner[3];
};

struct vertex_mainInput {
};
struct vertex_mainOutput {
    metal::float4 position [[position]];
    metal::float2 uv [[user(loc0), center_perspective]];
};
vertex vertex_mainOutput vertex_main(
  uint vertex_index [[vertex_id]]
) {
    type_3 positions = type_3 {{metal::float2(-1.0, -1.0), metal::float2(3.0, -1.0), metal::float2(-1.0, 3.0)}};
    VertexOutput output = {};
    metal::float2 position = positions.inner[metal::min(unsigned(vertex_index), 2u)];
    output.position = metal::float4(position, 0.0, 1.0);
    output.uv = metal::float2((position.x + 1.0) * 0.5, (1.0 - position.y) * 0.5);
    VertexOutput _e31 = output;
    const auto _tmp = _e31;
    return vertex_mainOutput { _tmp.position, _tmp.uv };
}
