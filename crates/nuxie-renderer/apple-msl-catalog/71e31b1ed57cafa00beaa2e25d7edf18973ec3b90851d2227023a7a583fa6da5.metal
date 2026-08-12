// language: metal2.4
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
constant bool Yg = false;
constant bool Zg = false;

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
    thread float& i1_1_,
    thread metal::float4& C1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    uint phi_1128_ = {};
    bool phi_789_ = {};
    float phi_1133_ = {};
    float phi_1132_ = {};
    float phi_1134_ = {};
    float phi_1137_ = {};
    float phi_1136_ = {};
    bool phi_826_ = {};
    float phi_1139_ = {};
    uint phi_1165_ = {};
    float phi_1138_ = {};
    uint phi_1164_ = {};
    metal::float4 phi_1162_ = {};
    uint phi_1179_ = {};
    metal::float4 phi_1175_ = {};
    metal::float3 phi_1176_ = {};
    bool local = {};
    bool local_1 = {};
    metal::float4 _e55_ = gl_FragCoord_1_;
    metal::float2 _e56_ = _e55_.xy;
    metal::uint2 _e59_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e56_)));
    uint _e61_ = n.m6_;
    int _e90_ = as_type<int>(((((_e59_.y >> as_type<uint>(5u)) * (((_e61_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e59_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e59_.x & 28u) << as_type<uint>(5u)) + ((_e59_.y & 28u) << as_type<uint>(2)))) + (((_e59_.y & 3u) << as_type<uint>(2)) + (_e59_.x & 3u)));
    uint _e93_ = q4_.c2_[metal::min(unsigned(_e90_), (_buffer_sizes.size7 - 0 - 4) / 4)];
    uint _e95_ = _e93_ >> as_type<uint>(17u);
    uint _e96_ = B0_1_;
    if (_e95_ == _e96_) {
        phi_1128_ = _e93_;
    } else {
        phi_1128_ = (_e96_ << as_type<uint>(17u)) + 65536u;
    }
    uint _e102_ = phi_1128_;
    float _e103_ = i1_1_;
    q4_.c2_[metal::min(unsigned(_e90_), (_buffer_sizes.size7 - 0 - 4) / 4)] = _e102_ + as_type<uint>(naga_f2i32(metal::rint(_e103_ * 2048.0)));
    phi_1179_ = 0u;
    phi_1175_ = metal::float4(0.0, 0.0, 0.0, 0.0);
    if (_e95_ != _e96_) {
        float _e113_ = (static_cast<float>(_e93_ & 131071u) * 0.00048828125) + -32.0;
        metal::uint2 _e116_ = AD.c2_[metal::min(unsigned(_e95_), (_buffer_sizes.size0 - 0 - 8) / 8)];
        phi_1132_ = _e113_;
        if ((_e116_.x & 768u) != 0u) {
            float _e120_ = metal::abs(_e113_);
            phi_789_ = ch;
            if (ch) {
                phi_789_ = (_e116_.x & 512u) != 0u;
            }
            bool _e124_ = phi_789_;
            phi_1133_ = _e120_;
            if (_e124_) {
                phi_1133_ = 1.0 - metal::abs((metal::fract(_e120_ * 0.5) * 2.0) + -1.0);
            }
            float _e132_ = phi_1133_;
            phi_1132_ = _e132_;
        }
        float _e134_ = phi_1132_;
        float _e135_ = metal::clamp(_e134_, 0.0, 1.0);
        phi_1136_ = _e135_;
        if (Yg) {
            uint _e137_ = _e116_.x >> as_type<uint>(16u);
            phi_1137_ = _e135_;
            if (_e137_ != 0u) {
                uint _e141_ = h0_.c2_[metal::min(unsigned(_e90_), (_buffer_sizes.size1 - 0 - 4) / 4)];
                if (_e137_ == (_e141_ >> as_type<uint>(16))) {
                    phi_1134_ = metal::min(_e135_, float2(as_type<half2>(_e141_)).x);
                } else {
                    phi_1134_ = 0.0;
                }
                float _e149_ = phi_1134_;
                phi_1137_ = _e149_;
            }
            float _e151_ = phi_1137_;
            phi_1136_ = _e151_;
        }
        float _e153_ = phi_1136_;
        phi_826_ = Zg;
        if (Zg) {
            phi_826_ = (_e116_.x & 1024u) != 0u;
        }
        bool _e157_ = phi_826_;
        phi_1139_ = _e153_;
        if (_e157_) {
            uint _e158_ = _e95_ * 4u;
            metal::float4 _e162_ = RB.c2_[metal::min(unsigned(_e158_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float4 _e173_ = RB.c2_[metal::min(unsigned(_e158_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float2 _e178_ = _e173_.zw;
            metal::float2 _e180_ = (metal::abs((metal::float2x2(metal::float2(_e162_.x, _e162_.y), metal::float2(_e162_.z, _e162_.w)) * _e56_) + _e173_.xy) * _e178_) - _e178_;
            phi_1139_ = metal::min(_e153_, metal::clamp(metal::min(_e180_.x, _e180_.y) + 0.5, 0.0, 1.0));
        }
        float _e188_ = phi_1139_;
        uint _e189_ = _e116_.x & 15u;
        if (_e189_ <= 1u) {
            if (Yg) {
                local = _e189_ == 0u;
            } else {
                local = false;
            }
            bool _e194_ = local;
            phi_1165_ = 0u;
            if (_e194_) {
                phi_1165_ = _e116_.y | as_type<uint>(half2(metal::float2(_e188_, 0.0)));
            }
            uint _e199_ = phi_1165_;
            phi_1164_ = _e199_;
            phi_1162_ = metal::select(metal::unpack_unorm4x8_to_float(_e116_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e194_));
        } else {
            uint _e202_ = _e95_ * 4u;
            metal::float4 _e205_ = RB.c2_[metal::min(unsigned(_e202_), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float4 _e216_ = RB.c2_[metal::min(unsigned(_e202_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float2 _e219_ = (metal::float2x2(metal::float2(_e205_.x, _e205_.y), metal::float2(_e205_.z, _e205_.w)) * _e56_) + _e216_.xy;
            if (_e189_ == 2u) {
                phi_1138_ = _e219_.x;
            } else {
                phi_1138_ = metal::length(_e219_);
            }
            float _e224_ = phi_1138_;
            metal::float4 _e233_ = KD.sample(Mb, metal::float2((metal::clamp(_e224_, 0.0, 1.0) * _e216_.z) + _e216_.w, as_type<float>(_e116_.y)), metal::level(0.0));
            phi_1164_ = 0u;
            phi_1162_ = _e233_;
        }
        uint _e235_ = phi_1164_;
        metal::float4 _e237_ = phi_1162_;
        float _e239_ = _e237_.w * _e188_;
        metal::float3 _e241_ = _e237_.xyz * _e239_;
        phi_1179_ = _e235_;
        phi_1175_ = metal::float4(_e241_.x, _e241_.y, _e241_.z, _e239_);
    }
    uint _e247_ = phi_1179_;
    metal::float4 _e249_ = phi_1175_;
    metal::float3 _e250_ = _e249_.xyz;
    float _e253_ = n.z3_;
    float _e255_ = n.A3_;
    if (fh) {
        local_1 = _e249_.w != 0.0;
    } else {
        local_1 = false;
    }
    bool _e309 = local_1;
    if (_e309) {
        phi_1176_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e55_.x) + (0.00583715 * _e55_.y))) * _e253_) + _e255_) + _e250_;
    } else {
        phi_1176_ = _e250_;
    }
    metal::float3 _e271_ = phi_1176_;
    metal::float4 _e277_ = metal::float4(_e271_.x, _e249_.y, _e249_.z, _e249_.w);
    metal::float4 _e283_ = metal::float4(_e277_.x, _e271_.y, _e277_.z, _e277_.w);
    C1_ = metal::float4(_e283_.x, _e283_.y, _e271_.z, _e283_.w);
    if (_e247_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e90_), (_buffer_sizes.size1 - 0 - 4) / 4)] = _e247_;
    }
    return;
}

struct main_Input {
    uint B0_ [[user(loc1), flat]];
    float i1_ [[user(loc0), flat]];
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
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    uint B0_1_ = {};
    float i1_1_ = {};
    metal::float4 C1_ = {};
    const auto B0_ = varyings.B0_;
    const auto i1_ = varyings.i1_;
    gl_FragCoord_1_ = gl_FragCoord;
    B0_1_ = B0_;
    i1_1_ = i1_;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, n, q4_, B0_1_, i1_1_, C1_, _buffer_sizes);
    metal::float4 _e7_ = C1_;
    return main_Output { _e7_ };
}
