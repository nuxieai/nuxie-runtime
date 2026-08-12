// language: metal4.0
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
constant bool fh = true;
constant bool eh = true;
constant bool ch = false;
constant bool Yg = true;
constant bool Zg = true;
constant bool ah = true;

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
    device j0Bd& j0_,
    constant CC& n,
    device q4Bd& q4_,
    thread uint& B0_1_,
    thread float& i1_1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    metal::float3 local = {};
    metal::float3 local_1_ = {};
    metal::float3 local_2_ = {};
    uint phi_3455_ = {};
    bool phi_1436_ = {};
    float phi_3460_ = {};
    float phi_3459_ = {};
    float phi_3461_ = {};
    float phi_3464_ = {};
    float phi_3463_ = {};
    bool phi_1473_ = {};
    float phi_3466_ = {};
    uint phi_4156_ = {};
    float phi_3465_ = {};
    uint phi_4155_ = {};
    metal::float4 phi_3489_ = {};
    bool phi_1592_ = {};
    uint phi_3493_ = {};
    bool phi_1601_ = {};
    float phi_3508_ = {};
    metal::float4 phi_3994_ = {};
    int phi_3930_ = {};
    metal::float4 phi_4151_ = {};
    uint phi_4182_ = {};
    metal::float4 phi_4176_ = {};
    metal::float3 phi_4177_ = {};
    metal::float4 phi_4179_ = {};
    bool local_1 = {};
    bool local_2 = {};
    metal::float4 _e77_ = gl_FragCoord_1_;
    metal::float2 _e78_ = _e77_.xy;
    metal::uint2 _e81_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e78_)));
    uint _e83_ = n.m6_;
    int _e112_ = as_type<int>(((((_e81_.y >> as_type<uint>(5u)) * (((_e83_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e81_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e81_.x & 28u) << as_type<uint>(5u)) + ((_e81_.y & 28u) << as_type<uint>(2)))) + (((_e81_.y & 3u) << as_type<uint>(2)) + (_e81_.x & 3u)));
    uint _e115_ = q4_.c2_[metal::min(unsigned(_e112_), (_buffer_sizes.size8 - 0 - 4) / 4)];
    uint _e117_ = _e115_ >> as_type<uint>(17u);
    uint _e118_ = B0_1_;
    if (_e117_ == _e118_) {
        phi_3455_ = _e115_;
    } else {
        phi_3455_ = (_e118_ << as_type<uint>(17u)) + 65536u;
    }
    uint _e124_ = phi_3455_;
    float _e125_ = i1_1_;
    q4_.c2_[metal::min(unsigned(_e112_), (_buffer_sizes.size8 - 0 - 4) / 4)] = _e124_ + as_type<uint>(naga_f2i32(metal::rint(_e125_ * 2048.0)));
    phi_4182_ = 0u;
    phi_4176_ = metal::float4(0.0, 0.0, 0.0, 0.0);
    if (_e117_ != _e118_) {
        float _e135_ = (static_cast<float>(_e115_ & 131071u) * 0.00048828125) + -32.0;
        metal::uint2 _e138_ = AD.c2_[metal::min(unsigned(_e117_), (_buffer_sizes.size0 - 0 - 8) / 8)];
        phi_3459_ = _e135_;
        if ((_e138_.x & 768u) != 0u) {
            float _e142_ = metal::abs(_e135_);
            phi_1436_ = ch;
            if (ch) {
                phi_1436_ = (_e138_.x & 512u) != 0u;
            }
            bool _e146_ = phi_1436_;
            phi_3460_ = _e142_;
            if (_e146_) {
                phi_3460_ = 1.0 - metal::abs((metal::fract(_e142_ * 0.5) * 2.0) + -1.0);
            }
            float _e154_ = phi_3460_;
            phi_3459_ = _e154_;
        }
        float _e156_ = phi_3459_;
        float _e157_ = metal::clamp(_e156_, 0.0, 1.0);
        phi_3463_ = _e157_;
        if (Yg) {
            uint _e159_ = _e138_.x >> as_type<uint>(16u);
            phi_3464_ = _e157_;
            if (_e159_ != 0u) {
                uint _e163_ = h0_.c2_[metal::min(unsigned(_e112_), (_buffer_sizes.size1 - 0 - 4) / 4)];
                if (_e159_ == (_e163_ >> as_type<uint>(16))) {
                    phi_3461_ = metal::min(_e157_, float2(as_type<half2>(_e163_)).x);
                } else {
                    phi_3461_ = 0.0;
                }
                float _e171_ = phi_3461_;
                phi_3464_ = _e171_;
            }
            float _e173_ = phi_3464_;
            phi_3463_ = _e173_;
        }
        float _e175_ = phi_3463_;
        phi_1473_ = Zg;
        if (Zg) {
            phi_1473_ = (_e138_.x & 1024u) != 0u;
        }
        bool _e179_ = phi_1473_;
        phi_3466_ = _e175_;
        if (_e179_) {
            uint _e180_ = _e117_ * 4u;
            metal::float4 _e184_ = RB.c2_[metal::min(unsigned(_e180_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float4 _e195_ = RB.c2_[metal::min(unsigned(_e180_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float2 _e200_ = _e195_.zw;
            metal::float2 _e202_ = (metal::abs((metal::float2x2(metal::float2(_e184_.x, _e184_.y), metal::float2(_e184_.z, _e184_.w)) * _e78_) + _e195_.xy) * _e200_) - _e200_;
            phi_3466_ = metal::min(_e175_, metal::clamp(metal::min(_e202_.x, _e202_.y) + 0.5, 0.0, 1.0));
        }
        float _e210_ = phi_3466_;
        uint _e211_ = _e138_.x & 15u;
        if (_e211_ <= 1u) {
            if (Yg) {
                local_1 = _e211_ == 0u;
            } else {
                local_1 = false;
            }
            bool _e216_ = local_1;
            phi_4156_ = 0u;
            if (_e216_) {
                phi_4156_ = _e138_.y | as_type<uint>(half2(metal::float2(_e210_, 0.0)));
            }
            uint _e221_ = phi_4156_;
            phi_4155_ = _e221_;
            phi_3489_ = metal::select(metal::unpack_unorm4x8_to_float(_e138_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e216_));
        } else {
            uint _e224_ = _e117_ * 4u;
            metal::float4 _e227_ = RB.c2_[metal::min(unsigned(_e224_), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float4 _e238_ = RB.c2_[metal::min(unsigned(_e224_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
            metal::float2 _e241_ = (metal::float2x2(metal::float2(_e227_.x, _e227_.y), metal::float2(_e227_.z, _e227_.w)) * _e78_) + _e238_.xy;
            if (_e211_ == 2u) {
                phi_3465_ = _e241_.x;
            } else {
                phi_3465_ = metal::length(_e241_);
            }
            float _e246_ = phi_3465_;
            metal::float4 _e255_ = KD.sample(Mb, metal::float2((metal::clamp(_e246_, 0.0, 1.0) * _e238_.z) + _e238_.w, as_type<float>(_e138_.y)), metal::level(0.0));
            phi_4155_ = 0u;
            phi_3489_ = _e255_;
        }
        uint _e257_ = phi_4155_;
        metal::float4 _e259_ = phi_3489_;
        float _e261_ = _e259_.w * _e210_;
        metal::float4 _e266_ = metal::float4(_e259_.x, _e259_.y, _e259_.z, _e261_);
        phi_1592_ = ah;
        if (ah) {
            phi_1592_ = _e261_ != 0.0;
        }
        bool _e269_ = phi_1592_;
        phi_3493_ = uint {};
        phi_1601_ = _e269_;
        if (_e269_) {
            uint _e272_ = (_e138_.x >> as_type<uint>(4)) & 15u;
            phi_3493_ = _e272_;
            phi_1601_ = _e272_ != 0u;
        }
        uint _e275_ = phi_3493_;
        bool _e277_ = phi_1601_;
        phi_4151_ = _e266_;
        if (_e277_) {
            uint _e280_ = j0_.c2_[metal::min(unsigned(_e112_), (_buffer_sizes.size6 - 0 - 4) / 4)];
            metal::float4 _e281_ = metal::unpack_unorm4x8_to_float(_e280_);
            metal::float3 _e282_ = _e266_.xyz;
            local_2_ = _e282_;
            metal::float3 _e283_ = _e281_.xyz;
            if (_e281_.w != 0.0) {
                phi_3508_ = 1.0 / _e281_.w;
            } else {
                phi_3508_ = 0.0;
            }
            float _e288_ = phi_3508_;
            metal::float3 _e289_ = _e283_ * _e288_;
            local = _e289_;
            switch(as_type<int>(_e275_)) {
                case 11: {
                    metal::float3 _e291_ = local_2_;
                    local_1_ = _e291_ * _e289_;
                    break;
                }
                case 1: {
                    metal::float3 _e293_ = local_2_;
                    local_1_ = (_e293_ + _e289_) - (_e293_ * _e289_);
                    break;
                }
                case 2: {
                    metal::float3 _e297_ = local_2_;
                    metal::float3 _e298_ = _e297_ * _e289_;
                    local_1_ = metal::select(_e298_, ((_e297_ + _e289_) - _e298_) - metal::float3(0.5, 0.5, 0.5), _e289_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                    break;
                }
                case 3: {
                    metal::float3 _e305_ = local_2_;
                    local_1_ = metal::min(_e305_, _e289_);
                    break;
                }
                case 4: {
                    metal::float3 _e307_ = local_2_;
                    local_1_ = metal::max(_e307_, _e289_);
                    break;
                }
                case 5: {
                    metal::float3 _e310_ = metal::clamp(_e283_, metal::float3(0.0, 0.0, 0.0), _e281_.www);
                    metal::float4 _e316_ = metal::float4(_e310_.x, float {}, float {}, float {});
                    metal::float4 _e322_ = metal::float4(_e316_.x, _e310_.y, _e316_.z, _e316_.w);
                    metal::float3 _e329_ = local_2_;
                    metal::float3 _e332_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e329_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e281_.w;
                    metal::float3 _e333_ = metal::float4(_e322_.x, _e322_.y, _e310_.z, _e322_.w).xyz;
                    local_1_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e333_ / _e332_), metal::sign(_e333_), _e332_ == metal::float3(0.0, 0.0, 0.0));
                    break;
                }
                case 6: {
                    metal::float3 _e339_ = local_2_;
                    local_2_ = metal::clamp(_e339_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    metal::float3 _e342_ = metal::clamp(_e283_, metal::float3(0.0, 0.0, 0.0), _e281_.www);
                    metal::float4 _e348_ = metal::float4(_e342_.x, _e281_.y, _e281_.z, _e281_.w);
                    metal::float4 _e354_ = metal::float4(_e348_.x, _e342_.y, _e348_.z, _e348_.w);
                    phi_3994_ = metal::float4(_e354_.x, _e354_.y, _e342_.z, _e354_.w);
                    if (_e281_.w == 0.0) {
                        phi_3994_ = metal::float4(_e342_.x, _e342_.y, _e342_.z, 1.0);
                    }
                    metal::float4 _e364_ = phi_3994_;
                    metal::float3 _e368_ = metal::float3(_e364_.w) - _e364_.xyz;
                    metal::float3 _e369_ = local_2_;
                    local_1_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e368_ / (_e369_ * _e364_.w)), metal::sign(_e368_), _e369_ == metal::float3(0.0, 0.0, 0.0));
                    break;
                }
                case 7: {
                    metal::float3 _e377_ = local_2_;
                    metal::float3 _e378_ = _e377_ * _e289_;
                    local_1_ = metal::select(_e378_, ((_e377_ + _e289_) - _e378_) - metal::float3(0.5, 0.5, 0.5), _e377_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                    break;
                }
                case 8: {
                    phi_3930_ = 0;
                    uint2 loop_bound = uint2(4294967295u);
                    bool loop_init = true;
                    while(true) {
                        if (metal::all(loop_bound == uint2(0u))) { break; }
                        loop_bound -= uint2(loop_bound.y == 0u, 1u);
                        if (!loop_init) {
                            phi_3930_ = as_type<int>(as_type<uint>(phi_3930_) + as_type<uint>(1));
                        }
                        loop_init = false;
                        int _e386_ = phi_3930_;
                        if (_e386_ < 3) {
                            float _e389_ = local_2_[metal::min(unsigned(_e386_), 2u)];
                            if (_e389_ <= 0.5) {
                                float _e392_ = local[metal::min(unsigned(_e386_), 2u)];
                                local_1_[metal::min(unsigned(_e386_), 2u)] = 1.0 - _e392_;
                            } else {
                                float _e396_ = local[metal::min(unsigned(_e386_), 2u)];
                                if (_e396_ <= 0.25) {
                                    float _e398_ = local[metal::min(unsigned(_e386_), 2u)];
                                    float _e401_ = local[metal::min(unsigned(_e386_), 2u)];
                                    local_1_[metal::min(unsigned(_e386_), 2u)] = (((16.0 * _e398_) - 12.0) * _e401_) + 3.0;
                                } else {
                                    float _e405_ = local[metal::min(unsigned(_e386_), 2u)];
                                    local_1_[metal::min(unsigned(_e386_), 2u)] = metal::rsqrt(_e405_) - 1.0;
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                    }
                    metal::float3 _e410_ = local_2_;
                    metal::float3 _e414_ = local_1_;
                    local_1_ = _e289_ + ((_e289_ * ((_e410_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e414_);
                    break;
                }
                case 9: {
                    metal::float3 _e417_ = local_2_;
                    local_1_ = metal::abs(_e289_ - _e417_);
                    break;
                }
                case 10: {
                    metal::float3 _e420_ = local_2_;
                    local_1_ = (_e420_ + _e289_) - ((_e420_ * 2.0) * _e289_);
                    break;
                }
                case 12: {
                    if (eh) {
                        metal::float3 _e425_ = local_2_;
                        metal::float3 _e426_ = metal::clamp(_e425_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                        local_2_ = _e426_;
                        metal::float3 _e441_ = _e426_ - metal::float3(metal::min(metal::min(_e426_.x, _e426_.y), _e426_.z));
                        metal::float3 _e449_ = _e441_ * ((metal::max(metal::max(_e289_.x, _e289_.y), _e289_.z) - metal::min(metal::min(_e289_.x, _e289_.y), _e289_.z)) / metal::max(0.000062, metal::max(metal::max(_e441_.x, _e441_.y), _e441_.z)));
                        float _e450_ = metal::dot(_e289_, metal::float3(0.3, 0.59, 0.11));
                        metal::float3 _e453_ = _e449_ - metal::float3(metal::dot(_e449_, metal::float3(0.3, 0.59, 0.11)));
                        metal::float2 _e466_ = metal::float2(_e450_, 1.0 - _e450_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e453_.x, _e453_.y), _e453_.z)), metal::max(metal::max(_e453_.x, _e453_.y), _e453_.z)));
                        local_1_ = (_e453_ * metal::min(1.0, metal::min(_e466_.x, _e466_.y))) + metal::float3(_e450_);
                    }
                    break;
                }
                case 13: {
                    if (eh) {
                        metal::float3 _e474_ = local_2_;
                        metal::float3 _e475_ = metal::clamp(_e474_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                        local_2_ = _e475_;
                        metal::float3 _e490_ = _e289_ - metal::float3(metal::min(metal::min(_e289_.x, _e289_.y), _e289_.z));
                        metal::float3 _e498_ = _e490_ * ((metal::max(metal::max(_e475_.x, _e475_.y), _e475_.z) - metal::min(metal::min(_e475_.x, _e475_.y), _e475_.z)) / metal::max(0.000062, metal::max(metal::max(_e490_.x, _e490_.y), _e490_.z)));
                        float _e499_ = metal::dot(_e289_, metal::float3(0.3, 0.59, 0.11));
                        metal::float3 _e502_ = _e498_ - metal::float3(metal::dot(_e498_, metal::float3(0.3, 0.59, 0.11)));
                        metal::float2 _e515_ = metal::float2(_e499_, 1.0 - _e499_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e502_.x, _e502_.y), _e502_.z)), metal::max(metal::max(_e502_.x, _e502_.y), _e502_.z)));
                        local_1_ = (_e502_ * metal::min(1.0, metal::min(_e515_.x, _e515_.y))) + metal::float3(_e499_);
                    }
                    break;
                }
                case 14: {
                    if (eh) {
                        metal::float3 _e523_ = local_2_;
                        metal::float3 _e524_ = metal::clamp(_e523_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                        local_2_ = _e524_;
                        float _e525_ = metal::dot(_e289_, metal::float3(0.3, 0.59, 0.11));
                        metal::float3 _e528_ = _e524_ - metal::float3(metal::dot(_e524_, metal::float3(0.3, 0.59, 0.11)));
                        metal::float2 _e541_ = metal::float2(_e525_, 1.0 - _e525_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e528_.x, _e528_.y), _e528_.z)), metal::max(metal::max(_e528_.x, _e528_.y), _e528_.z)));
                        local_1_ = (_e528_ * metal::min(1.0, metal::min(_e541_.x, _e541_.y))) + metal::float3(_e525_);
                    }
                    break;
                }
                case 15: {
                    if (eh) {
                        metal::float3 _e549_ = local_2_;
                        metal::float3 _e550_ = metal::clamp(_e549_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                        local_2_ = _e550_;
                        float _e551_ = metal::dot(_e550_, metal::float3(0.3, 0.59, 0.11));
                        metal::float3 _e554_ = _e289_ - metal::float3(metal::dot(_e289_, metal::float3(0.3, 0.59, 0.11)));
                        metal::float2 _e567_ = metal::float2(_e551_, 1.0 - _e551_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e554_.x, _e554_.y), _e554_.z)), metal::max(metal::max(_e554_.x, _e554_.y), _e554_.z)));
                        local_1_ = (_e554_ * metal::min(1.0, metal::min(_e567_.x, _e567_.y))) + metal::float3(_e551_);
                    }
                    break;
                }
                default: {
                    break;
                }
            }
            metal::float3 _e575_ = local_1_;
            metal::float3 _e577_ = metal::mix(_e282_, _e575_, metal::float3(_e281_.w));
            phi_4151_ = metal::float4(_e577_.x, _e577_.y, _e577_.z, _e261_);
        }
        metal::float4 _e583_ = phi_4151_;
        metal::float3 _e586_ = _e583_.xyz * _e583_.w;
        metal::float4 _e592_ = metal::float4(_e586_.x, _e583_.y, _e583_.z, _e583_.w);
        metal::float4 _e598_ = metal::float4(_e592_.x, _e586_.y, _e592_.z, _e592_.w);
        phi_4182_ = _e257_;
        phi_4176_ = metal::float4(_e598_.x, _e598_.y, _e586_.z, _e598_.w);
    }
    uint _e606_ = phi_4182_;
    metal::float4 _e608_ = phi_4176_;
    metal::float3 _e609_ = _e608_.xyz;
    float _e612_ = n.z3_;
    float _e614_ = n.A3_;
    if (fh) {
        local_2 = _e608_.w != 0.0;
    } else {
        local_2 = false;
    }
    bool _e855 = local_2;
    if (_e855) {
        phi_4177_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e77_.x) + (0.00583715 * _e77_.y))) * _e612_) + _e614_) + _e609_;
    } else {
        phi_4177_ = _e609_;
    }
    metal::float3 _e630_ = phi_4177_;
    metal::float4 _e636_ = metal::float4(_e630_.x, _e608_.y, _e608_.z, _e608_.w);
    metal::float4 _e642_ = metal::float4(_e636_.x, _e630_.y, _e636_.z, _e636_.w);
    metal::float4 _e648_ = metal::float4(_e642_.x, _e642_.y, _e630_.z, _e642_.w);
    switch(as_type<int>(0u)) {
        default: {
            if (_e608_.w == 0.0) {
                break;
            }
            float _e651_ = 1.0 - _e608_.w;
            phi_4179_ = _e648_;
            if (_e651_ != 0.0) {
                uint _e655_ = j0_.c2_[metal::min(unsigned(_e112_), (_buffer_sizes.size6 - 0 - 4) / 4)];
                phi_4179_ = _e648_ + (metal::unpack_unorm4x8_to_float(_e655_) * _e651_);
            }
            metal::float4 _e660_ = phi_4179_;
            j0_.c2_[metal::min(unsigned(_e112_), (_buffer_sizes.size6 - 0 - 4) / 4)] = metal::pack_float_to_unorm4x8(_e660_);
            break;
        }
    }
    if (_e606_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e112_), (_buffer_sizes.size1 - 0 - 4) / 4)] = _e606_;
    }
    return;
}

struct main_Input {
    uint B0_ [[user(loc1), flat]];
    float i1_ [[user(loc0), flat]];
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
, device q4Bd& q4_ [[buffer(6)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    uint B0_1_ = {};
    float i1_1_ = {};
    const auto B0_ = varyings.B0_;
    const auto i1_ = varyings.i1_;
    gl_FragCoord_1_ = gl_FragCoord;
    B0_1_ = B0_;
    i1_1_ = i1_;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, j0_, n, q4_, B0_1_, i1_1_, _buffer_sizes);
    return;
}
