// language: metal3.2
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size6;
    uint size7;
    uint size8;
    uint size9;
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
struct type_6 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_6 gl_ClipDistance;
    type_6 gl_CullDistance;
    char _pad4[4];
};
typedef metal::uint4 type_8[1];
struct bg {
    type_8 c2_;
};
typedef metal::uint2 type_10[1];
struct Je {
    type_10 c2_;
};
typedef metal::float4 type_11[1];
struct Ke {
    type_11 c2_;
};
struct cg {
    type_8 c2_;
};

void main_1_(
    thread int& gl_VertexIndex_1_,
    constant CC& n,
    thread gl_PerVertex& unnamed
) {
    int phi_170_ = {};
    int phi_173_ = {};
    int _e22_ = gl_VertexIndex_1_;
    if ((_e22_ & 1) == 0) {
        int _e27_ = n.R7_.x;
        phi_170_ = _e27_;
    } else {
        int _e30_ = n.R7_.z;
        phi_170_ = _e30_;
    }
    int _e32_ = phi_170_;
    if ((_e22_ & 2) == 0) {
        int _e37_ = n.R7_.y;
        phi_173_ = _e37_;
    } else {
        int _e40_ = n.R7_.w;
        phi_173_ = _e40_;
    }
    int _e42_ = phi_173_;
    metal::float2 _e44_ = static_cast<metal::float2>(metal::int2(_e32_, _e42_));
    float _e46_ = n.ff;
    float _e48_ = n.gf;
    unnamed.gl_Position = metal::float4((_e44_.x * _e46_) - 1.0, (_e44_.y * _e48_) - metal::sign(_e48_), 0.0, 1.0);
    return;
}

struct main_Input {
};
struct main_Output {
    metal::float4 member [[position]];
};
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, uint gl_InstanceIndex [[instance_id]]
, constant CC& n [[buffer(0)]]
) {
    int gl_VertexIndex_1_ = {};
    int gl_InstanceIndex_1_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_6 {}, type_6 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    main_1_(gl_VertexIndex_1_, n, unnamed);
    metal::float4 _e8_ = unnamed.gl_Position;
    return main_Output { _e8_ };
}
