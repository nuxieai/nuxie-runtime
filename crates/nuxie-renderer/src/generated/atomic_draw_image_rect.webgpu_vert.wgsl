struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Eg {
    g2_: array<vec4<u32>>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct Fg {
    g2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) member: f32,
    @location(0) member_1: vec2<f32>,
    @location(3) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: vec4<f32>,
    @location(5) @interpolate(flat, either) member_4: u32,
    @location(6) @interpolate(flat, either) member_5: u32,
    @location(2) member_6: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override Fh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> FC_1: vec4<f32>;
var<private> Z4_: f32;
var<private> WB_1: vec4<f32>;
var<private> OD_1: vec4<f32>;
var<private> c2_: vec2<f32>;
var<private> AD_1: vec4<f32>;
var<private> NB_1: vec4<f32>;
var<private> O0_: vec4<f32>;
var<private> RB_1: vec4<f32>;
var<private> K1_: vec4<f32>;
var<private> XB_1: u32;
var<private> B3_: u32;
var<private> YB_1: u32;
var<private> D1_: u32;
var<private> ZB_1: u32;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> NC_1: vec4<f32>;
var<private> PD_1: vec4<f32>;
var<private> N5_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> OB: Eg;
@group(0) @binding(3)
var<storage> CD: kf;
@group(0) @binding(4)
var<storage> PB: lf;
@group(0) @binding(5)
var<storage> HD: Fg;
@group(3) @binding(9)
var ca: sampler;
var<private> LC_1: u32;

fn main_1() {
    var phi_228_: bool;
    var phi_752_: vec2<f32>;
    var phi_754_: vec2<f32>;
    var phi_753_: vec2<f32>;
    var phi_755_: vec2<f32>;
    var phi_592_: bool;
    var phi_756_: vec4<f32>;
    var phi_769_: vec4<f32>;
    var phi_770_: vec4<f32>;

    let _e48 = FC_1[2u];
    let _e49 = (_e48 == 0f);
    phi_228_ = _e49;
    if !(_e49) {
        let _e52 = FC_1[3u];
        phi_228_ = (_e52 == 0f);
    }
    let _e55 = phi_228_;
    Z4_ = select(1f, 0f, _e55);
    let _e57 = FC_1;
    let _e58 = _e57.xy;
    let _e59 = WB_1;
    let _e64 = vec2<f32>(_e59.x, _e59.y);
    let _e65 = vec2<f32>(_e59.z, _e59.w);
    let _e66 = mat2x2<f32>(_e64, _e65);
    let _e68 = transpose(_naga_inverse_2x2_f32(_e66));
    phi_753_ = _e58;
    if !(_e55) {
        let _e78 = ((0.5f * (abs(_e68[1].x) + abs(_e68[1].y))) / dot(_e65, _e68[1]));
        if (_e78 >= 0.5f) {
            let _e90 = Z4_;
            Z4_ = (_e90 * (0.5f / _e78));
            phi_752_ = vec2<f32>(0.5f, _e58.y);
        } else {
            phi_752_ = vec2<f32>((_e57.x + (_e78 * _e48)), _e58.y);
        }
        let _e93 = phi_752_;
        let _e102 = ((0.5f * (abs(_e68[0].x) + abs(_e68[0].y))) / dot(_e64, _e68[0]));
        if (_e102 >= 0.5f) {
            let _e116 = Z4_;
            Z4_ = (_e116 * (0.5f / _e102));
            phi_754_ = vec2<f32>(_e93.x, 0.5f);
        } else {
            let _e105 = FC_1[3u];
            phi_754_ = vec2<f32>(_e93.x, (_e93.y + (_e102 * _e105)));
        }
        let _e119 = phi_754_;
        phi_753_ = _e119;
    }
    let _e121 = phi_753_;
    let _e122 = OD_1;
    let _e131 = AD_1;
    c2_ = ((mat2x2<f32>(vec2<f32>(_e122.x, _e122.y), vec2<f32>(_e122.z, _e122.w)) * _e121) + _e131.xy);
    let _e135 = NB_1;
    let _e137 = ((_e66 * _e121) + _e135.xy);
    phi_755_ = _e137;
    if _e55 {
        let _e139 = (_e68 * _e57.zw);
        phi_755_ = (_e137 + ((_e139 * ((abs(_e139.x) + abs(_e139.y)) / dot(_e139, _e139))) * 0.5f));
    }
    let _e151 = phi_755_;
    if Fh {
        let _e152 = RB_1;
        let _e157 = vec2<f32>(_e152.x, _e152.y);
        let _e158 = vec2<f32>(_e152.z, _e152.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e164 = (abs(_e157) + abs(_e158));
                let _e166 = (_e164.x != 0f);
                phi_592_ = _e166;
                if _e166 {
                    phi_592_ = (_e164.y != 0f);
                }
                let _e170 = phi_592_;
                if _e170 {
                    let _e174 = ((mat2x2<f32>(_e157, _e158) * _e151) + _e135.zw);
                    let _e175 = -(_e174);
                    let _e181 = (vec2<f32>(1f, 1f) / _e164).xyxy;
                    phi_756_ = (((vec4<f32>(_e174.x, _e174.y, _e175.x, _e175.y) * _e181) + _e181) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_756_ = _e135.zwzw;
                    break;
                }
            }
        }
        let _e186 = phi_756_;
        O0_ = _e186;
    }
    let _e187 = XB_1;
    K1_ = unpack4x8unorm(_e187);
    let _e189 = YB_1;
    B3_ = _e189;
    let _e190 = ZB_1;
    D1_ = _e190;
    let _e192 = j.Hf;
    let _e194 = j.If;
    let _e204 = NC_1[3u];
    if (_e204 != 0f) {
        let _e206 = PD_1;
        let _e215 = NC_1;
        let _e217 = NC_1[2u];
        let _e222 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, _e217);
        let _e224 = ((mat2x2<f32>(vec2<f32>(_e206.x, _e206.y), vec2<f32>(_e206.z, _e206.w)) * _e151) + _e131.zw);
        if (_e215.x > 0.9f) {
            phi_769_ = vec4<f32>(_e222.x, _e222.y, 2f, _e222.w);
        } else {
            phi_769_ = vec4<f32>(_e222.x, _e222.y, _e215.y, _e222.w);
        }
        let _e239 = phi_769_;
        if (_e204 == 2f) {
            let _e265 = vec4<f32>(_e224.x, _e239.y, _e239.z, _e239.w);
            phi_770_ = vec4<f32>(_e265.x, 0f, _e265.z, _e265.w);
        } else {
            let _e247 = vec4<f32>(_e239.x, _e239.y, -(_e239.z), _e239.w);
            let _e253 = vec4<f32>(_e224.x, _e247.y, _e247.z, _e247.w);
            phi_770_ = vec4<f32>(_e253.x, _e224.y, _e253.z, _e253.w);
        }
        let _e272 = phi_770_;
        N5_ = _e272;
    }
    unnamed.gl_Position = vec4<f32>(((_e151.x * _e192) - 1f), ((_e151.y * _e194) - sign(_e194)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) FC: vec4<f32>, @location(2) WB: vec4<f32>, @location(9) OD: vec4<f32>, @location(11) AD: vec4<f32>, @location(4) NB: vec4<f32>, @location(3) RB: vec4<f32>, @location(5) XB: u32, @location(6) YB: u32, @location(7) ZB: u32, @location(12) NC: vec4<f32>, @location(10) PD: vec4<f32>, @location(8) LC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    FC_1 = FC;
    WB_1 = WB;
    OD_1 = OD;
    AD_1 = AD;
    NB_1 = NB;
    RB_1 = RB;
    XB_1 = XB;
    YB_1 = YB;
    ZB_1 = ZB;
    NC_1 = NC;
    PD_1 = PD;
    LC_1 = LC;
    main_1();
    let _e39 = Z4_;
    let _e40 = c2_;
    let _e41 = O0_;
    let _e42 = K1_;
    let _e43 = B3_;
    let _e44 = D1_;
    let _e45 = N5_;
    let _e46 = unnamed.gl_Position;
    return VertexOutput(_e39, _e40, _e41, _e42, _e43, _e44, _e45, _e46);
}

fn _naga_inverse_2x2_f32(m: mat2x2<f32>) -> mat2x2<f32> {
    var adj: mat2x2<f32>;
    adj[0][0] = m[1][1];
    adj[0][1] = -m[0][1];
    adj[1][0] = -m[1][0];
    adj[1][1] = m[0][0];

    let det: f32 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
    return adj * (1 / det);
}
