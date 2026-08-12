// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size2;
    uint size3;
    uint size4;
    uint size11;
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
typedef metal::atomic_uint type_10[1];
struct q4Bd_1_ {
    type_10 c2_;
};
constant bool fh = true;
constant bool ch = false;
constant bool Yg = false;
constant bool Zg = true;
constant bool bh = true;

metal::int2 naga_f2i32(metal::float2 value) {
    return static_cast<metal::int2>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

uint naga_f2u32(float value) {
    return static_cast<uint>(metal::clamp(value, 0.0, 4294967000.0));
}

void main_1_(
    metal::texture2d<float, metal::access::sample> XC,
    metal::sampler aa,
    device Je const& AD,
    device h0Bd& h0_,
    device Ke const& RB,
    thread metal::float4& gl_FragCoord_1_,
    metal::texture2d<float, metal::access::sample> KD,
    metal::sampler Mb,
    constant CC& n,
    thread metal::float4& O_1_,
    thread uint& B0_1_,
    device q4Bd_1_& q4_,
    thread metal::float4& C1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    bool phi_794_ = {};
    bool phi_807_ = {};
    float phi_1657_ = {};
    float phi_1665_ = {};
    float phi_1673_ = {};
    float phi_1672_ = {};
    bool phi_1288_ = {};
    float phi_1676_ = {};
    float phi_1675_ = {};
    float phi_1677_ = {};
    float phi_1680_ = {};
    float phi_1679_ = {};
    bool phi_1325_ = {};
    float phi_1682_ = {};
    uint phi_1718_ = {};
    float phi_1681_ = {};
    uint phi_1717_ = {};
    metal::float4 phi_1715_ = {};
    uint phi_1733_ = {};
    metal::float4 phi_1728_ = {};
    metal::float3 phi_1730_ = {};
    bool local = {};
    bool local_1 = {};
    metal::float4 _e73_ = gl_FragCoord_1_;
    metal::float2 _e74_ = _e73_.xy;
    metal::uint2 _e77_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e74_)));
    uint _e79_ = n.m6_;
    int _e108_ = as_type<int>(((((_e77_.y >> as_type<uint>(5u)) * (((_e79_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e77_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e77_.x & 28u) << as_type<uint>(5u)) + ((_e77_.y & 28u) << as_type<uint>(2)))) + (((_e77_.y & 3u) << as_type<uint>(2)) + (_e77_.x & 3u)));
    phi_794_ = bh;
    if (bh) {
        metal::float4 _e109_ = O_1_;
        phi_794_ = _e109_.x < -1.5;
    }
    bool _e113_ = phi_794_;
    if (_e113_) {
        metal::float4 _e114_ = O_1_;
        metal::float4 _e118_ = XC.sample(aa, metal::float2(3.0 + _e114_.x, 0.0), metal::level(0.0));
        metal::float4 _e124_ = XC.sample(aa, metal::float2(1.0 - _e114_.y, 0.0), metal::level(0.0));
        phi_1672_ = (1.0 - _e118_.x) - _e124_.x;
    } else {
        phi_807_ = bh;
        if (bh) {
            metal::float4 _e127_ = O_1_;
            phi_807_ = _e127_.y < -1.5;
        }
        bool _e131_ = phi_807_;
        if (_e131_) {
            metal::float4 _e132_ = O_1_;
            float _e135_ = metal::max(_e132_.w, 0.0);
            if (_e132_.z >= 0.0) {
                metal::float4 _e138_ = XC.sample(aa, metal::float2(_e135_, 0.0), metal::level(0.0));
                phi_1657_ = _e138_.x;
            } else {
                phi_1657_ = 0.0;
            }
            float _e141_ = phi_1657_;
            phi_1665_ = _e141_;
            if (metal::abs(_e132_.z) < 1000.0) {
                float _e148_ = -2.0 - _e132_.y;
                float _e150_ = (_e148_ - _e135_) * 0.5984134;
                metal::float4 _e153_ = metal::float4(_e135_) + (metal::float4(0.20888568, 0.62665707, 1.0444285, 1.4621998) * _e150_);
                metal::float4 _e159_ = (_e153_ * -(_e132_.z)) + metal::float4((_e148_ * _e132_.z) + (metal::abs(_e132_.x) - 0.25));
                metal::float4 _e162_ = XC.sample(aa, metal::float2(_e159_.x, 0.0), metal::level(0.0));
                metal::float4 _e165_ = XC.sample(aa, metal::float2(_e159_.y, 0.0), metal::level(0.0));
                metal::float4 _e168_ = XC.sample(aa, metal::float2(_e159_.z, 0.0), metal::level(0.0));
                metal::float4 _e171_ = XC.sample(aa, metal::float2(_e159_.w, 0.0), metal::level(0.0));
                metal::float4 _e177_ = _e153_ * 5.0959306;
                phi_1665_ = _e141_ + (metal::dot(metal::float4(_e162_.x, _e165_.x, _e168_.x, _e171_.x), metal::exp2((metal::float4(2.5479653, 2.5479653, 2.5479653, 2.5479653) - _e177_) * (_e177_ + metal::float4(-2.5479653, -2.5479653, -2.5479653, -2.5479653)))) * _e150_);
            }
            float _e186_ = phi_1665_;
            phi_1673_ = _e186_ * metal::sign(_e132_.x);
        } else {
            float _e191_ = O_1_.x;
            float _e193_ = O_1_.y;
            phi_1673_ = metal::min(metal::min(_e191_, metal::abs(_e193_)), 1.0);
        }
        float _e198_ = phi_1673_;
        phi_1672_ = _e198_;
    }
    float _e200_ = phi_1672_;
    uint _e204_ = naga_f2u32(metal::rint((_e200_ * 2048.0) + 65536.0));
    uint _e205_ = B0_1_;
    uint _e208_ = (_e205_ << as_type<uint>(17u)) | _e204_;
    uint _e247 = metal::atomic_fetch_max_explicit(&q4_.c2_[metal::min(unsigned(_e108_), (_buffer_sizes.size11 - 0 - 4) / 4)], _e208_, metal::memory_order_relaxed);
    uint _e213_ = _e247 >> as_type<uint>(17u);
    if (_e213_ == _e205_) {
        metal::float4 _e215_ = O_1_;
        if (_e215_.y < 0.0) {
            uint _e265 = metal::atomic_fetch_add_explicit(&q4_.c2_[metal::min(unsigned(_e108_), (_buffer_sizes.size11 - 0 - 4) / 4)], (_e204_ + (_e247 - metal::max(_e208_, _e247))) - 65536u, metal::memory_order_relaxed);
        }
        phi_1733_ = 0u;
        phi_1728_ = metal::float4(0.0, 0.0, 0.0, 0.0);
    } else {
        float _e226_ = (static_cast<float>(_e247 & 131071u) * 0.00048828125) + -32.0;
        metal::uint2 _e229_ = AD.c2_[metal::min(unsigned(_e213_), (_buffer_sizes.size2 - 0 - 8) / 8)];
        phi_1675_ = _e226_;
        if ((_e229_.x & 768u) != 0u) {
            float _e233_ = metal::abs(_e226_);
            phi_1288_ = ch;
            if (ch) {
                phi_1288_ = (_e229_.x & 512u) != 0u;
            }
            bool _e237_ = phi_1288_;
            phi_1676_ = _e233_;
            if (_e237_) {
                phi_1676_ = 1.0 - metal::abs((metal::fract(_e233_ * 0.5) * 2.0) + -1.0);
            }
            float _e245_ = phi_1676_;
            phi_1675_ = _e245_;
        }
        float _e247_ = phi_1675_;
        float _e248_ = metal::clamp(_e247_, 0.0, 1.0);
        phi_1679_ = _e248_;
        if (Yg) {
            uint _e250_ = _e229_.x >> as_type<uint>(16u);
            phi_1680_ = _e248_;
            if (_e250_ != 0u) {
                uint _e254_ = h0_.c2_[metal::min(unsigned(_e108_), (_buffer_sizes.size3 - 0 - 4) / 4)];
                if (_e250_ == (_e254_ >> as_type<uint>(16))) {
                    phi_1677_ = metal::min(_e248_, float2(as_type<half2>(_e254_)).x);
                } else {
                    phi_1677_ = 0.0;
                }
                float _e262_ = phi_1677_;
                phi_1680_ = _e262_;
            }
            float _e264_ = phi_1680_;
            phi_1679_ = _e264_;
        }
        float _e266_ = phi_1679_;
        phi_1325_ = Zg;
        if (Zg) {
            phi_1325_ = (_e229_.x & 1024u) != 0u;
        }
        bool _e270_ = phi_1325_;
        phi_1682_ = _e266_;
        if (_e270_) {
            uint _e271_ = _e213_ * 4u;
            metal::float4 _e275_ = RB.c2_[metal::min(unsigned(_e271_ + 2u), (_buffer_sizes.size4 - 0 - 16) / 16)];
            metal::float4 _e286_ = RB.c2_[metal::min(unsigned(_e271_ + 3u), (_buffer_sizes.size4 - 0 - 16) / 16)];
            metal::float2 _e291_ = _e286_.zw;
            metal::float2 _e293_ = (metal::abs((metal::float2x2(metal::float2(_e275_.x, _e275_.y), metal::float2(_e275_.z, _e275_.w)) * _e74_) + _e286_.xy) * _e291_) - _e291_;
            phi_1682_ = metal::min(_e266_, metal::clamp(metal::min(_e293_.x, _e293_.y) + 0.5, 0.0, 1.0));
        }
        float _e301_ = phi_1682_;
        uint _e302_ = _e229_.x & 15u;
        if (_e302_ <= 1u) {
            if (Yg) {
                local = _e302_ == 0u;
            } else {
                local = false;
            }
            bool _e307_ = local;
            phi_1718_ = 0u;
            if (_e307_) {
                phi_1718_ = _e229_.y | as_type<uint>(half2(metal::float2(_e301_, 0.0)));
            }
            uint _e312_ = phi_1718_;
            phi_1717_ = _e312_;
            phi_1715_ = metal::select(metal::unpack_unorm4x8_to_float(_e229_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e307_));
        } else {
            uint _e315_ = _e213_ * 4u;
            metal::float4 _e318_ = RB.c2_[metal::min(unsigned(_e315_), (_buffer_sizes.size4 - 0 - 16) / 16)];
            metal::float4 _e329_ = RB.c2_[metal::min(unsigned(_e315_ + 1u), (_buffer_sizes.size4 - 0 - 16) / 16)];
            metal::float2 _e332_ = (metal::float2x2(metal::float2(_e318_.x, _e318_.y), metal::float2(_e318_.z, _e318_.w)) * _e74_) + _e329_.xy;
            if (_e302_ == 2u) {
                phi_1681_ = _e332_.x;
            } else {
                phi_1681_ = metal::length(_e332_);
            }
            float _e337_ = phi_1681_;
            metal::float4 _e346_ = KD.sample(Mb, metal::float2((metal::clamp(_e337_, 0.0, 1.0) * _e329_.z) + _e329_.w, as_type<float>(_e229_.y)), metal::level(0.0));
            phi_1717_ = 0u;
            phi_1715_ = _e346_;
        }
        uint _e348_ = phi_1717_;
        metal::float4 _e350_ = phi_1715_;
        float _e352_ = _e350_.w * _e301_;
        metal::float3 _e354_ = _e350_.xyz * _e352_;
        phi_1733_ = _e348_;
        phi_1728_ = metal::float4(_e354_.x, _e354_.y, _e354_.z, _e352_);
    }
    uint _e360_ = phi_1733_;
    metal::float4 _e362_ = phi_1728_;
    metal::float3 _e363_ = _e362_.xyz;
    float _e366_ = n.z3_;
    float _e368_ = n.A3_;
    if (fh) {
        local_1 = _e362_.w != 0.0;
    } else {
        local_1 = false;
    }
    bool _e476 = local_1;
    if (_e476) {
        phi_1730_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e73_.x) + (0.00583715 * _e73_.y))) * _e366_) + _e368_) + _e363_;
    } else {
        phi_1730_ = _e363_;
    }
    metal::float3 _e384_ = phi_1730_;
    metal::float4 _e390_ = metal::float4(_e384_.x, _e362_.y, _e362_.z, _e362_.w);
    metal::float4 _e396_ = metal::float4(_e390_.x, _e384_.y, _e390_.z, _e390_.w);
    C1_ = metal::float4(_e396_.x, _e396_.y, _e384_.z, _e396_.w);
    if (_e360_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e108_), (_buffer_sizes.size3 - 0 - 4) / 4)] = _e360_;
    }
    return;
}

struct main_Input {
    metal::float4 O [[user(loc0), center_perspective]];
    uint B0_ [[user(loc1), flat]];
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
, metal::float4 gl_FragCoord [[position]]
, metal::texture2d<float, metal::access::sample> XC [[texture(1)]]
, metal::sampler aa [[sampler(2)]]
, device Je const& AD [[buffer(2)]]
, device h0Bd& h0_ [[buffer(5)]]
, device Ke const& RB [[buffer(3)]]
, metal::texture2d<float, metal::access::sample> KD [[texture(0)]]
, metal::sampler Mb [[sampler(1)]]
, constant CC& n [[buffer(0)]]
, device q4Bd_1_& q4_ [[buffer(6)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 O_1_ = {};
    uint B0_1_ = {};
    metal::float4 C1_ = {};
    const auto O = varyings.O;
    const auto B0_ = varyings.B0_;
    gl_FragCoord_1_ = gl_FragCoord;
    O_1_ = O;
    B0_1_ = B0_;
    main_1_(XC, aa, AD, h0_, RB, gl_FragCoord_1_, KD, Mb, n, O_1_, B0_1_, q4_, C1_, _buffer_sizes);
    metal::float4 _e7_ = C1_;
    return main_Output { _e7_ };
}
