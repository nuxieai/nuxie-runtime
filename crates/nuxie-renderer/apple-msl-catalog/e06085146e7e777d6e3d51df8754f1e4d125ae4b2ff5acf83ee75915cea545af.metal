// language: metal3.2
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size0;
    uint size1;
    uint size2;
    uint size12;
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

void main_1_(
    device Je const& AD,
    device h0Bd& h0_,
    device Ke const& RB,
    thread metal::float4& gl_FragCoord_1_,
    metal::texture2d<float, metal::access::sample> KD,
    metal::sampler Mb,
    constant CC& n,
    metal::texture2d<float, metal::access::sample> IC,
    metal::sampler S5_,
    thread metal::float2& X1_1_,
    thread float& R4_1_,
    thread metal::float4& L0_1_,
    device q4Bd& q4_,
    thread uint& w3_1_,
    thread float& H1_1_,
    thread metal::float4& C1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    float phi_1252_ = {};
    bool phi_850_ = {};
    float phi_1199_ = {};
    float phi_1198_ = {};
    float phi_1200_ = {};
    float phi_1203_ = {};
    float phi_1202_ = {};
    bool phi_887_ = {};
    float phi_1205_ = {};
    uint phi_1232_ = {};
    float phi_1204_ = {};
    uint phi_1231_ = {};
    metal::float4 phi_1229_ = {};
    bool phi_640_ = {};
    uint phi_1243_ = {};
    float phi_1258_ = {};
    float phi_1259_ = {};
    metal::float3 phi_1280_ = {};
    bool local = {};
    bool local_1 = {};
    metal::float4 _e58_ = gl_FragCoord_1_;
    metal::float2 _e59_ = _e58_.xy;
    metal::uint2 _e62_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e59_)));
    uint _e64_ = n.m6_;
    int _e93_ = as_type<int>(((((_e62_.y >> as_type<uint>(5u)) * (((_e64_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e62_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e62_.x & 28u) << as_type<uint>(5u)) + ((_e62_.y & 28u) << as_type<uint>(2)))) + (((_e62_.y & 3u) << as_type<uint>(2)) + (_e62_.x & 3u)));
    metal::float2 _e94_ = X1_1_;
    metal::float4 _e95_ = IC.sample(S5_, _e94_);
    float _e96_ = R4_1_;
    float _e97_ = metal::min(_e96_, 1.0);
    phi_1252_ = _e97_;
    if (Zg) {
        metal::float4 _e98_ = L0_1_;
        metal::float2 _e101_ = metal::min(_e98_.xy, _e98_.zw);
        phi_1252_ = metal::clamp(metal::min(_e101_.x, _e101_.y), 0.0, _e97_);
    }
    float _e107_ = phi_1252_;
    uint _e110_ = q4_.c2_[metal::min(unsigned(_e93_), (_buffer_sizes.size12 - 0 - 4) / 4)];
    uint _e112_ = _e110_ >> as_type<uint>(17u);
    float _e116_ = (static_cast<float>(_e110_ & 131071u) * 0.00048828125) + -32.0;
    metal::uint2 _e119_ = AD.c2_[metal::min(unsigned(_e112_), (_buffer_sizes.size0 - 0 - 8) / 8)];
    phi_1198_ = _e116_;
    if ((_e119_.x & 768u) != 0u) {
        float _e123_ = metal::abs(_e116_);
        phi_850_ = ch;
        if (ch) {
            phi_850_ = (_e119_.x & 512u) != 0u;
        }
        bool _e127_ = phi_850_;
        phi_1199_ = _e123_;
        if (_e127_) {
            phi_1199_ = 1.0 - metal::abs((metal::fract(_e123_ * 0.5) * 2.0) + -1.0);
        }
        float _e135_ = phi_1199_;
        phi_1198_ = _e135_;
    }
    float _e137_ = phi_1198_;
    float _e138_ = metal::clamp(_e137_, 0.0, 1.0);
    phi_1202_ = _e138_;
    if (Yg) {
        uint _e140_ = _e119_.x >> as_type<uint>(16u);
        phi_1203_ = _e138_;
        if (_e140_ != 0u) {
            uint _e144_ = h0_.c2_[metal::min(unsigned(_e93_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            if (_e140_ == (_e144_ >> as_type<uint>(16))) {
                phi_1200_ = metal::min(_e138_, float2(as_type<half2>(_e144_)).x);
            } else {
                phi_1200_ = 0.0;
            }
            float _e152_ = phi_1200_;
            phi_1203_ = _e152_;
        }
        float _e154_ = phi_1203_;
        phi_1202_ = _e154_;
    }
    float _e156_ = phi_1202_;
    phi_887_ = Zg;
    if (Zg) {
        phi_887_ = (_e119_.x & 1024u) != 0u;
    }
    bool _e160_ = phi_887_;
    phi_1205_ = _e156_;
    if (_e160_) {
        uint _e161_ = _e112_ * 4u;
        metal::float4 _e165_ = RB.c2_[metal::min(unsigned(_e161_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e176_ = RB.c2_[metal::min(unsigned(_e161_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e181_ = _e176_.zw;
        metal::float2 _e183_ = (metal::abs((metal::float2x2(metal::float2(_e165_.x, _e165_.y), metal::float2(_e165_.z, _e165_.w)) * _e59_) + _e176_.xy) * _e181_) - _e181_;
        phi_1205_ = metal::min(_e156_, metal::clamp(metal::min(_e183_.x, _e183_.y) + 0.5, 0.0, 1.0));
    }
    float _e191_ = phi_1205_;
    uint _e192_ = _e119_.x & 15u;
    if (_e192_ <= 1u) {
        if (Yg) {
            local = _e192_ == 0u;
        } else {
            local = false;
        }
        bool _e197_ = local;
        phi_1232_ = 0u;
        if (_e197_) {
            phi_1232_ = _e119_.y | as_type<uint>(half2(metal::float2(_e191_, 0.0)));
        }
        uint _e202_ = phi_1232_;
        phi_1231_ = _e202_;
        phi_1229_ = metal::select(metal::unpack_unorm4x8_to_float(_e119_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e197_));
    } else {
        uint _e205_ = _e112_ * 4u;
        metal::float4 _e208_ = RB.c2_[metal::min(unsigned(_e205_), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e219_ = RB.c2_[metal::min(unsigned(_e205_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e222_ = (metal::float2x2(metal::float2(_e208_.x, _e208_.y), metal::float2(_e208_.z, _e208_.w)) * _e59_) + _e219_.xy;
        if (_e192_ == 2u) {
            phi_1204_ = _e222_.x;
        } else {
            phi_1204_ = metal::length(_e222_);
        }
        float _e227_ = phi_1204_;
        metal::float4 _e236_ = KD.sample(Mb, metal::float2((metal::clamp(_e227_, 0.0, 1.0) * _e219_.z) + _e219_.w, as_type<float>(_e119_.y)), metal::level(0.0));
        phi_1231_ = 0u;
        phi_1229_ = _e236_;
    }
    uint _e238_ = phi_1231_;
    metal::float4 _e240_ = phi_1229_;
    float _e242_ = _e240_.w * _e191_;
    metal::float3 _e244_ = _e240_.xyz * _e242_;
    phi_640_ = Yg;
    if (Yg) {
        uint _e249_ = w3_1_;
        phi_640_ = _e249_ != 0u;
    }
    bool _e252_ = phi_640_;
    phi_1259_ = _e107_;
    if (_e252_) {
        if (_e238_ != 0u) {
            phi_1243_ = _e238_;
        } else {
            uint _e256_ = h0_.c2_[metal::min(unsigned(_e93_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            phi_1243_ = _e256_;
        }
        uint _e258_ = phi_1243_;
        uint _e259_ = w3_1_;
        if (_e259_ == (_e258_ >> as_type<uint>(16))) {
            phi_1258_ = metal::min(_e107_, float2(as_type<half2>(_e258_)).x);
        } else {
            phi_1258_ = 0.0;
        }
        float _e267_ = phi_1258_;
        phi_1259_ = _e267_;
    }
    float _e269_ = phi_1259_;
    float _e270_ = H1_1_;
    metal::float4 _e272_ = _e95_ * (_e269_ * _e270_);
    metal::float4 _e276_ = (metal::float4(_e244_.x, _e244_.y, _e244_.z, _e242_) * (1.0 - _e272_.w)) + _e272_;
    metal::float3 _e277_ = _e276_.xyz;
    float _e280_ = n.z3_;
    float _e282_ = n.A3_;
    if (fh) {
        local_1 = _e276_.w != 0.0;
    } else {
        local_1 = false;
    }
    bool _e338 = local_1;
    if (_e338) {
        phi_1280_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e58_.x) + (0.00583715 * _e58_.y))) * _e280_) + _e282_) + _e277_;
    } else {
        phi_1280_ = _e277_;
    }
    metal::float3 _e298_ = phi_1280_;
    metal::float4 _e304_ = metal::float4(_e298_.x, _e276_.y, _e276_.z, _e276_.w);
    metal::float4 _e310_ = metal::float4(_e304_.x, _e298_.y, _e304_.z, _e304_.w);
    C1_ = metal::float4(_e310_.x, _e310_.y, _e298_.z, _e310_.w);
    if (_e238_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e93_), (_buffer_sizes.size1 - 0 - 4) / 4)] = _e238_;
    }
    q4_.c2_[metal::min(unsigned(_e93_), (_buffer_sizes.size12 - 0 - 4) / 4)] = 65536u;
    return;
}

struct main_Input {
    metal::float2 X1_ [[user(loc0), center_perspective]];
    float R4_ [[user(loc1), center_perspective]];
    metal::float4 L0_ [[user(loc2), center_perspective]];
    uint w3_ [[user(loc4), flat]];
    float H1_ [[user(loc3), flat]];
    uint A1_ [[user(loc5), flat]];
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
, metal::texture2d<float, metal::access::sample> IC [[texture(3)]]
, metal::sampler S5_ [[sampler(0)]]
, device q4Bd& q4_ [[buffer(6)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    metal::float2 X1_1_ = {};
    float R4_1_ = {};
    metal::float4 L0_1_ = {};
    uint w3_1_ = {};
    float H1_1_ = {};
    metal::float4 C1_ = {};
    uint A1_1_ = {};
    const auto X1_ = varyings.X1_;
    const auto R4_ = varyings.R4_;
    const auto L0_ = varyings.L0_;
    const auto w3_ = varyings.w3_;
    const auto H1_ = varyings.H1_;
    const auto A1_ = varyings.A1_;
    gl_FragCoord_1_ = gl_FragCoord;
    X1_1_ = X1_;
    R4_1_ = R4_;
    L0_1_ = L0_;
    w3_1_ = w3_;
    H1_1_ = H1_;
    A1_1_ = A1_;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, n, IC, S5_, X1_1_, R4_1_, L0_1_, q4_, w3_1_, H1_1_, C1_, _buffer_sizes);
    metal::float4 _e15_ = C1_;
    return main_Output { _e15_ };
}
