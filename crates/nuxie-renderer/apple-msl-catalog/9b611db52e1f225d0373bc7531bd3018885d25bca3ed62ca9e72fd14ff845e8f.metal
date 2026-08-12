// language: metal3.1
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size1;
    uint size6;
    uint size9;
    uint buffer_size30;
};

struct type_2 {
    float inner[4];
};
struct type_3 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_2 gl_ClipDistance;
    type_3 gl_CullDistance;
    char _pad4[8];
};
typedef metal::uint4 type_6[1];
struct bg {
    type_6 c2_;
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
typedef metal::uint2 type_11[1];
struct Je {
    type_11 c2_;
};
typedef metal::float4 type_12[1];
struct Ke {
    type_12 c2_;
};
struct VertexOutput {
    metal::float4 gl_Position;
    type_2 gl_ClipDistance;
    metal::float2 member;
    float member_1_;
    float member_2_;
    metal::float4 member_3_;
};
constant bool Yg = false;
constant bool ah = false;
constant bool Zg = true;
metal::float3 unpackFloat32x3_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11) {
    return metal::float3(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8));
}

void main_1_(
    thread gl_PerVertex& unnamed,
    device bg const& PB,
    constant CC& n,
    thread metal::float3& KB_1_,
    thread metal::float2& C2_,
    device Je const& AD,
    thread float& I3_,
    thread float& e2_,
    device Ke const& RB,
    thread metal::float4& f1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    uint phi_679_ = {};
    float phi_680_ = {};
    float phi_681_ = {};
    metal::float4 phi_682_ = {};
    bool local = {};
    metal::float3 _e42_ = KB_1_;
    uint _e45_ = as_type<uint>(_e42_.z) & 65535u;
    uint _e46_ = _e45_ * 4u;
    uint _e47_ = _e46_ + 2u;
    metal::uint4 _e50_ = PB.c2_[metal::min(unsigned(_e47_), (_buffer_sizes.size1 - 0 - 16) / 16)];
    metal::float2 _e52_ = _e42_.xy;
    metal::float3 _e54_ = as_type<metal::float3>(_e50_.yzw);
    metal::float2 _e60_ = n.Bg;
    C2_ = ((_e52_ * _e54_.x) + _e54_.yz) * _e60_;
    metal::uint2 _e64_ = AD.c2_[metal::min(unsigned(_e45_), (_buffer_sizes.size6 - 0 - 8) / 8)];
    uint _e66_ = _e64_.x & 15u;
    if (Yg) {
        bool _e67_ = _e66_ == 0u;
        if (_e67_) {
            phi_679_ = _e64_.y;
        } else {
            phi_679_ = _e64_.x;
        }
        uint _e70_ = phi_679_;
        uint _e72_ = _e70_ >> as_type<uint>(16);
        uint _e74_ = n.Z5_;
        if (_e72_ == 0u) {
            phi_680_ = 0.0;
        } else {
            phi_680_ = float2(as_type<half2>(((_e72_ + 1023u) * _e74_))).x;
        }
        float _e81_ = phi_680_;
        phi_681_ = _e81_;
        if (_e67_) {
            phi_681_ = -(_e81_);
        }
        float _e84_ = phi_681_;
        I3_ = _e84_;
    }
    if (ah) {
        e2_ = static_cast<float>((_e64_.x >> as_type<uint>(4)) & 15u);
    }
    if (Zg) {
        metal::float4 _e91_ = RB.c2_[metal::min(unsigned(_e47_), (_buffer_sizes.size9 - 0 - 16) / 16)];
        metal::float4 _e102_ = RB.c2_[metal::min(unsigned(_e46_ + 3u), (_buffer_sizes.size9 - 0 - 16) / 16)];
        if (metal::any(_e91_ != metal::float4(0.0, 0.0, 0.0, 0.0))) {
            metal::float2 _e117_ = (metal::float2x2(metal::float2(_e91_.x, _e91_.y), metal::float2(_e91_.z, _e91_.w)) * _e52_) + _e102_.xy;
            unnamed.gl_ClipDistance.inner[0] = _e117_.x + 1.0;
            unnamed.gl_ClipDistance.inner[1] = _e117_.y + 1.0;
            unnamed.gl_ClipDistance.inner[2] = 1.0 - _e117_.x;
            unnamed.gl_ClipDistance.inner[3] = 1.0 - _e117_.y;
        } else {
            float _e107_ = _e102_.x - 0.5;
            unnamed.gl_ClipDistance.inner[3] = _e107_;
            unnamed.gl_ClipDistance.inner[2] = _e107_;
            unnamed.gl_ClipDistance.inner[1] = _e107_;
            unnamed.gl_ClipDistance.inner[0] = _e107_;
        }
    }
    if (_e66_ == 1u) {
        metal::float4 _e180_ = metal::unpack_unorm4x8_to_float(_e64_.y);
        if (ah) {
            phi_682_ = _e180_;
        } else {
            metal::float3 _e183_ = _e180_.xyz * _e180_.w;
            metal::float4 _e189_ = metal::float4(_e183_.x, _e180_.y, _e180_.z, _e180_.w);
            metal::float4 _e195_ = metal::float4(_e189_.x, _e183_.y, _e189_.z, _e189_.w);
            phi_682_ = metal::float4(_e195_.x, _e195_.y, _e183_.z, _e195_.w);
        }
        metal::float4 _e203_ = phi_682_;
        f1_ = _e203_;
    } else {
        metal::float4 _e135_ = RB.c2_[metal::min(unsigned(_e46_), (_buffer_sizes.size9 - 0 - 16) / 16)];
        metal::float4 _e146_ = RB.c2_[metal::min(unsigned(_e46_ + 1u), (_buffer_sizes.size9 - 0 - 16) / 16)];
        metal::float2 _e149_ = (metal::float2x2(metal::float2(_e135_.x, _e135_.y), metal::float2(_e135_.z, _e135_.w)) * _e52_) + _e146_.xy;
        bool _e150_ = _e66_ == 2u;
        if (!(_e150_)) {
            local = _e66_ == 3u;
        } else {
            local = true;
        }
        bool _e190 = local;
        if (_e190) {
            f1_.w = -(as_type<float>(_e64_.y));
            if (_e146_.z > 0.9) {
                f1_.z = 2.0;
            } else {
                f1_.z = _e146_.w;
            }
            if (_e150_) {
                f1_.y = 0.0;
                f1_.x = _e149_.x;
            } else {
                float _e170_ = f1_.z;
                f1_.z = -(_e170_);
                f1_.x = _e149_.x;
                f1_.y = _e149_.y;
            }
        } else {
            f1_ = metal::float4(_e149_.x, _e149_.y, as_type<float>(_e64_.y), -2.0 - _e146_.z);
        }
    }
    float _e205_ = n.ff;
    float _e207_ = n.gf;
    metal::float4 _e215_ = metal::float4((_e42_.x * _e205_) - 1.0, (_e42_.y * _e207_) - metal::sign(_e207_), 0.0, 1.0);
    unnamed.gl_Position = metal::float4(_e215_.x, _e215_.y, 1.0 - (static_cast<float>(_e50_.x) * 0.000061035156), _e215_.w);
    return;
}

struct main_Output {
    metal::float4 gl_Position [[position]];
    float gl_ClipDistance [[clip_distance]] [4];
    metal::float2 member [[user(loc1), center_perspective]];
    float member_1_ [[user(loc4), flat]];
    float member_2_ [[user(loc6), flat]];
    metal::float4 member_3_ [[user(loc0), center_perspective]];
};
struct vb_30_type { metal::uchar data[12]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, device bg const& PB [[buffer(1)]]
, constant CC& n [[buffer(0)]]
, device Je const& AD [[buffer(2)]]
, device Ke const& RB [[buffer(3)]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(5)]]
) {
    metal::float3 KB = {};
    if (gl_VertexIndex < (_buffer_sizes.buffer_size30 / 12)) {
        const vb_30_type vb_30_elem = vb_30_in[gl_VertexIndex];
        KB = unpackFloat32x3_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7], vb_30_elem.data[8], vb_30_elem.data[9], vb_30_elem.data[10], vb_30_elem.data[11]);
    }
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_2 {}, type_3 {}};
    int gl_VertexIndex_1_ = {};
    metal::float3 KB_1_ = {};
    metal::float2 C2_ = {};
    float I3_ = {};
    float e2_ = {};
    metal::float4 f1_ = {};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    KB_1_ = KB;
    main_1_(unnamed, PB, n, KB_1_, C2_, AD, I3_, e2_, RB, f1_, _buffer_sizes);
    metal::float4 _e12_ = unnamed.gl_Position;
    type_2 _e13_ = unnamed.gl_ClipDistance;
    metal::float2 _e14_ = C2_;
    float _e15_ = I3_;
    float _e16_ = e2_;
    metal::float4 _e17_ = f1_;
    const auto _tmp = VertexOutput {_e12_, _e13_, _e14_, _e15_, _e16_, _e17_};
    return main_Output { _tmp.gl_Position, {_tmp.gl_ClipDistance.inner[0],_tmp.gl_ClipDistance.inner[1],_tmp.gl_ClipDistance.inner[2],_tmp.gl_ClipDistance.inner[3]}, _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.member_3_ };
}
