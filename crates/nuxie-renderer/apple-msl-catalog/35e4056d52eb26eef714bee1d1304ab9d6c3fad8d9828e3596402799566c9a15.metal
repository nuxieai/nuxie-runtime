// language: metal2.4
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
    metal::texture2d<float, metal::access::sample> BD,
    metal::sampler Q9_,
    thread metal::float2& C2_1_,
    constant _mslBufferSizes& _buffer_sizes
) {
    metal::float3 local = {};
    metal::float3 local_1_ = {};
    metal::float3 local_2_ = {};
    bool phi_1442_ = {};
    float phi_3464_ = {};
    float phi_3463_ = {};
    float phi_3465_ = {};
    float phi_3468_ = {};
    float phi_3467_ = {};
    bool phi_1479_ = {};
    float phi_3470_ = {};
    uint phi_4116_ = {};
    float phi_3469_ = {};
    uint phi_4115_ = {};
    metal::float4 phi_3491_ = {};
    bool phi_1598_ = {};
    uint phi_3495_ = {};
    bool phi_1607_ = {};
    float phi_3509_ = {};
    metal::float4 phi_3963_ = {};
    int phi_3903_ = {};
    metal::float4 phi_4111_ = {};
    metal::float3 phi_4136_ = {};
    metal::float4 phi_4138_ = {};
    bool local_1 = {};
    bool local_2 = {};
    metal::float4 _e79_ = gl_FragCoord_1_;
    metal::float2 _e80_ = _e79_.xy;
    metal::uint2 _e83_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e80_)));
    uint _e85_ = n.m6_;
    int _e114_ = as_type<int>(((((_e83_.y >> as_type<uint>(5u)) * (((_e85_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e83_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e83_.x & 28u) << as_type<uint>(5u)) + ((_e83_.y & 28u) << as_type<uint>(2)))) + (((_e83_.y & 3u) << as_type<uint>(2)) + (_e83_.x & 3u)));
    uint _e117_ = q4_.c2_[metal::min(unsigned(_e114_), (_buffer_sizes.size8 - 0 - 4) / 4)];
    uint _e119_ = _e117_ >> as_type<uint>(17u);
    uint _e120_ = B0_1_;
    metal::float2 _e124_ = C2_1_;
    metal::float4 _e125_ = BD.sample(Q9_, _e124_, metal::level(0.0));
    q4_.c2_[metal::min(unsigned(_e114_), (_buffer_sizes.size8 - 0 - 4) / 4)] = ((_e120_ << as_type<uint>(17u)) + 65536u) + as_type<uint>(naga_f2i32(metal::rint(metal::clamp(_e125_.x, 0.0, 1.0) * 2048.0)));
    float _e136_ = (static_cast<float>(_e117_ & 131071u) * 0.00048828125) + -32.0;
    metal::uint2 _e139_ = AD.c2_[metal::min(unsigned(_e119_), (_buffer_sizes.size0 - 0 - 8) / 8)];
    phi_3463_ = _e136_;
    if ((_e139_.x & 768u) != 0u) {
        float _e143_ = metal::abs(_e136_);
        phi_1442_ = ch;
        if (ch) {
            phi_1442_ = (_e139_.x & 512u) != 0u;
        }
        bool _e147_ = phi_1442_;
        phi_3464_ = _e143_;
        if (_e147_) {
            phi_3464_ = 1.0 - metal::abs((metal::fract(_e143_ * 0.5) * 2.0) + -1.0);
        }
        float _e155_ = phi_3464_;
        phi_3463_ = _e155_;
    }
    float _e157_ = phi_3463_;
    float _e158_ = metal::clamp(_e157_, 0.0, 1.0);
    phi_3467_ = _e158_;
    if (Yg) {
        uint _e160_ = _e139_.x >> as_type<uint>(16u);
        phi_3468_ = _e158_;
        if (_e160_ != 0u) {
            uint _e164_ = h0_.c2_[metal::min(unsigned(_e114_), (_buffer_sizes.size1 - 0 - 4) / 4)];
            if (_e160_ == (_e164_ >> as_type<uint>(16))) {
                phi_3465_ = metal::min(_e158_, float2(as_type<half2>(_e164_)).x);
            } else {
                phi_3465_ = 0.0;
            }
            float _e172_ = phi_3465_;
            phi_3468_ = _e172_;
        }
        float _e174_ = phi_3468_;
        phi_3467_ = _e174_;
    }
    float _e176_ = phi_3467_;
    phi_1479_ = Zg;
    if (Zg) {
        phi_1479_ = (_e139_.x & 1024u) != 0u;
    }
    bool _e180_ = phi_1479_;
    phi_3470_ = _e176_;
    if (_e180_) {
        uint _e181_ = _e119_ * 4u;
        metal::float4 _e185_ = RB.c2_[metal::min(unsigned(_e181_ + 2u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e196_ = RB.c2_[metal::min(unsigned(_e181_ + 3u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e201_ = _e196_.zw;
        metal::float2 _e203_ = (metal::abs((metal::float2x2(metal::float2(_e185_.x, _e185_.y), metal::float2(_e185_.z, _e185_.w)) * _e80_) + _e196_.xy) * _e201_) - _e201_;
        phi_3470_ = metal::min(_e176_, metal::clamp(metal::min(_e203_.x, _e203_.y) + 0.5, 0.0, 1.0));
    }
    float _e211_ = phi_3470_;
    uint _e212_ = _e139_.x & 15u;
    if (_e212_ <= 1u) {
        if (Yg) {
            local_1 = _e212_ == 0u;
        } else {
            local_1 = false;
        }
        bool _e217_ = local_1;
        phi_4116_ = 0u;
        if (_e217_) {
            phi_4116_ = _e139_.y | as_type<uint>(half2(metal::float2(_e211_, 0.0)));
        }
        uint _e222_ = phi_4116_;
        phi_4115_ = _e222_;
        phi_3491_ = metal::select(metal::unpack_unorm4x8_to_float(_e139_.y), metal::float4(0.0, 0.0, 0.0, 0.0), metal::bool4(_e217_));
    } else {
        uint _e225_ = _e119_ * 4u;
        metal::float4 _e228_ = RB.c2_[metal::min(unsigned(_e225_), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float4 _e239_ = RB.c2_[metal::min(unsigned(_e225_ + 1u), (_buffer_sizes.size2 - 0 - 16) / 16)];
        metal::float2 _e242_ = (metal::float2x2(metal::float2(_e228_.x, _e228_.y), metal::float2(_e228_.z, _e228_.w)) * _e80_) + _e239_.xy;
        if (_e212_ == 2u) {
            phi_3469_ = _e242_.x;
        } else {
            phi_3469_ = metal::length(_e242_);
        }
        float _e247_ = phi_3469_;
        metal::float4 _e256_ = KD.sample(Mb, metal::float2((metal::clamp(_e247_, 0.0, 1.0) * _e239_.z) + _e239_.w, as_type<float>(_e139_.y)), metal::level(0.0));
        phi_4115_ = 0u;
        phi_3491_ = _e256_;
    }
    uint _e258_ = phi_4115_;
    metal::float4 _e260_ = phi_3491_;
    float _e262_ = _e260_.w * _e211_;
    metal::float4 _e267_ = metal::float4(_e260_.x, _e260_.y, _e260_.z, _e262_);
    phi_1598_ = ah;
    if (ah) {
        phi_1598_ = _e262_ != 0.0;
    }
    bool _e270_ = phi_1598_;
    phi_3495_ = uint {};
    phi_1607_ = _e270_;
    if (_e270_) {
        uint _e273_ = (_e139_.x >> as_type<uint>(4)) & 15u;
        phi_3495_ = _e273_;
        phi_1607_ = _e273_ != 0u;
    }
    uint _e276_ = phi_3495_;
    bool _e278_ = phi_1607_;
    phi_4111_ = _e267_;
    if (_e278_) {
        uint _e281_ = j0_.c2_[metal::min(unsigned(_e114_), (_buffer_sizes.size6 - 0 - 4) / 4)];
        metal::float4 _e282_ = metal::unpack_unorm4x8_to_float(_e281_);
        metal::float3 _e283_ = _e267_.xyz;
        local_2_ = _e283_;
        metal::float3 _e284_ = _e282_.xyz;
        if (_e282_.w != 0.0) {
            phi_3509_ = 1.0 / _e282_.w;
        } else {
            phi_3509_ = 0.0;
        }
        float _e289_ = phi_3509_;
        metal::float3 _e290_ = _e284_ * _e289_;
        local = _e290_;
        switch(as_type<int>(_e276_)) {
            case 11: {
                metal::float3 _e292_ = local_2_;
                local_1_ = _e292_ * _e290_;
                break;
            }
            case 1: {
                metal::float3 _e294_ = local_2_;
                local_1_ = (_e294_ + _e290_) - (_e294_ * _e290_);
                break;
            }
            case 2: {
                metal::float3 _e298_ = local_2_;
                metal::float3 _e299_ = _e298_ * _e290_;
                local_1_ = metal::select(_e299_, ((_e298_ + _e290_) - _e299_) - metal::float3(0.5, 0.5, 0.5), _e290_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 3: {
                metal::float3 _e306_ = local_2_;
                local_1_ = metal::min(_e306_, _e290_);
                break;
            }
            case 4: {
                metal::float3 _e308_ = local_2_;
                local_1_ = metal::max(_e308_, _e290_);
                break;
            }
            case 5: {
                metal::float3 _e311_ = metal::clamp(_e284_, metal::float3(0.0, 0.0, 0.0), _e282_.www);
                metal::float4 _e317_ = metal::float4(_e311_.x, float {}, float {}, float {});
                metal::float4 _e323_ = metal::float4(_e317_.x, _e311_.y, _e317_.z, _e317_.w);
                metal::float3 _e330_ = local_2_;
                metal::float3 _e333_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e330_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e282_.w;
                metal::float3 _e334_ = metal::float4(_e323_.x, _e323_.y, _e311_.z, _e323_.w).xyz;
                local_1_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e334_ / _e333_), metal::sign(_e334_), _e333_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 6: {
                metal::float3 _e340_ = local_2_;
                local_2_ = metal::clamp(_e340_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                metal::float3 _e343_ = metal::clamp(_e284_, metal::float3(0.0, 0.0, 0.0), _e282_.www);
                metal::float4 _e349_ = metal::float4(_e343_.x, _e282_.y, _e282_.z, _e282_.w);
                metal::float4 _e355_ = metal::float4(_e349_.x, _e343_.y, _e349_.z, _e349_.w);
                phi_3963_ = metal::float4(_e355_.x, _e355_.y, _e343_.z, _e355_.w);
                if (_e282_.w == 0.0) {
                    phi_3963_ = metal::float4(_e343_.x, _e343_.y, _e343_.z, 1.0);
                }
                metal::float4 _e365_ = phi_3963_;
                metal::float3 _e369_ = metal::float3(_e365_.w) - _e365_.xyz;
                metal::float3 _e370_ = local_2_;
                local_1_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e369_ / (_e370_ * _e365_.w)), metal::sign(_e369_), _e370_ == metal::float3(0.0, 0.0, 0.0));
                break;
            }
            case 7: {
                metal::float3 _e378_ = local_2_;
                metal::float3 _e379_ = _e378_ * _e290_;
                local_1_ = metal::select(_e379_, ((_e378_ + _e290_) - _e379_) - metal::float3(0.5, 0.5, 0.5), _e378_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
                break;
            }
            case 8: {
                phi_3903_ = 0;
                uint2 loop_bound = uint2(4294967295u);
                bool loop_init = true;
                while(true) {
                    if (metal::all(loop_bound == uint2(0u))) { break; }
                    loop_bound -= uint2(loop_bound.y == 0u, 1u);
                    if (!loop_init) {
                        phi_3903_ = as_type<int>(as_type<uint>(phi_3903_) + as_type<uint>(1));
                    }
                    loop_init = false;
                    int _e387_ = phi_3903_;
                    if (_e387_ < 3) {
                        float _e390_ = local_2_[metal::min(unsigned(_e387_), 2u)];
                        if (_e390_ <= 0.5) {
                            float _e393_ = local[metal::min(unsigned(_e387_), 2u)];
                            local_1_[metal::min(unsigned(_e387_), 2u)] = 1.0 - _e393_;
                        } else {
                            float _e397_ = local[metal::min(unsigned(_e387_), 2u)];
                            if (_e397_ <= 0.25) {
                                float _e399_ = local[metal::min(unsigned(_e387_), 2u)];
                                float _e402_ = local[metal::min(unsigned(_e387_), 2u)];
                                local_1_[metal::min(unsigned(_e387_), 2u)] = (((16.0 * _e399_) - 12.0) * _e402_) + 3.0;
                            } else {
                                float _e406_ = local[metal::min(unsigned(_e387_), 2u)];
                                local_1_[metal::min(unsigned(_e387_), 2u)] = metal::rsqrt(_e406_) - 1.0;
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                }
                metal::float3 _e411_ = local_2_;
                metal::float3 _e415_ = local_1_;
                local_1_ = _e290_ + ((_e290_ * ((_e411_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e415_);
                break;
            }
            case 9: {
                metal::float3 _e418_ = local_2_;
                local_1_ = metal::abs(_e290_ - _e418_);
                break;
            }
            case 10: {
                metal::float3 _e421_ = local_2_;
                local_1_ = (_e421_ + _e290_) - ((_e421_ * 2.0) * _e290_);
                break;
            }
            case 12: {
                if (eh) {
                    metal::float3 _e426_ = local_2_;
                    metal::float3 _e427_ = metal::clamp(_e426_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e427_;
                    metal::float3 _e442_ = _e427_ - metal::float3(metal::min(metal::min(_e427_.x, _e427_.y), _e427_.z));
                    metal::float3 _e450_ = _e442_ * ((metal::max(metal::max(_e290_.x, _e290_.y), _e290_.z) - metal::min(metal::min(_e290_.x, _e290_.y), _e290_.z)) / metal::max(0.000062, metal::max(metal::max(_e442_.x, _e442_.y), _e442_.z)));
                    float _e451_ = metal::dot(_e290_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e454_ = _e450_ - metal::float3(metal::dot(_e450_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e467_ = metal::float2(_e451_, 1.0 - _e451_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e454_.x, _e454_.y), _e454_.z)), metal::max(metal::max(_e454_.x, _e454_.y), _e454_.z)));
                    local_1_ = (_e454_ * metal::min(1.0, metal::min(_e467_.x, _e467_.y))) + metal::float3(_e451_);
                }
                break;
            }
            case 13: {
                if (eh) {
                    metal::float3 _e475_ = local_2_;
                    metal::float3 _e476_ = metal::clamp(_e475_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e476_;
                    metal::float3 _e491_ = _e290_ - metal::float3(metal::min(metal::min(_e290_.x, _e290_.y), _e290_.z));
                    metal::float3 _e499_ = _e491_ * ((metal::max(metal::max(_e476_.x, _e476_.y), _e476_.z) - metal::min(metal::min(_e476_.x, _e476_.y), _e476_.z)) / metal::max(0.000062, metal::max(metal::max(_e491_.x, _e491_.y), _e491_.z)));
                    float _e500_ = metal::dot(_e290_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e503_ = _e499_ - metal::float3(metal::dot(_e499_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e516_ = metal::float2(_e500_, 1.0 - _e500_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e503_.x, _e503_.y), _e503_.z)), metal::max(metal::max(_e503_.x, _e503_.y), _e503_.z)));
                    local_1_ = (_e503_ * metal::min(1.0, metal::min(_e516_.x, _e516_.y))) + metal::float3(_e500_);
                }
                break;
            }
            case 14: {
                if (eh) {
                    metal::float3 _e524_ = local_2_;
                    metal::float3 _e525_ = metal::clamp(_e524_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e525_;
                    float _e526_ = metal::dot(_e290_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e529_ = _e525_ - metal::float3(metal::dot(_e525_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e542_ = metal::float2(_e526_, 1.0 - _e526_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e529_.x, _e529_.y), _e529_.z)), metal::max(metal::max(_e529_.x, _e529_.y), _e529_.z)));
                    local_1_ = (_e529_ * metal::min(1.0, metal::min(_e542_.x, _e542_.y))) + metal::float3(_e526_);
                }
                break;
            }
            case 15: {
                if (eh) {
                    metal::float3 _e550_ = local_2_;
                    metal::float3 _e551_ = metal::clamp(_e550_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                    local_2_ = _e551_;
                    float _e552_ = metal::dot(_e551_, metal::float3(0.3, 0.59, 0.11));
                    metal::float3 _e555_ = _e290_ - metal::float3(metal::dot(_e290_, metal::float3(0.3, 0.59, 0.11)));
                    metal::float2 _e568_ = metal::float2(_e552_, 1.0 - _e552_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e555_.x, _e555_.y), _e555_.z)), metal::max(metal::max(_e555_.x, _e555_.y), _e555_.z)));
                    local_1_ = (_e555_ * metal::min(1.0, metal::min(_e568_.x, _e568_.y))) + metal::float3(_e552_);
                }
                break;
            }
            default: {
                break;
            }
        }
        metal::float3 _e576_ = local_1_;
        metal::float3 _e578_ = metal::mix(_e283_, _e576_, metal::float3(_e282_.w));
        phi_4111_ = metal::float4(_e578_.x, _e578_.y, _e578_.z, _e262_);
    }
    metal::float4 _e584_ = phi_4111_;
    metal::float3 _e587_ = _e584_.xyz * _e584_.w;
    metal::float4 _e593_ = metal::float4(_e587_.x, _e584_.y, _e584_.z, _e584_.w);
    metal::float4 _e599_ = metal::float4(_e593_.x, _e587_.y, _e593_.z, _e593_.w);
    metal::float4 _e605_ = metal::float4(_e599_.x, _e599_.y, _e587_.z, _e599_.w);
    metal::float3 _e606_ = _e605_.xyz;
    float _e608_ = n.z3_;
    float _e610_ = n.A3_;
    if (fh) {
        local_2 = _e584_.w != 0.0;
    } else {
        local_2 = false;
    }
    bool _e849 = local_2;
    if (_e849) {
        phi_4136_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e79_.x) + (0.00583715 * _e79_.y))) * _e608_) + _e610_) + _e606_;
    } else {
        phi_4136_ = _e606_;
    }
    metal::float3 _e626_ = phi_4136_;
    metal::float4 _e632_ = metal::float4(_e626_.x, _e605_.y, _e605_.z, _e605_.w);
    metal::float4 _e638_ = metal::float4(_e632_.x, _e626_.y, _e632_.z, _e632_.w);
    metal::float4 _e644_ = metal::float4(_e638_.x, _e638_.y, _e626_.z, _e638_.w);
    switch(as_type<int>(0u)) {
        default: {
            if (_e584_.w == 0.0) {
                break;
            }
            float _e647_ = 1.0 - _e584_.w;
            phi_4138_ = _e644_;
            if (_e647_ != 0.0) {
                uint _e651_ = j0_.c2_[metal::min(unsigned(_e114_), (_buffer_sizes.size6 - 0 - 4) / 4)];
                phi_4138_ = _e644_ + (metal::unpack_unorm4x8_to_float(_e651_) * _e647_);
            }
            metal::float4 _e656_ = phi_4138_;
            j0_.c2_[metal::min(unsigned(_e114_), (_buffer_sizes.size6 - 0 - 4) / 4)] = metal::pack_float_to_unorm4x8(_e656_);
            break;
        }
    }
    if (_e258_ != 0u) {
        h0_.c2_[metal::min(unsigned(_e114_), (_buffer_sizes.size1 - 0 - 4) / 4)] = _e258_;
    }
    return;
}

struct main_Input {
    uint B0_ [[user(loc1), flat]];
    metal::float2 C2_ [[user(loc0), center_perspective]];
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
, metal::texture2d<float, metal::access::sample> BD [[texture(2)]]
, metal::sampler Q9_ [[sampler(3)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    uint B0_1_ = {};
    metal::float2 C2_1_ = {};
    const auto B0_ = varyings.B0_;
    const auto C2_ = varyings.C2_;
    gl_FragCoord_1_ = gl_FragCoord;
    B0_1_ = B0_;
    C2_1_ = C2_;
    main_1_(AD, h0_, RB, gl_FragCoord_1_, KD, Mb, j0_, n, q4_, B0_1_, BD, Q9_, C2_1_, _buffer_sizes);
    return;
}
