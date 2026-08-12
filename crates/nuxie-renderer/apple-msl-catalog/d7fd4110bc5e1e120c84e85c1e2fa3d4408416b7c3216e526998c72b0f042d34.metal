// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint buffer_size30;
};

struct VertexInput {
    metal::float2 position;
    char _pad1[8];
    metal::float4 color;
};
struct VertexOutput {
    metal::float4 position;
    metal::float4 color;
};
metal::float2 unpackFloat32x2_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7) {
    return metal::float2(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4));
}
metal::float4 unpackFloat32x4_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11, uint b12, uint b13, uint b14, uint b15) {
    return metal::float4(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8), as_type<float>(b15 << 24 | b14 << 16 | b13 << 8 | b12));
}

struct vertex_mainOutput {
    metal::float4 position [[position]];
    metal::float4 color [[user(loc0), center_perspective]];
};
struct vb_30_type { metal::uchar data[24]; };
vertex vertex_mainOutput vertex_main(
  uint v_id [[vertex_id]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(0)]]
) {
    metal::float2 position = {};
    metal::float4 color = {};
    if (v_id < (_buffer_sizes.buffer_size30 / 24)) {
        const vb_30_type vb_30_elem = vb_30_in[v_id];
        position = unpackFloat32x2_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7]);
        color = unpackFloat32x4_(vb_30_elem.data[8], vb_30_elem.data[9], vb_30_elem.data[10], vb_30_elem.data[11], vb_30_elem.data[12], vb_30_elem.data[13], vb_30_elem.data[14], vb_30_elem.data[15], vb_30_elem.data[16], vb_30_elem.data[17], vb_30_elem.data[18], vb_30_elem.data[19], vb_30_elem.data[20], vb_30_elem.data[21], vb_30_elem.data[22], vb_30_elem.data[23]);
    }
    const VertexInput input = { position, {}, color };
    VertexOutput output = {};
    output.position = metal::float4(input.position, 0.0, 1.0);
    output.color = input.color;
    VertexOutput _e9 = output;
    const auto _tmp = _e9;
    return vertex_mainOutput { _tmp.position, _tmp.color };
}
