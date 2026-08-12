// language: metal4.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size2;
    uint size3;
    uint size4;
    uint size8;
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
typedef metal::atomic_uint type_11[1];
struct q4Bd_1_ {
    type_11 c2_;
};
constant bool fh = true;
constant bool eh = true;
constant bool ch = false;
constant bool Yg = true;
constant bool Zg = true;
constant bool ah = true;
constant bool bh = false;

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
    device j0Bd& j0_,
    constant CC& n,
    thread metal::float4& O_1_,
    thread uint& B0_1_,
    device q4Bd_1_& q4_,
    constant _mslBufferSizes& _buffer_sizes
) {
    metal::float3 local = {};
    metal::float3 local_1_ = {};
    metal::float3 local_2_ = {};
    bool phi_1427_ = {};
    bool phi_1440_ = {};
    float phi_3981_ = {};
    float phi_3989_ = {};
    float phi_3997_ = {};
    float phi_3996_ = {};
    bool phi_1932_ = {};
    float phi_4000_ = {};
    float phi_3999_ = {};
    float phi_4001_ = {};
    float phi_4004_ = {};
    float phi_4003_ = {};
    bool phi_1969_ = {};
    float phi_4006_ = {};
    uint phi_4916_ = {};
    float phi_4005_ = {};
    uint phi_4915_ = {};
    metal::float4 phi_4039_ = {};
    bool phi_2088_ = {};
    uint phi_4043_ = {};
    bool phi_2097_ = {};
    float phi_4063_ = {};
    metal::float4 phi_4709_ = {};
    int phi_4625_ = {};
    metal::float4 phi_4911_ = {};
    uint phi_4943_ = {};
    metal::float4 phi_4936_ = {};
    metal::float3 phi_4938_ = {};
    metal::float4 phi_4940_ = {};
    bool local_1 = {};
    bool local_2 = {};
    metal::float4 _e93_ = gl_FragCoord_1_;
    metal::float2 _e94_ = _e93_.xy;
    metal::uint2 _e97_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e94_)));
    uint _e99_ = n.m6_;
    int _e128_ = as_type<int>(((((_e97_.y >> as_type<uint>(5u)) * (((_e99_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e97_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e97_.x & 28u) << as_type<uint>(5u)) + ((_e97_.y & 28u) << as_type<uint>(2)))) + (((_e97_.y & 3u) << as_type<uint>(2)) + (_e97_.x & 3u)));
    phi_1427_ = bh;
    if (bh) {
        metal::float4 _e129_ = O_1_;
        phi_1427_ = _e129_.x < -1.5;
    }
    bool _e133_ = phi_1427_;
    if (_e133_) {
        metal::float4 _e134_ = O_1_;
        metal::float4 _e138_ = XC.sample(aa, metal::float2(3.0 + _e134_.x, 0.0), metal::level(0.0));
        metal::float4 _e144_ = XC.sample(aa, metal::float2(1.0 - _e134_.y, 0.0), metal::level(0.0));
        phi_3996_ = (1.0 - _e138_.x) - _e144_.x;
    } else {
        phi_1440_ = bh;
        if (bh) {
            metal::float4 _e147_ = O_1_;
            phi_1440_ = _e147_.y < -1.5;
        }
        bool _e151_ = phi_1440_;
        if (_e151_) {
            metal::float4 _e152_ = O_1_;
            float _e155_ = metal::max(_e152_.w, 0.0);
            if (_e152_.z >= 0.0) {
                metal::float4 _e158_ = XC.sample(aa, metal::float2(_e155_, 0.0), metal::level(0.0));
                phi_3981_ = _e158_.x;
            } else {
                phi_3981_ = 0.0;
            }
            float _e161_ = phi_3981_;
            phi_3989_ = _e161_;
            if (metal::abs(_e152_.z) < 1000.0) {
                float _e168_ = -2.0 - _e152_.y;
                float _e170_ = (_e168_ - _e155_) * 0.5984134;
                metal::float4 _e173_ = metal::float4(_e155_) + (metal::float4(0.20888568, 0.62665707, 1.0444285, 1.4621998) * _e170_);
                metal::float4 _e179_ = (_e173_ * -(_e152_.z)) + metal::float4((_e168_ * _e152_.z) + (metal::abs(_e152_.x) - 0.25));
                metal::float4 _e182_ = XC.sample(aa, metal::float2(_e179_.x, 0.0), metal::level(0.0));
                metal::float4 _e185_ = XC.sample(aa, metal::float2(_e179_.y, 0.0), metal::level(0.0));
                metal::float4 _e188_ = XC.sample(aa, metal::float2(_e179_.z, 0.0), metal::level(0.0));
                metal::float4 _e191_ = XC.sample(aa, metal::float2(_e179_.w, 0.0), metal::level(0.0));
                metal::float4 _e197_ = _e173_ * 5.0959306;
                phi_3989_ = _e161_ + (metal::dot(metal::float4(_e182_.x, _e185_.x, _e188_.x, _e191_.x), metal::exp2((metal::float4(2.5479653, 2.5479653, 2.5479653, 2.5479653) - _e197_) * (_e197_ + metal::float4(-2.5479653, -2.5479653, -2.5479653, -2.5479653)))) * _e170_);
            }
            float _e206_ = phi_3989_;
            phi_3997_ = _e206_ * metal::sign(_e152_.x);
        } else {
            float _e211_ = O_1_.x;
            float _e213_ = O_1_.y;
            phi_3997_ = metal::min(metal::min(_e211_, metal::abs(_e213_)), 1.0);
        }
        float _e218_ = phi_3997_;
        phi_3996_ = _e218_;
    }
    float _e220_ = phi_3996_;
    uint _e224_ = naga_f2u32(metal::rint((_e220_ * 2048.0) + 65536.0));
    uint _e225_ = B0_1_;
    uint _e228_ = (_e225_ << as_type<uint>(17u)) | _e224_;
    uint _e258 = metal::atomic_fetch_max_explicit(&q4_.c2_[metal::min(unsigned(_e128_), (_buffer_sizes.size12 - 0 - 4) / 4)], _e228_, metal::memory_order_relaxed);
    uint _e233_ = _e258 >> as_type<uint>(17u);
    if (_e233_ == _e225_) {
        metal::float4 _e235_ = O_1_;
        if (_e235_.y < 0.0) {
            uint _e276 = metal::atomic_fetch_add_explicit(&q4_.c2_[metal::min(unsigned(_e128_), (_buffer_sizes.size12 - 0 - 4) / 4)], (_e224_ + (_e258 - metal::max(_e228_, _e258))) - 65536u, metal::memory_order_relaxed);
        }
        phi_4943_ = 0u;
        phi_4936_ = metal::float4(0.0, 0.0, 0.0, 0.0);
    } else {
        float _e246_ = (static_cast<float>(_e258 & 131071u) * 0.00048828125) + -32.0;
        metal::uint2 _e249_ = AD.c2_[metal::min(unsigned(_e233_), (_buffer_sizes.size2 - 0 - 8) / 8)];
        phi_3999_ = _e246_;
        if ((_e249_.x & 768u) != 0u) {
            float _e253_ = metal::abs(_e246_);
            phi_1932_ = ch;
            if (ch) {
                phi_1932_ = (_e249_.x & 512u) != 0u;
            }
            bool _e257_ = phi_1932_;
            phi_4000_ = _e253_;
            if (_e257_) {
                phi_4000_ = 1.0 - metal::abs((metal::fract(_e253_ * 0.5) * 2.0) + -1.0);
            }
            float _e265_ = phi_4000_;
            phi_3999_ = _e265_;
        }
        float _e267_ = phi_3999_;
        float _e268_ = metal::clamp(_e267_, 0.0, 1.0);
        phi_4003_ = _e268_;
        if (Yg) {
            uint _e270_ = _e249_.x >> as_type<uint>(16u);
            phi_4004_ = _e268_;
            if (_e270_ != 0u) {
                uint _e274_ = h0_.c2_[metal::min(unsigned(_e128_), (_buffer_sizes.size3 - 0 - 4) / 4)];
                if (_e270_ == (_e274_ >> as_type<uint>(16))) {
                    phi_4001_ = metal::min(_e268_, float2(as_type<half2>(_e274_)).x);
                } else {
                    phi_4001_ = 0.0;
                }
                float _e282_ = phi_4001_;
                phi_4004_ = _e282_;
            }
            float _e284_ = phi_4004_;
            phi_4003_ = _e284_;
        }
        float _e286_ = phi_4003_;
        phi_1969_ = Zg;
        if (Zg) {
            phi_1969_ = (_e249_.x & 1024u) != 0u;
        }
        bool _e290_ = phi_1969_;
        phi_4006_ = _e286_;
        if (_e290_) {
            uint _e291_ = _e233_ * 4u;
            metal::float4 _e295_ = RB.c2_[metal::min(unsigned(_e291_ + 2u), (_buffer_sizes.size4 - 0 - 16) / 16)];
            metal::float4 _e306_ = RB.c2_[metal::min(unsigned(_e291_ + 3u), (_buffer_sizes.size4 - 0 - 16) / 16)];
            metal::float2 _e311_ = _e306_.zw;
            metal::float2 _e313_ = (metal::abs((metal::float2x2(metal::float2(_e295_.x, _e295_.y), metal::float2(_e295_.z, _e295_.w)) * _e94_) + _e306_.xy) * _e311_) - _e311_;
            phi_4006_ = metal::min(_e286_, metal::clamp(metal::min(_e313_.x, _e313_.y) + 0.5, 0.0, 1.0));
        }
        float _e321_ = phi_4006_;
        uint _e322_ = _e249_.x & 15u;
        if (_e322_ <= 1u) {
            if (Yg) {
                local_1 = _e322_ == 0u;
            } else {
                local_1 = false;
            }
            bool _e327_ = local_1;
            phi_4916_ = 0u;
            if (_e327_) {
                phi_4916_ = _e249_.y | as_type<uint>(half2(metal::float2(_e321_, 0.0)));
            }
            uint _e332_ = phi_4916_;
            phi_4915_ = _e332_;
            phi_4039_ = metal::select(metal::unpack_unorm4x8_to_float(_e249_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e327_));
        } else {
            uint _e335_ = _e233_ * 4u;
            metal::float4 _e338_ = RB.c2_[metal::min(unsigned(_e335_), (_buffer_sizes.size4 - 0 - 16) / 16)];
            metal::float4 _e349_ = RB.c2_[metal::min(unsigned(_e335_ + 1u), (_buffer_sizes.size4 - 0 - 16) / 16)];
            metal::float2 _e352_ = (metal::float2x2(metal::float2(_e338_.x, _e338_.y), metal::float2(_e338_.z, _e338_.w)) * _e94_) + _e349_.xy;
            if (_e322_ == 2u) {
                phi_4005_ = _e352_.x;
            } else {
                phi_4005_ = metal::length(_e352_);
            }
            float _e357_ = phi_4005_;
            metal::float4 _e366_ = KD.sample(Mb, metal::float2((metal::clamp(_e357_, 0.0, 1.0) * _e349_.z) + _e349_.w, as_type<float>(_e249_.y)), metal::level(0.0));
            phi_4915_ = 0u;
            phi_4039_ = _e366_;
        }
        uint _e368_ = phi_4915_;
        metal::float4 _e370_ = phi_4039_;
        float _e372_ = _e370_.w * _e321_;
        metal::float4 _e377_ = metal::float4(_e370_.x, _e370_.y, _e370_.z, _e372_);
        phi_2088_ = ah;
        if (ah) {
            phi_2088_ = _e372_ != 0.0;
        }
        bool _e380_ = phi_2088_;
        phi_4043_ = uint {};
        phi_2097_ = _e380_;
        if (_e380_) {
            uint _e383_ = (_e249_.x >> as_type<uint>(4)) & 15u;
            phi_4043_ = _e383_;
            phi_2097_ = _e383_ != 0u;
        }
        uint _e386_ = phi_4043_;
        bool _e388_ = phi_2097_;
        phi_4911_ = _e377_;
        if (_e388_) {
            uint _e391_ = j0_.c2_[metal::min(unsigned(_e128_), (_buffer_sizes.size8 - 0 - 4) / 4)];
            metal::float4 _e392_ = metal::unpack_unorm4x8_to_float(_e391_);
            metal::float3 _e393_ = _e377_.xyz;
            local_2_ = _e393_;
            metal::float3 _e394_ = _e392_.xyz;
            if (_e392_.w != 0.0) {
                phi_4063_ = 1.0 / _e392_.w;
            } else {
                phi_4063_ = 0.0;
            }
            float _e399_ = phi_4063_;
            metal::float3 _e400_ = _e394_ * _e399_;
            local = _e400_;
            switch(as_type<int>(_e386_)) {
                case 11: {
                    metal::float3 _e402_ = local_2_;
                    local_1_ = _e402_ * _e400_;
                    break;
                }
                case 1: {
                    metal::float3 _e404_ = local_2_;
                    local_1_ = (_e404_ + _e400_) - (_e404_ * _e400_);
                    break;
                }
                case 2: {
                    metal::float3 _e408_ = local_2_;
                    metal::float3 _e409_ = _e408_ * _e400_;
                    local_1_ = metal::select(_e409_, ((_e408_ + _e400_) - _e409_) - metal::float3(0.5, 0.5, 0.5), _e400_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                    break;
                }
                case 3: {
                    metal::float3 _e416_ = local_2_;
                    local_1_ = metal::min(_e416_, _e400_);
                    break;
                }
                case 4: {
                    metal::float3 _e418_ = local_2_;
                    local_1_ = metal::max(_e418_, _e400_);
                    break;
                }
                case 5: {
                    metal::float3 _e421_ = metal::clamp(_e394_, metal::float3(0.0, 0.0, 0.0), _e392_.www);
                    metal::float4 _e427_ = metal::float4(_e421_.x, float {}, float {}, float {});
                    metal::float4 _e433_ = metal::float4(_e427_.x, _e421_.y, _e427_.z, _e427_.w);
                    metal::float3 _e440_ = local_2_;
                    metal::float3 _e443_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e440_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e392_.w;
                    metal::float3 _e444_ = metal::float4(_e433_.x, _e433_.y, _e421_.z, _e433_.w).xyz;
                    local_1_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e444_ / _e443_), metal::sign(_e444_), _e443_ == metal::float3(0.0, 0.0, 0.0));
                    break;
                }
                case 6: {
                    metal::float3 _e450_ = local_2_;
                    local_2_ = metal::clamp(_e450_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    metal::float3 _e453_ = metal::clamp(_e394_, metal::float3(0.0, 0.0, 0.0), _e392_.www);
                    metal::float4 _e459_ = metal::float4(_e453_.x, _e392_.y, _e392_.z, _e392_.w);
                    metal::float4 _e465_ = metal::float4(_e459_.x, _e453_.y, _e459_.z, _e459_.w);
                    phi_4709_ = metal::float4(_e465_.x, _e465_.y, _e453_.z, _e465_.w);
                    if (_e392_.w == 0.0) {
                        phi_4709_ = metal::float4(_e453_.x, _e453_.y, _e453_.z, 1.0);
                    }
                    metal::float4 _e475_ = phi_4709_;
                    metal::float3 _e479_ = metal::float3(_e475_.w) - _e475_.xyz;
                    metal::float3 _e480_ = local_2_;
                    local_1_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e479_ / (_e480_ * _e475_.w)), metal::sign(_e479_), _e480_ == metal::float3(0.0, 0.0, 0.0));
                    break;
                }
                case 7: {
                    metal::float3 _e488_ = local_2_;
                    metal::float3 _e489_ = _e488_ * _e400_;
                    local_1_ = metal::select(_e489_, ((_e488_ + _e400_) - _e489_) - metal::float3(0.5, 0.5, 0.5), _e488_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                    break;
                }
                case 8: {
                    phi_4625_ = 0;
                    uint2 loop_bound = uint2(4294967295u);
                    bool loop_init = true;
                    while(true) {
                        if (metal::all(loop_bound == uint2(0u))) { break; }
                        loop_bound -= uint2(loop_bound.y == 0u, 1u);
                        if (!loop_init) {
                            phi_4625_ = as_type<int>(as_type<uint>(phi_4625_) + as_type<uint>(1));
                        }
                        loop_init = false;
                        int _e497_ = phi_4625_;
                        if (_e497_ < 3) {
                            float _e500_ = local_2_[metal::min(unsigned(_e497_), 2u)];
                            if (_e500_ <= 0.5) {
                                float _e503_ = local[metal::min(unsigned(_e497_), 2u)];
                                local_1_[metal::min(unsigned(_e497_), 2u)] = 1.0 - _e503_;
                            } else {
                                float _e507_ = local[metal::min(unsigned(_e497_), 2u)];
                                if (_e507_ <= 0.25) {
                                    float _e509_ = local[metal::min(unsigned(_e497_), 2u)];
                                    float _e512_ = local[metal::min(unsigned(_e497_), 2u)];
                                    local_1_[metal::min(unsigned(_e497_), 2u)] = (((16.0 * _e509_) - 12.0) * _e512_) + 3.0;
                                } else {
                                    float _e516_ = local[metal::min(unsigned(_e497_), 2u)];
                                    local_1_[metal::min(unsigned(_e497_), 2u)] = metal::rsqrt(_e516_) - 1.0;
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                    }
                    metal::float3 _e521_ = local_2_;
                    metal::float3 _e525_ = local_1_;
                    local_1_ = _e400_ + ((_e400_ * ((_e521_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e525_);
                    break;
                }
                case 9: {
                    metal::float3 _e528_ = local_2_;
                    local_1_ = metal::abs(_e400_ - _e528_);
                    break;
                }
                case 10: {
                    metal::float3 _e531_ = local_2_;
                    local_1_ = (_e531_ + _e400_) - ((_e531_ * 2.0) * _e400_);
                    break;
                }
                case 12: {
                    if (eh) {
                        metal::float3 _e536_ = local_2_;
                        metal::float3 _e537_ = metal::clamp(_e536_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                        local_2_ = _e537_;
                        metal::float3 _e552_ = _e537_ - metal::float3(metal::min(metal::min(_e537_.x, _e537_.y), _e537_.z));
                        metal::float3 _e560_ = _e552_ * ((metal::max(metal::max(_e400_.x, _e400_.y), _e400_.z) - metal::min(metal::min(_e400_.x, _e400_.y), _e400_.z)) / metal::max(0.000062, metal::max(metal::max(_e552_.x, _e552_.y), _e552_.z)));
                        float _e561_ = metal::dot(_e400_, metal::float3(0.3, 0.59, 0.11));
                        metal::float3 _e564_ = _e560_ - metal::float3(metal::dot(_e560_, metal::float3(0.3, 0.59, 0.11)));
                        metal::float2 _e577_ = metal::float2(_e561_, 1.0 - _e561_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e564_.x, _e564_.y), _e564_.z)), metal::max(metal::max(_e564_.x, _e564_.y), _e564_.z)));
                        local_1_ = (_e564_ * metal::min(1.0, metal::min(_e577_.x, _e577_.y))) + metal::float3(_e561_);
                    }
                    break;
                }
                case 13: {
                    if (eh) {
                        metal::float3 _e585_ = local_2_;
                        metal::float3 _e586_ = metal::clamp(_e585_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                        local_2_ = _e586_;
                        metal::float3 _e601_ = _e400_ - metal::float3(metal::min(metal::min(_e400_.x, _e400_.y), _e400_.z));
                        metal::float3 _e609_ = _e601_ * ((metal::max(metal::max(_e586_.x, _e586_.y), _e586_.z) - metal::min(metal::min(_e586_.x, _e586_.y), _e586_.z)) / metal::max(0.000062, metal::max(metal::max(_e601_.x, _e601_.y), _e601_.z)));
                        float _e610_ = metal::dot(_e400_, metal::float3(0.3, 0.59, 0.11));
                        metal::float3 _e613_ = _e609_ - metal::float3(metal::dot(_e609_, metal::float3(0.3, 0.59, 0.11)));
                        metal::float2 _e626_ = metal::float2(_e610_, 1.0 - _e610_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e613_.x, _e613_.y), _e613_.z)), metal::max(metal::max(_e613_.x, _e613_.y), _e613_.z)));
                        local_1_ = (_e613_ * metal::min(1.0, metal::min(_e626_.x, _e626_.y))) + metal::float3(_e610_);
                    }
                    break;
                }
                case 14: {
                    if (eh) {
                        metal::float3 _e634_ = local_2_;
                        metal::float3 _e635_ = metal::clamp(_e634_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                        local_2_ = _e635_;
                        float _e636_ = metal::dot(_e400_, metal::float3(0.3, 0.59, 0.11));
                        metal::float3 _e639_ = _e635_ - metal::float3(metal::dot(_e635_, metal::float3(0.3, 0.59, 0.11)));
                        metal::float2 _e652_ = metal::float2(_e636_, 1.0 - _e636_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e639_.x, _e639_.y), _e639_.z)), metal::max(metal::max(_e639_.x, _e639_.y), _e639_.z)));
                        local_1_ = (_e639_ * metal::min(1.0, metal::min(_e652_.x, _e652_.y))) + metal::float3(_e636_);
                    }
                    break;
                }
                case 15: {
                    if (eh) {
                        metal::float3 _e660_ = local_2_;
                        metal::float3 _e661_ = metal::clamp(_e660_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                        local_2_ = _e661_;
                        float _e662_ = metal::dot(_e661_, metal::float3(0.3, 0.59, 0.11));
                        metal::float3 _e665_ = _e400_ - metal::float3(metal::dot(_e400_, metal::float3(0.3, 0.59, 0.11)));
                        metal::float2 _e678_ = metal::float2(_e662_, 1.0 - _e662_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e665_.x, _e665_.y), _e665_.z)), metal::max(metal::max(_e665_.x, _e665_.y), _e665_.z)));
                        local_1_ = (_e665_ * metal::min(1.0, metal::min(_e678_.x, _e678_.y))) + metal::float3(_e662_);
                    }
                    break;
                }
                default: {
                    break;
                }
            }
            metal::float3 _e686_ = local_1_;
            metal::float3 _e688_ = metal::mix(_e393_, _e686_, metal::float3(_e392_.w));
            phi_4911_ = metal::float4(_e688_.x, _e688_.y, _e688_.z, _e372_);
        }
        metal::float4 _e694_ = phi_4911_;
        metal::float3 _e697_ = _e694_.xyz * _e694_.w;
        metal::float4 _e703_ = metal::float4(_e697_.x, _e694_.y, _e694_.z, _e694_.w);
        metal::float4 _e709_ = metal::float4(_e703_.x, _e697_.y, _e703_.z, _e703_.w);
        phi_4943_ = _e368_;
        phi_4936_ = metal::float4(_e709_.x, _e709_.y, _e697_.z, _e709_.w);
    }
    uint _e717_ = phi_4943_;
    metal::float4 _e719_ = phi_4936_;
    metal::float3 _e720_ = _e719_.xyz;
    float _e723_ = n.z3_;
    float _e725_ = n.A3_;
    if (fh) {
        local_2 = _e719_.w != 0.0;
    } else {
        local_2 = false;
    }
    bool _e1022 = local_2;
    if (_e1022) {
        phi_4938_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e93_.x) + (0.00583715 * _e93_.y))) * _e723_) + _e725_) + _e720_;
    } else {
        phi_4938_ = _e720_;
    }
    metal::float3 _e741_ = phi_4938_;
    metal::float4 _e747_ = metal::float4(_e741_.x, _e719_.y, _e719_.z, _e719_.w);
    metal::float4 _e753_ = metal::float4(_e747_.x, _e741_.y, _e747_.z, _e747_.w);
    metal::float4 _e759_ = metal::float4(_e753_.x, _e753_.y, _e741_.z, _e753_.w);
    switch(as_type<int>(0u)) {
        default: {
            if (_e719_.w == 0.0) {
                break;
            }
            float _e762_ = 1.0 - _e719_.w;
            phi_4940_ = _e759_;
            if (_e762_ != 0.0) {
                uint _e766_ = j0_.c2_[metal::min(unsigned(_e128_), (_buffer_sizes.size8 - 0 - 4) / 4)];
                phi_4940_ = _e759_ + (metal::unpack_unorm4x8_to_float(_e766_) * _e762_);
            }
            metal::float4 _e771_ = phi_4940_;
            j0_.c2_[metal::min(unsigned(_e128_), (_buffer_sizes.size8 - 0 - 4) / 4)] = metal::pack_float_to_unorm4x8(_e771_);
            break;
        }
    }
    if (_e717_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e128_), (_buffer_sizes.size3 - 0 - 4) / 4)] = _e717_;
    }
    return;
}

struct main_Input {
    metal::float4 O [[user(loc0), center_perspective]];
    uint B0_ [[user(loc1), flat]];
};
fragment void main_(
  main_Input varyings [[stage_in]]
, metal::float4 gl_FragCoord [[position]]
, metal::texture2d<float, metal::access::sample> XC [[texture(1)]]
, metal::sampler aa [[sampler(2)]]
, device Je const& AD [[buffer(2)]]
, device h0Bd& h0_ [[buffer(5)]]
, device Ke const& RB [[buffer(3)]]
, metal::texture2d<float, metal::access::sample> KD [[texture(0)]]
, metal::sampler Mb [[sampler(1)]]
, device j0Bd& j0_ [[buffer(4)]]
, constant CC& n [[buffer(0)]]
, device q4Bd_1_& q4_ [[buffer(6)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 O_1_ = {};
    uint B0_1_ = {};
    const auto O = varyings.O;
    const auto B0_ = varyings.B0_;
    gl_FragCoord_1_ = gl_FragCoord;
    O_1_ = O;
    B0_1_ = B0_;
    main_1_(XC, aa, AD, h0_, RB, gl_FragCoord_1_, KD, Mb, j0_, n, O_1_, B0_1_, q4_, _buffer_sizes);
    return;
}
