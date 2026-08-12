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
    thread metal::float4& Jg
) {
    Jg = metal::float4(0.0, 0.0, 0.0, 0.0);
    return;
}

struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
) {
    metal::float4 Jg = {};
    main_1_(Jg);
    metal::float4 _e1_ = Jg;
    return main_Output { _e1_ };
}
