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

void main_1_(
    metal::texture2d<float, metal::access::sample> JC,
    metal::sampler cf,
    thread metal::float2& X1_1_,
    thread metal::float4& Jg
) {
    metal::float2 _e6_ = X1_1_;
    metal::float4 _e7_ = JC.sample(cf, _e6_, metal::level(0.0));
    Jg = _e7_;
    return;
}

struct main_Input {
    metal::float2 X1_ [[user(loc0), center_perspective]];
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
, metal::texture2d<float, metal::access::sample> JC [[texture(0)]]
, metal::sampler cf [[sampler(0)]]
) {
    metal::float2 X1_1_ = {};
    metal::float4 Jg = {};
    const auto X1_ = varyings.X1_;
    X1_1_ = X1_;
    main_1_(JC, cf, X1_1_, Jg);
    metal::float4 _e3_ = Jg;
    return main_Output { _e3_ };
}
