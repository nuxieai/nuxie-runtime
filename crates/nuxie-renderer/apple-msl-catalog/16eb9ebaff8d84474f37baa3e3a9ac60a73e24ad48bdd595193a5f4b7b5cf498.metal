// language: metal3.1
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size0;
    uint size1;
    uint size2;
    uint size7;
};

typedef metal::uint2 type_2[1];
struct Je {
    type_2 c2_;
};
typedef uint type_3[1];
struct h0Bd {
    type_3 c2_;
};
typedef metal::float4 type_6[1];
struct Ke {
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
struct q4Bd {
    type_3 c2_;
};
constant bool fh = true;
constant bool ch = false;
constant bool Yg = true;
constant bool Zg = true;

metal::int2 naga_f2i32(metal::float2 value) {
    return static_cast<metal::int2>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

int naga_f2i32(float value) {
    return static_cast<int>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

void main_1_(
    device Je const& AD,
    device h0Bd& h0_,
    device Ke const& RB,
    thread metal::float4& gl_FragCoord_1_,
    metal::texture2d<float, metal::access::sample> KD,
    metal::sampler Mb,
    constant CC& n,
    device q4Bd& q4_,
    thread uint& B0_1_,
    metal::texture2d<float, metal::access::sample> BD,
    metal::sampler Q9_,
    thread metal::float2& C2_1_,
    thread metal::float4& C1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    bool phi_795_ = {};
    float phi_1137_ = {};
    float phi_1136_ = {};
    float phi_1138_ = {};
    float phi_1141_ = {};
    float phi_1140_ = {};
    bool phi_832_ = {};
    float phi_1143_ = {};
    uint phi_1167_ = {};
    float phi_1142_ = {};
    uint phi_1166_ = {};
    metal::float4 phi_1164_ = {};
    metal::float3 phi_1177_ = {};
    bool local = {};
    bool local_1 = {};
    metal::float4 _e57_ = gl_FragCoord_1_;
    metal::float2 _e58_ = _e57_.xy;
    metal::uint2 _e61_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e58_)));
    uint _e63_ = n.m6_;
    int _e92_ = as_type<int>(((((_e61_.y >> as_type<uint>(5u)) * (((_e63_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e61_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e61_.x & 28u) << as_type<uint>(5u)) + ((_e61_.y & 28u) << as_type<uint>(2)))) + (((_e61_.y & 3u) << as_type<uint>(2)) + (_e61_.x & 3u)));
    uint _e95_ = q4_.c2_[metal::min(unsigned(_e92_), (_buffer_sizes.size7 - 0 - 4) / 4)];
    uint _e97_ = _e95_ >> as_type<uint>(17u);
    uint _e98_ = B0_1_;
    metal::float2 _e102_ = C2_1_;
    metal::float4 _e103_ = BD.sample(Q9_, _e102_, metal::level(0.0));
    q4_.c2_[metal::min(unsigned(_e92_), (_buffer_sizes.size7 - 0 - 4) / 4)] = ((_e98_ << as_type<uint>(17u)) + 65536u) + as_type<uint>(naga_f2i32(metal::rint(metal::clamp(_e103_.x, 0.0, 1.0) * 2048.0)));
    float _e114_ = (static_cast<float>(_e95_ & 131071u) * 0.00048828125) + -32.0;
    metal::uint2 _e117_ = AD.c2_[metal::min(unsigned(_e97_), (_buffer_sizes.size0 - 0 - 8) / 8)];
    phi_1136_ = _e114_;
    if ((_e117_.x & 768u) != 0u) {
        float _e121_ = metal::abs(_e114_);
        phi_795_ = ch;
        if (ch) {
            phi_795_ = (_e117_.x & 512u) != 0u;
        }
        bool _e125_ = phi_795_;
        phi_1137_ = _e121_;
        if (_e125_) {
            phi_1137_ = 1.0 - metal::abs((metal::fract(_e121_ * 0.5) * 2.0) + -1.0);
        }
        float _e133_ = phi_1137_;
        phi_1136_ = _e133_;
    }
    float _e135_ = phi_1136_;
    float _e136_ = metal::clamp(_e135_, 0.0, 1.0);
    phi_1140_ = _e136_;
    if (Yg) {
        uint _e138_ = _e117_.x >> as_type<uint>(16u);
        phi_1141_ = _e136_;
        if (_e138_ != 0u) {
            uint _e142_ = h0_.c2_[metal::min(unsigned(_e92_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            if (_e138_ == (_e142_ >> as_type<uint>(16))) {
                phi_1138_ = metal::min(_e136_, float2(as_type<half2>(_e142_)).x);
            } else {
                phi_1138_ = 0.0;
            }
            float _e150_ = phi_1138_;
            phi_1141_ = _e150_;
        }
        float _e152_ = phi_1141_;
        phi_1140_ = _e152_;
    }
    float _e154_ = phi_1140_;
    phi_832_ = Zg;
    if (Zg) {
        phi_832_ = (_e117_.x & 1024u) != 0u;
    }
    bool _e158_ = phi_832_;
    phi_1143_ = _e154_;
    if (_e158_) {
        uint _e159_ = _e97_ * 4u;
        metal::float4 _e163_ = RB.c2_[metal::min(unsigned(_e159_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e174_ = RB.c2_[metal::min(unsigned(_e159_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e179_ = _e174_.zw;
        metal::float2 _e181_ = (metal::abs((metal::float2x2(metal::float2(_e163_.x, _e163_.y), metal::float2(_e163_.z, _e163_.w)) * _e58_) + _e174_.xy) * _e179_) - _e179_;
        phi_1143_ = metal::min(_e154_, metal::clamp(metal::min(_e181_.x, _e181_.y) + 0.5, 0.0, 1.0));
    }
    float _e189_ = phi_1143_;
    uint _e190_ = _e117_.x & 15u;
    if (_e190_ <= 1u) {
        if (Yg) {
            local = _e190_ == 0u;
        } else {
            local = false;
        }
        bool _e195_ = local;
        phi_1167_ = 0u;
        if (_e195_) {
            phi_1167_ = _e117_.y | as_type<uint>(half2(metal::float2(_e189_, 0.0)));
        }
        uint _e200_ = phi_1167_;
        phi_1166_ = _e200_;
        phi_1164_ = metal::select(metal::unpack_unorm4x8_to_float(_e117_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e195_));
    } else {
        uint _e203_ = _e97_ * 4u;
        metal::float4 _e206_ = RB.c2_[metal::min(unsigned(_e203_), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e217_ = RB.c2_[metal::min(unsigned(_e203_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e220_ = (metal::float2x2(metal::float2(_e206_.x, _e206_.y), metal::float2(_e206_.z, _e206_.w)) * _e58_) + _e217_.xy;
        if (_e190_ == 2u) {
            phi_1142_ = _e220_.x;
        } else {
            phi_1142_ = metal::length(_e220_);
        }
        float _e225_ = phi_1142_;
        metal::float4 _e234_ = KD.sample(Mb, metal::float2((metal::clamp(_e225_, 0.0, 1.0) * _e217_.z) + _e217_.w, as_type<float>(_e117_.y)), metal::level(0.0));
        phi_1166_ = 0u;
        phi_1164_ = _e234_;
    }
    uint _e236_ = phi_1166_;
    metal::float4 _e238_ = phi_1164_;
    float _e240_ = _e238_.w * _e189_;
    metal::float3 _e242_ = _e238_.xyz * _e240_;
    metal::float4 _e246_ = metal::float4(_e242_.x, _e242_.y, _e242_.z, _e240_);
    metal::float3 _e247_ = _e246_.xyz;
    float _e249_ = n.z3_;
    float _e251_ = n.A3_;
    if (fh) {
        local_1 = _e240_ != 0.0;
    } else {
        local_1 = false;
    }
    bool _e302 = local_1;
    if (_e302) {
        phi_1177_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e57_.x) + (0.00583715 * _e57_.y))) * _e249_) + _e251_) + _e247_;
    } else {
        phi_1177_ = _e247_;
    }
    metal::float3 _e267_ = phi_1177_;
    metal::float4 _e273_ = metal::float4(_e267_.x, _e246_.y, _e246_.z, _e246_.w);
    metal::float4 _e279_ = metal::float4(_e273_.x, _e267_.y, _e273_.z, _e273_.w);
    C1_ = metal::float4(_e279_.x, _e279_.y, _e267_.z, _e279_.w);
    if (_e236_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e92_), (_buffer_sizes.size1 - 0 - 4) / 4)] = _e236_;
    }
    return;
}

struct main_Input {
    uint B0_ [[user(loc1), flat]];
    metal::float2 C2_ [[user(loc0), center_perspective]];
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
, metal::float4 gl_FragCoord [[position]]
, device Je const& AD [[buffer(2)]]
, device h0Bd& h0_ [[buffer(5)]]
, device Ke const& RB [[buffer(3)]]
, metal::texture2d<float, metal::access::sample> KD [[texture(0)]]
, metal::sampler Mb [[sampler(1)]]
, constant CC& n [[buffer(0)]]
, device q4Bd& q4_ [[buffer(6)]]
, metal::texture2d<float, metal::access::sample> BD [[texture(2)]]
, metal::sampler Q9_ [[sampler(3)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    uint B0_1_ = {};
    metal::float2 C2_1_ = {};
    metal::float4 C1_ = {};
    const auto B0_ = varyings.B0_;
    const auto C2_ = varyings.C2_;
    gl_FragCoord_1_ = gl_FragCoord;
    B0_1_ = B0_;
    C2_1_ = C2_;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, n, q4_, B0_1_, BD, Q9_, C2_1_, C1_, _buffer_sizes);
    metal::float4 _e7_ = C1_;
    return main_Output { _e7_ };
}
