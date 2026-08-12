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
    metal::texture2d<float, metal::access::sample> XC,
    metal::sampler aa,
    thread float& Jg,
    thread metal::float4& O_1_,
    thread bool& gl_FrontFacing_1_
) {
    float phi_419_ = {};
    float phi_423_ = {};
    float phi_424_ = {};
    metal::float4 _e25_ = O_1_;
    bool _e26_ = gl_FrontFacing_1_;
    float _e29_ = metal::max(_e25_.w, 0.0);
    if (_e25_.z >= 0.0) {
        metal::float4 _e32_ = XC.sample(aa, metal::float2(_e29_, 0.0), metal::level(0.0));
        phi_419_ = _e32_.x;
    } else {
        phi_419_ = 0.0;
    }
    float _e35_ = phi_419_;
    phi_423_ = _e35_;
    if (metal::abs(_e25_.z) < 1000.0) {
        float _e42_ = -2.0 - _e25_.y;
        float _e44_ = (_e42_ - _e29_) * 0.5984134;
        metal::float4 _e47_ = metal::float4(_e29_) + (metal::float4(0.20888568, 0.62665707, 1.0444285, 1.4621998) * _e44_);
        metal::float4 _e53_ = (_e47_ * -(_e25_.z)) + metal::float4((_e42_ * _e25_.z) + (metal::abs(_e25_.x) - 0.25));
        metal::float4 _e56_ = XC.sample(aa, metal::float2(_e53_.x, 0.0), metal::level(0.0));
        metal::float4 _e59_ = XC.sample(aa, metal::float2(_e53_.y, 0.0), metal::level(0.0));
        metal::float4 _e62_ = XC.sample(aa, metal::float2(_e53_.z, 0.0), metal::level(0.0));
        metal::float4 _e65_ = XC.sample(aa, metal::float2(_e53_.w, 0.0), metal::level(0.0));
        metal::float4 _e71_ = _e47_ * 5.0959306;
        phi_423_ = _e35_ + (metal::dot(metal::float4(_e56_.x, _e59_.x, _e62_.x, _e65_.x), metal::exp2((metal::float4(2.5479653, 2.5479653, 2.5479653, 2.5479653) - _e71_) * (_e71_ + metal::float4(-2.5479653, -2.5479653, -2.5479653, -2.5479653)))) * _e44_);
    }
    float _e80_ = phi_423_;
    float _e83_ = _e80_ * metal::sign(_e25_.x);
    phi_424_ = _e83_;
    if (!(_e26_)) {
        phi_424_ = -(_e83_);
    }
    float _e87_ = phi_424_;
    Jg = _e87_;
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
, bool gl_FrontFacing [[front_facing]]
, metal::texture2d<float, metal::access::sample> XC [[texture(2)]]
, metal::sampler aa [[sampler(2)]]
) {
    float Jg = {};
    metal::float4 O_1_ = {};
    bool gl_FrontFacing_1_ = {};
    const auto O = varyings.O;
    O_1_ = O;
    gl_FrontFacing_1_ = gl_FrontFacing;
    main_1_(XC, aa, Jg, O_1_, gl_FrontFacing_1_);
    float _e5_ = Jg;
    return main_Output { _e5_ };
}
