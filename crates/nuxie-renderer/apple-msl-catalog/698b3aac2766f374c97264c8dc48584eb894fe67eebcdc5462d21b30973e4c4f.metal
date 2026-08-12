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

void main_1_(
    metal::texture2d<float, metal::access::sample> XC,
    metal::sampler aa,
    thread float& Jg,
    thread metal::float4& O_1_
) {
    metal::float4 _e12_ = O_1_;
    metal::float4 _e16_ = XC.sample(aa, metal::float2(3.0 + _e12_.x, 0.0), metal::level(0.0));
    metal::float4 _e22_ = XC.sample(aa, metal::float2(1.0 - _e12_.y, 0.0), metal::level(0.0));
    Jg = (1.0 - _e16_.x) - _e22_.x;
    return;
}

struct main_Input {
    metal::float4 O [[user(loc0), center_perspective]];
};
struct main_Output {
    float member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
, metal::texture2d<float, metal::access::sample> XC [[texture(2)]]
, metal::sampler aa [[sampler(2)]]
) {
    float Jg = {};
    metal::float4 O_1_ = {};
    const auto O = varyings.O;
    O_1_ = O;
    main_1_(XC, aa, Jg, O_1_);
    float _e3_ = Jg;
    return main_Output { _e3_ };
}
