// language: metal3.0
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
constant bool Zg = true;

metal::int2 naga_f2i32(metal::float2 value) {
    return static_cast<metal::int2>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

void main_1_(
    device Je const& AD,
    device h0Bd const& h0_,
    device Ke const& RB,
    thread metal::float4& gl_FragCoord_1_,
    metal::texture2d<float, metal::access::sample> KD,
    metal::sampler Mb,
    constant CC& n,
    device q4Bd const& q4_,
    thread metal::float4& C1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    bool phi_680_ = {};
    float phi_986_ = {};
    float phi_985_ = {};
    float phi_987_ = {};
    float phi_990_ = {};
    float phi_989_ = {};
    bool phi_717_ = {};
    float phi_1003_ = {};
    float phi_991_ = {};
    metal::float4 phi_1005_ = {};
    metal::float3 phi_1007_ = {};
    bool local = {};
    bool local_1 = {};
    metal::float4 _e51_ = gl_FragCoord_1_;
    metal::float2 _e52_ = _e51_.xy;
    metal::uint2 _e55_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e52_)));
    uint _e57_ = n.m6_;
    int _e86_ = as_type<int>(((((_e55_.y >> as_type<uint>(5u)) * (((_e57_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e55_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e55_.x & 28u) << as_type<uint>(5u)) + ((_e55_.y & 28u) << as_type<uint>(2)))) + (((_e55_.y & 3u) << as_type<uint>(2)) + (_e55_.x & 3u)));
    uint _e89_ = q4_.c2_[metal::min(unsigned(_e86_), (_buffer_sizes.size7 - 0 - 4) / 4)];
    float _e93_ = (static_cast<float>(_e89_ & 131071u) * 0.00048828125) + -32.0;
    uint _e95_ = _e89_ >> as_type<uint>(17u);
    metal::uint2 _e98_ = AD.c2_[metal::min(unsigned(_e95_), (_buffer_sizes.size0 - 0 - 8) / 8)];
    phi_985_ = _e93_;
    if ((_e98_.x & 768u) != 0u) {
        float _e102_ = metal::abs(_e93_);
        phi_680_ = ch;
        if (ch) {
            phi_680_ = (_e98_.x & 512u) != 0u;
        }
        bool _e106_ = phi_680_;
        phi_986_ = _e102_;
        if (_e106_) {
            phi_986_ = 1.0 - metal::abs((metal::fract(_e102_ * 0.5) * 2.0) + -1.0);
        }
        float _e114_ = phi_986_;
        phi_985_ = _e114_;
    }
    float _e116_ = phi_985_;
    float _e117_ = metal::clamp(_e116_, 0.0, 1.0);
    phi_989_ = _e117_;
    if (Yg) {
        uint _e119_ = _e98_.x >> as_type<uint>(16u);
        phi_990_ = _e117_;
        if (_e119_ != 0u) {
            uint _e123_ = h0_.c2_[metal::min(unsigned(_e86_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            if (_e119_ == (_e123_ >> as_type<uint>(16))) {
                phi_987_ = metal::min(_e117_, float2(as_type<half2>(_e123_)).x);
            } else {
                phi_987_ = 0.0;
            }
            float _e131_ = phi_987_;
            phi_990_ = _e131_;
        }
        float _e133_ = phi_990_;
        phi_989_ = _e133_;
    }
    float _e135_ = phi_989_;
    phi_717_ = Zg;
    if (Zg) {
        phi_717_ = (_e98_.x & 1024u) != 0u;
    }
    bool _e139_ = phi_717_;
    phi_1003_ = _e135_;
    if (_e139_) {
        uint _e140_ = _e95_ * 4u;
        metal::float4 _e144_ = RB.c2_[metal::min(unsigned(_e140_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e155_ = RB.c2_[metal::min(unsigned(_e140_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e160_ = _e155_.zw;
        metal::float2 _e162_ = (metal::abs((metal::float2x2(metal::float2(_e144_.x, _e144_.y), metal::float2(_e144_.z, _e144_.w)) * _e52_) + _e155_.xy) * _e160_) - _e160_;
        phi_1003_ = metal::min(_e135_, metal::clamp(metal::min(_e162_.x, _e162_.y) + 0.5, 0.0, 1.0));
    }
    float _e170_ = phi_1003_;
    uint _e171_ = _e98_.x & 15u;
    if (_e171_ <= 1u) {
        if (Yg) {
            local = _e171_ == 0u;
        } else {
            local = false;
        }
        bool _e199 = local;
        phi_1005_ = metal::select(metal::unpack_unorm4x8_to_float(_e98_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e199));
    } else {
        uint _e179_ = _e95_ * 4u;
        metal::float4 _e182_ = RB.c2_[metal::min(unsigned(_e179_), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e193_ = RB.c2_[metal::min(unsigned(_e179_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e196_ = (metal::float2x2(metal::float2(_e182_.x, _e182_.y), metal::float2(_e182_.z, _e182_.w)) * _e52_) + _e193_.xy;
        if (_e171_ == 2u) {
            phi_991_ = _e196_.x;
        } else {
            phi_991_ = metal::length(_e196_);
        }
        float _e201_ = phi_991_;
        metal::float4 _e210_ = KD.sample(Mb, metal::float2((metal::clamp(_e201_, 0.0, 1.0) * _e193_.z) + _e193_.w, as_type<float>(_e98_.y)), metal::level(0.0));
        phi_1005_ = _e210_;
    }
    metal::float4 _e212_ = phi_1005_;
    float _e214_ = _e212_.w * _e170_;
    metal::float3 _e216_ = _e212_.xyz * _e214_;
    metal::float4 _e220_ = metal::float4(_e216_.x, _e216_.y, _e216_.z, _e214_);
    metal::float3 _e221_ = _e220_.xyz;
    float _e223_ = n.z3_;
    float _e225_ = n.A3_;
    if (fh) {
        local_1 = _e214_ != 0.0;
    } else {
        local_1 = false;
    }
    bool _e265 = local_1;
    if (_e265) {
        phi_1007_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e51_.x) + (0.00583715 * _e51_.y))) * _e223_) + _e225_) + _e221_;
    } else {
        phi_1007_ = _e221_;
    }
    metal::float3 _e241_ = phi_1007_;
    metal::float4 _e247_ = metal::float4(_e241_.x, _e220_.y, _e220_.z, _e220_.w);
    metal::float4 _e253_ = metal::float4(_e247_.x, _e241_.y, _e247_.z, _e247_.w);
    C1_ = metal::float4(_e253_.x, _e253_.y, _e241_.z, _e253_.w);
    return;
}

struct main_Input {
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  metal::float4 gl_FragCoord [[position]]
, device Je const& AD [[buffer(2)]]
, device h0Bd const& h0_ [[buffer(5)]]
, device Ke const& RB [[buffer(3)]]
, metal::texture2d<float, metal::access::sample> KD [[texture(0)]]
, metal::sampler Mb [[sampler(1)]]
, constant CC& n [[buffer(0)]]
, device q4Bd const& q4_ [[buffer(6)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 C1_ = {};
    gl_FragCoord_1_ = gl_FragCoord;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, n, q4_, C1_, _buffer_sizes);
    metal::float4 _e3_ = C1_;
    return main_Output { _e3_ };
}
