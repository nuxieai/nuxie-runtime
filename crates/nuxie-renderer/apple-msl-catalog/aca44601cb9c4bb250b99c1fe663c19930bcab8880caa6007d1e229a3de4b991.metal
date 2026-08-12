// language: metal3.2
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

void main_1_(
    metal::texture2d<float, metal::access::sample> IC,
    metal::sampler S5_,
    thread metal::float2& E5_1_,
    constant CC& n,
    thread float& H1_1_,
    thread metal::float4& gl_FragCoord_1_,
    thread metal::float4& Jg
) {
    metal::float3 phi_204_ = {};
    bool local = {};
    metal::float2 _e18_ = E5_1_;
    float _e20_ = n.qd;
    metal::float4 _e21_ = IC.sample(S5_, _e18_, metal::bias(_e20_));
    float _e22_ = H1_1_;
    metal::float4 _e23_ = _e21_ * _e22_;
    metal::float3 _e24_ = _e23_.xyz;
    metal::float4 _e26_ = gl_FragCoord_1_;
    float _e28_ = n.z3_;
    float _e30_ = n.A3_;
    if (fh) {
        local = _e23_.w != 0.0;
    } else {
        local = false;
    }
    bool _e28 = local;
    if (_e28) {
        phi_204_ = metal::float3((metal::fract(52.982918 * metal::fract((0.06711056 * _e26_.x) + (0.00583715 * _e26_.y))) * _e28_) + _e30_) + _e24_;
    } else {
        phi_204_ = _e24_;
    }
    metal::float3 _e46_ = phi_204_;
    metal::float4 _e52_ = metal::float4(_e46_.x, _e23_.y, _e23_.z, _e23_.w);
    metal::float4 _e58_ = metal::float4(_e52_.x, _e46_.y, _e52_.z, _e52_.w);
    Jg = metal::float4(_e58_.x, _e58_.y, _e46_.z, _e58_.w);
    return;
}

struct main_Input {
    metal::float2 E5_ [[user(loc0), center_perspective]];
    float H1_ [[user(loc3), flat]];
    float I3_ [[user(loc1), flat]];
    uint A1_ [[user(loc4), flat]];
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
) {
    metal::float2 E5_1_ = {};
    float H1_1_ = {};
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 Jg = {};
    float I3_1_ = {};
    uint A1_1_ = {};
    const auto E5_ = varyings.E5_;
    const auto H1_ = varyings.H1_;
    const auto I3_ = varyings.I3_;
    const auto A1_ = varyings.A1_;
    E5_1_ = E5_;
    H1_1_ = H1_;
    gl_FragCoord_1_ = gl_FragCoord;
    I3_1_ = I3_;
    A1_1_ = A1_;
    main_1_(IC, S5_, E5_1_, n, H1_1_, gl_FragCoord_1_, Jg);
    metal::float4 _e11_ = Jg;
    return main_Output { _e11_ };
}
