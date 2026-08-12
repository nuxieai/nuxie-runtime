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

void main_1_(
    thread metal::float4& Jg,
    thread metal::float4& R6_1_
) {
    metal::float4 _e3_ = R6_1_;
    Jg = _e3_;
    return;
}

struct main_Input {
    metal::float4 R6_ [[user(loc0), center_perspective]];
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
) {
    metal::float4 Jg = {};
    metal::float4 R6_1_ = {};
    const auto R6_ = varyings.R6_;
    R6_1_ = R6_;
    main_1_(Jg, R6_1_);
    metal::float4 _e3_1 = Jg;
    return main_Output { _e3_1 };
}
