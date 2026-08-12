// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint buffer_size30;
    uint buffer_size29;
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
    float member;
    char _pad1[4];
    metal::float2 member_1_;
    metal::float4 member_2_;
    float member_3_;
    uint member_4_;
    uint member_5_;
    char _pad6[4];
    metal::float4 gl_Position;
};
constant bool Zg = true;
metal::float4 unpackFloat32x4_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11, uint b12, uint b13, uint b14, uint b15) {
    return metal::float4(as_type<float>(b3 << 24 | b2 << 16 | b1 << 8 | b0), as_type<float>(b7 << 24 | b6 << 16 | b5 << 8 | b4), as_type<float>(b11 << 24 | b10 << 16 | b9 << 8 | b8), as_type<float>(b15 << 24 | b14 << 16 | b13 << 8 | b12));
}
metal::uint4 unpackUint32x4_(uint b0, uint b1, uint b2, uint b3, uint b4, uint b5, uint b6, uint b7, uint b8, uint b9, uint b10, uint b11, uint b12, uint b13, uint b14, uint b15) {
    return metal::uint4((b3 << 24 | b2 << 16 | b1 << 8 | b0), (b7 << 24 | b6 << 16 | b5 << 8 | b4), (b11 << 24 | b10 << 16 | b9 << 8 | b8), (b15 << 24 | b14 << 16 | b13 << 8 | b12));
}

metal::float2x2 _naga_inverse_2x2_f32_(
    metal::float2x2 m
) {
    metal::float2x2 adj = {};
    adj[0].x = m[1].y;
    adj[0].y = -(m[0].y);
    adj[1].x = -(m[1].x);
    adj[1].y = m[0].x;
    float det = (m[0].x * m[1].y) - (m[1].x * m[0].y);
    metal::float2x2 _e31 = adj;
    return _e31 * (1.0 / det);
}

void main_1_(
    thread metal::float4& HC_1_,
    thread float& R4_,
    thread metal::float4& WB_1_,
    thread metal::float2& X1_,
    thread metal::float4& NB_1_,
    thread metal::float4& L0_,
    thread metal::float4& QB_1_,
    thread float& H1_,
    thread metal::uint4& IB_1_,
    thread uint& w3_,
    thread uint& A1_,
    constant CC& n,
    thread gl_PerVertex& unnamed
) {
    bool phi_175_ = {};
    metal::float2 phi_566_ = {};
    metal::float2 phi_568_ = {};
    metal::float2 phi_567_ = {};
    metal::float2 phi_569_ = {};
    bool phi_481_ = {};
    metal::float4 phi_570_ = {};
    float _e37_ = HC_1_.z;
    bool _e38_ = _e37_ == 0.0;
    phi_175_ = _e38_;
    if (!(_e38_)) {
        float _e41_ = HC_1_.w;
        phi_175_ = _e41_ == 0.0;
    }
    bool _e44_ = phi_175_;
    R4_ = _e44_ ? 0.0 : 1.0;
    metal::float4 _e46_ = HC_1_;
    metal::float2 _e47_ = _e46_.xy;
    metal::float4 _e48_ = WB_1_;
    metal::float2 _e53_ = metal::float2(_e48_.x, _e48_.y);
    metal::float2 _e54_ = metal::float2(_e48_.z, _e48_.w);
    metal::float2x2 _e55_ = metal::float2x2(_e53_, _e54_);
    metal::float2x2 _e35 = _naga_inverse_2x2_f32_(_e55_);
    metal::float2x2 _e57_ = metal::transpose(_e35);
    phi_567_ = _e47_;
    if (!(_e44_)) {
        float _e67_ = (0.5 * (metal::abs(_e57_[1].x) + metal::abs(_e57_[1].y))) / metal::dot(_e54_, _e57_[1]);
        if (_e67_ >= 0.5) {
            float _e79_ = R4_;
            R4_ = _e79_ * (0.5 / _e67_);
            phi_566_ = metal::float2(0.5, _e47_.y);
        } else {
            phi_566_ = metal::float2(_e46_.x + (_e67_ * _e37_), _e47_.y);
        }
        metal::float2 _e82_ = phi_566_;
        float _e91_ = (0.5 * (metal::abs(_e57_[0].x) + metal::abs(_e57_[0].y))) / metal::dot(_e53_, _e57_[0]);
        if (_e91_ >= 0.5) {
            float _e105_ = R4_;
            R4_ = _e105_ * (0.5 / _e91_);
            phi_568_ = metal::float2(_e82_.x, 0.5);
        } else {
            float _e94_ = HC_1_.w;
            phi_568_ = metal::float2(_e82_.x, _e82_.y + (_e91_ * _e94_));
        }
        metal::float2 _e108_ = phi_568_;
        phi_567_ = _e108_;
    }
    metal::float2 _e110_ = phi_567_;
    X1_ = _e110_;
    metal::float4 _e112_ = NB_1_;
    metal::float2 _e114_ = (_e55_ * _e110_) + _e112_.xy;
    phi_569_ = _e114_;
    if (_e44_) {
        metal::float2 _e116_ = _e57_ * _e46_.zw;
        phi_569_ = _e114_ + ((_e116_ * ((metal::abs(_e116_.x) + metal::abs(_e116_.y)) / metal::dot(_e116_, _e116_))) * 0.5);
    }
    metal::float2 _e128_ = phi_569_;
    if (Zg) {
        metal::float4 _e129_ = QB_1_;
        metal::float2 _e134_ = metal::float2(_e129_.x, _e129_.y);
        metal::float2 _e135_ = metal::float2(_e129_.z, _e129_.w);
        switch(as_type<int>(0u)) {
            default: {
                metal::float2 _e141_ = metal::abs(_e134_) + metal::abs(_e135_);
                bool _e143_ = _e141_.x != 0.0;
                phi_481_ = _e143_;
                if (_e143_) {
                    phi_481_ = _e141_.y != 0.0;
                }
                bool _e147_ = phi_481_;
                if (_e147_) {
                    metal::float2 _e151_ = (metal::float2x2(_e134_, _e135_) * _e128_) + _e112_.zw;
                    metal::float2 _e152_ = -(_e151_);
                    metal::float4 _e158_ = (metal::float2(1.0, 1.0) / _e141_).xyxy;
                    phi_570_ = ((metal::float4(_e151_.x, _e151_.y, _e152_.x, _e152_.y) * _e158_) + _e158_) + metal::float4(0.5, 0.5, 0.5, 0.5);
                    break;
                } else {
                    phi_570_ = _e112_.zwzw;
                    break;
                }
                break;
            }
        }
        metal::float4 _e163_ = phi_570_;
        L0_ = _e163_;
    }
    uint _e165_ = IB_1_.x;
    H1_ = as_type<float>(_e165_);
    uint _e168_ = IB_1_.y;
    w3_ = _e168_;
    uint _e170_ = IB_1_.z;
    A1_ = _e170_;
    float _e172_ = n.ff;
    float _e174_ = n.gf;
    unnamed.gl_Position = metal::float4((_e128_.x * _e172_) - 1.0, (_e128_.y * _e174_) - metal::sign(_e174_), 0.0, 1.0);
    return;
}

struct main_Output {
    float member [[user(loc1), center_perspective]];
    metal::float2 member_1_ [[user(loc0), center_perspective]];
    metal::float4 member_2_ [[user(loc2), center_perspective]];
    float member_3_ [[user(loc3), flat]];
    uint member_4_ [[user(loc4), flat]];
    uint member_5_ [[user(loc5), flat]];
    metal::float4 gl_Position [[position]];
};
struct vb_30_type { metal::uchar data[16]; };
struct vb_29_type { metal::uchar data[64]; };
vertex main_Output main_(
  uint gl_VertexIndex [[vertex_id]]
, uint gl_InstanceIndex [[instance_id]]
, constant CC& n [[buffer(0)]]
, const device vb_30_type* vb_30_in [[buffer(30)]]
, const device vb_29_type* vb_29_in [[buffer(29)]]
, constant _mslBufferSizes& _buffer_sizes [[buffer(3)]]
) {
    metal::float4 HC = {};
    if (gl_VertexIndex < (_buffer_sizes.buffer_size30 / 16)) {
        const vb_30_type vb_30_elem = vb_30_in[gl_VertexIndex];
        HC = unpackFloat32x4_(vb_30_elem.data[0], vb_30_elem.data[1], vb_30_elem.data[2], vb_30_elem.data[3], vb_30_elem.data[4], vb_30_elem.data[5], vb_30_elem.data[6], vb_30_elem.data[7], vb_30_elem.data[8], vb_30_elem.data[9], vb_30_elem.data[10], vb_30_elem.data[11], vb_30_elem.data[12], vb_30_elem.data[13], vb_30_elem.data[14], vb_30_elem.data[15]);
    }
    metal::float4 WB = {};
    metal::float4 QB = {};
    metal::float4 NB = {};
    metal::uint4 IB = {};
    if (gl_InstanceIndex < (_buffer_sizes.buffer_size29 / 64)) {
        const vb_29_type vb_29_elem = vb_29_in[gl_InstanceIndex];
        WB = unpackFloat32x4_(vb_29_elem.data[0], vb_29_elem.data[1], vb_29_elem.data[2], vb_29_elem.data[3], vb_29_elem.data[4], vb_29_elem.data[5], vb_29_elem.data[6], vb_29_elem.data[7], vb_29_elem.data[8], vb_29_elem.data[9], vb_29_elem.data[10], vb_29_elem.data[11], vb_29_elem.data[12], vb_29_elem.data[13], vb_29_elem.data[14], vb_29_elem.data[15]);
        QB = unpackFloat32x4_(vb_29_elem.data[16], vb_29_elem.data[17], vb_29_elem.data[18], vb_29_elem.data[19], vb_29_elem.data[20], vb_29_elem.data[21], vb_29_elem.data[22], vb_29_elem.data[23], vb_29_elem.data[24], vb_29_elem.data[25], vb_29_elem.data[26], vb_29_elem.data[27], vb_29_elem.data[28], vb_29_elem.data[29], vb_29_elem.data[30], vb_29_elem.data[31]);
        NB = unpackFloat32x4_(vb_29_elem.data[32], vb_29_elem.data[33], vb_29_elem.data[34], vb_29_elem.data[35], vb_29_elem.data[36], vb_29_elem.data[37], vb_29_elem.data[38], vb_29_elem.data[39], vb_29_elem.data[40], vb_29_elem.data[41], vb_29_elem.data[42], vb_29_elem.data[43], vb_29_elem.data[44], vb_29_elem.data[45], vb_29_elem.data[46], vb_29_elem.data[47]);
        IB = unpackUint32x4_(vb_29_elem.data[48], vb_29_elem.data[49], vb_29_elem.data[50], vb_29_elem.data[51], vb_29_elem.data[52], vb_29_elem.data[53], vb_29_elem.data[54], vb_29_elem.data[55], vb_29_elem.data[56], vb_29_elem.data[57], vb_29_elem.data[58], vb_29_elem.data[59], vb_29_elem.data[60], vb_29_elem.data[61], vb_29_elem.data[62], vb_29_elem.data[63]);
    }
    int gl_VertexIndex_1_ = {};
    int gl_InstanceIndex_1_ = {};
    metal::float4 HC_1_ = {};
    float R4_ = {};
    metal::float4 WB_1_ = {};
    metal::float2 X1_ = {};
    metal::float4 NB_1_ = {};
    metal::float4 L0_ = {};
    metal::float4 QB_1_ = {};
    float H1_ = {};
    metal::uint4 IB_1_ = {};
    uint w3_ = {};
    uint A1_ = {};
    gl_PerVertex unnamed = gl_PerVertex {metal::float4(0.0, 0.0, 0.0, 1.0), 1.0, type_6 {}, type_6 {}};
    gl_VertexIndex_1_ = static_cast<int>(gl_VertexIndex);
    gl_InstanceIndex_1_ = static_cast<int>(gl_InstanceIndex);
    HC_1_ = HC;
    WB_1_ = WB;
    NB_1_ = NB;
    QB_1_ = QB;
    IB_1_ = IB;
    main_1_(HC_1_, R4_, WB_1_, X1_, NB_1_, L0_, QB_1_, H1_, IB_1_, w3_, A1_, n, unnamed);
    float _e24_ = R4_;
    metal::float2 _e25_ = X1_;
    metal::float4 _e26_ = L0_;
    float _e27_ = H1_;
    uint _e28_ = w3_;
    uint _e29_ = A1_;
    metal::float4 _e30_ = unnamed.gl_Position;
    const auto _tmp = VertexOutput {_e24_, {}, _e25_, _e26_, _e27_, _e28_, _e29_, {}, _e30_};
    return main_Output { _tmp.member, _tmp.member_1_, _tmp.member_2_, _tmp.member_3_, _tmp.member_4_, _tmp.member_5_, _tmp.gl_Position };
}
