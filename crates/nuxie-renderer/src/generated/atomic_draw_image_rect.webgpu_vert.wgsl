struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Cg {
    e2_: array<vec4<u32>>,
}

struct jf {
    e2_: array<vec2<u32>>,
}

struct kf {
    e2_: array<vec4<f32>>,
}

struct Dg {
    e2_: array<vec4<u32>>,
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

@id(1) override Dh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> GC_1: vec4<f32>;
var<private> V4_: f32;
var<private> WB_1: vec4<f32>;
var<private> PD_1: vec4<f32>;
var<private> Z1_: vec2<f32>;
var<private> BD_1: vec4<f32>;
var<private> NB_1: vec4<f32>;
var<private> M0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> H1_: vec4<f32>;
var<private> XB_1: u32;
var<private> y3_: u32;
var<private> YB_1: u32;
var<private> A1_: u32;
var<private> ZB_1: u32;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> OC_1: vec4<f32>;
var<private> QD_1: vec4<f32>;
var<private> N5_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> PB: Cg;
@group(0) @binding(3)
var<storage> DD: jf;
@group(0) @binding(4)
var<storage> QB: kf;
@group(0) @binding(5)
var<storage> ID: Dg;
@group(3) @binding(9)
var ea: sampler;
var<private> MC_1: u32;

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

    let _e48 = GC_1[2u];
    let _e49 = (_e48 == 0f);
    phi_228_ = _e49;
    if !(_e49) {
        let _e52 = GC_1[3u];
        phi_228_ = (_e52 == 0f);
    }
    let _e55 = phi_228_;
    V4_ = select(1f, 0f, _e55);
    let _e57 = GC_1;
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
            let _e90 = V4_;
            V4_ = (_e90 * (0.5f / _e78));
            phi_752_ = vec2<f32>(0.5f, _e58.y);
        } else {
            phi_752_ = vec2<f32>((_e57.x + (_e78 * _e48)), _e58.y);
        }
        let _e93 = phi_752_;
        let _e102 = ((0.5f * (abs(_e68[0].x) + abs(_e68[0].y))) / dot(_e64, _e68[0]));
        if (_e102 >= 0.5f) {
            let _e116 = V4_;
            V4_ = (_e116 * (0.5f / _e102));
            phi_754_ = vec2<f32>(_e93.x, 0.5f);
        } else {
            let _e105 = GC_1[3u];
            phi_754_ = vec2<f32>(_e93.x, (_e93.y + (_e102 * _e105)));
        }
        let _e119 = phi_754_;
        phi_753_ = _e119;
    }
    let _e121 = phi_753_;
    let _e122 = PD_1;
    let _e131 = BD_1;
    Z1_ = ((mat2x2<f32>(vec2<f32>(_e122.x, _e122.y), vec2<f32>(_e122.z, _e122.w)) * _e121) + _e131.xy);
    let _e135 = NB_1;
    let _e137 = ((_e66 * _e121) + _e135.xy);
    phi_755_ = _e137;
    if _e55 {
        let _e139 = (_e68 * _e57.zw);
        phi_755_ = (_e137 + ((_e139 * ((abs(_e139.x) + abs(_e139.y)) / dot(_e139, _e139))) * 0.5f));
    }
    let _e151 = phi_755_;
    if Dh {
        let _e152 = SB_1;
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
        M0_ = _e186;
    }
    let _e187 = XB_1;
    H1_ = unpack4x8unorm(_e187);
    let _e189 = YB_1;
    y3_ = _e189;
    let _e190 = ZB_1;
    A1_ = _e190;
    let _e192 = l.Ff;
    let _e194 = l.Gf;
    let _e204 = OC_1[3u];
    if (_e204 != 0f) {
        let _e206 = QD_1;
        let _e215 = OC_1;
        let _e217 = OC_1[2u];
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
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) GC: vec4<f32>, @location(2) WB: vec4<f32>, @location(9) PD: vec4<f32>, @location(11) BD: vec4<f32>, @location(4) NB: vec4<f32>, @location(3) SB: vec4<f32>, @location(5) XB: u32, @location(6) YB: u32, @location(7) ZB: u32, @location(12) OC: vec4<f32>, @location(10) QD: vec4<f32>, @location(8) MC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    GC_1 = GC;
    WB_1 = WB;
    PD_1 = PD;
    BD_1 = BD;
    NB_1 = NB;
    SB_1 = SB;
    XB_1 = XB;
    YB_1 = YB;
    ZB_1 = ZB;
    OC_1 = OC;
    QD_1 = QD;
    MC_1 = MC;
    main_1();
    let _e39 = V4_;
    let _e40 = Z1_;
    let _e41 = M0_;
    let _e42 = H1_;
    let _e43 = y3_;
    let _e44 = A1_;
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
