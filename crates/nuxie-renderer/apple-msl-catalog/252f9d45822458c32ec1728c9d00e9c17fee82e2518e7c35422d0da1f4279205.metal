// language: metal3.1
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size0;
    uint size1;
    uint size2;
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
    thread metal::float4& L0_1_,
    device q4Bd& q4_,
    thread uint& w3_1_,
    thread float& H1_1_,
    thread metal::float4& C1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    float phi_1248_ = {};
    bool phi_846_ = {};
    float phi_1195_ = {};
    float phi_1194_ = {};
    float phi_1196_ = {};
    float phi_1199_ = {};
    float phi_1198_ = {};
    bool phi_883_ = {};
    float phi_1201_ = {};
    uint phi_1228_ = {};
    float phi_1200_ = {};
    uint phi_1227_ = {};
    metal::float4 phi_1225_ = {};
    bool phi_635_ = {};
    uint phi_1239_ = {};
    float phi_1254_ = {};
    float phi_1255_ = {};
    metal::float3 phi_1276_ = {};
    bool local = {};
    bool local_1 = {};
    metal::float4 _e57_ = gl_FragCoord_1_;
    metal::float2 _e58_ = _e57_.xy;
    metal::uint2 _e61_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e58_)));
    uint _e63_ = n.m6_;
    int _e92_ = as_type<int>(((((_e61_.y >> as_type<uint>(5u)) * (((_e63_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e61_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e61_.x & 28u) << as_type<uint>(5u)) + ((_e61_.y & 28u) << as_type<uint>(2)))) + (((_e61_.y & 3u) << as_type<uint>(2)) + (_e61_.x & 3u)));
    metal::float2 _e93_ = X1_1_;
    metal::float4 _e94_ = IC.sample(S5_, _e93_);
    phi_1248_ = 1.0;
    if (Zg) {
        metal::float4 _e95_ = L0_1_;
        metal::float2 _e98_ = metal::min(_e95_.xy, _e95_.zw);
        phi_1248_ = metal::clamp(metal::min(_e98_.x, _e98_.y), 0.0, 1.0);
    }
    float _e104_ = phi_1248_;
    uint _e107_ = q4_.c2_[metal::min(unsigned(_e92_), (_buffer_sizes.size11 - 0 - 4) / 4)];
    uint _e109_ = _e107_ >> as_type<uint>(17u);
    float _e113_ = (static_cast<float>(_e107_ & 131071u) * 0.00048828125) + -32.0;
    metal::uint2 _e116_ = AD.c2_[metal::min(unsigned(_e109_), (_buffer_sizes.size0 - 0 - 8) / 8)];
    phi_1194_ = _e113_;
    if ((_e116_.x & 768u) != 0u) {
        float _e120_ = metal::abs(_e113_);
        phi_846_ = ch;
        if (ch) {
            phi_846_ = (_e116_.x & 512u) != 0u;
        }
        bool _e124_ = phi_846_;
        phi_1195_ = _e120_;
        if (_e124_) {
            phi_1195_ = 1.0 - metal::abs((metal::fract(_e120_ * 0.5) * 2.0) + -1.0);
        }
        float _e132_ = phi_1195_;
        phi_1194_ = _e132_;
    }
    float _e134_ = phi_1194_;
    float _e135_ = metal::clamp(_e134_, 0.0, 1.0);
    phi_1198_ = _e135_;
    if (Yg) {
        uint _e137_ = _e116_.x >> as_type<uint>(16u);
        phi_1199_ = _e135_;
        if (_e137_ != 0u) {
            uint _e141_ = h0_.c2_[metal::min(unsigned(_e92_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            if (_e137_ == (_e141_ >> as_type<uint>(16))) {
                phi_1196_ = metal::min(_e135_, float2(as_type<half2>(_e141_)).x);
            } else {
                phi_1196_ = 0.0;
            }
            float _e149_ = phi_1196_;
            phi_1199_ = _e149_;
        }
        float _e151_ = phi_1199_;
        phi_1198_ = _e151_;
    }
    float _e153_ = phi_1198_;
    phi_883_ = Zg;
    if (Zg) {
        phi_883_ = (_e116_.x & 1024u) != 0u;
    }
    bool _e157_ = phi_883_;
    phi_1201_ = _e153_;
    if (_e157_) {
        uint _e158_ = _e109_ * 4u;
        metal::float4 _e162_ = RB.c2_[metal::min(unsigned(_e158_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e173_ = RB.c2_[metal::min(unsigned(_e158_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e178_ = _e173_.zw;
        metal::float2 _e180_ = (metal::abs((metal::float2x2(metal::float2(_e162_.x, _e162_.y), metal::float2(_e162_.z, _e162_.w)) * _e58_) + _e173_.xy) * _e178_) - _e178_;
        phi_1201_ = metal::min(_e153_, metal::clamp(metal::min(_e180_.x, _e180_.y) + 0.5, 0.0, 1.0));
    }
    float _e188_ = phi_1201_;
    uint _e189_ = _e116_.x & 15u;
    if (_e189_ <= 1u) {
        if (Yg) {
            local = _e189_ == 0u;
        } else {
            local = false;
        }
        bool _e194_ = local;
        phi_1228_ = 0u;
        if (_e194_) {
            phi_1228_ = _e116_.y | as_type<uint>(half2(metal::float2(_e188_, 0.0)));
        }
        uint _e199_ = phi_1228_;
        phi_1227_ = _e199_;
        phi_1225_ = metal::select(metal::unpack_unorm4x8_to_float(_e116_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e194_));
    } else {
        uint _e202_ = _e109_ * 4u;
        metal::float4 _e205_ = RB.c2_[metal::min(unsigned(_e202_), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e216_ = RB.c2_[metal::min(unsigned(_e202_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e219_ = (metal::float2x2(metal::float2(_e205_.x, _e205_.y), metal::float2(_e205_.z, _e205_.w)) * _e58_) + _e216_.xy;
        if (_e189_ == 2u) {
            phi_1200_ = _e219_.x;
        } else {
            phi_1200_ = metal::length(_e219_);
        }
        float _e224_ = phi_1200_;
        metal::float4 _e233_ = KD.sample(Mb, metal::float2((metal::clamp(_e224_, 0.0, 1.0) * _e216_.z) + _e216_.w, as_type<float>(_e116_.y)), metal::level(0.0));
        phi_1227_ = 0u;
        phi_1225_ = _e233_;
    }
    uint _e235_ = phi_1227_;
    metal::float4 _e237_ = phi_1225_;
    float _e239_ = _e237_.w * _e188_;
    metal::float3 _e241_ = _e237_.xyz * _e239_;
    phi_635_ = Yg;
    if (Yg) {
        uint _e246_ = w3_1_;
        phi_635_ = _e246_ != 0u;
    }
    bool _e249_ = phi_635_;
    phi_1255_ = _e104_;
    if (_e249_) {
        if (_e235_ != 0u) {
            phi_1239_ = _e235_;
        } else {
            uint _e253_ = h0_.c2_[metal::min(unsigned(_e92_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            phi_1239_ = _e253_;
        }
        uint _e255_ = phi_1239_;
        uint _e256_ = w3_1_;
        if (_e256_ == (_e255_ >> as_type<uint>(16))) {
            phi_1254_ = metal::min(_e104_, float2(as_type<half2>(_e255_)).x);
        } else {
            phi_1254_ = 0.0;
        }
        float _e264_ = phi_1254_;
        phi_1255_ = _e264_;
    }
    float _e266_ = phi_1255_;
    float _e267_ = H1_1_;
    metal::float4 _e269_ = _e94_ * (_e266_ * _e267_);
    metal::float4 _e273_ = (metal::float4(_e241_.x, _e241_.y, _e241_.z, _e239_) * (1.0 - _e269_.w)) + _e269_;
    metal::float3 _e274_ = _e273_.xyz;
    float _e277_ = n.z3_;
    float _e279_ = n.A3_;
    if (fh) {
        local_1 = _e273_.w != 0.0;
    } else {
        local_1 = false;
    }
    bool _e336 = local_1;
    if (_e336) {
        phi_1276_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e57_.x) + (0.00583715 * _e57_.y))) * _e277_) + _e279_) + _e274_;
    } else {
        phi_1276_ = _e274_;
    }
    metal::float3 _e295_ = phi_1276_;
    metal::float4 _e301_ = metal::float4(_e295_.x, _e273_.y, _e273_.z, _e273_.w);
    metal::float4 _e307_ = metal::float4(_e301_.x, _e295_.y, _e301_.z, _e301_.w);
    C1_ = metal::float4(_e307_.x, _e307_.y, _e295_.z, _e307_.w);
    if (_e235_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e92_), (_buffer_sizes.size1 - 0 - 4) / 4)] = _e235_;
    }
    q4_.c2_[metal::min(unsigned(_e92_), (_buffer_sizes.size11 - 0 - 4) / 4)] = 65536u;
    return;
}

struct main_Input {
    metal::float2 X1_ [[user(loc0), center_perspective]];
    metal::float4 L0_ [[user(loc1), center_perspective]];
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
    metal::float4 L0_1_ = {};
    uint w3_1_ = {};
    float H1_1_ = {};
    metal::float4 C1_ = {};
    uint A1_1_ = {};
    const auto X1_ = varyings.X1_;
    const auto L0_ = varyings.L0_;
    const auto w3_ = varyings.w3_;
    const auto H1_ = varyings.H1_;
    const auto A1_ = varyings.A1_;
    gl_FragCoord_1_ = gl_FragCoord;
    X1_1_ = X1_;
    L0_1_ = L0_;
    w3_1_ = w3_;
    H1_1_ = H1_;
    A1_1_ = A1_;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, n, IC, S5_, X1_1_, L0_1_, q4_, w3_1_, H1_1_, C1_, _buffer_sizes);
    metal::float4 _e13_ = C1_;
    return main_Output { _e13_ };
}
