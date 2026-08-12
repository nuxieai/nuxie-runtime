// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size0;
    uint size1;
    uint size2;
    uint size6;
    uint size13;
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
    thread float& R4_1_,
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
    float phi_5922_ = {};
    bool phi_1533_ = {};
    float phi_5192_ = {};
    float phi_5191_ = {};
    float phi_5193_ = {};
    float phi_5196_ = {};
    float phi_5195_ = {};
    bool phi_1570_ = {};
    float phi_5198_ = {};
    uint phi_5889_ = {};
    float phi_5197_ = {};
    uint phi_5888_ = {};
    metal::float4 phi_5222_ = {};
    bool phi_1689_ = {};
    uint phi_5226_ = {};
    bool phi_1698_ = {};
    float phi_5241_ = {};
    metal::float4 phi_5727_ = {};
    int phi_5663_ = {};
    metal::float4 phi_5884_ = {};
    bool phi_1275_ = {};
    uint phi_5910_ = {};
    float phi_5938_ = {};
    float phi_7317_ = {};
    bool phi_1303_ = {};
    float phi_5974_ = {};
    float phi_5975_ = {};
    metal::float4 phi_7004_ = {};
    int phi_6872_ = {};
    metal::float4 phi_7329_ = {};
    metal::float3 phi_7342_ = {};
    metal::float4 phi_7344_ = {};
    bool local_1 = {};
    bool local_2 = {};
    metal::float4 _e83_ = gl_FragCoord_1_;
    metal::float2 _e84_ = _e83_.xy;
    metal::uint2 _e87_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e84_)));
    uint _e89_ = n.m6_;
    int _e118_ = as_type<int>(((((_e87_.y >> as_type<uint>(5u)) * (((_e89_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e87_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e87_.x & 28u) << as_type<uint>(5u)) + ((_e87_.y & 28u) << as_type<uint>(2)))) + (((_e87_.y & 3u) << as_type<uint>(2)) + (_e87_.x & 3u)));
    metal::float2 _e119_ = X1_1_;
    metal::float4 _e120_ = IC.sample(S5_, _e119_);
    float _e121_ = R4_1_;
    float _e122_ = metal::min(_e121_, 1.0);
    phi_5922_ = _e122_;
    if (Zg) {
        metal::float4 _e123_ = L0_1_;
        metal::float2 _e126_ = metal::min(_e123_.xy, _e123_.zw);
        phi_5922_ = metal::clamp(metal::min(_e126_.x, _e126_.y), 0.0, _e122_);
    }
    float _e132_ = phi_5922_;
    uint _e135_ = q4_.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size13 - 0 - 4) / 4)];
    uint _e137_ = _e135_ >> as_type<uint>(17u);
    float _e141_ = (static_cast<float>(_e135_ & 131071u) * 0.00048828125) + -32.0;
    metal::uint2 _e144_ = AD.c2_[metal::min(unsigned(_e137_), (_buffer_sizes.size0 - 0 - 8) / 8)];
    phi_5191_ = _e141_;
    if ((_e144_.x & 768u) != 0u) {
        float _e148_ = metal::abs(_e141_);
        phi_1533_ = ch;
        if (ch) {
            phi_1533_ = (_e144_.x & 512u) != 0u;
        }
        bool _e152_ = phi_1533_;
        phi_5192_ = _e148_;
        if (_e152_) {
            phi_5192_ = 1.0 - metal::abs((metal::fract(_e148_ * 0.5) * 2.0) + -1.0);
        }
        float _e160_ = phi_5192_;
        phi_5191_ = _e160_;
    }
    float _e162_ = phi_5191_;
    float _e163_ = metal::clamp(_e162_, 0.0, 1.0);
    phi_5195_ = _e163_;
    if (Yg) {
        uint _e165_ = _e144_.x >> as_type<uint>(16u);
        phi_5196_ = _e163_;
        if (_e165_ != 0u) {
            uint _e169_ = h0_.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            if (_e165_ == (_e169_ >> as_type<uint>(16))) {
                phi_5193_ = metal::min(_e163_, float2(as_type<half2>(_e169_)).x);
            } else {
                phi_5193_ = 0.0;
            }
            float _e177_ = phi_5193_;
            phi_5196_ = _e177_;
        }
        float _e179_ = phi_5196_;
        phi_5195_ = _e179_;
    }
    float _e181_ = phi_5195_;
    phi_1570_ = Zg;
    if (Zg) {
        phi_1570_ = (_e144_.x & 1024u) != 0u;
    }
    bool _e185_ = phi_1570_;
    phi_5198_ = _e181_;
    if (_e185_) {
        uint _e186_ = _e137_ * 4u;
        metal::float4 _e190_ = RB.c2_[metal::min(unsigned(_e186_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e201_ = RB.c2_[metal::min(unsigned(_e186_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e206_ = _e201_.zw;
        metal::float2 _e208_ = (metal::abs((metal::float2x2(metal::float2(_e190_.x, _e190_.y), metal::float2(_e190_.z, _e190_.w)) * _e84_) + _e201_.xy) * _e206_) - _e206_;
        phi_5198_ = metal::min(_e181_, metal::clamp(metal::min(_e208_.x, _e208_.y) + 0.5, 0.0, 1.0));
    }
    float _e216_ = phi_5198_;
    uint _e217_ = _e144_.x & 15u;
    if (_e217_ <= 1u) {
        if (Yg) {
            local_1 = _e217_ == 0u;
        } else {
            local_1 = false;
        }
        bool _e222_ = local_1;
        phi_5889_ = 0u;
        if (_e222_) {
            phi_5889_ = _e144_.y | as_type<uint>(half2(metal::float2(_e216_, 0.0)));
        }
        uint _e227_ = phi_5889_;
        phi_5888_ = _e227_;
        phi_5222_ = metal::select(metal::unpack_unorm4x8_to_float(_e144_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e222_));
    } else {
        uint _e230_ = _e137_ * 4u;
        metal::float4 _e233_ = RB.c2_[metal::min(unsigned(_e230_), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e244_ = RB.c2_[metal::min(unsigned(_e230_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e247_ = (metal::float2x2(metal::float2(_e233_.x, _e233_.y), metal::float2(_e233_.z, _e233_.w)) * _e84_) + _e244_.xy;
        if (_e217_ == 2u) {
            phi_5197_ = _e247_.x;
        } else {
            phi_5197_ = metal::length(_e247_);
        }
        float _e252_ = phi_5197_;
        metal::float4 _e261_ = KD.sample(Mb, metal::float2((metal::clamp(_e252_, 0.0, 1.0) * _e244_.z) + _e244_.w, as_type<float>(_e144_.y)), metal::level(0.0));
        phi_5888_ = 0u;
        phi_5222_ = _e261_;
    }
    uint _e263_ = phi_5888_;
    metal::float4 _e265_ = phi_5222_;
    float _e267_ = _e265_.w * _e216_;
    metal::float4 _e272_ = metal::float4(_e265_.x, _e265_.y, _e265_.z, _e267_);
    phi_1689_ = ah;
    if (ah) {
        phi_1689_ = _e267_ != 0.0;
    }
    bool _e275_ = phi_1689_;
    phi_5226_ = uint {};
    phi_1698_ = _e275_;
    if (_e275_) {
        uint _e278_ = (_e144_.x >> as_type<uint>(4)) & 15u;
        phi_5226_ = _e278_;
        phi_1698_ = _e278_ != 0u;
    }
    uint _e281_ = phi_5226_;
    bool _e283_ = phi_1698_;
    phi_5884_ = _e272_;
    if (_e283_) {
        uint _e286_ = j0_.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size6 - 0 - 4) / 4)];
        metal::float4 _e287_ = metal::unpack_unorm4x8_to_float(_e286_);
        metal::float3 _e288_ = _e272_.xyz;
        local_5_ = _e288_;
        metal::float3 _e289_ = _e287_.xyz;
        if (_e287_.w != 0.0) {
            phi_5241_ = 1.0 / _e287_.w;
        } else {
            phi_5241_ = 0.0;
        }
        float _e294_ = phi_5241_;
        metal::float3 _e295_ = _e289_ * _e294_;
        local_3_ = _e295_;
        switch(as_type<int>(_e281_)) {
            case 11: {
                metal::float3 _e297_ = local_5_;
                local_4_ = _e297_ * _e295_;
                break;
            }
            case 1: {
                metal::float3 _e299_ = local_5_;
                local_4_ = (_e299_ + _e295_) - (_e299_ * _e295_);
                break;
            }
            case 2: {
                metal::float3 _e303_ = local_5_;
                metal::float3 _e304_ = _e303_ * _e295_;
                local_4_ = metal::select(_e304_, ((_e303_ + _e295_) - _e304_) - metal::float3(0.5, 0.5, 0.5), _e295_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 3: {
                metal::float3 _e311_ = local_5_;
                local_4_ = metal::min(_e311_, _e295_);
                break;
            }
            case 4: {
                metal::float3 _e313_ = local_5_;
                local_4_ = metal::max(_e313_, _e295_);
                break;
            }
            case 5: {
                metal::float3 _e316_ = metal::clamp(_e289_, metal::float3(0.0, 0.0, 0.0), _e287_.www);
                metal::float4 _e322_ = metal::float4(_e316_.x, float {}, float {}, float {});
                metal::float4 _e328_ = metal::float4(_e322_.x, _e316_.y, _e322_.z, _e322_.w);
                metal::float3 _e335_ = local_5_;
                metal::float3 _e338_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e335_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e287_.w;
                metal::float3 _e339_ = metal::float4(_e328_.x, _e328_.y, _e316_.z, _e328_.w).xyz;
                local_4_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e339_ / _e338_), metal::sign(_e339_), _e338_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 6: {
                metal::float3 _e345_ = local_5_;
                local_5_ = metal::clamp(_e345_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                metal::float3 _e348_ = metal::clamp(_e289_, metal::float3(0.0, 0.0, 0.0), _e287_.www);
                metal::float4 _e354_ = metal::float4(_e348_.x, _e287_.y, _e287_.z, _e287_.w);
                metal::float4 _e360_ = metal::float4(_e354_.x, _e348_.y, _e354_.z, _e354_.w);
                phi_5727_ = metal::float4(_e360_.x, _e360_.y, _e348_.z, _e360_.w);
                if (_e287_.w == 0.0) {
                    phi_5727_ = metal::float4(_e348_.x, _e348_.y, _e348_.z, 1.0);
                }
                metal::float4 _e370_ = phi_5727_;
                metal::float3 _e374_ = metal::float3(_e370_.w) - _e370_.xyz;
                metal::float3 _e375_ = local_5_;
                local_4_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e374_ / (_e375_ * _e370_.w)), metal::sign(_e374_), _e375_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 7: {
                metal::float3 _e383_ = local_5_;
                metal::float3 _e384_ = _e383_ * _e295_;
                local_4_ = metal::select(_e384_, ((_e383_ + _e295_) - _e384_) - metal::float3(0.5, 0.5, 0.5), _e383_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 8: {
                phi_5663_ = 0;
                uint2 loop_bound = uint2(4294967295u);
                bool loop_init = true;
                while(true) {
                    if (metal::all(loop_bound == uint2(0u))) { break; }
                    loop_bound -= uint2(loop_bound.y == 0u, 1u);
                    if (!loop_init) {
                        phi_5663_ = as_type<int>(as_type<uint>(phi_5663_) + as_type<uint>(1));
                    }
                    loop_init = false;
                    int _e392_ = phi_5663_;
                    if (_e392_ < 3) {
                        float _e395_ = local_5_[metal::min(unsigned(_e392_), 2u)];
                        if (_e395_ <= 0.5) {
                            float _e398_ = local_3_[metal::min(unsigned(_e392_), 2u)];
                            local_4_[metal::min(unsigned(_e392_), 2u)] = 1.0 - _e398_;
                        } else {
                            float _e402_ = local_3_[metal::min(unsigned(_e392_), 2u)];
                            if (_e402_ <= 0.25) {
                                float _e404_ = local_3_[metal::min(unsigned(_e392_), 2u)];
                                float _e407_ = local_3_[metal::min(unsigned(_e392_), 2u)];
                                local_4_[metal::min(unsigned(_e392_), 2u)] = (((16.0 * _e404_) - 12.0) * _e407_) + 3.0;
                            } else {
                                float _e411_ = local_3_[metal::min(unsigned(_e392_), 2u)];
                                local_4_[metal::min(unsigned(_e392_), 2u)] = metal::rsqrt(_e411_) - 1.0;
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                }
                metal::float3 _e416_ = local_5_;
                metal::float3 _e420_ = local_4_;
                local_4_ = _e295_ + ((_e295_ * ((_e416_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e420_);
                break;
            }
            case 9: {
                metal::float3 _e423_ = local_5_;
                local_4_ = metal::abs(_e295_ - _e423_);
                break;
            }
            case 10: {
                metal::float3 _e426_ = local_5_;
                local_4_ = (_e426_ + _e295_) - ((_e426_ * 2.0) * _e295_);
                break;
            }
            case 12: {
                if (eh) {
                    metal::float3 _e431_ = local_5_;
                    metal::float3 _e432_ = metal::clamp(_e431_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_5_ = _e432_;
                    metal::float3 _e447_ = _e432_ - metal::float3(metal::min(metal::min(_e432_.x, _e432_.y), _e432_.z));
                    metal::float3 _e455_ = _e447_ * ((metal::max(metal::max(_e295_.x, _e295_.y), _e295_.z) - metal::min(metal::min(_e295_.x, _e295_.y), _e295_.z)) / metal::max(0.000062, metal::max(metal::max(_e447_.x, _e447_.y), _e447_.z)));
                    float _e456_ = metal::dot(_e295_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e459_ = _e455_ - metal::float3(metal::dot(_e455_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e472_ = metal::float2(_e456_, 1.0 - _e456_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e459_.x, _e459_.y), _e459_.z)), metal::max(metal::max(_e459_.x, _e459_.y), _e459_.z)));
                    local_4_ = (_e459_ * metal::min(1.0, metal::min(_e472_.x, _e472_.y))) + metal::float3(_e456_);
                }
                break;
            }
            case 13: {
                if (eh) {
                    metal::float3 _e480_ = local_5_;
                    metal::float3 _e481_ = metal::clamp(_e480_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_5_ = _e481_;
                    metal::float3 _e496_ = _e295_ - metal::float3(metal::min(metal::min(_e295_.x, _e295_.y), _e295_.z));
                    metal::float3 _e504_ = _e496_ * ((metal::max(metal::max(_e481_.x, _e481_.y), _e481_.z) - metal::min(metal::min(_e481_.x, _e481_.y), _e481_.z)) / metal::max(0.000062, metal::max(metal::max(_e496_.x, _e496_.y), _e496_.z)));
                    float _e505_ = metal::dot(_e295_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e508_ = _e504_ - metal::float3(metal::dot(_e504_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e521_ = metal::float2(_e505_, 1.0 - _e505_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e508_.x, _e508_.y), _e508_.z)), metal::max(metal::max(_e508_.x, _e508_.y), _e508_.z)));
                    local_4_ = (_e508_ * metal::min(1.0, metal::min(_e521_.x, _e521_.y))) + metal::float3(_e505_);
                }
                break;
            }
            case 14: {
                if (eh) {
                    metal::float3 _e529_ = local_5_;
                    metal::float3 _e530_ = metal::clamp(_e529_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_5_ = _e530_;
                    float _e531_ = metal::dot(_e295_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e534_ = _e530_ - metal::float3(metal::dot(_e530_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e547_ = metal::float2(_e531_, 1.0 - _e531_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e534_.x, _e534_.y), _e534_.z)), metal::max(metal::max(_e534_.x, _e534_.y), _e534_.z)));
                    local_4_ = (_e534_ * metal::min(1.0, metal::min(_e547_.x, _e547_.y))) + metal::float3(_e531_);
                }
                break;
            }
            case 15: {
                if (eh) {
                    metal::float3 _e555_ = local_5_;
                    metal::float3 _e556_ = metal::clamp(_e555_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_5_ = _e556_;
                    float _e557_ = metal::dot(_e556_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e560_ = _e295_ - metal::float3(metal::dot(_e295_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e573_ = metal::float2(_e557_, 1.0 - _e557_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e560_.x, _e560_.y), _e560_.z)), metal::max(metal::max(_e560_.x, _e560_.y), _e560_.z)));
                    local_4_ = (_e560_ * metal::min(1.0, metal::min(_e573_.x, _e573_.y))) + metal::float3(_e557_);
                }
                break;
            }
            default: {
                break;
            }
        }
        metal::float3 _e581_ = local_4_;
        metal::float3 _e583_ = metal::mix(_e288_, _e581_, metal::float3(_e287_.w));
        phi_5884_ = metal::float4(_e583_.x, _e583_.y, _e583_.z, _e267_);
    }
    metal::float4 _e589_ = phi_5884_;
    metal::float3 _e592_ = _e589_.xyz * _e589_.w;
    metal::float4 _e598_ = metal::float4(_e592_.x, _e589_.y, _e589_.z, _e589_.w);
    metal::float4 _e604_ = metal::float4(_e598_.x, _e592_.y, _e598_.z, _e598_.w);
    metal::float4 _e610_ = metal::float4(_e604_.x, _e604_.y, _e592_.z, _e604_.w);
    phi_1275_ = Yg;
    if (Yg) {
        uint _e611_ = w3_1_;
        phi_1275_ = _e611_ != 0u;
    }
    bool _e614_ = phi_1275_;
    phi_7317_ = _e132_;
    if (_e614_) {
        if (_e263_ != 0u) {
            phi_5910_ = _e263_;
        } else {
            uint _e618_ = h0_.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            phi_5910_ = _e618_;
        }
        uint _e620_ = phi_5910_;
        uint _e621_ = w3_1_;
        if (_e621_ == (_e620_ >> as_type<uint>(16))) {
            phi_5938_ = metal::min(_e132_, float2(as_type<half2>(_e620_)).x);
        } else {
            phi_5938_ = 0.0;
        }
        float _e629_ = phi_5938_;
        phi_7317_ = _e629_;
    }
    float _e631_ = phi_7317_;
    phi_1303_ = ah;
    if (ah) {
        uint _e632_ = A1_1_;
        phi_1303_ = _e632_ != 0u;
    }
    bool _e635_ = phi_1303_;
    phi_7329_ = _e120_;
    if (_e635_) {
        uint _e638_ = j0_.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size6 - 0 - 4) / 4)];
        metal::float4 _e642_ = (metal::unpack_unorm4x8_to_float(_e638_) * (1.0 - _e589_.w)) + _e610_;
        if (_e120_.w != 0.0) {
            phi_5974_ = 1.0 / _e120_.w;
        } else {
            phi_5974_ = 0.0;
        }
        float _e648_ = phi_5974_;
        metal::float3 _e649_ = _e120_.xyz * _e648_;
        uint _e650_ = A1_1_;
        local_2_ = _e649_;
        metal::float3 _e651_ = _e642_.xyz;
        if (_e642_.w != 0.0) {
            phi_5975_ = 1.0 / _e642_.w;
        } else {
            phi_5975_ = 0.0;
        }
        float _e656_ = phi_5975_;
        metal::float3 _e657_ = _e651_ * _e656_;
        local = _e657_;
        switch(as_type<int>(_e650_)) {
            case 11: {
                metal::float3 _e659_ = local_2_;
                local_1_ = _e659_ * _e657_;
                break;
            }
            case 1: {
                metal::float3 _e661_ = local_2_;
                local_1_ = (_e661_ + _e657_) - (_e661_ * _e657_);
                break;
            }
            case 2: {
                metal::float3 _e665_ = local_2_;
                metal::float3 _e666_ = _e665_ * _e657_;
                local_1_ = metal::select(_e666_, ((_e665_ + _e657_) - _e666_) - metal::float3(0.5, 0.5, 0.5), _e657_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 3: {
                metal::float3 _e673_ = local_2_;
                local_1_ = metal::min(_e673_, _e657_);
                break;
            }
            case 4: {
                metal::float3 _e675_ = local_2_;
                local_1_ = metal::max(_e675_, _e657_);
                break;
            }
            case 5: {
                metal::float3 _e678_ = metal::clamp(_e651_, metal::float3(0.0, 0.0, 0.0), _e642_.www);
                metal::float4 _e684_ = metal::float4(_e678_.x, float {}, float {}, float {});
                metal::float4 _e690_ = metal::float4(_e684_.x, _e678_.y, _e684_.z, _e684_.w);
                metal::float3 _e697_ = local_2_;
                metal::float3 _e700_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e697_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e642_.w;
                metal::float3 _e701_ = metal::float4(_e690_.x, _e690_.y, _e678_.z, _e690_.w).xyz;
                local_1_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e701_ / _e700_), metal::sign(_e701_), _e700_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 6: {
                metal::float3 _e707_ = local_2_;
                local_2_ = metal::clamp(_e707_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                metal::float3 _e710_ = metal::clamp(_e651_, metal::float3(0.0, 0.0, 0.0), _e642_.www);
                metal::float4 _e716_ = metal::float4(_e710_.x, _e642_.y, _e642_.z, _e642_.w);
                metal::float4 _e722_ = metal::float4(_e716_.x, _e710_.y, _e716_.z, _e716_.w);
                phi_7004_ = metal::float4(_e722_.x, _e722_.y, _e710_.z, _e722_.w);
                if (_e642_.w == 0.0) {
                    phi_7004_ = metal::float4(_e710_.x, _e710_.y, _e710_.z, 1.0);
                }
                metal::float4 _e732_ = phi_7004_;
                metal::float3 _e736_ = metal::float3(_e732_.w) - _e732_.xyz;
                metal::float3 _e737_ = local_2_;
                local_1_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e736_ / (_e737_ * _e732_.w)), metal::sign(_e736_), _e737_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 7: {
                metal::float3 _e745_ = local_2_;
                metal::float3 _e746_ = _e745_ * _e657_;
                local_1_ = metal::select(_e746_, ((_e745_ + _e657_) - _e746_) - metal::float3(0.5, 0.5, 0.5), _e745_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 8: {
                phi_6872_ = 0;
                uint2 loop_bound_1 = uint2(4294967295u);
                bool loop_init_1 = true;
                while(true) {
                    if (metal::all(loop_bound_1 == uint2(0u))) { break; }
                    loop_bound_1 -= uint2(loop_bound_1.y == 0u, 1u);
                    if (!loop_init_1) {
                        phi_6872_ = as_type<int>(as_type<uint>(phi_6872_) + as_type<uint>(1));
                    }
                    loop_init_1 = false;
                    int _e754_ = phi_6872_;
                    if (_e754_ < 3) {
                        float _e757_ = local_2_[metal::min(unsigned(_e754_), 2u)];
                        if (_e757_ <= 0.5) {
                            float _e760_ = local[metal::min(unsigned(_e754_), 2u)];
                            local_1_[metal::min(unsigned(_e754_), 2u)] = 1.0 - _e760_;
                        } else {
                            float _e764_ = local[metal::min(unsigned(_e754_), 2u)];
                            if (_e764_ <= 0.25) {
                                float _e766_ = local[metal::min(unsigned(_e754_), 2u)];
                                float _e769_ = local[metal::min(unsigned(_e754_), 2u)];
                                local_1_[metal::min(unsigned(_e754_), 2u)] = (((16.0 * _e766_) - 12.0) * _e769_) + 3.0;
                            } else {
                                float _e773_ = local[metal::min(unsigned(_e754_), 2u)];
                                local_1_[metal::min(unsigned(_e754_), 2u)] = metal::rsqrt(_e773_) - 1.0;
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                }
                metal::float3 _e778_ = local_2_;
                metal::float3 _e782_ = local_1_;
                local_1_ = _e657_ + ((_e657_ * ((_e778_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e782_);
                break;
            }
            case 9: {
                metal::float3 _e785_ = local_2_;
                local_1_ = metal::abs(_e657_ - _e785_);
                break;
            }
            case 10: {
                metal::float3 _e788_ = local_2_;
                local_1_ = (_e788_ + _e657_) - ((_e788_ * 2.0) * _e657_);
                break;
            }
            case 12: {
                if (eh) {
                    metal::float3 _e793_ = local_2_;
                    metal::float3 _e794_ = metal::clamp(_e793_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e794_;
                    metal::float3 _e809_ = _e794_ - metal::float3(metal::min(metal::min(_e794_.x, _e794_.y), _e794_.z));
                    metal::float3 _e817_ = _e809_ * ((metal::max(metal::max(_e657_.x, _e657_.y), _e657_.z) - metal::min(metal::min(_e657_.x, _e657_.y), _e657_.z)) / metal::max(0.000062, metal::max(metal::max(_e809_.x, _e809_.y), _e809_.z)));
                    float _e818_ = metal::dot(_e657_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e821_ = _e817_ - metal::float3(metal::dot(_e817_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e834_ = metal::float2(_e818_, 1.0 - _e818_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e821_.x, _e821_.y), _e821_.z)), metal::max(metal::max(_e821_.x, _e821_.y), _e821_.z)));
                    local_1_ = (_e821_ * metal::min(1.0, metal::min(_e834_.x, _e834_.y))) + metal::float3(_e818_);
                }
                break;
            }
            case 13: {
                if (eh) {
                    metal::float3 _e842_ = local_2_;
                    metal::float3 _e843_ = metal::clamp(_e842_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e843_;
                    metal::float3 _e858_ = _e657_ - metal::float3(metal::min(metal::min(_e657_.x, _e657_.y), _e657_.z));
                    metal::float3 _e866_ = _e858_ * ((metal::max(metal::max(_e843_.x, _e843_.y), _e843_.z) - metal::min(metal::min(_e843_.x, _e843_.y), _e843_.z)) / metal::max(0.000062, metal::max(metal::max(_e858_.x, _e858_.y), _e858_.z)));
                    float _e867_ = metal::dot(_e657_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e870_ = _e866_ - metal::float3(metal::dot(_e866_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e883_ = metal::float2(_e867_, 1.0 - _e867_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e870_.x, _e870_.y), _e870_.z)), metal::max(metal::max(_e870_.x, _e870_.y), _e870_.z)));
                    local_1_ = (_e870_ * metal::min(1.0, metal::min(_e883_.x, _e883_.y))) + metal::float3(_e867_);
                }
                break;
            }
            case 14: {
                if (eh) {
                    metal::float3 _e891_ = local_2_;
                    metal::float3 _e892_ = metal::clamp(_e891_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e892_;
                    float _e893_ = metal::dot(_e657_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e896_ = _e892_ - metal::float3(metal::dot(_e892_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e909_ = metal::float2(_e893_, 1.0 - _e893_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e896_.x, _e896_.y), _e896_.z)), metal::max(metal::max(_e896_.x, _e896_.y), _e896_.z)));
                    local_1_ = (_e896_ * metal::min(1.0, metal::min(_e909_.x, _e909_.y))) + metal::float3(_e893_);
                }
                break;
            }
            case 15: {
                if (eh) {
                    metal::float3 _e917_ = local_2_;
                    metal::float3 _e918_ = metal::clamp(_e917_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e918_;
                    float _e919_ = metal::dot(_e918_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e922_ = _e657_ - metal::float3(metal::dot(_e657_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e935_ = metal::float2(_e919_, 1.0 - _e919_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e922_.x, _e922_.y), _e922_.z)), metal::max(metal::max(_e922_.x, _e922_.y), _e922_.z)));
                    local_1_ = (_e922_ * metal::min(1.0, metal::min(_e935_.x, _e935_.y))) + metal::float3(_e919_);
                }
                break;
            }
            default: {
                break;
            }
        }
        metal::float3 _e943_ = local_1_;
        metal::float3 _e946_ = metal::mix(_e649_, _e943_, metal::float3(_e642_.w)) * _e120_.w;
        metal::float4 _e952_ = metal::float4(_e946_.x, _e120_.y, _e120_.z, _e120_.w);
        metal::float4 _e958_ = metal::float4(_e952_.x, _e946_.y, _e952_.z, _e952_.w);
        phi_7329_ = metal::float4(_e958_.x, _e958_.y, _e946_.z, _e958_.w);
    }
    metal::float4 _e966_ = phi_7329_;
    float _e967_ = H1_1_;
    metal::float4 _e969_ = _e966_ * (_e631_ * _e967_);
    metal::float4 _e973_ = (_e610_ * (1.0 - _e969_.w)) + _e969_;
    metal::float3 _e974_ = _e973_.xyz;
    float _e977_ = n.z3_;
    float _e979_ = n.A3_;
    if (fh) {
        local_2 = _e973_.w != 0.0;
    } else {
        local_2 = false;
    }
    bool _e1432 = local_2;
    if (_e1432) {
        phi_7342_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e83_.x) + (0.00583715 * _e83_.y))) * _e977_) + _e979_) + _e974_;
    } else {
        phi_7342_ = _e974_;
    }
    metal::float3 _e995_ = phi_7342_;
    metal::float4 _e1001_ = metal::float4(_e995_.x, _e973_.y, _e973_.z, _e973_.w);
    metal::float4 _e1007_ = metal::float4(_e1001_.x, _e995_.y, _e1001_.z, _e1001_.w);
    metal::float4 _e1013_ = metal::float4(_e1007_.x, _e1007_.y, _e995_.z, _e1007_.w);
    switch(as_type<int>(0u)) {
        default: {
            if (_e973_.w == 0.0) {
                break;
            }
            float _e1016_ = 1.0 - _e973_.w;
            phi_7344_ = _e1013_;
            if (_e1016_ != 0.0) {
                uint _e1020_ = j0_.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size6 - 0 - 4) / 4)];
                phi_7344_ = _e1013_ + (metal::unpack_unorm4x8_to_float(_e1020_) * _e1016_);
            }
            metal::float4 _e1025_ = phi_7344_;
            j0_.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size6 - 0 - 4) / 4)] = metal::pack_float_to_unorm4x8(_e1025_);
            break;
        }
    }
    if (_e263_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size1 - 0 - 4) / 4)] = _e263_;
    }
    q4_.c2_[metal::min(unsigned(_e118_), (_buffer_sizes.size13 - 0 - 4) / 4)] = 65536u;
    return;
}

struct main_Input {
    metal::float2 X1_ [[user(loc0), center_perspective]];
    float R4_ [[user(loc1), center_perspective]];
    metal::float4 L0_ [[user(loc2), center_perspective]];
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
    float R4_1_ = {};
    metal::float4 L0_1_ = {};
    uint w3_1_ = {};
    uint A1_1_ = {};
    float H1_1_ = {};
    const auto X1_ = varyings.X1_;
    const auto R4_ = varyings.R4_;
    const auto L0_ = varyings.L0_;
    const auto w3_ = varyings.w3_;
    const auto A1_ = varyings.A1_;
    const auto H1_ = varyings.H1_;
    gl_FragCoord_1_ = gl_FragCoord;
    X1_1_ = X1_;
    R4_1_ = R4_;
    L0_1_ = L0_;
    w3_1_ = w3_;
    A1_1_ = A1_;
    H1_1_ = H1_;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, j0_, n, IC, S5_, X1_1_, R4_1_, L0_1_, q4_, w3_1_, A1_1_, H1_1_, _buffer_sizes);
    return;
}
