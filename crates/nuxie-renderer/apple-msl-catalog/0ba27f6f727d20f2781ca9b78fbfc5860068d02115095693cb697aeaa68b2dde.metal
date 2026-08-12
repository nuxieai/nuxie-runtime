// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint buffer_size30;
};

struct CC {
    float ec;
    float od;
    float ff;
    float gf;
    uint m6_;
    uint Fg;
    uint Re;
    uint Se;
    metal::int4 R7_;
    metal::float2 Bg;
    metal::float2 pd;
    uint a2_;
    float Gg;
    uint Z5_;
    float P2_;
    float qd;
    uint Me;
    float z3_;
    float A3_;
    float rd;
    uint yg;
    char _pad21[8];
};
struct type_6 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_6 gl_ClipDistance;
    type_6 gl_CullDistance;
    char _pad4[4];
};
metal::float3 unpackFloat32x3_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11) {
    return metal::float3(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8));
}

void main_1_(
    thread metal::float3& KB_1_,
    constant CC& n,
    thread gl_PerVertex& unnamed
) {
    metal::float3 _e13_ = KB_1_;
    float _e15_ = n.ff;
    float _e17_ = n.gf;
    metal::float4 _e25_ = metal::float4((_e13_.x * _e15_) - 1.0, (_e13_.y * _e17_) - metal::sign(_e17_), 0.0, 1.0);
    float _e27_ = KB_1_.z;
    unnamed.gl_Position = metal::float4(_e25_.x, _e25_.y, 1.0 - (static_cast<float>(as_type<uint>(_e27_) & 65535u) * 0.000061035156), _e25_.w);
    return;
}

struct main_Output {
    metal::float4 member [[position]];
};
struct vb_30_type { metal::uchar data[12]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, constant CC& n [[buffer(0)]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(1)]]
) {
    metal::float3 KB = {};
    if (gl_VertexIndex < (_buffer_sizes.buffer_size30 / 12)) {
        const vb_30_type vb_30_elem = vb_30_in[gl_VertexIndex];
        KB = unpackFloat32x3_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7], vb_30_elem.data[8], vb_30_elem.data[9], vb_30_elem.data[10], vb_30_elem.data[11]);
    }
    int gl_VertexIndex_1_ = {};
    metal::float3 KB_1_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_6 {}, type_6 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    KB_1_ = KB;
    main_1_(KB_1_, n, unnamed);
    metal::float4 _e7_ = unnamed.gl_Position;
    return main_Output { _e7_ };
}
