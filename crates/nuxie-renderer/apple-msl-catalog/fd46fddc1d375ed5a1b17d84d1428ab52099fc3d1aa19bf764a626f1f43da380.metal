// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size2;
    uint size3;
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
typedef uint type_4[1];
struct q4Bd {
    type_4 c2_;
};
struct h0Bd {
    type_4 c2_;
};
constant bool Yg = false;

metal::int2 naga_f2i32(metal::float2 value) {
    return static_cast<metal::int2>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

void main_1_(
    thread metal::float4& gl_FragCoord_1_,
    constant CC& n,
    device q4Bd& q4_,
    device h0Bd& h0_,
    constant _mslBufferSizes& _buffer_sizes
) {
    metal::float4 _e25_ = gl_FragCoord_1_;
    metal::uint2 _e29_ = as_type<metal::uint2>(naga_f2i32(metal::floor(_e25_.xy)));
    uint _e31_ = n.m6_;
    int _e60_ = as_type<int>(((((_e29_.y >> as_type<uint>(5u)) * (((_e31_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e29_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e29_.x & 28u) << as_type<uint>(5u)) + ((_e29_.y & 28u) << as_type<uint>(2)))) + (((_e29_.y & 3u) << as_type<uint>(2)) + (_e29_.x & 3u)));
    uint _e62_ = n.Se;
    q4_.c2_[metal::min(unsigned(_e60_), (_buffer_sizes.size2 - 0 - 4) / 4)] = _e62_;
    if (Yg) {
        h0_.c2_[metal::min(unsigned(_e60_), (_buffer_sizes.size3 - 0 - 4) / 4)] = 0u;
    }
    metal::discard_fragment();
}

struct main_Input {
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  metal::float4 gl_FragCoord [[position]]
, constant CC& n [[buffer(0)]]
, device q4Bd& q4_ [[buffer(6)]]
, device h0Bd& h0_ [[buffer(5)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    metal::float4 C1_ = {};
    gl_FragCoord_1_ = gl_FragCoord;
    main_1_(gl_FragCoord_1_, n, q4_, h0_, _buffer_sizes);
    metal::float4 _e3_ = C1_;
    return main_Output { _e3_ };
}
