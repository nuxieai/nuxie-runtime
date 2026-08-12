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
struct VertexOutput {
    metal::float2 member;
    float member_1_;
    float member_2_;
    metal::float4 member_3_;
    metal::float4 gl_Position;
};
constant bool Yg = false;
constant bool ah = false;
metal::float3 unpackFloat32x3_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11) {
    return metal::float3(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8));
}

void main_1_(
    metal::texture2d<uint, metal::access::sample> PB,
    constant CC& n,
    thread metal::float3& KB_1_,
    thread metal::float2& C2_,
    metal::texture2d<uint, metal::access::sample> AD,
    thread float& I3_,
    thread float& e2_,
    metal::texture2d<float, metal::access::sample> RB,
    thread metal::float4& f1_,
    thread gl_PerVertex& unnamed
) {
    uint phi_631_ = {};
    float phi_632_ = {};
    float phi_633_ = {};
    metal::float4 phi_634_ = {};
    bool local = {};
    metal::float3 _e40_ = KB_1_;
    uint _e42_ = as_type<uint>(_e40_.z);
    uint _e43_ = _e42_ & 65535u;
    uint _e44_ = _e43_ * 4u;
    uint _e45_ = _e44_ + 2u;
    uint clamped_lod_e24 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
    metal::uint4 _e52_ = PB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e45_ & 127u), as_type<int>(_e45_ >> as_type<uint>(7)))), metal::uint2(PB.get_width(clamped_lod_e24), PB.get_height(clamped_lod_e24)) - 1), clamped_lod_e24);
    metal::float2 _e54_ = _e40_.xy;
    metal::float3 _e56_ = as_type<metal::float3>(_e52_.yzw);
    metal::float2 _e62_ = n.Bg;
    C2_ = ((_e54_ * _e56_.x) + _e56_.yz) * _e62_;
    uint clamped_lod_e47 = metal::min(uint(0), AD.get_num_mip_levels() - 1);
    metal::uint4 _e70_ = AD.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e42_ & 127u), as_type<int>(_e43_ >> as_type<uint>(7)))), metal::uint2(AD.get_width(clamped_lod_e47), AD.get_height(clamped_lod_e47)) - 1), clamped_lod_e47);
    uint _e72_ = _e70_.x & 15u;
    if (Yg) {
        bool _e73_ = _e72_ == 0u;
        if (_e73_) {
            phi_631_ = _e70_.y;
        } else {
            phi_631_ = _e70_.x;
        }
        uint _e76_ = phi_631_;
        uint _e78_ = _e76_ >> as_type<uint>(16);
        uint _e80_ = n.Z5_;
        if (_e78_ == 0u) {
            phi_632_ = 0.0;
        } else {
            phi_632_ = float2(as_type<half2>(((_e78_ + 1023u) * _e80_))).x;
        }
        float _e87_ = phi_632_;
        phi_633_ = _e87_;
        if (_e73_) {
            phi_633_ = -(_e87_);
        }
        float _e90_ = phi_633_;
        I3_ = _e90_;
    }
    if (ah) {
        e2_ = static_cast<float>((_e70_.x >> as_type<uint>(4)) & 15u);
    }
    if (_e72_ == 1u) {
        metal::float4 _e151_ = metal::unpack_unorm4x8_to_float(_e70_.y);
        if (ah) {
            phi_634_ = _e151_;
        } else {
            metal::float3 _e154_ = _e151_.xyz * _e151_.w;
            metal::float4 _e160_ = metal::float4(_e154_.x, _e151_.y, _e151_.z, _e151_.w);
            metal::float4 _e166_ = metal::float4(_e160_.x, _e154_.y, _e160_.z, _e160_.w);
            phi_634_ = metal::float4(_e166_.x, _e166_.y, _e154_.z, _e166_.w);
        }
        metal::float4 _e174_ = phi_634_;
        f1_ = _e174_;
    } else {
        uint clamped_lod_e119 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
        metal::float4 _e102_ = RB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e44_ & 127u), as_type<int>(_e44_ >> as_type<uint>(7)))), metal::uint2(RB.get_width(clamped_lod_e119), RB.get_height(clamped_lod_e119)) - 1), clamped_lod_e119);
        uint _e110_ = _e44_ + 1u;
        uint clamped_lod_e132 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
        metal::float4 _e117_ = RB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e110_ & 127u), as_type<int>(_e110_ >> as_type<uint>(7)))), metal::uint2(RB.get_width(clamped_lod_e132), RB.get_height(clamped_lod_e132)) - 1), clamped_lod_e132);
        metal::float2 _e120_ = (metal::float2x2(metal::float2(_e102_.x, _e102_.y), metal::float2(_e102_.z, _e102_.w)) * _e54_) + _e117_.xy;
        bool _e121_ = _e72_ == 2u;
        if (!(_e121_)) {
            local = _e72_ == 3u;
        } else {
            local = true;
        }
        bool _e151 = local;
        if (_e151) {
            f1_.w = -(as_type<float>(_e70_.y));
            if (_e117_.z > 0.9) {
                f1_.z = 2.0;
            } else {
                f1_.z = _e117_.w;
            }
            if (_e121_) {
                f1_.y = 0.0;
                f1_.x = _e120_.x;
            } else {
                float _e141_ = f1_.z;
                f1_.z = -(_e141_);
                f1_.x = _e120_.x;
                f1_.y = _e120_.y;
            }
        } else {
            f1_ = metal::float4(_e120_.x, _e120_.y, as_type<float>(_e70_.y), -2.0 - _e117_.z);
        }
    }
    float _e176_ = n.ff;
    float _e178_ = n.gf;
    metal::float4 _e186_ = metal::float4((_e40_.x * _e176_) - 1.0, (_e40_.y * _e178_) - metal::sign(_e178_), 0.0, 1.0);
    unnamed.gl_Position = metal::float4(_e186_.x, _e186_.y, 1.0 - (static_cast<float>(_e52_.x) * 0.000061035156), _e186_.w);
    return;
}

struct main_Output {
    metal::float2 member [[user(loc1), center_perspective]];
    float member_1_ [[user(loc4), flat]];
    float member_2_ [[user(loc6), flat]];
    metal::float4 member_3_ [[user(loc0), center_perspective]];
    metal::float4 gl_Position [[position]];
};
struct vb_30_type { metal::uchar data[12]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, metal::texture2d<uint, metal::access::sample> PB [[texture(0)]]
, constant CC& n [[buffer(0)]]
, metal::texture2d<uint, metal::access::sample> AD [[texture(1)]]
, metal::texture2d<float, metal::access::sample> RB [[texture(2)]]
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
    metal::float2 C2_ = {};
    float I3_ = {};
    float e2_ = {};
    metal::float4 f1_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_6 {}, type_6 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    KB_1_ = KB;
    main_1_(PB, n, KB_1_, C2_, AD, I3_, e2_, RB, f1_, unnamed);
    metal::float2 _e11_ = C2_;
    float _e12_ = I3_;
    float _e13_ = e2_;
    metal::float4 _e14_ = f1_;
    metal::float4 _e15_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e11_, _e12_, _e13_, _e14_, _e15_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.member_3_, _tmp.gl_Position };
}
