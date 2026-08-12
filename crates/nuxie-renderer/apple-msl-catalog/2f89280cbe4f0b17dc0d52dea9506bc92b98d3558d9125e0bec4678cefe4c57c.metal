// language: metal4.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint buffer_size30;
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
struct VertexOutput {
    metal::float4 member;
    metal::float4 gl_Position;
};
metal::uint4 unpackUint32x4_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11, uint b12, uint b13, uint b14, uint b15) {
    return metal::uint4((b3 << 24 | b2 << 16 | b1 << 8 | b0), (b7 << 24 | b6 << 16 | b5 << 8 | b4), (b11 << 24 | b10 << 16 | b9 << 8 | b8), (b15 << 24 | b14 << 16 | b13 << 8 | b12));
}

void main_1_(
    thread int& gl_VertexIndex_1_,
    thread metal::uint4& KC_1_,
    constant CC& n,
    thread metal::float4& R6_,
    thread gl_PerVertex& unnamed
) {
    uint phi_239_ = {};
    float phi_240_ = {};
    float phi_243_ = {};
    float phi_242_ = {};
    float phi_247_ = {};
    float phi_246_ = {};
    uint phi_244_ = {};
    bool local = {};
    bool local_1 = {};
    int _e31_ = gl_VertexIndex_1_;
    int _e33_ = _e31_ >> as_type<uint>(1);
    bool _e34_ = _e33_ <= 1;
    if (_e34_) {
        uint _e36_ = KC_1_.x;
        phi_239_ = _e36_ & 65535u;
    } else {
        uint _e39_ = KC_1_.x;
        phi_239_ = _e39_ >> as_type<uint>(16);
    }
    uint _e43_ = phi_239_;
    float _e45_ = static_cast<float>(_e43_) * 0.000015258789;
    float _e48_ = ((_e31_ & 1) == 0) ? 0.0 : 1.0;
    float _e50_ = n.ec;
    phi_240_ = _e48_;
    if (_e50_ < 0.0) {
        phi_240_ = 1.0 - _e48_;
    }
    float _e54_ = phi_240_;
    uint _e56_ = KC_1_.y;
    phi_242_ = _e45_;
    if ((_e56_ & 2147483648u) != 0u) {
        local = _e33_ == 0;
    } else {
        local = false;
    }
    bool _e56 = local;
    if (_e56) {
        if ((_e56_ & 536870912u) != 0u) {
            phi_243_ = 0.0;
        } else {
            phi_243_ = _e45_ - 0.001953125;
        }
        float _e68_ = phi_243_;
        phi_242_ = _e68_;
    }
    float _e70_ = phi_242_;
    phi_246_ = _e70_;
    if ((_e56_ & 1073741824u) != 0u) {
        local_1 = _e33_ == 3;
    } else {
        local_1 = false;
    }
    bool _e75 = local_1;
    if (_e75) {
        if ((_e56_ & 536870912u) != 0u) {
            phi_247_ = 1.0;
        } else {
            phi_247_ = _e70_ + 0.001953125;
        }
        float _e79_ = phi_247_;
        phi_246_ = _e79_;
    }
    float _e81_ = phi_246_;
    if (_e34_) {
        uint _e83_ = KC_1_.z;
        phi_244_ = _e83_;
    } else {
        uint _e85_ = KC_1_.w;
        phi_244_ = _e85_;
    }
    uint _e87_ = phi_244_;
    R6_ = static_cast<metal::float4>((metal::uint4(_e87_) >> as_type<metal::uint4>(metal::uint4(16u, 8u, 0u, 24u))) & metal::uint4(255u, 255u, 255u, 255u)) * metal::float4(0.003921569, 0.003921569, 0.003921569, 0.003921569);
    unnamed.gl_Position = metal::float4((_e81_ * 2.0) - 1.0, ((static_cast<float>(_e56_ & 536870911u) + _e54_) * _e50_) - metal::sign(_e50_), 0.0, 1.0);
    return;
}

struct main_Output {
    metal::float4 member [[user(loc0), center_perspective]];
    metal::float4 gl_Position [[position]];
};
struct vb_30_type { metal::uchar data[16]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, constant CC& n [[buffer(0)]]
, uint i_id [[instance_id]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(1)]]
) {
    metal::uint4 KC = {};
    if (i_id < (_buffer_sizes.buffer_size30 / 16)) {
        const vb_30_type vb_30_elem = vb_30_in[i_id];
        KC = unpackUint32x4_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7], vb_30_elem.data[8], vb_30_elem.data[9], vb_30_elem.data[10], vb_30_elem.data[11], vb_30_elem.data[12], vb_30_elem.data[13], vb_30_elem.data[14], vb_30_elem.data[15]);
    }
    int gl_VertexIndex_1_ = {};
    metal::uint4 KC_1_ = {};
    metal::float4 R6_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_6 {}, type_6 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    KC_1_ = KC;
    main_1_(gl_VertexIndex_1_, KC_1_, n, R6_, unnamed);
    metal::float4 _e8_ = R6_;
    metal::float4 _e9_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e8_, _e9_};
    return main_Output { _tmp.member, _tmp.gl_Position };
}
