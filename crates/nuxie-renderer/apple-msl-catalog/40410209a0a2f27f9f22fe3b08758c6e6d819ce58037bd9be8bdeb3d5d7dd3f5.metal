// language: metal3.1
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size0;
    uint size1;
    uint size2;
    uint size6;
    uint size8;
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
struct j0Bd {
    type_3 c2_;
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
constant bool eh = true;
constant bool ch = false;
constant bool Yg = true;
constant bool Zg = true;
constant bool ah = true;

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
    device j0Bd const& j0_,
    constant CC& n,
    device q4Bd const& q4_,
    thread metal::float4& C1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    metal::float3 local = {};
    metal::float3 local_1_ = {};
    metal::float3 local_2_ = {};
    bool phi_1234_ = {};
    float phi_3140_ = {};
    float phi_3139_ = {};
    float phi_3141_ = {};
    float phi_3144_ = {};
    float phi_3143_ = {};
    bool phi_1271_ = {};
    float phi_3157_ = {};
    float phi_3145_ = {};
    metal::float4 phi_3159_ = {};
    bool phi_1384_ = {};
    uint phi_3163_ = {};
    bool phi_1393_ = {};
    float phi_3177_ = {};
    metal::float4 phi_3632_ = {};
    int phi_3572_ = {};
    metal::float4 phi_3780_ = {};
    metal::float4 phi_3781_ = {};
    bool local_1 = {};
    metal::float4 _e68_ = gl_FragCoord_1_;
    metal::float2 _e69_ = _e68_.xy;
    metal::uint2 _e72_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e69_)));
    uint _e74_ = n.m6_;
    int _e103_ = as_type<int>(((((_e72_.y >> as_type<uint>(5u)) * (((_e74_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e72_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e72_.x & 28u) << as_type<uint>(5u)) + ((_e72_.y & 28u) << as_type<uint>(2)))) + (((_e72_.y & 3u) << as_type<uint>(2)) + (_e72_.x & 3u)));
    uint _e106_ = q4_.c2_[metal::min(unsigned(_e103_), (_buffer_sizes.size8 - 0 - 4) / 4)];
    float _e110_ = (static_cast<float>(_e106_ & 131071u) * 0.00048828125) + -32.0;
    uint _e112_ = _e106_ >> as_type<uint>(17u);
    metal::uint2 _e115_ = AD.c2_[metal::min(unsigned(_e112_), (_buffer_sizes.size0 - 0 - 8) / 8)];
    phi_3139_ = _e110_;
    if ((_e115_.x & 768u) != 0u) {
        float _e119_ = metal::abs(_e110_);
        phi_1234_ = ch;
        if (ch) {
            phi_1234_ = (_e115_.x & 512u) != 0u;
        }
        bool _e123_ = phi_1234_;
        phi_3140_ = _e119_;
        if (_e123_) {
            phi_3140_ = 1.0 - metal::abs((metal::fract(_e119_ * 0.5) * 2.0) + -1.0);
        }
        float _e131_ = phi_3140_;
        phi_3139_ = _e131_;
    }
    float _e133_ = phi_3139_;
    float _e134_ = metal::clamp(_e133_, 0.0, 1.0);
    phi_3143_ = _e134_;
    if (Yg) {
        uint _e136_ = _e115_.x >> as_type<uint>(16u);
        phi_3144_ = _e134_;
        if (_e136_ != 0u) {
            uint _e140_ = h0_.c2_[metal::min(unsigned(_e103_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            if (_e136_ == (_e140_ >> as_type<uint>(16))) {
                phi_3141_ = metal::min(_e134_, float2(as_type<half2>(_e140_)).x);
            } else {
                phi_3141_ = 0.0;
            }
            float _e148_ = phi_3141_;
            phi_3144_ = _e148_;
        }
        float _e150_ = phi_3144_;
        phi_3143_ = _e150_;
    }
    float _e152_ = phi_3143_;
    phi_1271_ = Zg;
    if (Zg) {
        phi_1271_ = (_e115_.x & 1024u) != 0u;
    }
    bool _e156_ = phi_1271_;
    phi_3157_ = _e152_;
    if (_e156_) {
        uint _e157_ = _e112_ * 4u;
        metal::float4 _e161_ = RB.c2_[metal::min(unsigned(_e157_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e172_ = RB.c2_[metal::min(unsigned(_e157_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e177_ = _e172_.zw;
        metal::float2 _e179_ = (metal::abs((metal::float2x2(metal::float2(_e161_.x, _e161_.y), metal::float2(_e161_.z, _e161_.w)) * _e69_) + _e172_.xy) * _e177_) - _e177_;
        phi_3157_ = metal::min(_e152_, metal::clamp(metal::min(_e179_.x, _e179_.y) + 0.5, 0.0, 1.0));
    }
    float _e187_ = phi_3157_;
    uint _e188_ = _e115_.x & 15u;
    if (_e188_ <= 1u) {
        if (Yg) {
            local_1 = _e188_ == 0u;
        } else {
            local_1 = false;
        }
        bool _e209 = local_1;
        phi_3159_ = metal::select(metal::unpack_unorm4x8_to_float(_e115_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e209));
    } else {
        uint _e196_ = _e112_ * 4u;
        metal::float4 _e199_ = RB.c2_[metal::min(unsigned(_e196_), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e210_ = RB.c2_[metal::min(unsigned(_e196_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e213_ = (metal::float2x2(metal::float2(_e199_.x, _e199_.y), metal::float2(_e199_.z, _e199_.w)) * _e69_) + _e210_.xy;
        if (_e188_ == 2u) {
            phi_3145_ = _e213_.x;
        } else {
            phi_3145_ = metal::length(_e213_);
        }
        float _e218_ = phi_3145_;
        metal::float4 _e227_ = KD.sample(Mb, metal::float2((metal::clamp(_e218_, 0.0, 1.0) * _e210_.z) + _e210_.w, as_type<float>(_e115_.y)), metal::level(0.0));
        phi_3159_ = _e227_;
    }
    metal::float4 _e229_ = phi_3159_;
    float _e231_ = _e229_.w * _e187_;
    metal::float4 _e236_ = metal::float4(_e229_.x, _e229_.y, _e229_.z, _e231_);
    phi_1384_ = ah;
    if (ah) {
        phi_1384_ = _e231_ != 0.0;
    }
    bool _e239_ = phi_1384_;
    phi_3163_ = uint {};
    phi_1393_ = _e239_;
    if (_e239_) {
        uint _e242_ = (_e115_.x >> as_type<uint>(4)) & 15u;
        phi_3163_ = _e242_;
        phi_1393_ = _e242_ != 0u;
    }
    uint _e245_ = phi_3163_;
    bool _e247_ = phi_1393_;
    phi_3780_ = _e236_;
    if (_e247_) {
        uint _e250_ = j0_.c2_[metal::min(unsigned(_e103_), (_buffer_sizes.size6 - 0 - 4) / 4)];
        metal::float4 _e251_ = metal::unpack_unorm4x8_to_float(_e250_);
        metal::float3 _e252_ = _e236_.xyz;
        local_2_ = _e252_;
        metal::float3 _e253_ = _e251_.xyz;
        if (_e251_.w != 0.0) {
            phi_3177_ = 1.0 / _e251_.w;
        } else {
            phi_3177_ = 0.0;
        }
        float _e258_ = phi_3177_;
        metal::float3 _e259_ = _e253_ * _e258_;
        local = _e259_;
        switch(as_type<int>(_e245_)) {
            case 11: {
                metal::float3 _e261_ = local_2_;
                local_1_ = _e261_ * _e259_;
                break;
            }
            case 1: {
                metal::float3 _e263_ = local_2_;
                local_1_ = (_e263_ + _e259_) - (_e263_ * _e259_);
                break;
            }
            case 2: {
                metal::float3 _e267_ = local_2_;
                metal::float3 _e268_ = _e267_ * _e259_;
                local_1_ = metal::select(_e268_, ((_e267_ + _e259_) - _e268_) - metal::float3(0.5, 0.5, 0.5), _e259_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 3: {
                metal::float3 _e275_ = local_2_;
                local_1_ = metal::min(_e275_, _e259_);
                break;
            }
            case 4: {
                metal::float3 _e277_ = local_2_;
                local_1_ = metal::max(_e277_, _e259_);
                break;
            }
            case 5: {
                metal::float3 _e280_ = metal::clamp(_e253_, metal::float3(0.0, 0.0, 0.0), _e251_.www);
                metal::float4 _e286_ = metal::float4(_e280_.x, float {}, float {}, float {});
                metal::float4 _e292_ = metal::float4(_e286_.x, _e280_.y, _e286_.z, _e286_.w);
                metal::float3 _e299_ = local_2_;
                metal::float3 _e302_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e299_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e251_.w;
                metal::float3 _e303_ = metal::float4(_e292_.x, _e292_.y, _e280_.z, _e292_.w).xyz;
                local_1_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e303_ / _e302_), metal::sign(_e303_), _e302_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 6: {
                metal::float3 _e309_ = local_2_;
                local_2_ = metal::clamp(_e309_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                metal::float3 _e312_ = metal::clamp(_e253_, metal::float3(0.0, 0.0, 0.0), _e251_.www);
                metal::float4 _e318_ = metal::float4(_e312_.x, _e251_.y, _e251_.z, _e251_.w);
                metal::float4 _e324_ = metal::float4(_e318_.x, _e312_.y, _e318_.z, _e318_.w);
                phi_3632_ = metal::float4(_e324_.x, _e324_.y, _e312_.z, _e324_.w);
                if (_e251_.w == 0.0) {
                    phi_3632_ = metal::float4(_e312_.x, _e312_.y, _e312_.z, 1.0);
                }
                metal::float4 _e334_ = phi_3632_;
                metal::float3 _e338_ = metal::float3(_e334_.w) - _e334_.xyz;
                metal::float3 _e339_ = local_2_;
                local_1_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e338_ / (_e339_ * _e334_.w)), metal::sign(_e338_), _e339_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 7: {
                metal::float3 _e347_ = local_2_;
                metal::float3 _e348_ = _e347_ * _e259_;
                local_1_ = metal::select(_e348_, ((_e347_ + _e259_) - _e348_) - metal::float3(0.5, 0.5, 0.5), _e347_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 8: {
                phi_3572_ = 0;
                uint2 loop_bound = uint2(4294967295u);
                bool loop_init = true;
                while(true) {
                    if (metal::all(loop_bound == uint2(0u))) { break; }
                    loop_bound -= uint2(loop_bound.y == 0u, 1u);
                    if (!loop_init) {
                        phi_3572_ = as_type<int>(as_type<uint>(phi_3572_) + as_type<uint>(1));
                    }
                    loop_init = false;
                    int _e356_ = phi_3572_;
                    if (_e356_ < 3) {
                        float _e359_ = local_2_[metal::min(unsigned(_e356_), 2u)];
                        if (_e359_ <= 0.5) {
                            float _e362_ = local[metal::min(unsigned(_e356_), 2u)];
                            local_1_[metal::min(unsigned(_e356_), 2u)] = 1.0 - _e362_;
                        } else {
                            float _e366_ = local[metal::min(unsigned(_e356_), 2u)];
                            if (_e366_ <= 0.25) {
                                float _e368_ = local[metal::min(unsigned(_e356_), 2u)];
                                float _e371_ = local[metal::min(unsigned(_e356_), 2u)];
                                local_1_[metal::min(unsigned(_e356_), 2u)] = (((16.0 * _e368_) - 12.0) * _e371_) + 3.0;
                            } else {
                                float _e375_ = local[metal::min(unsigned(_e356_), 2u)];
                                local_1_[metal::min(unsigned(_e356_), 2u)] = metal::rsqrt(_e375_) - 1.0;
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                }
                metal::float3 _e380_ = local_2_;
                metal::float3 _e384_ = local_1_;
                local_1_ = _e259_ + ((_e259_ * ((_e380_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e384_);
                break;
            }
            case 9: {
                metal::float3 _e387_ = local_2_;
                local_1_ = metal::abs(_e259_ - _e387_);
                break;
            }
            case 10: {
                metal::float3 _e390_ = local_2_;
                local_1_ = (_e390_ + _e259_) - ((_e390_ * 2.0) * _e259_);
                break;
            }
            case 12: {
                if (eh) {
                    metal::float3 _e395_ = local_2_;
                    metal::float3 _e396_ = metal::clamp(_e395_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e396_;
                    metal::float3 _e411_ = _e396_ - metal::float3(metal::min(metal::min(_e396_.x, _e396_.y), _e396_.z));
                    metal::float3 _e419_ = _e411_ * ((metal::max(metal::max(_e259_.x, _e259_.y), _e259_.z) - metal::min(metal::min(_e259_.x, _e259_.y), _e259_.z)) / metal::max(0.000062, metal::max(metal::max(_e411_.x, _e411_.y), _e411_.z)));
                    float _e420_ = metal::dot(_e259_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e423_ = _e419_ - metal::float3(metal::dot(_e419_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e436_ = metal::float2(_e420_, 1.0 - _e420_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e423_.x, _e423_.y), _e423_.z)), metal::max(metal::max(_e423_.x, _e423_.y), _e423_.z)));
                    local_1_ = (_e423_ * metal::min(1.0, metal::min(_e436_.x, _e436_.y))) + metal::float3(_e420_);
                }
                break;
            }
            case 13: {
                if (eh) {
                    metal::float3 _e444_ = local_2_;
                    metal::float3 _e445_ = metal::clamp(_e444_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e445_;
                    metal::float3 _e460_ = _e259_ - metal::float3(metal::min(metal::min(_e259_.x, _e259_.y), _e259_.z));
                    metal::float3 _e468_ = _e460_ * ((metal::max(metal::max(_e445_.x, _e445_.y), _e445_.z) - metal::min(metal::min(_e445_.x, _e445_.y), _e445_.z)) / metal::max(0.000062, metal::max(metal::max(_e460_.x, _e460_.y), _e460_.z)));
                    float _e469_ = metal::dot(_e259_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e472_ = _e468_ - metal::float3(metal::dot(_e468_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e485_ = metal::float2(_e469_, 1.0 - _e469_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e472_.x, _e472_.y), _e472_.z)), metal::max(metal::max(_e472_.x, _e472_.y), _e472_.z)));
                    local_1_ = (_e472_ * metal::min(1.0, metal::min(_e485_.x, _e485_.y))) + metal::float3(_e469_);
                }
                break;
            }
            case 14: {
                if (eh) {
                    metal::float3 _e493_ = local_2_;
                    metal::float3 _e494_ = metal::clamp(_e493_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e494_;
                    float _e495_ = metal::dot(_e259_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e498_ = _e494_ - metal::float3(metal::dot(_e494_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e511_ = metal::float2(_e495_, 1.0 - _e495_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e498_.x, _e498_.y), _e498_.z)), metal::max(metal::max(_e498_.x, _e498_.y), _e498_.z)));
                    local_1_ = (_e498_ * metal::min(1.0, metal::min(_e511_.x, _e511_.y))) + metal::float3(_e495_);
                }
                break;
            }
            case 15: {
                if (eh) {
                    metal::float3 _e519_ = local_2_;
                    metal::float3 _e520_ = metal::clamp(_e519_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e520_;
                    float _e521_ = metal::dot(_e520_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e524_ = _e259_ - metal::float3(metal::dot(_e259_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e537_ = metal::float2(_e521_, 1.0 - _e521_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e524_.x, _e524_.y), _e524_.z)), metal::max(metal::max(_e524_.x, _e524_.y), _e524_.z)));
                    local_1_ = (_e524_ * metal::min(1.0, metal::min(_e537_.x, _e537_.y))) + metal::float3(_e521_);
                }
                break;
            }
            default: {
                break;
            }
        }
        metal::float3 _e545_ = local_1_;
        metal::float3 _e547_ = metal::mix(_e252_, _e545_, metal::float3(_e251_.w));
        phi_3780_ = metal::float4(_e547_.x, _e547_.y, _e547_.z, _e231_);
    }
    metal::float4 _e553_ = phi_3780_;
    metal::float3 _e556_ = _e553_.xyz * _e553_.w;
    metal::float4 _e562_ = metal::float4(_e556_.x, _e553_.y, _e553_.z, _e553_.w);
    metal::float4 _e568_ = metal::float4(_e562_.x, _e556_.y, _e562_.z, _e562_.w);
    metal::float4 _e574_ = metal::float4(_e568_.x, _e568_.y, _e556_.z, _e568_.w);
    float _e575_ = 1.0 - _e553_.w;
    phi_3781_ = _e574_;
    if (_e575_ != 0.0) {
        uint _e579_ = j0_.c2_[metal::min(unsigned(_e103_), (_buffer_sizes.size6 - 0 - 4) / 4)];
        phi_3781_ = _e574_ + (metal::unpack_unorm4x8_to_float(_e579_) * _e575_);
    }
    metal::float4 _e584_ = phi_3781_;
    C1_ = _e584_;
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
, device j0Bd const& j0_ [[buffer(4)]]
, constant CC& n [[buffer(0)]]
, device q4Bd const& q4_ [[buffer(6)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 C1_ = {};
    gl_FragCoord_1_ = gl_FragCoord;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, j0_, n, q4_, C1_, _buffer_sizes);
    metal::float4 _e3_ = C1_;
    return main_Output { _e3_ };
}
