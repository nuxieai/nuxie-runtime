// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint buffer_size30;
    uint buffer_size29;
    uint buffer_size28;
};

struct type_2 {
    float inner[4];
};
struct type_3 {
    float inner[1];
};
struct gl_PerVertex {
    metal::float4 gl_Position;
    float gl_PointSize;
    type_2 gl_ClipDistance;
    type_3 gl_CullDistance;
    char _pad4[8];
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
    metal::float4 gl_Position;
    type_2 gl_ClipDistance;
    metal::float2 member;
    float member_1_;
    float member_2_;
    uint member_3_;
    char _pad6[12];
};
constant bool Yg = true;
constant bool Zg = true;
metal::float2 unpackFloat32x2_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7) {
    return metal::float2(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4));
}
metal::float4 unpackFloat32x4_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11, uint b12, uint b13, uint b14, uint b15) {
    return metal::float4(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8), as_type<float>(b15 << 24 | b14 << 16 | b13 << 8 | b12));
}
metal::uint4 unpackUint32x4_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11, uint b12, uint b13, uint b14, uint b15) {
    return metal::uint4((b3 << 24 | b2 << 16 | b1 << 8 | b0), (b7 << 24 | b6 << 16 | b5 << 8 | b4), (b11 << 24 | b10 << 16 | b9 << 8 | b8), (b15 << 24 | b14 << 16 | b13 << 8 | b12));
}

void main_1_(
    thread gl_PerVertex& unnamed,
    thread metal::float4& WB_1_,
    thread metal::float2& OC_1_,
    thread metal::float4& NB_1_,
    thread metal::float2& E5_,
    thread metal::float2& PC_1_,
    thread float& I3_,
    thread metal::uint4& IB_1_,
    constant CC& n,
    thread metal::float4& QB_1_,
    thread float& H1_,
    thread uint& A1_
) {
    float phi_384_ = {};
    metal::float4 _e31_ = WB_1_;
    metal::float2 _e39_ = OC_1_;
    metal::float4 _e41_ = NB_1_;
    metal::float2 _e43_ = (metal::float2x2(metal::float2(_e31_.x, _e31_.y), metal::float2(_e31_.z, _e31_.w)) * _e39_) + _e41_.xy;
    metal::float2 _e44_ = PC_1_;
    E5_ = _e44_;
    if (Yg) {
        uint _e46_ = IB_1_.y;
        uint _e48_ = n.Z5_;
        if (_e46_ == 0u) {
            phi_384_ = 0.0;
        } else {
            phi_384_ = float2(as_type<half2>(((_e46_ + 1023u) * _e48_))).x;
        }
        float _e55_ = phi_384_;
        I3_ = _e55_;
    }
    if (Zg) {
        metal::float4 _e56_ = QB_1_;
        if (metal::any(_e56_ != metal::float4(0.0, 0.0, 0.0, 0.0))) {
            metal::float2 _e68_ = (metal::float2x2(metal::float2(_e56_.x, _e56_.y), metal::float2(_e56_.z, _e56_.w)) * _e43_) + _e41_.zw;
            unnamed.gl_ClipDistance.inner[0] = _e68_.x + 1.0;
            unnamed.gl_ClipDistance.inner[1] = _e68_.y + 1.0;
            unnamed.gl_ClipDistance.inner[2] = 1.0 - _e68_.x;
            unnamed.gl_ClipDistance.inner[3] = 1.0 - _e68_.y;
        } else {
            float _e84_ = _e41_.z - 0.5;
            unnamed.gl_ClipDistance.inner[3] = _e84_;
            unnamed.gl_ClipDistance.inner[2] = _e84_;
            unnamed.gl_ClipDistance.inner[1] = _e84_;
            unnamed.gl_ClipDistance.inner[0] = _e84_;
        }
    }
    float _e94_ = n.ff;
    float _e96_ = n.gf;
    metal::float4 _e104_ = metal::float4((_e43_.x * _e94_) - 1.0, (_e43_.y * _e96_) - metal::sign(_e96_), 0.0, 1.0);
    uint _e106_ = IB_1_.w;
    uint _e116_ = IB_1_.x;
    H1_ = as_type<float>(_e116_);
    uint _e119_ = IB_1_.z;
    A1_ = _e119_;
    unnamed.gl_Position = metal::float4(_e104_.x, _e104_.y, 1.0 - (static_cast<float>(_e106_) * 0.000061035156), _e104_.w);
    return;
}

struct main_Output {
    metal::float4 gl_Position [[position]];
    float gl_ClipDistance [[clip_distance]] [4];
    metal::float2 member [[user(loc0), center_perspective]];
    float member_1_ [[user(loc1), flat]];
    float member_2_ [[user(loc3), flat]];
    uint member_3_ [[user(loc4), flat]];
};
struct vb_30_type { metal::uchar data[8]; };
struct vb_29_type { metal::uchar data[8]; };
struct vb_28_type { metal::uchar data[64]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, constant CC& n [[buffer(0)]]
, uint i_id [[instance_id]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, const device vb_29_type* vb_29_in [[buffer(29)]]
, const device vb_28_type* vb_28_in [[buffer(28)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(1)]]
) {
    metal::float2 OC = {};
    if (gl_VertexIndex < (_buffer_sizes.buffer_size30 / 8)) {
        const vb_30_type vb_30_elem = vb_30_in[gl_VertexIndex];
        OC = unpackFloat32x2_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7]);
    }
    metal::float2 PC = {};
    if (gl_VertexIndex < (_buffer_sizes.buffer_size29 / 8)) {
        const vb_29_type vb_29_elem = vb_29_in[gl_VertexIndex];
        PC = unpackFloat32x2_(vb_29_elem.data[0], vb_29_elem.data[1], vb_29_elem.data[2], vb_29_elem.data[3], vb_29_elem.data[4], vb_29_elem.data[5], vb_29_elem.data[6], vb_29_elem.data[7]);
    }
    metal::float4 WB = {};
    metal::float4 QB = {};
    metal::float4 NB = {};
    metal::uint4 IB = {};
    if (i_id < (_buffer_sizes.buffer_size28 / 64)) {
        const vb_28_type vb_28_elem = vb_28_in[i_id];
        WB = unpackFloat32x4_(vb_28_elem.data[0], vb_28_elem.data[1], vb_28_elem.data[2], vb_28_elem.data[3], vb_28_elem.data[4], vb_28_elem.data[5], vb_28_elem.data[6], vb_28_elem.data[7], vb_28_elem.data[8], vb_28_elem.data[9], vb_28_elem.data[10], vb_28_elem.data[11], vb_28_elem.data[12], vb_28_elem.data[13], vb_28_elem.data[14], vb_28_elem.data[15]);
        QB = unpackFloat32x4_(vb_28_elem.data[16], vb_28_elem.data[17], vb_28_elem.data[18], vb_28_elem.data[19], vb_28_elem.data[20], vb_28_elem.data[21], vb_28_elem.data[22], vb_28_elem.data[23], vb_28_elem.data[24], vb_28_elem.data[25], vb_28_elem.data[26], vb_28_elem.data[27], vb_28_elem.data[28], vb_28_elem.data[29], vb_28_elem.data[30], vb_28_elem.data[31]);
        NB = unpackFloat32x4_(vb_28_elem.data[32], vb_28_elem.data[33], vb_28_elem.data[34], vb_28_elem.data[35], vb_28_elem.data[36], vb_28_elem.data[37], vb_28_elem.data[38], vb_28_elem.data[39], vb_28_elem.data[40], vb_28_elem.data[41], vb_28_elem.data[42], vb_28_elem.data[43], vb_28_elem.data[44], vb_28_elem.data[45], vb_28_elem.data[46], vb_28_elem.data[47]);
        IB = unpackUint32x4_(vb_28_elem.data[48], vb_28_elem.data[49], vb_28_elem.data[50], vb_28_elem.data[51], vb_28_elem.data[52], vb_28_elem.data[53], vb_28_elem.data[54], vb_28_elem.data[55], vb_28_elem.data[56], vb_28_elem.data[57], vb_28_elem.data[58], vb_28_elem.data[59], vb_28_elem.data[60], vb_28_elem.data[61], vb_28_elem.data[62], vb_28_elem.data[63]);
    }
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_2 {}, type_3 {}};
    int gl_VertexIndex_1_ = {};
    metal::float4 WB_1_ = {};
    metal::float2 OC_1_ = {};
    metal::float4 NB_1_ = {};
    metal::float2 E5_ = {};
    metal::float2 PC_1_ = {};
    float I3_ = {};
    metal::uint4 IB_1_ = {};
    metal::float4 QB_1_ = {};
    float H1_ = {};
    uint A1_ = {};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    WB_1_ = WB;
    OC_1_ = OC;
    NB_1_ = NB;
    PC_1_ = PC;
    IB_1_ = IB;
    QB_1_ = QB;
    main_1_(unnamed, WB_1_, OC_1_, NB_1_, E5_, PC_1_, I3_, IB_1_, n, QB_1_, H1_, A1_);
    metal::float4 _e22_ = unnamed.gl_Position;
    type_2 _e23_ = unnamed.gl_ClipDistance;
    metal::float2 _e24_ = E5_;
    float _e25_ = I3_;
    float _e26_ = H1_;
    uint _e27_ = A1_;
    const auto _tmp = VertexOutput {_e22_, _e23_, _e24_, _e25_, _e26_, _e27_};
    return main_Output { _tmp.gl_Position, {_tmp.gl_ClipDistance.inner[0],_tmp.gl_ClipDistance.inner[1],_tmp.gl_ClipDistance.inner[2],_tmp.gl_ClipDistance.inner[3]}, _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.member_3_ };
}
