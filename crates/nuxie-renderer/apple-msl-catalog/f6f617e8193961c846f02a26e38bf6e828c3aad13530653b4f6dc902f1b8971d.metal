// language: metal3.1
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

metal::int2 naga_f2i32(metal::float2 value) {
    return static_cast<metal::int2>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

void main_1_(
    metal::texture2d<float, metal::access::sample> IC,
    metal::sampler S5_,
    thread metal::float2& E5_1_,
    constant CC& n,
    thread float& H1_1_,
    thread uint& A1_1_,
    metal::texture2d<float, metal::access::sample> SD,
    thread metal::float4& gl_FragCoord_1_,
    thread metal::float4& Jg
) {
    metal::float3 local = {};
    metal::float3 local_1_ = {};
    metal::float3 local_2_ = {};
    float phi_2475_ = {};
    float phi_2477_ = {};
    metal::float4 phi_2548_ = {};
    int phi_2536_ = {};
    metal::float3 phi_2588_ = {};
    bool local_1 = {};
    metal::float2 _e42_ = E5_1_;
    float _e44_ = n.qd;
    metal::float4 _e45_ = IC.sample(S5_, _e42_, metal::bias(_e44_));
    float _e46_ = H1_1_;
    metal::float4 _e47_ = _e45_ * _e46_;
    bool _e50_ = _e47_.w != 0.0;
    if (_e50_) {
        phi_2475_ = 1.0 / _e47_.w;
    } else {
        phi_2475_ = 0.0;
    }
    float _e53_ = phi_2475_;
    metal::float3 _e54_ = _e47_.xyz * _e53_;
    metal::float4 _e60_ = metal::float4(_e54_.x, _e47_.y, _e47_.z, _e47_.w);
    metal::float4 _e66_ = metal::float4(_e60_.x, _e54_.y, _e60_.z, _e60_.w);
    metal::float4 _e72_ = metal::float4(_e66_.x, _e66_.y, _e54_.z, _e66_.w);
    uint _e73_ = A1_1_;
    metal::float4 _e74_ = gl_FragCoord_1_;
    uint clamped_lod_e53 = metal::min(uint(0), SD.get_num_mip_levels() - 1);
    metal::float4 _e78_ = SD.read(metal::min(metal::uint2(naga_f2i32(metal::floor(_e74_.xy))), metal::uint2(SD.get_width(clamped_lod_e53), SD.get_height(clamped_lod_e53)) - 1), clamped_lod_e53);
    metal::float3 _e79_ = _e72_.xyz;
    local_2_ = _e79_;
    metal::float3 _e80_ = _e78_.xyz;
    if (_e78_.w != 0.0) {
        phi_2477_ = 1.0 / _e78_.w;
    } else {
        phi_2477_ = 0.0;
    }
    float _e85_ = phi_2477_;
    metal::float3 _e86_ = _e80_ * _e85_;
    local = _e86_;
    switch(as_type<int>(_e73_)) {
        case 11: {
            metal::float3 _e88_ = local_2_;
            local_1_ = _e88_ * _e86_;
            break;
        }
        case 1: {
            metal::float3 _e90_ = local_2_;
            local_1_ = (_e90_ + _e86_) - (_e90_ * _e86_);
            break;
        }
        case 2: {
            metal::float3 _e94_ = local_2_;
            metal::float3 _e95_ = _e94_ * _e86_;
            local_1_ = metal::select(_e95_, ((_e94_ + _e86_) - _e95_) - metal::float3(0.5, 0.5, 0.5), _e86_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
            break;
        }
        case 3: {
            metal::float3 _e102_ = local_2_;
            local_1_ = metal::min(_e102_, _e86_);
            break;
        }
        case 4: {
            metal::float3 _e104_ = local_2_;
            local_1_ = metal::max(_e104_, _e86_);
            break;
        }
        case 5: {
            metal::float3 _e107_ = metal::clamp(_e80_, metal::float3(0.0, 0.0, 0.0), _e78_.www);
            metal::float4 _e113_ = metal::float4(_e107_.x, float {}, float {}, float {});
            metal::float4 _e119_ = metal::float4(_e113_.x, _e107_.y, _e113_.z, _e113_.w);
            metal::float3 _e126_ = local_2_;
            metal::float3 _e129_ = metal::clamp(metal::float3(1.0, 1.0, 1.0) - _e126_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0)) * _e78_.w;
            metal::float3 _e130_ = metal::float4(_e119_.x, _e119_.y, _e107_.z, _e119_.w).xyz;
            local_1_ = metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e130_ / _e129_), metal::sign(_e130_), _e129_ == metal::float3(0.0, 0.0, 0.0));
            break;
        }
        case 6: {
            metal::float3 _e136_ = local_2_;
            local_2_ = metal::clamp(_e136_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
            metal::float3 _e139_ = metal::clamp(_e80_, metal::float3(0.0, 0.0, 0.0), _e78_.www);
            metal::float4 _e145_ = metal::float4(_e139_.x, _e78_.y, _e78_.z, _e78_.w);
            metal::float4 _e151_ = metal::float4(_e145_.x, _e139_.y, _e145_.z, _e145_.w);
            phi_2548_ = metal::float4(_e151_.x, _e151_.y, _e139_.z, _e151_.w);
            if (_e78_.w == 0.0) {
                phi_2548_ = metal::float4(_e139_.x, _e139_.y, _e139_.z, 1.0);
            }
            metal::float4 _e161_ = phi_2548_;
            metal::float3 _e165_ = metal::float3(_e161_.w) - _e161_.xyz;
            metal::float3 _e166_ = local_2_;
            local_1_ = metal::float3(1.0, 1.0, 1.0) - metal::select(metal::min(metal::float3(1.0, 1.0, 1.0), _e165_ / (_e166_ * _e161_.w)), metal::sign(_e165_), _e166_ == metal::float3(0.0, 0.0, 0.0));
            break;
        }
        case 7: {
            metal::float3 _e174_ = local_2_;
            metal::float3 _e175_ = _e174_ * _e86_;
            local_1_ = metal::select(_e175_, ((_e174_ + _e86_) - _e175_) - metal::float3(0.5, 0.5, 0.5), _e174_ > metal::float3(0.5, 0.5, 0.5)) * 2.0;
            break;
        }
        case 8: {
            phi_2536_ = 0;
            uint2 loop_bound = uint2(4294967295u);
            bool loop_init = true;
            while(true) {
                if (metal::all(loop_bound == uint2(0u))) { break; }
                loop_bound -= uint2(loop_bound.y == 0u, 1u);
                if (!loop_init) {
                    phi_2536_ = as_type<int>(as_type<uint>(phi_2536_) + as_type<uint>(1));
                }
                loop_init = false;
                int _e183_ = phi_2536_;
                if (_e183_ < 3) {
                    float _e186_ = local_2_[metal::min(unsigned(_e183_), 2u)];
                    if (_e186_ <= 0.5) {
                        float _e189_ = local[metal::min(unsigned(_e183_), 2u)];
                        local_1_[metal::min(unsigned(_e183_), 2u)] = 1.0 - _e189_;
                    } else {
                        float _e193_ = local[metal::min(unsigned(_e183_), 2u)];
                        if (_e193_ <= 0.25) {
                            float _e195_ = local[metal::min(unsigned(_e183_), 2u)];
                            float _e198_ = local[metal::min(unsigned(_e183_), 2u)];
                            local_1_[metal::min(unsigned(_e183_), 2u)] = (((16.0 * _e195_) - 12.0) * _e198_) + 3.0;
                        } else {
                            float _e202_ = local[metal::min(unsigned(_e183_), 2u)];
                            local_1_[metal::min(unsigned(_e183_), 2u)] = metal::rsqrt(_e202_) - 1.0;
                        }
                    }
                    continue;
                } else {
                    break;
                }
            }
            metal::float3 _e207_ = local_2_;
            metal::float3 _e211_ = local_1_;
            local_1_ = _e86_ + ((_e86_ * ((_e207_ * 2.0) - metal::float3(1.0, 1.0, 1.0))) * _e211_);
            break;
        }
        case 9: {
            metal::float3 _e214_ = local_2_;
            local_1_ = metal::abs(_e86_ - _e214_);
            break;
        }
        case 10: {
            metal::float3 _e217_ = local_2_;
            local_1_ = (_e217_ + _e86_) - ((_e217_ * 2.0) * _e86_);
            break;
        }
        case 12: {
            if (eh) {
                metal::float3 _e222_ = local_2_;
                metal::float3 _e223_ = metal::clamp(_e222_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e223_;
                metal::float3 _e238_ = _e223_ - metal::float3(metal::min(metal::min(_e223_.x, _e223_.y), _e223_.z));
                metal::float3 _e246_ = _e238_ * ((metal::max(metal::max(_e86_.x, _e86_.y), _e86_.z) - metal::min(metal::min(_e86_.x, _e86_.y), _e86_.z)) / metal::max(0.000062, metal::max(metal::max(_e238_.x, _e238_.y), _e238_.z)));
                float _e247_ = metal::dot(_e86_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e250_ = _e246_ - metal::float3(metal::dot(_e246_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e263_ = metal::float2(_e247_, 1.0 - _e247_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e250_.x, _e250_.y), _e250_.z)), metal::max(metal::max(_e250_.x, _e250_.y), _e250_.z)));
                local_1_ = (_e250_ * metal::min(1.0, metal::min(_e263_.x, _e263_.y))) + metal::float3(_e247_);
            }
            break;
        }
        case 13: {
            if (eh) {
                metal::float3 _e271_ = local_2_;
                metal::float3 _e272_ = metal::clamp(_e271_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e272_;
                metal::float3 _e287_ = _e86_ - metal::float3(metal::min(metal::min(_e86_.x, _e86_.y), _e86_.z));
                metal::float3 _e295_ = _e287_ * ((metal::max(metal::max(_e272_.x, _e272_.y), _e272_.z) - metal::min(metal::min(_e272_.x, _e272_.y), _e272_.z)) / metal::max(0.000062, metal::max(metal::max(_e287_.x, _e287_.y), _e287_.z)));
                float _e296_ = metal::dot(_e86_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e299_ = _e295_ - metal::float3(metal::dot(_e295_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e312_ = metal::float2(_e296_, 1.0 - _e296_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e299_.x, _e299_.y), _e299_.z)), metal::max(metal::max(_e299_.x, _e299_.y), _e299_.z)));
                local_1_ = (_e299_ * metal::min(1.0, metal::min(_e312_.x, _e312_.y))) + metal::float3(_e296_);
            }
            break;
        }
        case 14: {
            if (eh) {
                metal::float3 _e320_ = local_2_;
                metal::float3 _e321_ = metal::clamp(_e320_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e321_;
                float _e322_ = metal::dot(_e86_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e325_ = _e321_ - metal::float3(metal::dot(_e321_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e338_ = metal::float2(_e322_, 1.0 - _e322_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e325_.x, _e325_.y), _e325_.z)), metal::max(metal::max(_e325_.x, _e325_.y), _e325_.z)));
                local_1_ = (_e325_ * metal::min(1.0, metal::min(_e338_.x, _e338_.y))) + metal::float3(_e322_);
            }
            break;
        }
        case 15: {
            if (eh) {
                metal::float3 _e346_ = local_2_;
                metal::float3 _e347_ = metal::clamp(_e346_, metal::float3(0.0, 0.0, 0.0), metal::float3(1.0, 1.0, 1.0));
                local_2_ = _e347_;
                float _e348_ = metal::dot(_e347_, metal::float3(0.3, 0.59, 0.11));
                metal::float3 _e351_ = _e86_ - metal::float3(metal::dot(_e86_, metal::float3(0.3, 0.59, 0.11)));
                metal::float2 _e364_ = metal::float2(_e348_, 1.0 - _e348_) / metal::max(metal::float2(0.000062, 0.000062), metal::float2(-(metal::min(metal::min(_e351_.x, _e351_.y), _e351_.z)), metal::max(metal::max(_e351_.x, _e351_.y), _e351_.z)));
                local_1_ = (_e351_ * metal::min(1.0, metal::min(_e364_.x, _e364_.y))) + metal::float3(_e348_);
            }
            break;
        }
        default: {
            break;
        }
    }
    metal::float3 _e372_ = local_1_;
    metal::float3 _e374_ = metal::mix(_e79_, _e372_, metal::float3(_e78_.w));
    metal::float4 _e380_ = metal::float4(_e374_.x, _e72_.y, _e72_.z, _e72_.w);
    metal::float4 _e386_ = metal::float4(_e380_.x, _e374_.y, _e380_.z, _e380_.w);
    metal::float4 _e392_ = metal::float4(_e386_.x, _e386_.y, _e374_.z, _e386_.w);
    metal::float3 _e394_ = _e392_.xyz * _e47_.w;
    metal::float4 _e400_ = metal::float4(_e394_.x, _e392_.y, _e392_.z, _e392_.w);
    metal::float4 _e406_ = metal::float4(_e400_.x, _e394_.y, _e400_.z, _e400_.w);
    metal::float4 _e412_ = metal::float4(_e406_.x, _e406_.y, _e394_.z, _e406_.w);
    metal::float3 _e413_ = _e412_.xyz;
    metal::float4 _e414_ = gl_FragCoord_1_;
    float _e416_ = n.z3_;
    float _e418_ = n.A3_;
    if (fh) {
        local_1 = _e50_;
    } else {
        local_1 = false;
    }
    bool _e593 = local_1;
    if (_e593) {
        phi_2588_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e414_.x) + (0.00583715 * _e414_.y))) * _e416_) + _e418_) + _e413_;
    } else {
        phi_2588_ = _e413_;
    }
    metal::float3 _e433_ = phi_2588_;
    metal::float4 _e439_ = metal::float4(_e433_.x, _e412_.y, _e412_.z, _e412_.w);
    metal::float4 _e445_ = metal::float4(_e439_.x, _e433_.y, _e439_.z, _e439_.w);
    Jg = metal::float4(_e445_.x, _e445_.y, _e433_.z, _e445_.w);
    return;
}

struct main_Input {
    metal::float2 E5_ [[user(loc0), center_perspective]];
    float H1_ [[user(loc3), flat]];
    uint A1_ [[user(loc4), flat]];
    float I3_ [[user(loc1), flat]];
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
, metal::float4 gl_FragCoord [[position]]
, metal::texture2d<float, metal::access::sample> IC [[texture(1)]]
, metal::sampler S5_ [[sampler(0)]]
, constant CC& n [[buffer(0)]]
, metal::texture2d<float, metal::access::sample> SD [[texture(0)]]
) {
    metal::float2 E5_1_ = {};
    float H1_1_ = {};
    uint A1_1_ = {};
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 Jg = {};
    float I3_1_ = {};
    const auto E5_ = varyings.E5_;
    const auto H1_ = varyings.H1_;
    const auto A1_ = varyings.A1_;
    const auto I3_ = varyings.I3_;
    E5_1_ = E5_;
    H1_1_ = H1_;
    A1_1_ = A1_;
    gl_FragCoord_1_ = gl_FragCoord;
    I3_1_ = I3_;
    main_1_(IC, S5_, E5_1_, n, H1_1_, A1_1_, SD, gl_FragCoord_1_, Jg);
    metal::float4 _e11_ = Jg;
    return main_Output { _e11_ };
}
