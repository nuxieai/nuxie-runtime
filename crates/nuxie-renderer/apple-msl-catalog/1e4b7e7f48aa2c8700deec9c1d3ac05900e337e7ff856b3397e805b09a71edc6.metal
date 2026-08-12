// language: metal4.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size0;
    uint size1;
    uint size2;
    uint size6;
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
constant bool fh = true;
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
    device h0Bd& h0_,
    device Ke const& RB,
    thread metal::float4& gl_FragCoord_1_,
    metal::texture2d<float, metal::access::sample> KD,
    metal::sampler Mb,
    device j0Bd& j0_,
    constant CC& n,
    metal::texture2d<float, metal::access::sample> IC,
    metal::sampler S5_,
    thread metal::float2& X1_1_,
    thread metal::float4& L0_1_,
    device q4Bd& q4_,
    thread uint& w3_1_,
    thread uint& A1_1_,
    thread float& H1_1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    metal::float3 local = {};
    metal::float3 local_1_ = {};
    metal::float3 local_2_ = {};
    metal::float3 local_3_ = {};
    metal::float3 local_4_ = {};
    metal::float3 local_5_ = {};
    float phi_5918_ = {};
    bool phi_1529_ = {};
    float phi_5188_ = {};
    float phi_5187_ = {};
    float phi_5189_ = {};
    float phi_5192_ = {};
    float phi_5191_ = {};
    bool phi_1566_ = {};
    float phi_5194_ = {};
    uint phi_5885_ = {};
    float phi_5193_ = {};
    uint phi_5884_ = {};
    metal::float4 phi_5218_ = {};
    bool phi_1685_ = {};
    uint phi_5222_ = {};
    bool phi_1694_ = {};
    float phi_5237_ = {};
    metal::float4 phi_5723_ = {};
    int phi_5659_ = {};
    metal::float4 phi_5880_ = {};
    bool phi_1270_ = {};
    uint phi_5906_ = {};
    float phi_5934_ = {};
    float phi_7313_ = {};
    bool phi_1298_ = {};
    float phi_5970_ = {};
    float phi_5971_ = {};
    metal::float4 phi_7000_ = {};
    int phi_6868_ = {};
    metal::float4 phi_7325_ = {};
    metal::float3 phi_7338_ = {};
    metal::float4 phi_7340_ = {};
    bool local_1 = {};
    bool local_2 = {};
    metal::float4 _e82_ = gl_FragCoord_1_;
    metal::float2 _e83_ = _e82_.xy;
    metal::uint2 _e86_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e83_)));
    uint _e88_ = n.m6_;
    int _e117_ = as_type<int>(((((_e86_.y >> as_type<uint>(5u)) * (((_e88_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e86_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e86_.x & 28u) << as_type<uint>(5u)) + ((_e86_.y & 28u) << as_type<uint>(2)))) + (((_e86_.y & 3u) << as_type<uint>(2)) + (_e86_.x & 3u)));
    metal::float2 _e118_ = X1_1_;
    metal::float4 _e119_ = IC.sample(S5_, _e118_);
    phi_5918_ = 1.0;
    if (Zg) {
        metal::float4 _e120_ = L0_1_;
        metal::float2 _e123_ = metal::min(_e120_.xy, _e120_.zw);
        phi_5918_ = metal::clamp(metal::min(_e123_.x, _e123_.y), 0.0, 1.0);
    }
    float _e129_ = phi_5918_;
    uint _e132_ = q4_.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size12 - 0 - 4) / 4)];
    uint _e134_ = _e132_ >> as_type<uint>(17u);
    float _e138_ = (static_cast<float>(_e132_ & 131071u) * 0.00048828125) + -32.0;
    metal::uint2 _e141_ = AD.c2_[metal::min(unsigned(_e134_), (_buffer_sizes.size0 - 0 - 8) / 8)];
    phi_5187_ = _e138_;
    if ((_e141_.x & 768u) != 0u) {
        float _e145_ = metal::abs(_e138_);
        phi_1529_ = ch;
        if (ch) {
            phi_1529_ = (_e141_.x & 512u) != 0u;
        }
        bool _e149_ = phi_1529_;
        phi_5188_ = _e145_;
        if (_e149_) {
            phi_5188_ = 1.0 - metal::abs((metal::fract(_e145_ * 0.5) * 2.0) + -1.0);
        }
        float _e157_ = phi_5188_;
        phi_5187_ = _e157_;
    }
    float _e159_ = phi_5187_;
    float _e160_ = metal::clamp(_e159_, 0.0, 1.0);
    phi_5191_ = _e160_;
    if (Yg) {
        uint _e162_ = _e141_.x >> as_type<uint>(16u);
        phi_5192_ = _e160_;
        if (_e162_ != 0u) {
            uint _e166_ = h0_.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            if (_e162_ == (_e166_ >> as_type<uint>(16))) {
                phi_5189_ = metal::min(_e160_, float2(as_type<half2>(_e166_)).x);
            } else {
                phi_5189_ = 0.0;
            }
            float _e174_ = phi_5189_;
            phi_5192_ = _e174_;
        }
        float _e176_ = phi_5192_;
        phi_5191_ = _e176_;
    }
    float _e178_ = phi_5191_;
    phi_1566_ = Zg;
    if (Zg) {
        phi_1566_ = (_e141_.x & 1024u) != 0u;
    }
    bool _e182_ = phi_1566_;
    phi_5194_ = _e178_;
    if (_e182_) {
        uint _e183_ = _e134_ * 4u;
        metal::float4 _e187_ = RB.c2_[metal::min(unsigned(_e183_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e198_ = RB.c2_[metal::min(unsigned(_e183_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e203_ = _e198_.zw;
        metal::float2 _e205_ = (metal::abs((metal::float2x2(metal::float2(_e187_.x, _e187_.y), metal::float2(_e187_.z, _e187_.w)) * _e83_) + _e198_.xy) * _e203_) - _e203_;
        phi_5194_ = metal::min(_e178_, metal::clamp(metal::min(_e205_.x, _e205_.y) + 0.5, 0.0, 1.0));
    }
    float _e213_ = phi_5194_;
    uint _e214_ = _e141_.x & 15u;
    if (_e214_ <= 1u) {
        if (Yg) {
            local_1 = _e214_ == 0u;
        } else {
            local_1 = false;
        }
        bool _e219_ = local_1;
        phi_5885_ = 0u;
        if (_e219_) {
            phi_5885_ = _e141_.y | as_type<uint>(half2(metal::float2(_e213_, 0.0)));
        }
        uint _e224_ = phi_5885_;
        phi_5884_ = _e224_;
        phi_5218_ = metal::select(metal::unpack_unorm4x8_to_float(_e141_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e219_));
    } else {
        uint _e227_ = _e134_ * 4u;
        metal::float4 _e230_ = RB.c2_[metal::min(unsigned(_e227_), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e241_ = RB.c2_[metal::min(unsigned(_e227_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e244_ = (metal::float2x2(metal::float2(_e230_.x, _e230_.y), metal::float2(_e230_.z, _e230_.w)) * _e83_) + _e241_.xy;
        if (_e214_ == 2u) {
            phi_5193_ = _e244_.x;
        } else {
            phi_5193_ = metal::length(_e244_);
        }
        float _e249_ = phi_5193_;
        metal::float4 _e258_ = KD.sample(Mb, metal::float2((metal::clamp(_e249_, 0.0, 1.0) * _e241_.z) + _e241_.w, as_type<float>(_e141_.y)), metal::level(0.0));
        phi_5884_ = 0u;
        phi_5218_ = _e258_;
    }
    uint _e260_ = phi_5884_;
    metal::float4 _e262_ = phi_5218_;
    float _e264_ = _e262_.w * _e213_;
    metal::float4 _e269_ = metal::float4(_e262_.x, _e262_.y, _e262_.z, _e264_);
    phi_1685_ = ah;
    if (ah) {
        phi_1685_ = _e264_ != 0.0;
    }
    bool _e272_ = phi_1685_;
    phi_5222_ = uint {};
    phi_1694_ = _e272_;
    if (_e272_) {
        uint _e275_ = (_e141_.x >> as_type<uint>(4)) & 15u;
        phi_5222_ = _e275_;
        phi_1694_ = _e275_ != 0u;
    }
    uint _e278_ = phi_5222_;
    bool _e280_ = phi_1694_;
    phi_5880_ = _e269_;
    if (_e280_) {
        uint _e283_ = j0_.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size6 - 0 - 4) / 4)];
        metal::float4 _e284_ = metal::unpack_unorm4x8_to_float(_e283_);
        metal::float3 _e285_ = _e269_.xyz;
        local_5_ = _e285_;
        metal::float3 _e286_ = _e284_.xyz;
        if (_e284_.w != 0.0) {
            phi_5237_ = 1.0 / _e284_.w;
        } else {
            phi_5237_ = 0.0;
        }
        float _e291_ = phi_5237_;
        metal::float3 _e292_ = _e286_ * _e291_;
        local_3_ = _e292_;
        switch(as_type<int>(_e278_)) {
            case 11: {
                metal::float3 _e294_ = local_5_;
                local_4_ = _e294_ * _e292_;
                break;
            }
            case 1: {
                metal::float3 _e296_ = local_5_;
                local_4_ = (_e296_ + _e292_) - (_e296_ * _e292_);
                break;
            }
            case 2: {
                metal::float3 _e300_ = local_5_;
                metal::float3 _e301_ = _e300_ * _e292_;
                local_4_ = metal::select(_e301_, ((_e300_ + _e292_) - _e301_) - metal::float3(0.5, 0.5, 0.5), _e292_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 3: {
                metal::float3 _e308_ = local_5_;
                local_4_ = metal::min(_e308_, _e292_);
                break;
            }
            case 4: {
                metal::float3 _e310_ = local_5_;
                local_4_ = metal::max(_e310_, _e292_);
                break;
            }
            case 5: {
                metal::float3 _e313_ = metal::clamp(_e286_, metal::float3(0.0, 0.0, 0.0), _e284_.www);
                metal::float4 _e319_ = metal::float4(_e313_.x, float {}, float {}, float {});
                metal::float4 _e325_ = metal::float4(_e319_.x, _e313_.y, _e319_.z, _e319_.w);
                metal::float3 _e332_ = local_5_;
                metal::float3 _e335_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e332_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e284_.w;
                metal::float3 _e336_ = metal::float4(_e325_.x, _e325_.y, _e313_.z, _e325_.w).xyz;
                local_4_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e336_ / _e335_), metal::sign(_e336_), _e335_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 6: {
                metal::float3 _e342_ = local_5_;
                local_5_ = metal::clamp(_e342_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                metal::float3 _e345_ = metal::clamp(_e286_, metal::float3(0.0, 0.0, 0.0), _e284_.www);
                metal::float4 _e351_ = metal::float4(_e345_.x, _e284_.y, _e284_.z, _e284_.w);
                metal::float4 _e357_ = metal::float4(_e351_.x, _e345_.y, _e351_.z, _e351_.w);
                phi_5723_ = metal::float4(_e357_.x, _e357_.y, _e345_.z, _e357_.w);
                if (_e284_.w == 0.0) {
                    phi_5723_ = metal::float4(_e345_.x, _e345_.y, _e345_.z, 1.0);
                }
                metal::float4 _e367_ = phi_5723_;
                metal::float3 _e371_ = metal::float3(_e367_.w) - _e367_.xyz;
                metal::float3 _e372_ = local_5_;
                local_4_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e371_ / (_e372_ * _e367_.w)), metal::sign(_e371_), _e372_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 7: {
                metal::float3 _e380_ = local_5_;
                metal::float3 _e381_ = _e380_ * _e292_;
                local_4_ = metal::select(_e381_, ((_e380_ + _e292_) - _e381_) - metal::float3(0.5, 0.5, 0.5), _e380_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 8: {
                phi_5659_ = 0;
                uint2 loop_bound = uint2(4294967295u);
                bool loop_init = true;
                while(true) {
                    if (metal::all(loop_bound == uint2(0u))) { break; }
                    loop_bound -= uint2(loop_bound.y == 0u, 1u);
                    if (!loop_init) {
                        phi_5659_ = as_type<int>(as_type<uint>(phi_5659_) + as_type<uint>(1));
                    }
                    loop_init = false;
                    int _e389_ = phi_5659_;
                    if (_e389_ < 3) {
                        float _e392_ = local_5_[metal::min(unsigned(_e389_), 2u)];
                        if (_e392_ <= 0.5) {
                            float _e395_ = local_3_[metal::min(unsigned(_e389_), 2u)];
                            local_4_[metal::min(unsigned(_e389_), 2u)] = 1.0 - _e395_;
                        } else {
                            float _e399_ = local_3_[metal::min(unsigned(_e389_), 2u)];
                            if (_e399_ <= 0.25) {
                                float _e401_ = local_3_[metal::min(unsigned(_e389_), 2u)];
                                float _e404_ = local_3_[metal::min(unsigned(_e389_), 2u)];
                                local_4_[metal::min(unsigned(_e389_), 2u)] = (((16.0 * _e401_) - 12.0) * _e404_) + 3.0;
                            } else {
                                float _e408_ = local_3_[metal::min(unsigned(_e389_), 2u)];
                                local_4_[metal::min(unsigned(_e389_), 2u)] = metal::rsqrt(_e408_) - 1.0;
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                }
                metal::float3 _e413_ = local_5_;
                metal::float3 _e417_ = local_4_;
                local_4_ = _e292_ + ((_e292_ * ((_e413_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e417_);
                break;
            }
            case 9: {
                metal::float3 _e420_ = local_5_;
                local_4_ = metal::abs(_e292_ - _e420_);
                break;
            }
            case 10: {
                metal::float3 _e423_ = local_5_;
                local_4_ = (_e423_ + _e292_) - ((_e423_ * 2.0) * _e292_);
                break;
            }
            case 12: {
                if (eh) {
                    metal::float3 _e428_ = local_5_;
                    metal::float3 _e429_ = metal::clamp(_e428_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_5_ = _e429_;
                    metal::float3 _e444_ = _e429_ - metal::float3(metal::min(metal::min(_e429_.x, _e429_.y), _e429_.z));
                    metal::float3 _e452_ = _e444_ * ((metal::max(metal::max(_e292_.x, _e292_.y), _e292_.z) - metal::min(metal::min(_e292_.x, _e292_.y), _e292_.z)) / metal::max(0.000062, metal::max(metal::max(_e444_.x, _e444_.y), _e444_.z)));
                    float _e453_ = metal::dot(_e292_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e456_ = _e452_ - metal::float3(metal::dot(_e452_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e469_ = metal::float2(_e453_, 1.0 - _e453_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e456_.x, _e456_.y), _e456_.z)), metal::max(metal::max(_e456_.x, _e456_.y), _e456_.z)));
                    local_4_ = (_e456_ * metal::min(1.0, metal::min(_e469_.x, _e469_.y))) + metal::float3(_e453_);
                }
                break;
            }
            case 13: {
                if (eh) {
                    metal::float3 _e477_ = local_5_;
                    metal::float3 _e478_ = metal::clamp(_e477_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_5_ = _e478_;
                    metal::float3 _e493_ = _e292_ - metal::float3(metal::min(metal::min(_e292_.x, _e292_.y), _e292_.z));
                    metal::float3 _e501_ = _e493_ * ((metal::max(metal::max(_e478_.x, _e478_.y), _e478_.z) - metal::min(metal::min(_e478_.x, _e478_.y), _e478_.z)) / metal::max(0.000062, metal::max(metal::max(_e493_.x, _e493_.y), _e493_.z)));
                    float _e502_ = metal::dot(_e292_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e505_ = _e501_ - metal::float3(metal::dot(_e501_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e518_ = metal::float2(_e502_, 1.0 - _e502_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e505_.x, _e505_.y), _e505_.z)), metal::max(metal::max(_e505_.x, _e505_.y), _e505_.z)));
                    local_4_ = (_e505_ * metal::min(1.0, metal::min(_e518_.x, _e518_.y))) + metal::float3(_e502_);
                }
                break;
            }
            case 14: {
                if (eh) {
                    metal::float3 _e526_ = local_5_;
                    metal::float3 _e527_ = metal::clamp(_e526_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_5_ = _e527_;
                    float _e528_ = metal::dot(_e292_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e531_ = _e527_ - metal::float3(metal::dot(_e527_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e544_ = metal::float2(_e528_, 1.0 - _e528_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e531_.x, _e531_.y), _e531_.z)), metal::max(metal::max(_e531_.x, _e531_.y), _e531_.z)));
                    local_4_ = (_e531_ * metal::min(1.0, metal::min(_e544_.x, _e544_.y))) + metal::float3(_e528_);
                }
                break;
            }
            case 15: {
                if (eh) {
                    metal::float3 _e552_ = local_5_;
                    metal::float3 _e553_ = metal::clamp(_e552_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_5_ = _e553_;
                    float _e554_ = metal::dot(_e553_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e557_ = _e292_ - metal::float3(metal::dot(_e292_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e570_ = metal::float2(_e554_, 1.0 - _e554_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e557_.x, _e557_.y), _e557_.z)), metal::max(metal::max(_e557_.x, _e557_.y), _e557_.z)));
                    local_4_ = (_e557_ * metal::min(1.0, metal::min(_e570_.x, _e570_.y))) + metal::float3(_e554_);
                }
                break;
            }
            default: {
                break;
            }
        }
        metal::float3 _e578_ = local_4_;
        metal::float3 _e580_ = metal::mix(_e285_, _e578_, metal::float3(_e284_.w));
        phi_5880_ = metal::float4(_e580_.x, _e580_.y, _e580_.z, _e264_);
    }
    metal::float4 _e586_ = phi_5880_;
    metal::float3 _e589_ = _e586_.xyz * _e586_.w;
    metal::float4 _e595_ = metal::float4(_e589_.x, _e586_.y, _e586_.z, _e586_.w);
    metal::float4 _e601_ = metal::float4(_e595_.x, _e589_.y, _e595_.z, _e595_.w);
    metal::float4 _e607_ = metal::float4(_e601_.x, _e601_.y, _e589_.z, _e601_.w);
    phi_1270_ = Yg;
    if (Yg) {
        uint _e608_ = w3_1_;
        phi_1270_ = _e608_ != 0u;
    }
    bool _e611_ = phi_1270_;
    phi_7313_ = _e129_;
    if (_e611_) {
        if (_e260_ != 0u) {
            phi_5906_ = _e260_;
        } else {
            uint _e615_ = h0_.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            phi_5906_ = _e615_;
        }
        uint _e617_ = phi_5906_;
        uint _e618_ = w3_1_;
        if (_e618_ == (_e617_ >> as_type<uint>(16))) {
            phi_5934_ = metal::min(_e129_, float2(as_type<half2>(_e617_)).x);
        } else {
            phi_5934_ = 0.0;
        }
        float _e626_ = phi_5934_;
        phi_7313_ = _e626_;
    }
    float _e628_ = phi_7313_;
    phi_1298_ = ah;
    if (ah) {
        uint _e629_ = A1_1_;
        phi_1298_ = _e629_ != 0u;
    }
    bool _e632_ = phi_1298_;
    phi_7325_ = _e119_;
    if (_e632_) {
        uint _e635_ = j0_.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size6 - 0 - 4) / 4)];
        metal::float4 _e639_ = (metal::unpack_unorm4x8_to_float(_e635_) * (1.0 - _e586_.w)) + _e607_;
        if (_e119_.w != 0.0) {
            phi_5970_ = 1.0 / _e119_.w;
        } else {
            phi_5970_ = 0.0;
        }
        float _e645_ = phi_5970_;
        metal::float3 _e646_ = _e119_.xyz * _e645_;
        uint _e647_ = A1_1_;
        local_2_ = _e646_;
        metal::float3 _e648_ = _e639_.xyz;
        if (_e639_.w != 0.0) {
            phi_5971_ = 1.0 / _e639_.w;
        } else {
            phi_5971_ = 0.0;
        }
        float _e653_ = phi_5971_;
        metal::float3 _e654_ = _e648_ * _e653_;
        local = _e654_;
        switch(as_type<int>(_e647_)) {
            case 11: {
                metal::float3 _e656_ = local_2_;
                local_1_ = _e656_ * _e654_;
                break;
            }
            case 1: {
                metal::float3 _e658_ = local_2_;
                local_1_ = (_e658_ + _e654_) - (_e658_ * _e654_);
                break;
            }
            case 2: {
                metal::float3 _e662_ = local_2_;
                metal::float3 _e663_ = _e662_ * _e654_;
                local_1_ = metal::select(_e663_, ((_e662_ + _e654_) - _e663_) - metal::float3(0.5, 0.5, 0.5), _e654_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 3: {
                metal::float3 _e670_ = local_2_;
                local_1_ = metal::min(_e670_, _e654_);
                break;
            }
            case 4: {
                metal::float3 _e672_ = local_2_;
                local_1_ = metal::max(_e672_, _e654_);
                break;
            }
            case 5: {
                metal::float3 _e675_ = metal::clamp(_e648_, metal::float3(0.0, 0.0, 0.0), _e639_.www);
                metal::float4 _e681_ = metal::float4(_e675_.x, float {}, float {}, float {});
                metal::float4 _e687_ = metal::float4(_e681_.x, _e675_.y, _e681_.z, _e681_.w);
                metal::float3 _e694_ = local_2_;
                metal::float3 _e697_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e694_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e639_.w;
                metal::float3 _e698_ = metal::float4(_e687_.x, _e687_.y, _e675_.z, _e687_.w).xyz;
                local_1_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e698_ / _e697_), metal::sign(_e698_), _e697_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 6: {
                metal::float3 _e704_ = local_2_;
                local_2_ = metal::clamp(_e704_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                metal::float3 _e707_ = metal::clamp(_e648_, metal::float3(0.0, 0.0, 0.0), _e639_.www);
                metal::float4 _e713_ = metal::float4(_e707_.x, _e639_.y, _e639_.z, _e639_.w);
                metal::float4 _e719_ = metal::float4(_e713_.x, _e707_.y, _e713_.z, _e713_.w);
                phi_7000_ = metal::float4(_e719_.x, _e719_.y, _e707_.z, _e719_.w);
                if (_e639_.w == 0.0) {
                    phi_7000_ = metal::float4(_e707_.x, _e707_.y, _e707_.z, 1.0);
                }
                metal::float4 _e729_ = phi_7000_;
                metal::float3 _e733_ = metal::float3(_e729_.w) - _e729_.xyz;
                metal::float3 _e734_ = local_2_;
                local_1_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e733_ / (_e734_ * _e729_.w)), metal::sign(_e733_), _e734_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 7: {
                metal::float3 _e742_ = local_2_;
                metal::float3 _e743_ = _e742_ * _e654_;
                local_1_ = metal::select(_e743_, ((_e742_ + _e654_) - _e743_) - metal::float3(0.5, 0.5, 0.5), _e742_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 8: {
                phi_6868_ = 0;
                uint2 loop_bound_1 = uint2(4294967295u);
                bool loop_init_1 = true;
                while(true) {
                    if (metal::all(loop_bound_1 == uint2(0u))) { break; }
                    loop_bound_1 -= uint2(loop_bound_1.y == 0u, 1u);
                    if (!loop_init_1) {
                        phi_6868_ = as_type<int>(as_type<uint>(phi_6868_) + as_type<uint>(1));
                    }
                    loop_init_1 = false;
                    int _e751_ = phi_6868_;
                    if (_e751_ < 3) {
                        float _e754_ = local_2_[metal::min(unsigned(_e751_), 2u)];
                        if (_e754_ <= 0.5) {
                            float _e757_ = local[metal::min(unsigned(_e751_), 2u)];
                            local_1_[metal::min(unsigned(_e751_), 2u)] = 1.0 - _e757_;
                        } else {
                            float _e761_ = local[metal::min(unsigned(_e751_), 2u)];
                            if (_e761_ <= 0.25) {
                                float _e763_ = local[metal::min(unsigned(_e751_), 2u)];
                                float _e766_ = local[metal::min(unsigned(_e751_), 2u)];
                                local_1_[metal::min(unsigned(_e751_), 2u)] = (((16.0 * _e763_) - 12.0) * _e766_) + 3.0;
                            } else {
                                float _e770_ = local[metal::min(unsigned(_e751_), 2u)];
                                local_1_[metal::min(unsigned(_e751_), 2u)] = metal::rsqrt(_e770_) - 1.0;
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                }
                metal::float3 _e775_ = local_2_;
                metal::float3 _e779_ = local_1_;
                local_1_ = _e654_ + ((_e654_ * ((_e775_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e779_);
                break;
            }
            case 9: {
                metal::float3 _e782_ = local_2_;
                local_1_ = metal::abs(_e654_ - _e782_);
                break;
            }
            case 10: {
                metal::float3 _e785_ = local_2_;
                local_1_ = (_e785_ + _e654_) - ((_e785_ * 2.0) * _e654_);
                break;
            }
            case 12: {
                if (eh) {
                    metal::float3 _e790_ = local_2_;
                    metal::float3 _e791_ = metal::clamp(_e790_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e791_;
                    metal::float3 _e806_ = _e791_ - metal::float3(metal::min(metal::min(_e791_.x, _e791_.y), _e791_.z));
                    metal::float3 _e814_ = _e806_ * ((metal::max(metal::max(_e654_.x, _e654_.y), _e654_.z) - metal::min(metal::min(_e654_.x, _e654_.y), _e654_.z)) / metal::max(0.000062, metal::max(metal::max(_e806_.x, _e806_.y), _e806_.z)));
                    float _e815_ = metal::dot(_e654_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e818_ = _e814_ - metal::float3(metal::dot(_e814_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e831_ = metal::float2(_e815_, 1.0 - _e815_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e818_.x, _e818_.y), _e818_.z)), metal::max(metal::max(_e818_.x, _e818_.y), _e818_.z)));
                    local_1_ = (_e818_ * metal::min(1.0, metal::min(_e831_.x, _e831_.y))) + metal::float3(_e815_);
                }
                break;
            }
            case 13: {
                if (eh) {
                    metal::float3 _e839_ = local_2_;
                    metal::float3 _e840_ = metal::clamp(_e839_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e840_;
                    metal::float3 _e855_ = _e654_ - metal::float3(metal::min(metal::min(_e654_.x, _e654_.y), _e654_.z));
                    metal::float3 _e863_ = _e855_ * ((metal::max(metal::max(_e840_.x, _e840_.y), _e840_.z) - metal::min(metal::min(_e840_.x, _e840_.y), _e840_.z)) / metal::max(0.000062, metal::max(metal::max(_e855_.x, _e855_.y), _e855_.z)));
                    float _e864_ = metal::dot(_e654_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e867_ = _e863_ - metal::float3(metal::dot(_e863_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e880_ = metal::float2(_e864_, 1.0 - _e864_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e867_.x, _e867_.y), _e867_.z)), metal::max(metal::max(_e867_.x, _e867_.y), _e867_.z)));
                    local_1_ = (_e867_ * metal::min(1.0, metal::min(_e880_.x, _e880_.y))) + metal::float3(_e864_);
                }
                break;
            }
            case 14: {
                if (eh) {
                    metal::float3 _e888_ = local_2_;
                    metal::float3 _e889_ = metal::clamp(_e888_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e889_;
                    float _e890_ = metal::dot(_e654_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e893_ = _e889_ - metal::float3(metal::dot(_e889_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e906_ = metal::float2(_e890_, 1.0 - _e890_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e893_.x, _e893_.y), _e893_.z)), metal::max(metal::max(_e893_.x, _e893_.y), _e893_.z)));
                    local_1_ = (_e893_ * metal::min(1.0, metal::min(_e906_.x, _e906_.y))) + metal::float3(_e890_);
                }
                break;
            }
            case 15: {
                if (eh) {
                    metal::float3 _e914_ = local_2_;
                    metal::float3 _e915_ = metal::clamp(_e914_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e915_;
                    float _e916_ = metal::dot(_e915_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e919_ = _e654_ - metal::float3(metal::dot(_e654_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e932_ = metal::float2(_e916_, 1.0 - _e916_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e919_.x, _e919_.y), _e919_.z)), metal::max(metal::max(_e919_.x, _e919_.y), _e919_.z)));
                    local_1_ = (_e919_ * metal::min(1.0, metal::min(_e932_.x, _e932_.y))) + metal::float3(_e916_);
                }
                break;
            }
            default: {
                break;
            }
        }
        metal::float3 _e940_ = local_1_;
        metal::float3 _e943_ = metal::mix(_e646_, _e940_, metal::float3(_e639_.w)) * _e119_.w;
        metal::float4 _e949_ = metal::float4(_e943_.x, _e119_.y, _e119_.z, _e119_.w);
        metal::float4 _e955_ = metal::float4(_e949_.x, _e943_.y, _e949_.z, _e949_.w);
        phi_7325_ = metal::float4(_e955_.x, _e955_.y, _e943_.z, _e955_.w);
    }
    metal::float4 _e963_ = phi_7325_;
    float _e964_ = H1_1_;
    metal::float4 _e966_ = _e963_ * (_e628_ * _e964_);
    metal::float4 _e970_ = (_e607_ * (1.0 - _e966_.w)) + _e966_;
    metal::float3 _e971_ = _e970_.xyz;
    float _e974_ = n.z3_;
    float _e976_ = n.A3_;
    if (fh) {
        local_2 = _e970_.w != 0.0;
    } else {
        local_2 = false;
    }
    bool _e1430 = local_2;
    if (_e1430) {
        phi_7338_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e82_.x) + (0.00583715 * _e82_.y))) * _e974_) + _e976_) + _e971_;
    } else {
        phi_7338_ = _e971_;
    }
    metal::float3 _e992_ = phi_7338_;
    metal::float4 _e998_ = metal::float4(_e992_.x, _e970_.y, _e970_.z, _e970_.w);
    metal::float4 _e1004_ = metal::float4(_e998_.x, _e992_.y, _e998_.z, _e998_.w);
    metal::float4 _e1010_ = metal::float4(_e1004_.x, _e1004_.y, _e992_.z, _e1004_.w);
    switch(as_type<int>(0u)) {
        default: {
            if (_e970_.w == 0.0) {
                break;
            }
            float _e1013_ = 1.0 - _e970_.w;
            phi_7340_ = _e1010_;
            if (_e1013_ != 0.0) {
                uint _e1017_ = j0_.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size6 - 0 - 4) / 4)];
                phi_7340_ = _e1010_ + (metal::unpack_unorm4x8_to_float(_e1017_) * _e1013_);
            }
            metal::float4 _e1022_ = phi_7340_;
            j0_.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size6 - 0 - 4) / 4)] = metal::pack_float_to_unorm4x8(_e1022_);
            break;
        }
    }
    if (_e260_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size1 - 0 - 4) / 4)] = _e260_;
    }
    q4_.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size12 - 0 - 4) / 4)] = 65536u;
    return;
}

struct main_Input {
    metal::float2 X1_ [[user(loc0), center_perspective]];
    metal::float4 L0_ [[user(loc1), center_perspective]];
    uint w3_ [[user(loc4), flat]];
    uint A1_ [[user(loc5), flat]];
    float H1_ [[user(loc3), flat]];
};
fragment void main_(
  main_Input varyings [[stage_in]]
, metal::float4 gl_FragCoord [[position]]
, device Je const& AD [[buffer(2)]]
, device h0Bd& h0_ [[buffer(5)]]
, device Ke const& RB [[buffer(3)]]
, metal::texture2d<float, metal::access::sample> KD [[texture(0)]]
, metal::sampler Mb [[sampler(1)]]
, device j0Bd& j0_ [[buffer(4)]]
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
    uint A1_1_ = {};
    float H1_1_ = {};
    const auto X1_ = varyings.X1_;
    const auto L0_ = varyings.L0_;
    const auto w3_ = varyings.w3_;
    const auto A1_ = varyings.A1_;
    const auto H1_ = varyings.H1_;
    gl_FragCoord_1_ = gl_FragCoord;
    X1_1_ = X1_;
    L0_1_ = L0_;
    w3_1_ = w3_;
    A1_1_ = A1_;
    H1_1_ = H1_;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, j0_, n, IC, S5_, X1_1_, L0_1_, q4_, w3_1_, A1_1_, H1_1_, _buffer_sizes);
    return;
}
