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
constant bool ah = false;

void main_1_(
    metal::texture2d<float, metal::access::sample> KD,
    metal::sampler Mb,
    metal::texture2d<float, metal::access::sample> IC,
    metal::sampler S5_,
    metal::texture2d<float, metal::access::sample> BD,
    metal::sampler Q9_,
    thread metal::float2& C2_1_,
    thread metal::float4& f1_1_,
    thread metal::float4& gl_FragCoord_1_,
    constant CC& n,
    thread metal::float4& Jg
) {
    metal::float4 phi_615_ = {};
    float phi_612_ = {};
    float phi_613_ = {};
    metal::float4 phi_617_ = {};
    float phi_608_ = {};
    metal::float4 phi_618_ = {};
    metal::float4 phi_616_ = {};
    metal::float4 phi_614_ = {};
    metal::float3 phi_619_ = {};
    bool local = {};
    metal::float2 _e29_ = C2_1_;
    metal::float4 _e30_ = BD.sample(Q9_, _e29_, metal::level(0.0));
    float _e32_ = metal::clamp(_e30_.x, 0.0, 1.0);
    metal::float4 _e33_ = f1_1_;
    if (_e33_.w >= 0.0) {
        if (ah) {
            phi_615_ = metal::float4(_e33_.x, _e33_.y, _e33_.z, _e33_.w * _e32_);
        } else {
            phi_615_ = _e33_ * _e32_;
        }
        metal::float4 _e45_ = phi_615_;
        phi_614_ = _e45_;
    } else {
        if (_e33_.w > -1.0) {
            if (_e33_.z > 0.0) {
                phi_612_ = _e33_.x;
            } else {
                phi_612_ = metal::length(_e33_.xy);
            }
            float _e53_ = phi_612_;
            float _e54_ = metal::clamp(_e53_, 0.0, 1.0);
            float _e55_ = metal::abs(_e33_.z);
            if (_e55_ > 1.0) {
                phi_613_ = (0.9980469 * _e54_) + 0.0009765625;
            } else {
                phi_613_ = (0.001953125 * _e54_) + _e55_;
            }
            float _e62_ = phi_613_;
            metal::float4 _e65_ = KD.sample(Mb, metal::float2(_e62_, -(_e33_.w)), metal::level(0.0));
            float _e67_ = _e65_.w * _e32_;
            metal::float4 _e72_ = metal::float4(_e65_.x, _e65_.y, _e65_.z, _e67_);
            if (ah) {
                phi_617_ = _e72_;
            } else {
                metal::float3 _e74_ = _e72_.xyz * _e67_;
                phi_617_ = metal::float4(_e74_.x, _e74_.y, _e74_.z, _e67_);
            }
            metal::float4 _e80_ = phi_617_;
            phi_616_ = _e80_;
        } else {
            metal::float4 _e83_ = IC.sample(S5_, _e33_.xy, metal::level(-2.0 - _e33_.w));
            float _e85_ = _e33_.z * _e32_;
            if (ah) {
                if (_e83_.w != 0.0) {
                    phi_608_ = 1.0 / _e83_.w;
                } else {
                    phi_608_ = 0.0;
                }
                float _e91_ = phi_608_;
                metal::float3 _e92_ = _e83_.xyz * _e91_;
                phi_618_ = metal::float4(_e92_.x, _e92_.y, _e92_.z, _e83_.w * _e85_);
            } else {
                phi_618_ = _e83_ * _e85_;
            }
            metal::float4 _e100_ = phi_618_;
            phi_616_ = _e100_;
        }
        metal::float4 _e102_ = phi_616_;
        phi_614_ = _e102_;
    }
    metal::float4 _e104_ = phi_614_;
    metal::float3 _e105_ = _e104_.xyz;
    metal::float4 _e107_ = gl_FragCoord_1_;
    float _e109_ = n.z3_;
    float _e111_ = n.A3_;
    if (fh) {
        local = _e104_.w != 0.0;
    } else {
        local = false;
    }
    bool _e125 = local;
    if (_e125) {
        phi_619_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e107_.x) + (0.00583715 * _e107_.y))) * _e109_) + _e111_) + _e105_;
    } else {
        phi_619_ = _e105_;
    }
    metal::float3 _e127_ = phi_619_;
    metal::float4 _e133_ = metal::float4(_e127_.x, _e104_.y, _e104_.z, _e104_.w);
    metal::float4 _e139_ = metal::float4(_e133_.x, _e127_.y, _e133_.z, _e133_.w);
    Jg = metal::float4(_e139_.x, _e139_.y, _e127_.z, _e139_.w);
    return;
}

struct main_Input {
    metal::float2 C2_ [[user(loc1), center_perspective]];
    metal::float4 f1_ [[user(loc0), center_perspective]];
    float I3_ [[user(loc4), flat]];
    float e2_ [[user(loc6), flat]];
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
, constant CC& n [[buffer(0)]]
) {
    metal::float2 C2_1_ = {};
    metal::float4 f1_1_ = {};
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 Jg = {};
    float I3_1_ = {};
    float e2_1_ = {};
    const auto C2_ = varyings.C2_;
    const auto f1_ = varyings.f1_;
    const auto I3_ = varyings.I3_;
    const auto e2_ = varyings.e2_;
    C2_1_ = C2_;
    f1_1_ = f1_;
    gl_FragCoord_1_ = gl_FragCoord;
    I3_1_ = I3_;
    e2_1_ = e2_;
    main_1_(KD, Mb, IC, S5_, BD, Q9_, C2_1_, f1_1_, gl_FragCoord_1_, n, Jg);
    metal::float4 _e11_ = Jg;
    return main_Output { _e11_ };
}
