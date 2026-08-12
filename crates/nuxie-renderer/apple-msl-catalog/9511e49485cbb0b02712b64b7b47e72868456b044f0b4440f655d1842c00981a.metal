// language: metal3.1
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct type_2 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_2 gl_ClipDistance;
    type_2 gl_CullDistance;
    char _pad4[4];
};
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
struct VertexOutput {
    metal::float2 member;
    char _pad1[8];
    metal::float4 gl_Position;
};

void main_1_(
    thread int& gl_VertexIndex_1_,
    thread metal::float2& X1_,
    thread gl_PerVertex& unnamed
) {
    int _e14_ = gl_VertexIndex_1_;
    float _e17_ = ((_e14_ & 1) == 0) ? -1.0 : 1.0;
    float _e20_ = ((_e14_ & 2) == 0) ? -1.0 : 1.0;
    X1_.x = (_e17_ * 0.5) + 0.5;
    X1_.y = (_e20_ * -0.5) + 0.5;
    unnamed.gl_Position = metal::float4(_e17_, _e20_, 0.0, 1.0);
    return;
}

struct main_Input {
};
struct main_Output {
    metal::float2 member [[user(loc0), center_perspective]];
    metal::float4 gl_Position [[position]];
};
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
) {
    int gl_VertexIndex_1_ = {};
    metal::float2 X1_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_2 {}, type_2 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    main_1_(gl_VertexIndex_1_, X1_, unnamed);
    metal::float2 _e6_ = X1_;
    metal::float4 _e7_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e6_, {}, _e7_};
    return main_Output { _tmp.member, _tmp.gl_Position };
}
