// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size0;
    uint size10;
    uint size11;
    uint size12;
    uint buffer_size30;
};

typedef metal::uint4 type_2[1];
struct bg {
    type_2 c2_;
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
struct type_8 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_8 gl_ClipDistance;
    type_8 gl_CullDistance;
    char _pad4[4];
};
typedef metal::uint2 type_10[1];
struct Je {
    type_10 c2_;
};
typedef metal::float4 type_11[1];
struct Ke {
    type_11 c2_;
};
struct cg {
    type_2 c2_;
};
struct VertexOutput {
    metal::float2 member;
    uint member_1_;
    char _pad2[4];
    metal::float4 gl_Position;
};
metal::float3 unpackFloat32x3_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11) {
    return metal::float3(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8));
}

void main_1_(
    device bg const& PB,
    constant CC& n,
    thread metal::float3& KB_1_,
    thread metal::float2& C2_,
    thread uint& B0_,
    thread gl_PerVertex& unnamed,
    constant _mslBufferSizes& _buffer_sizes
) {
    metal::float3 _e24_ = KB_1_;
    uint _e27_ = as_type<uint>(_e24_.z) & 65535u;
    metal::uint4 _e32_ = PB.c2_[metal::min(unsigned((_e27_ * 4u) + 2u), (_buffer_sizes.size0 - 0 - 16) / 16)];
    metal::float3 _e35_ = as_type<metal::float3>(_e32_.yzw);
    metal::float2 _e41_ = n.Bg;
    C2_ = ((_e24_.xy * _e35_.x) + _e35_.yz) * _e41_;
    B0_ = _e27_;
    float _e44_ = n.ff;
    float _e46_ = n.gf;
    unnamed.gl_Position = metal::float4((_e24_.x * _e44_) - 1.0, (_e24_.y * _e46_) - metal::sign(_e46_), 0.0, 1.0);
    return;
}

struct main_Output {
    metal::float2 member [[user(loc0), center_perspective]];
    uint member_1_ [[user(loc1), flat]];
    metal::float4 gl_Position [[position]];
};
struct vb_30_type { metal::uchar data[12]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, uint gl_InstanceIndex [[instance_id]]
, device bg const& PB [[buffer(1)]]
, constant CC& n [[buffer(0)]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(3)]]
) {
    metal::float3 KB = {};
    if (gl_VertexIndex < (_buffer_sizes.buffer_size30 / 12)) {
        const vb_30_type vb_30_elem = vb_30_in[gl_VertexIndex];
        KB = unpackFloat32x3_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7], vb_30_elem.data[8], vb_30_elem.data[9], vb_30_elem.data[10], vb_30_elem.data[11]);
    }
    int gl_VertexIndex_1_ = {};
    int gl_InstanceIndex_1_ = {};
    metal::float3 KB_1_ = {};
    metal::float2 C2_ = {};
    uint B0_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_8 {}, type_8 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    KB_1_ = KB;
    main_1_(PB, n, KB_1_, C2_, B0_, unnamed, _buffer_sizes);
    metal::float2 _e12_ = C2_;
    uint _e13_ = B0_;
    metal::float4 _e14_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e12_, _e13_, {}, _e14_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.gl_Position };
}
