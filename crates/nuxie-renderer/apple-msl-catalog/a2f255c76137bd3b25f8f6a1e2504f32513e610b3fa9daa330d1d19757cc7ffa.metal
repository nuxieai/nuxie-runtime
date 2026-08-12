// language: metal4.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

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
constant bool fh = true;
constant bool eh = true;
constant bool ah = true;

metal::int2 naga_f2i32(metal::float2 value) {
    return static_cast<metal::int2>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

uint naga_f2u32(float value) {
    return static_cast<uint>(metal::clamp(value, 0.0, 4294967000.0));
}

void main_1_(
    metal::texture2d<float, metal::access::sample> KD,
    metal::sampler Mb,
    metal::texture2d<float, metal::access::sample> IC,
    metal::sampler S5_,
    metal::texture2d<float, metal::access::sample> BD,
    metal::sampler Q9_,
    thread metal::float2& C2_1_,
    thread metal::float4& f1_1_,
    thread float& e2_1_,
    metal::texture2d<float, metal::access::sample> SD,
    thread metal::float4& gl_FragCoord_1_,
    constant CC& n,
    thread metal::float4& Jg
) {
    metal::float3 local = {};
    metal::float3 local_1_ = {};
    metal::float3 local_2_ = {};
    metal::float4 phi_2839_ = {};
    float phi_2836_ = {};
    float phi_2837_ = {};
    metal::float4 phi_2841_ = {};
    float phi_2832_ = {};
    metal::float4 phi_2842_ = {};
    metal::float4 phi_2840_ = {};
    metal::float4 phi_2838_ = {};
    float phi_2843_ = {};
    metal::float4 phi_3138_ = {};
    int phi_3098_ = {};
    metal::float3 phi_3241_ = {};
    bool local_1 = {};
    metal::float2 _e53_ = C2_1_;
    metal::float4 _e54_ = BD.sample(Q9_, _e53_, metal::level(0.0));
    float _e56_ = metal::clamp(_e54_.x, 0.0, 1.0);
    metal::float4 _e57_ = f1_1_;
    if (_e57_.w >= 0.0) {
        if (ah) {
            phi_2839_ = metal::float4(_e57_.x, _e57_.y, _e57_.z, _e57_.w * _e56_);
        } else {
            phi_2839_ = _e57_ * _e56_;
        }
        metal::float4 _e69_ = phi_2839_;
        phi_2838_ = _e69_;
    } else {
        if (_e57_.w > -1.0) {
            if (_e57_.z > 0.0) {
                phi_2836_ = _e57_.x;
            } else {
                phi_2836_ = metal::length(_e57_.xy);
            }
            float _e77_ = phi_2836_;
            float _e78_ = metal::clamp(_e77_, 0.0, 1.0);
            float _e79_ = metal::abs(_e57_.z);
            if (_e79_ > 1.0) {
                phi_2837_ = (0.9980469 * _e78_) + 0.0009765625;
            } else {
                phi_2837_ = (0.001953125 * _e78_) + _e79_;
            }
            float _e86_ = phi_2837_;
            metal::float4 _e89_ = KD.sample(Mb, metal::float2(_e86_, -(_e57_.w)), metal::level(0.0));
            float _e91_ = _e89_.w * _e56_;
            metal::float4 _e96_ = metal::float4(_e89_.x, _e89_.y, _e89_.z, _e91_);
            if (ah) {
                phi_2841_ = _e96_;
            } else {
                metal::float3 _e98_ = _e96_.xyz * _e91_;
                phi_2841_ = metal::float4(_e98_.x, _e98_.y, _e98_.z, _e91_);
            }
            metal::float4 _e104_ = phi_2841_;
            phi_2840_ = _e104_;
        } else {
            metal::float4 _e107_ = IC.sample(S5_, _e57_.xy, metal::level(-2.0 - _e57_.w));
            float _e109_ = _e57_.z * _e56_;
            if (ah) {
                if (_e107_.w != 0.0) {
                    phi_2832_ = 1.0 / _e107_.w;
                } else {
                    phi_2832_ = 0.0;
                }
                float _e115_ = phi_2832_;
                metal::float3 _e116_ = _e107_.xyz * _e115_;
                phi_2842_ = metal::float4(_e116_.x, _e116_.y, _e116_.z, _e107_.w * _e109_);
            } else {
                phi_2842_ = _e107_ * _e109_;
            }
            metal::float4 _e124_ = phi_2842_;
            phi_2840_ = _e124_;
        }
        metal::float4 _e126_ = phi_2840_;
        phi_2838_ = _e126_;
    }
    metal::float4 _e128_ = phi_2838_;
    float _e129_ = e2_1_;
    metal::float4 _e131_ = gl_FragCoord_1_;
    uint clamped_lod_e124 = metal::min(uint(0), SD.get_num_mip_levels() - 1);
    metal::float4 _e135_ = SD.read(metal::min(metal::uint2(naga_f2i32(metal::floor(_e131_.xy))), metal::uint2(SD.get_width(clamped_lod_e124), SD.get_height(clamped_lod_e124)) - 1), clamped_lod_e124);
    metal::float3 _e136_ = _e128_.xyz;
    local_2_ = _e136_;
    metal::float3 _e137_ = _e135_.xyz;
    if (_e135_.w != 0.0) {
        phi_2843_ = 1.0 / _e135_.w;
    } else {
        phi_2843_ = 0.0;
    }
    float _e142_ = phi_2843_;
    metal::float3 _e143_ = _e137_ * _e142_;
    local = _e143_;
    switch(as_type<int>(naga_f2u32(_e129_))) {
        case 11: {
            metal::float3 _e145_ = local_2_;
            local_1_ = _e145_ * _e143_;
            break;
        }
        case 1: {
            metal::float3 _e147_ = local_2_;
            local_1_ = (_e147_ + _e143_) - (_e147_ * _e143_);
            break;
        }
        case 2: {
            metal::float3 _e151_ = local_2_;
            metal::float3 _e152_ = _e151_ * _e143_;
            local_1_ = metal::select(_e152_, ((_e151_ + _e143_) - _e152_) - metal::float3(0.5, 0.5, 0.5), _e143_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
            break;
        }
        case 3: {
            metal::float3 _e159_ = local_2_;
            local_1_ = metal::min(_e159_, _e143_);
            break;
        }
        case 4: {
            metal::float3 _e161_ = local_2_;
            local_1_ = metal::max(_e161_, _e143_);
            break;
        }
        case 5: {
            metal::float3 _e164_ = metal::clamp(_e137_, metal::float3(0.0, 0.0, 0.0), _e135_.www);
            metal::float4 _e170_ = metal::float4(_e164_.x, float {}, float {}, float {});
            metal::float4 _e176_ = metal::float4(_e170_.x, _e164_.y, _e170_.z, _e170_.w);
            metal::float3 _e183_ = local_2_;
            metal::float3 _e186_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e183_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e135_.w;
            metal::float3 _e187_ = metal::float4(_e176_.x, _e176_.y, _e164_.z, _e176_.w).xyz;
            local_1_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e187_ / _e186_), metal::sign(_e187_), _e186_ == metal::float3(0.0, 0.0, 0.0));
            break;
        }
        case 6: {
            metal::float3 _e193_ = local_2_;
            local_2_ = metal::clamp(_e193_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
            metal::float3 _e196_ = metal::clamp(_e137_, metal::float3(0.0, 0.0, 0.0), _e135_.www);
            metal::float4 _e202_ = metal::float4(_e196_.x, _e135_.y, _e135_.z, _e135_.w);
            metal::float4 _e208_ = metal::float4(_e202_.x, _e196_.y, _e202_.z, _e202_.w);
            phi_3138_ = metal::float4(_e208_.x, _e208_.y, _e196_.z, _e208_.w);
            if (_e135_.w == 0.0) {
                phi_3138_ = metal::float4(_e196_.x, _e196_.y, _e196_.z, 1.0);
            }
            metal::float4 _e218_ = phi_3138_;
            metal::float3 _e222_ = metal::float3(_e218_.w) - _e218_.xyz;
            metal::float3 _e223_ = local_2_;
            local_1_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e222_ / (_e223_ * _e218_.w)), metal::sign(_e222_), _e223_ == metal::float3(0.0, 0.0, 0.0));
            break;
        }
        case 7: {
            metal::float3 _e231_ = local_2_;
            metal::float3 _e232_ = _e231_ * _e143_;
            local_1_ = metal::select(_e232_, ((_e231_ + _e143_) - _e232_) - metal::float3(0.5, 0.5, 0.5), _e231_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
            break;
        }
        case 8: {
            phi_3098_ = 0;
            uint2 loop_bound = uint2(4294967295u);
            bool loop_init = true;
            while(true) {
                if (metal::all(loop_bound == uint2(0u))) { break; }
                loop_bound -= uint2(loop_bound.y == 0u, 1u);
                if (!loop_init) {
                    phi_3098_ = as_type<int>(as_type<uint>(phi_3098_) + as_type<uint>(1));
                }
                loop_init = false;
                int _e240_ = phi_3098_;
                if (_e240_ < 3) {
                    float _e243_ = local_2_[metal::min(unsigned(_e240_), 2u)];
                    if (_e243_ <= 0.5) {
                        float _e246_ = local[metal::min(unsigned(_e240_), 2u)];
                        local_1_[metal::min(unsigned(_e240_), 2u)] = 1.0 - _e246_;
                    } else {
                        float _e250_ = local[metal::min(unsigned(_e240_), 2u)];
                        if (_e250_ <= 0.25) {
                            float _e252_ = local[metal::min(unsigned(_e240_), 2u)];
                            float _e255_ = local[metal::min(unsigned(_e240_), 2u)];
                            local_1_[metal::min(unsigned(_e240_), 2u)] = (((16.0 * _e252_) - 12.0) * _e255_) + 3.0;
                        } else {
                            float _e259_ = local[metal::min(unsigned(_e240_), 2u)];
                            local_1_[metal::min(unsigned(_e240_), 2u)] = metal::rsqrt(_e259_) - 1.0;
                        }
                    }
                    continue;
                } else {
                    break;
                }
            }
            metal::float3 _e264_ = local_2_;
            metal::float3 _e268_ = local_1_;
            local_1_ = _e143_ + ((_e143_ * ((_e264_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e268_);
            break;
        }
        case 9: {
            metal::float3 _e271_ = local_2_;
            local_1_ = metal::abs(_e143_ - _e271_);
            break;
        }
        case 10: {
            metal::float3 _e274_ = local_2_;
            local_1_ = (_e274_ + _e143_) - ((_e274_ * 2.0) * _e143_);
            break;
        }
        case 12: {
            if (eh) {
                metal::float3 _e279_ = local_2_;
                metal::float3 _e280_ = metal::clamp(_e279_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e280_;
                metal::float3 _e295_ = _e280_ - metal::float3(metal::min(metal::min(_e280_.x, _e280_.y), _e280_.z));
                metal::float3 _e303_ = _e295_ * ((metal::max(metal::max(_e143_.x, _e143_.y), _e143_.z) - metal::min(metal::min(_e143_.x, _e143_.y), _e143_.z)) / metal::max(0.000062, metal::max(metal::max(_e295_.x, _e295_.y), _e295_.z)));
                float _e304_ = metal::dot(_e143_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e307_ = _e303_ - metal::float3(metal::dot(_e303_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e320_ = metal::float2(_e304_, 1.0 - _e304_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e307_.x, _e307_.y), _e307_.z)), metal::max(metal::max(_e307_.x, _e307_.y), _e307_.z)));
                local_1_ = (_e307_ * metal::min(1.0, metal::min(_e320_.x, _e320_.y))) + metal::float3(_e304_);
            }
            break;
        }
        case 13: {
            if (eh) {
                metal::float3 _e328_ = local_2_;
                metal::float3 _e329_ = metal::clamp(_e328_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e329_;
                metal::float3 _e344_ = _e143_ - metal::float3(metal::min(metal::min(_e143_.x, _e143_.y), _e143_.z));
                metal::float3 _e352_ = _e344_ * ((metal::max(metal::max(_e329_.x, _e329_.y), _e329_.z) - metal::min(metal::min(_e329_.x, _e329_.y), _e329_.z)) / metal::max(0.000062, metal::max(metal::max(_e344_.x, _e344_.y), _e344_.z)));
                float _e353_ = metal::dot(_e143_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e356_ = _e352_ - metal::float3(metal::dot(_e352_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e369_ = metal::float2(_e353_, 1.0 - _e353_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e356_.x, _e356_.y), _e356_.z)), metal::max(metal::max(_e356_.x, _e356_.y), _e356_.z)));
                local_1_ = (_e356_ * metal::min(1.0, metal::min(_e369_.x, _e369_.y))) + metal::float3(_e353_);
            }
            break;
        }
        case 14: {
            if (eh) {
                metal::float3 _e377_ = local_2_;
                metal::float3 _e378_ = metal::clamp(_e377_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e378_;
                float _e379_ = metal::dot(_e143_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e382_ = _e378_ - metal::float3(metal::dot(_e378_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e395_ = metal::float2(_e379_, 1.0 - _e379_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e382_.x, _e382_.y), _e382_.z)), metal::max(metal::max(_e382_.x, _e382_.y), _e382_.z)));
                local_1_ = (_e382_ * metal::min(1.0, metal::min(_e395_.x, _e395_.y))) + metal::float3(_e379_);
            }
            break;
        }
        case 15: {
            if (eh) {
                metal::float3 _e403_ = local_2_;
                metal::float3 _e404_ = metal::clamp(_e403_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e404_;
                float _e405_ = metal::dot(_e404_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e408_ = _e143_ - metal::float3(metal::dot(_e143_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e421_ = metal::float2(_e405_, 1.0 - _e405_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e408_.x, _e408_.y), _e408_.z)), metal::max(metal::max(_e408_.x, _e408_.y), _e408_.z)));
                local_1_ = (_e408_ * metal::min(1.0, metal::min(_e421_.x, _e421_.y))) + metal::float3(_e405_);
            }
            break;
        }
        default: {
            break;
        }
    }
    metal::float3 _e429_ = local_1_;
    metal::float3 _e431_ = metal::mix(_e136_, _e429_, metal::float3(_e135_.w));
    metal::float4 _e437_ = metal::float4(_e431_.x, _e128_.y, _e128_.z, _e128_.w);
    metal::float4 _e443_ = metal::float4(_e437_.x, _e431_.y, _e437_.z, _e437_.w);
    metal::float4 _e449_ = metal::float4(_e443_.x, _e443_.y, _e431_.z, _e443_.w);
    metal::float3 _e452_ = _e449_.xyz * _e128_.w;
    metal::float4 _e458_ = metal::float4(_e452_.x, _e449_.y, _e449_.z, _e449_.w);
    metal::float4 _e464_ = metal::float4(_e458_.x, _e452_.y, _e458_.z, _e458_.w);
    metal::float4 _e470_ = metal::float4(_e464_.x, _e464_.y, _e452_.z, _e464_.w);
    metal::float3 _e471_ = _e470_.xyz;
    metal::float4 _e472_ = gl_FragCoord_1_;
    float _e474_ = n.z3_;
    float _e476_ = n.A3_;
    if (fh) {
        local_1 = _e128_.w != 0.0;
    } else {
        local_1 = false;
    }
    bool _e668 = local_1;
    if (_e668) {
        phi_3241_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e472_.x) + (0.00583715 * _e472_.y))) * _e474_) + _e476_) + _e471_;
    } else {
        phi_3241_ = _e471_;
    }
    metal::float3 _e492_ = phi_3241_;
    metal::float4 _e498_ = metal::float4(_e492_.x, _e470_.y, _e470_.z, _e470_.w);
    metal::float4 _e504_ = metal::float4(_e498_.x, _e492_.y, _e498_.z, _e498_.w);
    Jg = metal::float4(_e504_.x, _e504_.y, _e492_.z, _e504_.w);
    return;
}

struct main_Input {
    metal::float2 C2_ [[user(loc1), center_perspective]];
    metal::float4 f1_ [[user(loc0), center_perspective]];
    float e2_ [[user(loc6), flat]];
    float I3_ [[user(loc4), flat]];
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
, metal::float4 gl_FragCoord [[position]]
, metal::texture2d<float, metal::access::sample> KD [[texture(1)]]
, metal::sampler Mb [[sampler(1)]]
, metal::texture2d<float, metal::access::sample> IC [[texture(5)]]
, metal::sampler S5_ [[sampler(0)]]
, metal::texture2d<float, metal::access::sample> BD [[texture(3)]]
, metal::sampler Q9_ [[sampler(3)]]
, metal::texture2d<float, metal::access::sample> SD [[texture(4)]]
, constant CC& n [[buffer(0)]]
) {
    metal::float2 C2_1_ = {};
    metal::float4 f1_1_ = {};
    float e2_1_ = {};
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 Jg = {};
    float I3_1_ = {};
    const auto C2_ = varyings.C2_;
    const auto f1_ = varyings.f1_;
    const auto e2_ = varyings.e2_;
    const auto I3_ = varyings.I3_;
    C2_1_ = C2_;
    f1_1_ = f1_;
    e2_1_ = e2_;
    gl_FragCoord_1_ = gl_FragCoord;
    I3_1_ = I3_;
    main_1_(KD, Mb, IC, S5_, BD, Q9_, C2_1_, f1_1_, e2_1_, SD, gl_FragCoord_1_, n, Jg);
    metal::float4 _e11_ = Jg;
    return main_Output { _e11_ };
}
