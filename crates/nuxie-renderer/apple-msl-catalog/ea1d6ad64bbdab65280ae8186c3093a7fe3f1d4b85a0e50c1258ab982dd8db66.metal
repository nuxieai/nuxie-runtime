// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint buffer_size30;
    uint buffer_size29;
    uint buffer_size28;
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
    metal::float2 member;
    char _pad1[8];
    metal::float4 member_1_;
    float member_2_;
    uint member_3_;
    uint member_4_;
    char _pad5[4];
    metal::float4 gl_Position;
};
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
    thread metal::float4& WB_1_,
    thread metal::float2& OC_1_,
    thread metal::float4& NB_1_,
    thread metal::float2& X1_,
    thread metal::float2& PC_1_,
    thread metal::float4& L0_,
    thread metal::float4& QB_1_,
    thread float& H1_,
    thread metal::uint4& IB_1_,
    thread uint& w3_,
    thread uint& A1_,
    constant CC& n,
    thread gl_PerVertex& unnamed
) {
    bool phi_313_ = {};
    metal::float4 phi_376_ = {};
    metal::float4 _e35_ = WB_1_;
    metal::float2 _e43_ = OC_1_;
    metal::float4 _e45_ = NB_1_;
    metal::float2 _e47_ = (metal::float2x2(metal::float2(_e35_.x, _e35_.y), metal::float2(_e35_.z, _e35_.w)) * _e43_) + _e45_.xy;
    metal::float2 _e48_ = PC_1_;
    X1_ = _e48_;
    if (Zg) {
        metal::float4 _e49_ = QB_1_;
        metal::float2 _e54_ = metal::float2(_e49_.x, _e49_.y);
        metal::float2 _e55_ = metal::float2(_e49_.z, _e49_.w);
        switch(as_type<int>(0u)) {
            default: {
                metal::float2 _e61_ = metal::abs(_e54_) + metal::abs(_e55_);
                bool _e63_ = _e61_.x != 0.0;
                phi_313_ = _e63_;
                if (_e63_) {
                    phi_313_ = _e61_.y != 0.0;
                }
                bool _e67_ = phi_313_;
                if (_e67_) {
                    metal::float2 _e71_ = (metal::float2x2(_e54_, _e55_) * _e47_) + _e45_.zw;
                    metal::float2 _e72_ = -(_e71_);
                    metal::float4 _e78_ = (metal::float2(1.0, 1.0) / _e61_).xyxy;
                    phi_376_ = ((metal::float4(_e71_.x, _e71_.y, _e72_.x, _e72_.y) * _e78_) + _e78_) + metal::float4(0.5, 0.5, 0.5, 0.5);
                    break;
                } else {
                    phi_376_ = _e45_.zwzw;
                    break;
                }
                break;
            }
        }
        metal::float4 _e83_ = phi_376_;
        L0_ = _e83_;
    }
    uint _e85_ = IB_1_.x;
    H1_ = as_type<float>(_e85_);
    uint _e88_ = IB_1_.y;
    w3_ = _e88_;
    uint _e90_ = IB_1_.z;
    A1_ = _e90_;
    float _e92_ = n.ff;
    float _e94_ = n.gf;
    unnamed.gl_Position = metal::float4((_e47_.x * _e92_) - 1.0, (_e47_.y * _e94_) - metal::sign(_e94_), 0.0, 1.0);
    return;
}

struct main_Output {
    metal::float2 member [[user(loc0), center_perspective]];
    metal::float4 member_1_ [[user(loc1), center_perspective]];
    float member_2_ [[user(loc3), flat]];
    uint member_3_ [[user(loc4), flat]];
    uint member_4_ [[user(loc5), flat]];
    metal::float4 gl_Position [[position]];
};
struct vb_30_type { metal::uchar data[8]; };
struct vb_29_type { metal::uchar data[8]; };
struct vb_28_type { metal::uchar data[64]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, uint gl_InstanceIndex [[instance_id]]
, constant CC& n [[buffer(0)]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, const device vb_29_type* vb_29_in [[buffer(29)]]
, const device vb_28_type* vb_28_in [[buffer(28)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(3)]]
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
    if (gl_InstanceIndex < (_buffer_sizes.buffer_size28 / 64)) {
        const vb_28_type vb_28_elem = vb_28_in[gl_InstanceIndex];
        WB = unpackFloat32x4_(vb_28_elem.data[0], vb_28_elem.data[1], vb_28_elem.data[2], vb_28_elem.data[3], vb_28_elem.data[4], vb_28_elem.data[5], vb_28_elem.data[6], vb_28_elem.data[7], vb_28_elem.data[8], vb_28_elem.data[9], vb_28_elem.data[10], vb_28_elem.data[11], vb_28_elem.data[12], vb_28_elem.data[13], vb_28_elem.data[14], vb_28_elem.data[15]);
        QB = unpackFloat32x4_(vb_28_elem.data[16], vb_28_elem.data[17], vb_28_elem.data[18], vb_28_elem.data[19], vb_28_elem.data[20], vb_28_elem.data[21], vb_28_elem.data[22], vb_28_elem.data[23], vb_28_elem.data[24], vb_28_elem.data[25], vb_28_elem.data[26], vb_28_elem.data[27], vb_28_elem.data[28], vb_28_elem.data[29], vb_28_elem.data[30], vb_28_elem.data[31]);
        NB = unpackFloat32x4_(vb_28_elem.data[32], vb_28_elem.data[33], vb_28_elem.data[34], vb_28_elem.data[35], vb_28_elem.data[36], vb_28_elem.data[37], vb_28_elem.data[38], vb_28_elem.data[39], vb_28_elem.data[40], vb_28_elem.data[41], vb_28_elem.data[42], vb_28_elem.data[43], vb_28_elem.data[44], vb_28_elem.data[45], vb_28_elem.data[46], vb_28_elem.data[47]);
        IB = unpackUint32x4_(vb_28_elem.data[48], vb_28_elem.data[49], vb_28_elem.data[50], vb_28_elem.data[51], vb_28_elem.data[52], vb_28_elem.data[53], vb_28_elem.data[54], vb_28_elem.data[55], vb_28_elem.data[56], vb_28_elem.data[57], vb_28_elem.data[58], vb_28_elem.data[59], vb_28_elem.data[60], vb_28_elem.data[61], vb_28_elem.data[62], vb_28_elem.data[63]);
    }
    int gl_VertexIndex_1_ = {};
    int gl_InstanceIndex_1_ = {};
    metal::float4 WB_1_ = {};
    metal::float2 OC_1_ = {};
    metal::float4 NB_1_ = {};
    metal::float2 X1_ = {};
    metal::float2 PC_1_ = {};
    metal::float4 L0_ = {};
    metal::float4 QB_1_ = {};
    float H1_ = {};
    metal::uint4 IB_1_ = {};
    uint w3_ = {};
    uint A1_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_6 {}, type_6 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    WB_1_ = WB;
    OC_1_ = OC;
    NB_1_ = NB;
    PC_1_ = PC;
    QB_1_ = QB;
    IB_1_ = IB;
    main_1_(WB_1_, OC_1_, NB_1_, X1_, PC_1_, L0_, QB_1_, H1_, IB_1_, w3_, A1_, n, unnamed);
    metal::float2 _e25_ = X1_;
    metal::float4 _e26_ = L0_;
    float _e27_ = H1_;
    uint _e28_ = w3_;
    uint _e29_ = A1_;
    metal::float4 _e30_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e25_, {}, _e26_, _e27_, _e28_, _e29_, {}, _e30_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.member_3_, _tmp.member_4_, _tmp.gl_Position };
}
