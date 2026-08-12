// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size0;
    uint size5;
    uint size8;
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
typedef metal::uint2 type_8[1];
struct Je {
    type_8 c2_;
};
typedef metal::float4 type_10[1];
struct Ke {
    type_10 c2_;
};
struct type_11 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_11 gl_ClipDistance;
    type_11 gl_CullDistance;
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
constant bool ah = true;
metal::float3 unpackFloat32x3_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11) {
    return metal::float3(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8));
}

void main_1_(
    device bg const& PB,
    constant CC& n,
    thread metal::float3& KB_1_,
    thread metal::float2& C2_,
    device Je const& AD,
    thread float& I3_,
    thread float& e2_,
    device Ke const& RB,
    thread metal::float4& f1_,
    thread gl_PerVertex& unnamed,
    constant _mslBufferSizes& _buffer_sizes
) {
    uint phi_589_ = {};
    float phi_590_ = {};
    float phi_591_ = {};
    metal::float4 phi_592_ = {};
    bool local = {};
    metal::float3 _e38_ = KB_1_;
    uint _e41_ = as_type<uint>(_e38_.z) & 65535u;
    uint _e42_ = _e41_ * 4u;
    metal::uint4 _e46_ = PB.c2_[metal::min(unsigned(_e42_ + 2u), (_buffer_sizes.size0 - 0 - 16) / 16)];
    metal::float2 _e48_ = _e38_.xy;
    metal::float3 _e50_ = as_type<metal::float3>(_e46_.yzw);
    metal::float2 _e56_ = n.Bg;
    C2_ = ((_e48_ * _e50_.x) + _e50_.yz) * _e56_;
    metal::uint2 _e60_ = AD.c2_[metal::min(unsigned(_e41_), (_buffer_sizes.size5 - 0 - 8) / 8)];
    uint _e62_ = _e60_.x & 15u;
    if (Yg) {
        bool _e63_ = _e62_ == 0u;
        if (_e63_) {
            phi_589_ = _e60_.y;
        } else {
            phi_589_ = _e60_.x;
        }
        uint _e66_ = phi_589_;
        uint _e68_ = _e66_ >> as_type<uint>(16);
        uint _e70_ = n.Z5_;
        if (_e68_ == 0u) {
            phi_590_ = 0.0;
        } else {
            phi_590_ = float2(as_type<half2>(((_e68_ + 1023u) * _e70_))).x;
        }
        float _e77_ = phi_590_;
        phi_591_ = _e77_;
        if (_e63_) {
            phi_591_ = -(_e77_);
        }
        float _e80_ = phi_591_;
        I3_ = _e80_;
    }
    if (ah) {
        e2_ = static_cast<float>((_e60_.x >> as_type<uint>(4)) & 15u);
    }
    if (_e62_ == 1u) {
        metal::float4 _e133_ = metal::unpack_unorm4x8_to_float(_e60_.y);
        if (ah) {
            phi_592_ = _e133_;
        } else {
            metal::float3 _e136_ = _e133_.xyz * _e133_.w;
            metal::float4 _e142_ = metal::float4(_e136_.x, _e133_.y, _e133_.z, _e133_.w);
            metal::float4 _e148_ = metal::float4(_e142_.x, _e136_.y, _e142_.z, _e142_.w);
            phi_592_ = metal::float4(_e148_.x, _e148_.y, _e136_.z, _e148_.w);
        }
        metal::float4 _e156_ = phi_592_;
        f1_ = _e156_;
    } else {
        metal::float4 _e88_ = RB.c2_[metal::min(unsigned(_e42_), (_buffer_sizes.size8 - 0 - 16) / 16)];
        metal::float4 _e99_ = RB.c2_[metal::min(unsigned(_e42_ + 1u), (_buffer_sizes.size8 - 0 - 16) / 16)];
        metal::float2 _e102_ = (metal::float2x2(metal::float2(_e88_.x, _e88_.y), metal::float2(_e88_.z, _e88_.w)) * _e48_) + _e99_.xy;
        bool _e103_ = _e62_ == 2u;
        if (!(_e103_)) {
            local = _e62_ == 3u;
        } else {
            local = true;
        }
        bool _e123 = local;
        if (_e123) {
            f1_.w = -(as_type<float>(_e60_.y));
            if (_e99_.z > 0.9) {
                f1_.z = 2.0;
            } else {
                f1_.z = _e99_.w;
            }
            if (_e103_) {
                f1_.y = 0.0;
                f1_.x = _e102_.x;
            } else {
                float _e123_ = f1_.z;
                f1_.z = -(_e123_);
                f1_.x = _e102_.x;
                f1_.y = _e102_.y;
            }
        } else {
            f1_ = metal::float4(_e102_.x, _e102_.y, as_type<float>(_e60_.y), -2.0 - _e99_.z);
        }
    }
    float _e158_ = n.ff;
    float _e160_ = n.gf;
    metal::float4 _e168_ = metal::float4((_e38_.x * _e158_) - 1.0, (_e38_.y * _e160_) - metal::sign(_e160_), 0.0, 1.0);
    unnamed.gl_Position = metal::float4(_e168_.x, _e168_.y, 1.0 - (static_cast<float>(_e46_.x) * 0.000061035156), _e168_.w);
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
    int gl_VertexIndex_1_ = {};
    metal::float3 KB_1_ = {};
    metal::float2 C2_ = {};
    float I3_ = {};
    float e2_ = {};
    metal::float4 f1_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_11 {}, type_11 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    KB_1_ = KB;
    main_1_(PB, n, KB_1_, C2_, AD, I3_, e2_, RB, f1_, unnamed, _buffer_sizes);
    metal::float2 _e11_ = C2_;
    float _e12_ = I3_;
    float _e13_ = e2_;
    metal::float4 _e14_ = f1_;
    metal::float4 _e15_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e11_, _e12_, _e13_, _e14_, _e15_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.member_3_, _tmp.gl_Position };
}
