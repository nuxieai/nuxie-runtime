// language: metal3.1
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
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
struct VertexOutput {
    metal::float4 gl_Position;
    type_2 gl_ClipDistance;
    metal::float2 member;
    float member_1_;
    float member_2_;
    metal::float4 member_3_;
};
constant bool Yg = false;
constant bool ah = true;
constant bool Zg = true;
metal::float3 unpackFloat32x3_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11) {
    return metal::float3(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8));
}

void main_1_(
    thread gl_PerVertex& unnamed,
    metal::texture2d<uint, metal::access::sample> PB,
    constant CC& n,
    thread metal::float3& KB_1_,
    thread metal::float2& C2_,
    metal::texture2d<uint, metal::access::sample> AD,
    thread float& I3_,
    thread float& e2_,
    metal::texture2d<float, metal::access::sample> RB,
    thread metal::float4& f1_
) {
    uint phi_721_ = {};
    float phi_722_ = {};
    float phi_723_ = {};
    metal::float4 phi_724_ = {};
    bool local = {};
    metal::float3 _e44_ = KB_1_;
    uint _e46_ = as_type<uint>(_e44_.z);
    uint _e47_ = _e46_ & 65535u;
    uint _e48_ = _e47_ * 4u;
    uint _e49_ = _e48_ + 2u;
    metal::int2 _e55_ = metal::int2(as_type<int>(_e49_ & 127u), as_type<int>(_e49_ >> as_type<uint>(7)));
    uint clamped_lod_e24 = metal::min(uint(0), PB.get_num_mip_levels() - 1);
    metal::uint4 _e56_ = PB.read(metal::min(metal::uint2(_e55_), metal::uint2(PB.get_width(clamped_lod_e24), PB.get_height(clamped_lod_e24)) - 1), clamped_lod_e24);
    metal::float2 _e58_ = _e44_.xy;
    metal::float3 _e60_ = as_type<metal::float3>(_e56_.yzw);
    metal::float2 _e66_ = n.Bg;
    C2_ = ((_e58_ * _e60_.x) + _e60_.yz) * _e66_;
    uint clamped_lod_e47 = metal::min(uint(0), AD.get_num_mip_levels() - 1);
    metal::uint4 _e74_ = AD.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e46_ & 127u), as_type<int>(_e47_ >> as_type<uint>(7)))), metal::uint2(AD.get_width(clamped_lod_e47), AD.get_height(clamped_lod_e47)) - 1), clamped_lod_e47);
    uint _e76_ = _e74_.x & 15u;
    if (Yg) {
        bool _e77_ = _e76_ == 0u;
        if (_e77_) {
            phi_721_ = _e74_.y;
        } else {
            phi_721_ = _e74_.x;
        }
        uint _e80_ = phi_721_;
        uint _e82_ = _e80_ >> as_type<uint>(16);
        uint _e84_ = n.Z5_;
        if (_e82_ == 0u) {
            phi_722_ = 0.0;
        } else {
            phi_722_ = float2(as_type<half2>(((_e82_ + 1023u) * _e84_))).x;
        }
        float _e91_ = phi_722_;
        phi_723_ = _e91_;
        if (_e77_) {
            phi_723_ = -(_e91_);
        }
        float _e94_ = phi_723_;
        I3_ = _e94_;
    }
    if (ah) {
        e2_ = static_cast<float>((_e74_.x >> as_type<uint>(4)) & 15u);
    }
    if (Zg) {
        uint clamped_lod_e87 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
        metal::float4 _e99_ = RB.read(metal::min(metal::uint2(_e55_), metal::uint2(RB.get_width(clamped_lod_e87), RB.get_height(clamped_lod_e87)) - 1), clamped_lod_e87);
        uint _e107_ = _e48_ + 3u;
        uint clamped_lod_e100 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
        metal::float4 _e114_ = RB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e107_ & 127u), as_type<int>(_e107_ >> as_type<uint>(7)))), metal::uint2(RB.get_width(clamped_lod_e100), RB.get_height(clamped_lod_e100)) - 1), clamped_lod_e100);
        if (metal::any(_e99_ != metal::float4(0.0, 0.0, 0.0, 0.0))) {
            metal::float2 _e129_ = (metal::float2x2(metal::float2(_e99_.x, _e99_.y), metal::float2(_e99_.z, _e99_.w)) * _e58_) + _e114_.xy;
            unnamed.gl_ClipDistance.inner[0] = _e129_.x + 1.0;
            unnamed.gl_ClipDistance.inner[1] = _e129_.y + 1.0;
            unnamed.gl_ClipDistance.inner[2] = 1.0 - _e129_.x;
            unnamed.gl_ClipDistance.inner[3] = 1.0 - _e129_.y;
        } else {
            float _e119_ = _e114_.x - 0.5;
            unnamed.gl_ClipDistance.inner[3] = _e119_;
            unnamed.gl_ClipDistance.inner[2] = _e119_;
            unnamed.gl_ClipDistance.inner[1] = _e119_;
            unnamed.gl_ClipDistance.inner[0] = _e119_;
        }
    }
    if (_e76_ == 1u) {
        metal::float4 _e200_ = metal::unpack_unorm4x8_to_float(_e74_.y);
        if (ah) {
            phi_724_ = _e200_;
        } else {
            metal::float3 _e203_ = _e200_.xyz * _e200_.w;
            metal::float4 _e209_ = metal::float4(_e203_.x, _e200_.y, _e200_.z, _e200_.w);
            metal::float4 _e215_ = metal::float4(_e209_.x, _e203_.y, _e209_.z, _e209_.w);
            phi_724_ = metal::float4(_e215_.x, _e215_.y, _e203_.z, _e215_.w);
        }
        metal::float4 _e223_ = phi_724_;
        f1_ = _e223_;
    } else {
        uint clamped_lod_e192 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
        metal::float4 _e151_ = RB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e48_ & 127u), as_type<int>(_e48_ >> as_type<uint>(7)))), metal::uint2(RB.get_width(clamped_lod_e192), RB.get_height(clamped_lod_e192)) - 1), clamped_lod_e192);
        uint _e159_ = _e48_ + 1u;
        uint clamped_lod_e205 = metal::min(uint(0), RB.get_num_mip_levels() - 1);
        metal::float4 _e166_ = RB.read(metal::min(metal::uint2(metal::int2(as_type<int>(_e159_ & 127u), as_type<int>(_e159_ >> as_type<uint>(7)))), metal::uint2(RB.get_width(clamped_lod_e205), RB.get_height(clamped_lod_e205)) - 1), clamped_lod_e205);
        metal::float2 _e169_ = (metal::float2x2(metal::float2(_e151_.x, _e151_.y), metal::float2(_e151_.z, _e151_.w)) * _e58_) + _e166_.xy;
        bool _e170_ = _e76_ == 2u;
        if (!(_e170_)) {
            local = _e76_ == 3u;
        } else {
            local = true;
        }
        bool _e224 = local;
        if (_e224) {
            f1_.w = -(as_type<float>(_e74_.y));
            if (_e166_.z > 0.9) {
                f1_.z = 2.0;
            } else {
                f1_.z = _e166_.w;
            }
            if (_e170_) {
                f1_.y = 0.0;
                f1_.x = _e169_.x;
            } else {
                float _e190_ = f1_.z;
                f1_.z = -(_e190_);
                f1_.x = _e169_.x;
                f1_.y = _e169_.y;
            }
        } else {
            f1_ = metal::float4(_e169_.x, _e169_.y, as_type<float>(_e74_.y), -2.0 - _e166_.z);
        }
    }
    float _e225_ = n.ff;
    float _e227_ = n.gf;
    metal::float4 _e235_ = metal::float4((_e44_.x * _e225_) - 1.0, (_e44_.y * _e227_) - metal::sign(_e227_), 0.0, 1.0);
    unnamed.gl_Position = metal::float4(_e235_.x, _e235_.y, 1.0 - (static_cast<float>(_e56_.x) * 0.000061035156), _e235_.w);
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
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_2 {}, type_3 {}};
    int gl_VertexIndex_1_ = {};
    metal::float3 KB_1_ = {};
    metal::float2 C2_ = {};
    float I3_ = {};
    float e2_ = {};
    metal::float4 f1_ = {};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    KB_1_ = KB;
    main_1_(unnamed, PB, n, KB_1_, C2_, AD, I3_, e2_, RB, f1_);
    metal::float4 _e12_ = unnamed.gl_Position;
    type_2 _e13_ = unnamed.gl_ClipDistance;
    metal::float2 _e14_ = C2_;
    float _e15_ = I3_;
    float _e16_ = e2_;
    metal::float4 _e17_ = f1_;
    const auto _tmp = VertexOutput {_e12_, _e13_, _e14_, _e15_, _e16_, _e17_};
    return main_Output { _tmp.gl_Position, {_tmp.gl_ClipDistance.inner[0],_tmp.gl_ClipDistance.inner[1],_tmp.gl_ClipDistance.inner[2],_tmp.gl_ClipDistance.inner[3]}, _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.member_3_ };
}
