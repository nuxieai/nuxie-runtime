// language: metal3.0
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
constant bool ah = false;

void main_1_(
    metal::texture2d<float, metal::access::sample> KD,
    metal::sampler Mb,
    metal::texture2d<float, metal::access::sample> IC,
    metal::sampler S5_,
    thread metal::float4& f1_1_,
    thread metal::float4& gl_FragCoord_1_,
    constant CC& n,
    thread metal::float4& Jg
) {
    metal::float4 phi_588_ = {};
    float phi_585_ = {};
    float phi_586_ = {};
    metal::float4 phi_590_ = {};
    float phi_581_ = {};
    metal::float4 phi_591_ = {};
    metal::float4 phi_589_ = {};
    metal::float4 phi_587_ = {};
    metal::float3 phi_592_ = {};
    bool local = {};
    metal::float4 _e26_ = f1_1_;
    if (_e26_.w >= 0.0) {
        if (ah) {
            phi_588_ = metal::float4(_e26_.x, _e26_.y, _e26_.z, _e26_.w);
        } else {
            phi_588_ = _e26_ * 1.0;
        }
        metal::float4 _e37_ = phi_588_;
        phi_587_ = _e37_;
    } else {
        if (_e26_.w > -1.0) {
            if (_e26_.z > 0.0) {
                phi_585_ = _e26_.x;
            } else {
                phi_585_ = metal::length(_e26_.xy);
            }
            float _e45_ = phi_585_;
            float _e46_ = metal::clamp(_e45_, 0.0, 1.0);
            float _e47_ = metal::abs(_e26_.z);
            if (_e47_ > 1.0) {
                phi_586_ = (0.9980469 * _e46_) + 0.0009765625;
            } else {
                phi_586_ = (0.001953125 * _e46_) + _e47_;
            }
            float _e54_ = phi_586_;
            metal::float4 _e57_ = KD.sample(Mb, metal::float2(_e54_, -(_e26_.w)), metal::level(0.0));
            metal::float4 _e63_ = metal::float4(_e57_.x, _e57_.y, _e57_.z, _e57_.w);
            if (ah) {
                phi_590_ = _e63_;
            } else {
                metal::float3 _e65_ = _e63_.xyz * _e57_.w;
                phi_590_ = metal::float4(_e65_.x, _e65_.y, _e65_.z, _e57_.w);
            }
            metal::float4 _e71_ = phi_590_;
            phi_589_ = _e71_;
        } else {
            metal::float4 _e74_ = IC.sample(S5_, _e26_.xy, metal::level(-2.0 - _e26_.w));
            if (ah) {
                if (_e74_.w != 0.0) {
                    phi_581_ = 1.0 / _e74_.w;
                } else {
                    phi_581_ = 0.0;
                }
                float _e81_ = phi_581_;
                metal::float3 _e82_ = _e74_.xyz * _e81_;
                phi_591_ = metal::float4(_e82_.x, _e82_.y, _e82_.z, _e74_.w * _e26_.z);
            } else {
                phi_591_ = _e74_ * _e26_.z;
            }
            metal::float4 _e90_ = phi_591_;
            phi_589_ = _e90_;
        }
        metal::float4 _e92_ = phi_589_;
        phi_587_ = _e92_;
    }
    metal::float4 _e94_ = phi_587_;
    metal::float3 _e95_ = _e94_.xyz;
    metal::float4 _e97_ = gl_FragCoord_1_;
    float _e99_ = n.z3_;
    float _e101_ = n.A3_;
    if (fh) {
        local = _e94_.w != 0.0;
    } else {
        local = false;
    }
    bool _e116 = local;
    if (_e116) {
        phi_592_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e97_.x) + (0.00583715 * _e97_.y))) * _e99_) + _e101_) + _e95_;
    } else {
        phi_592_ = _e95_;
    }
    metal::float3 _e117_ = phi_592_;
    metal::float4 _e123_ = metal::float4(_e117_.x, _e94_.y, _e94_.z, _e94_.w);
    metal::float4 _e129_ = metal::float4(_e123_.x, _e117_.y, _e123_.z, _e123_.w);
    Jg = metal::float4(_e129_.x, _e129_.y, _e117_.z, _e129_.w);
    return;
}

struct main_Input {
    metal::float4 f1_ [[user(loc0), center_perspective]];
    metal::float2 U1_ [[user(loc4), flat]];
    float e2_ [[user(loc6), flat]];
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
, constant CC& n [[buffer(0)]]
) {
    metal::float4 f1_1_ = {};
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 Jg = {};
    metal::float2 U1_1_ = {};
    float e2_1_ = {};
    const auto f1_ = varyings.f1_;
    const auto U1_ = varyings.U1_;
    const auto e2_ = varyings.e2_;
    f1_1_ = f1_;
    gl_FragCoord_1_ = gl_FragCoord;
    U1_1_ = U1_;
    e2_1_ = e2_;
    main_1_(KD, Mb, IC, S5_, f1_1_, gl_FragCoord_1_, n, Jg);
    metal::float4 _e9_ = Jg;
    return main_Output { _e9_ };
}
