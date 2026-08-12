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
    metal::float4 phi_2819_ = {};
    float phi_2816_ = {};
    float phi_2817_ = {};
    metal::float4 phi_2821_ = {};
    float phi_2812_ = {};
    metal::float4 phi_2822_ = {};
    metal::float4 phi_2820_ = {};
    metal::float4 phi_2818_ = {};
    float phi_2823_ = {};
    metal::float4 phi_3118_ = {};
    int phi_3078_ = {};
    metal::float3 phi_3221_ = {};
    bool local_1 = {};
    metal::float4 _e50_ = f1_1_;
    if (_e50_.w >= 0.0) {
        if (ah) {
            phi_2819_ = metal::float4(_e50_.x, _e50_.y, _e50_.z, _e50_.w);
        } else {
            phi_2819_ = _e50_ * 1.0;
        }
        metal::float4 _e61_ = phi_2819_;
        phi_2818_ = _e61_;
    } else {
        if (_e50_.w > -1.0) {
            if (_e50_.z > 0.0) {
                phi_2816_ = _e50_.x;
            } else {
                phi_2816_ = metal::length(_e50_.xy);
            }
            float _e69_ = phi_2816_;
            float _e70_ = metal::clamp(_e69_, 0.0, 1.0);
            float _e71_ = metal::abs(_e50_.z);
            if (_e71_ > 1.0) {
                phi_2817_ = (0.9980469 * _e70_) + 0.0009765625;
            } else {
                phi_2817_ = (0.001953125 * _e70_) + _e71_;
            }
            float _e78_ = phi_2817_;
            metal::float4 _e81_ = KD.sample(Mb, metal::float2(_e78_, -(_e50_.w)), metal::level(0.0));
            metal::float4 _e87_ = metal::float4(_e81_.x, _e81_.y, _e81_.z, _e81_.w);
            if (ah) {
                phi_2821_ = _e87_;
            } else {
                metal::float3 _e89_ = _e87_.xyz * _e81_.w;
                phi_2821_ = metal::float4(_e89_.x, _e89_.y, _e89_.z, _e81_.w);
            }
            metal::float4 _e95_ = phi_2821_;
            phi_2820_ = _e95_;
        } else {
            metal::float4 _e98_ = IC.sample(S5_, _e50_.xy, metal::level(-2.0 - _e50_.w));
            if (ah) {
                if (_e98_.w != 0.0) {
                    phi_2812_ = 1.0 / _e98_.w;
                } else {
                    phi_2812_ = 0.0;
                }
                float _e105_ = phi_2812_;
                metal::float3 _e106_ = _e98_.xyz * _e105_;
                phi_2822_ = metal::float4(_e106_.x, _e106_.y, _e106_.z, _e98_.w * _e50_.z);
            } else {
                phi_2822_ = _e98_ * _e50_.z;
            }
            metal::float4 _e114_ = phi_2822_;
            phi_2820_ = _e114_;
        }
        metal::float4 _e116_ = phi_2820_;
        phi_2818_ = _e116_;
    }
    metal::float4 _e118_ = phi_2818_;
    float _e119_ = e2_1_;
    metal::float4 _e121_ = gl_FragCoord_1_;
    uint clamped_lod_e115 = metal::min(uint(0), SD.get_num_mip_levels() - 1);
    metal::float4 _e125_ = SD.read(metal::min(metal::uint2(naga_f2i32(metal::floor(_e121_.xy))), metal::uint2(SD.get_width(clamped_lod_e115), SD.get_height(clamped_lod_e115)) - 1), clamped_lod_e115);
    metal::float3 _e126_ = _e118_.xyz;
    local_2_ = _e126_;
    metal::float3 _e127_ = _e125_.xyz;
    if (_e125_.w != 0.0) {
        phi_2823_ = 1.0 / _e125_.w;
    } else {
        phi_2823_ = 0.0;
    }
    float _e132_ = phi_2823_;
    metal::float3 _e133_ = _e127_ * _e132_;
    local = _e133_;
    switch(as_type<int>(naga_f2u32(_e119_))) {
        case 11: {
            metal::float3 _e135_ = local_2_;
            local_1_ = _e135_ * _e133_;
            break;
        }
        case 1: {
            metal::float3 _e137_ = local_2_;
            local_1_ = (_e137_ + _e133_) - (_e137_ * _e133_);
            break;
        }
        case 2: {
            metal::float3 _e141_ = local_2_;
            metal::float3 _e142_ = _e141_ * _e133_;
            local_1_ = metal::select(_e142_, ((_e141_ + _e133_) - _e142_) - metal::float3(0.5, 0.5, 0.5), _e133_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
            break;
        }
        case 3: {
            metal::float3 _e149_ = local_2_;
            local_1_ = metal::min(_e149_, _e133_);
            break;
        }
        case 4: {
            metal::float3 _e151_ = local_2_;
            local_1_ = metal::max(_e151_, _e133_);
            break;
        }
        case 5: {
            metal::float3 _e154_ = metal::clamp(_e127_, metal::float3(0.0, 0.0, 0.0), _e125_.www);
            metal::float4 _e160_ = metal::float4(_e154_.x, float {}, float {}, float {});
            metal::float4 _e166_ = metal::float4(_e160_.x, _e154_.y, _e160_.z, _e160_.w);
            metal::float3 _e173_ = local_2_;
            metal::float3 _e176_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e173_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e125_.w;
            metal::float3 _e177_ = metal::float4(_e166_.x, _e166_.y, _e154_.z, _e166_.w).xyz;
            local_1_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e177_ / _e176_), metal::sign(_e177_), _e176_ == metal::float3(0.0, 0.0, 0.0));
            break;
        }
        case 6: {
            metal::float3 _e183_ = local_2_;
            local_2_ = metal::clamp(_e183_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
            metal::float3 _e186_ = metal::clamp(_e127_, metal::float3(0.0, 0.0, 0.0), _e125_.www);
            metal::float4 _e192_ = metal::float4(_e186_.x, _e125_.y, _e125_.z, _e125_.w);
            metal::float4 _e198_ = metal::float4(_e192_.x, _e186_.y, _e192_.z, _e192_.w);
            phi_3118_ = metal::float4(_e198_.x, _e198_.y, _e186_.z, _e198_.w);
            if (_e125_.w == 0.0) {
                phi_3118_ = metal::float4(_e186_.x, _e186_.y, _e186_.z, 1.0);
            }
            metal::float4 _e208_ = phi_3118_;
            metal::float3 _e212_ = metal::float3(_e208_.w) - _e208_.xyz;
            metal::float3 _e213_ = local_2_;
            local_1_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e212_ / (_e213_ * _e208_.w)), metal::sign(_e212_), _e213_ == metal::float3(0.0, 0.0, 0.0));
            break;
        }
        case 7: {
            metal::float3 _e221_ = local_2_;
            metal::float3 _e222_ = _e221_ * _e133_;
            local_1_ = metal::select(_e222_, ((_e221_ + _e133_) - _e222_) - metal::float3(0.5, 0.5, 0.5), _e221_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
            break;
        }
        case 8: {
            phi_3078_ = 0;
            uint2 loop_bound = uint2(4294967295u);
            bool loop_init = true;
            while(true) {
                if (metal::all(loop_bound == uint2(0u))) { break; }
                loop_bound -= uint2(loop_bound.y == 0u, 1u);
                if (!loop_init) {
                    phi_3078_ = as_type<int>(as_type<uint>(phi_3078_) + as_type<uint>(1));
                }
                loop_init = false;
                int _e230_ = phi_3078_;
                if (_e230_ < 3) {
                    float _e233_ = local_2_[metal::min(unsigned(_e230_), 2u)];
                    if (_e233_ <= 0.5) {
                        float _e236_ = local[metal::min(unsigned(_e230_), 2u)];
                        local_1_[metal::min(unsigned(_e230_), 2u)] = 1.0 - _e236_;
                    } else {
                        float _e240_ = local[metal::min(unsigned(_e230_), 2u)];
                        if (_e240_ <= 0.25) {
                            float _e242_ = local[metal::min(unsigned(_e230_), 2u)];
                            float _e245_ = local[metal::min(unsigned(_e230_), 2u)];
                            local_1_[metal::min(unsigned(_e230_), 2u)] = (((16.0 * _e242_) - 12.0) * _e245_) + 3.0;
                        } else {
                            float _e249_ = local[metal::min(unsigned(_e230_), 2u)];
                            local_1_[metal::min(unsigned(_e230_), 2u)] = metal::rsqrt(_e249_) - 1.0;
                        }
                    }
                    continue;
                } else {
                    break;
                }
            }
            metal::float3 _e254_ = local_2_;
            metal::float3 _e258_ = local_1_;
            local_1_ = _e133_ + ((_e133_ * ((_e254_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e258_);
            break;
        }
        case 9: {
            metal::float3 _e261_ = local_2_;
            local_1_ = metal::abs(_e133_ - _e261_);
            break;
        }
        case 10: {
            metal::float3 _e264_ = local_2_;
            local_1_ = (_e264_ + _e133_) - ((_e264_ * 2.0) * _e133_);
            break;
        }
        case 12: {
            if (eh) {
                metal::float3 _e269_ = local_2_;
                metal::float3 _e270_ = metal::clamp(_e269_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e270_;
                metal::float3 _e285_ = _e270_ - metal::float3(metal::min(metal::min(_e270_.x, _e270_.y), _e270_.z));
                metal::float3 _e293_ = _e285_ * ((metal::max(metal::max(_e133_.x, _e133_.y), _e133_.z) - metal::min(metal::min(_e133_.x, _e133_.y), _e133_.z)) / metal::max(0.000062, metal::max(metal::max(_e285_.x, _e285_.y), _e285_.z)));
                float _e294_ = metal::dot(_e133_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e297_ = _e293_ - metal::float3(metal::dot(_e293_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e310_ = metal::float2(_e294_, 1.0 - _e294_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e297_.x, _e297_.y), _e297_.z)), metal::max(metal::max(_e297_.x, _e297_.y), _e297_.z)));
                local_1_ = (_e297_ * metal::min(1.0, metal::min(_e310_.x, _e310_.y))) + metal::float3(_e294_);
            }
            break;
        }
        case 13: {
            if (eh) {
                metal::float3 _e318_ = local_2_;
                metal::float3 _e319_ = metal::clamp(_e318_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e319_;
                metal::float3 _e334_ = _e133_ - metal::float3(metal::min(metal::min(_e133_.x, _e133_.y), _e133_.z));
                metal::float3 _e342_ = _e334_ * ((metal::max(metal::max(_e319_.x, _e319_.y), _e319_.z) - metal::min(metal::min(_e319_.x, _e319_.y), _e319_.z)) / metal::max(0.000062, metal::max(metal::max(_e334_.x, _e334_.y), _e334_.z)));
                float _e343_ = metal::dot(_e133_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e346_ = _e342_ - metal::float3(metal::dot(_e342_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e359_ = metal::float2(_e343_, 1.0 - _e343_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e346_.x, _e346_.y), _e346_.z)), metal::max(metal::max(_e346_.x, _e346_.y), _e346_.z)));
                local_1_ = (_e346_ * metal::min(1.0, metal::min(_e359_.x, _e359_.y))) + metal::float3(_e343_);
            }
            break;
        }
        case 14: {
            if (eh) {
                metal::float3 _e367_ = local_2_;
                metal::float3 _e368_ = metal::clamp(_e367_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e368_;
                float _e369_ = metal::dot(_e133_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e372_ = _e368_ - metal::float3(metal::dot(_e368_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e385_ = metal::float2(_e369_, 1.0 - _e369_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e372_.x, _e372_.y), _e372_.z)), metal::max(metal::max(_e372_.x, _e372_.y), _e372_.z)));
                local_1_ = (_e372_ * metal::min(1.0, metal::min(_e385_.x, _e385_.y))) + metal::float3(_e369_);
            }
            break;
        }
        case 15: {
            if (eh) {
                metal::float3 _e393_ = local_2_;
                metal::float3 _e394_ = metal::clamp(_e393_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e394_;
                float _e395_ = metal::dot(_e394_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e398_ = _e133_ - metal::float3(metal::dot(_e133_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e411_ = metal::float2(_e395_, 1.0 - _e395_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e398_.x, _e398_.y), _e398_.z)), metal::max(metal::max(_e398_.x, _e398_.y), _e398_.z)));
                local_1_ = (_e398_ * metal::min(1.0, metal::min(_e411_.x, _e411_.y))) + metal::float3(_e395_);
            }
            break;
        }
        default: {
            break;
        }
    }
    metal::float3 _e419_ = local_1_;
    metal::float3 _e421_ = metal::mix(_e126_, _e419_, metal::float3(_e125_.w));
    metal::float4 _e427_ = metal::float4(_e421_.x, _e118_.y, _e118_.z, _e118_.w);
    metal::float4 _e433_ = metal::float4(_e427_.x, _e421_.y, _e427_.z, _e427_.w);
    metal::float4 _e439_ = metal::float4(_e433_.x, _e433_.y, _e421_.z, _e433_.w);
    metal::float3 _e442_ = _e439_.xyz * _e118_.w;
    metal::float4 _e448_ = metal::float4(_e442_.x, _e439_.y, _e439_.z, _e439_.w);
    metal::float4 _e454_ = metal::float4(_e448_.x, _e442_.y, _e448_.z, _e448_.w);
    metal::float4 _e460_ = metal::float4(_e454_.x, _e454_.y, _e442_.z, _e454_.w);
    metal::float3 _e461_ = _e460_.xyz;
    metal::float4 _e462_ = gl_FragCoord_1_;
    float _e464_ = n.z3_;
    float _e466_ = n.A3_;
    if (fh) {
        local_1 = _e118_.w != 0.0;
    } else {
        local_1 = false;
    }
    bool _e659 = local_1;
    if (_e659) {
        phi_3221_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e462_.x) + (0.00583715 * _e462_.y))) * _e464_) + _e466_) + _e461_;
    } else {
        phi_3221_ = _e461_;
    }
    metal::float3 _e482_ = phi_3221_;
    metal::float4 _e488_ = metal::float4(_e482_.x, _e460_.y, _e460_.z, _e460_.w);
    metal::float4 _e494_ = metal::float4(_e488_.x, _e482_.y, _e488_.z, _e488_.w);
    Jg = metal::float4(_e494_.x, _e494_.y, _e482_.z, _e494_.w);
    return;
}

struct main_Input {
    metal::float4 f1_ [[user(loc0), center_perspective]];
    float e2_ [[user(loc6), flat]];
    metal::float2 U1_ [[user(loc4), flat]];
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
, metal::float4 gl_FragCoord [[position]]
, metal::texture2d<float, metal::access::sample> KD [[texture(0)]]
, metal::sampler Mb [[sampler(1)]]
, metal::texture2d<float, metal::access::sample> IC [[texture(3)]]
, metal::sampler S5_ [[sampler(0)]]
, metal::texture2d<float, metal::access::sample> SD [[texture(2)]]
, constant CC& n [[buffer(0)]]
) {
    metal::float4 f1_1_ = {};
    float e2_1_ = {};
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 Jg = {};
    metal::float2 U1_1_ = {};
    const auto f1_ = varyings.f1_;
    const auto e2_ = varyings.e2_;
    const auto U1_ = varyings.U1_;
    f1_1_ = f1_;
    e2_1_ = e2_;
    gl_FragCoord_1_ = gl_FragCoord;
    U1_1_ = U1_;
    main_1_(KD, Mb, IC, S5_, f1_1_, e2_1_, SD, gl_FragCoord_1_, n, Jg);
    metal::float4 _e9_ = Jg;
    return main_Output { _e9_ };
}
