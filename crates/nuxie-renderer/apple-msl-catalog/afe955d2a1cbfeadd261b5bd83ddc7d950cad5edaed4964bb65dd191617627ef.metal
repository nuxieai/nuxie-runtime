// language: metal4.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size2;
    uint size4;
    uint size5;
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
struct j0Bd {
    type_4 c2_;
};
struct q4Bd {
    type_4 c2_;
};
struct h0Bd {
    type_4 c2_;
};
constant bool kh = false;
constant bool lh = true;
constant bool Yg = false;

metal::int2 naga_f2i32(metal::float2 value) {
    return static_cast<metal::int2>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

void main_1_(
    thread metal::float4& gl_FragCoord_1_,
    constant CC& n,
    device j0Bd& j0_,
    metal::texture2d<float, metal::access::sample> IC,
    device q4Bd& q4_,
    device h0Bd& h0_,
    constant _mslBufferSizes& _buffer_sizes
) {
    metal::float4 _e28_ = gl_FragCoord_1_;
    metal::int2 _e31_ = naga_f2i32(metal::floor(_e28_.xy));
    metal::uint2 _e32_ = as_type<metal::uint2>(_e31_);
    uint _e34_ = n.m6_;
    int _e63_ = as_type<int>(((((_e32_.y >> as_type<uint>(5u)) * (((_e34_ + 31u) & 4294967264u) << as_type<uint>(5u))) + ((_e32_.x >> as_type<uint>(5u)) << as_type<uint>(10u))) + (((_e32_.x & 28u) << as_type<uint>(5u)) + ((_e32_.y & 28u) << as_type<uint>(2)))) + (((_e32_.y & 3u) << as_type<uint>(2)) + (_e32_.x & 3u)));
    if (kh) {
        uint _e65_ = n.Re;
        j0_.c2_[metal::min(unsigned(_e63_), (_buffer_sizes.size2 - 0 - 4) / 4)] = metal::pack_float_to_unorm4x8(metal::unpack_unorm4x8_to_float(_e65_));
    }
    if (lh) {
        uint clamped_lod_e67 = metal::min(uint(0), IC.get_num_mip_levels() - 1);
        metal::float4 _e70_ = IC.read(metal::min(metal::uint2(_e31_), metal::uint2(IC.get_width(clamped_lod_e67), IC.get_height(clamped_lod_e67)) - 1), clamped_lod_e67);
        j0_.c2_[metal::min(unsigned(_e63_), (_buffer_sizes.size2 - 0 - 4) / 4)] = metal::pack_float_to_unorm4x8(_e70_);
    }
    uint _e75_ = n.Se;
    q4_.c2_[metal::min(unsigned(_e63_), (_buffer_sizes.size4 - 0 - 4) / 4)] = _e75_;
    if (Yg) {
        h0_.c2_[metal::min(unsigned(_e63_), (_buffer_sizes.size5 - 0 - 4) / 4)] = 0u;
    }
    return;
}

struct main_Input {
};
fragment void main_(
  metal::float4 gl_FragCoord [[position]]
, constant CC& n [[buffer(0)]]
, device j0Bd& j0_ [[buffer(4)]]
, metal::texture2d<float, metal::access::sample> IC [[texture(3)]]
, device q4Bd& q4_ [[buffer(6)]]
, device h0Bd& h0_ [[buffer(5)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(7)]]
) {
    metal::float4 gl_FragCoord_1_ = {};
    gl_FragCoord_1_ = gl_FragCoord;
    main_1_(gl_FragCoord_1_, n, j0_, IC, q4_, h0_, _buffer_sizes);
    return;
}
