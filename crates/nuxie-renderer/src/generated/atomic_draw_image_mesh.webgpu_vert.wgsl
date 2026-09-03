struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ig {
    d2_: array<vec4<u32>>,
}

struct Ne {
    d2_: array<vec2<u32>>,
}

struct Oe {
    d2_: array<vec4<f32>>,
}

struct jg {
    d2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec4<f32>,
    @location(3) @interpolate(flat, either) member_2: f32,
    @location(4) @interpolate(flat, either) member_3: u32,
    @location(5) @interpolate(flat, either) member_4: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(1) override gh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> XB_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> OB_1: vec4<f32>;
var<private> Y1_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> M0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> H1_: f32;
var<private> IB_1: vec4<u32>;
var<private> x3_: u32;
var<private> A1_: u32;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> QB: ig;
@group(0) @binding(3)
var<storage> BD: Ne;
@group(0) @binding(4)
var<storage> RB: Oe;
@group(0) @binding(5)
var<storage> FD: jg;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_315_: bool;
    var phi_378_: vec4<f32>;

    let _e35 = XB_1;
    let _e43 = PC_1;
    let _e45 = OB_1;
    let _e47 = ((mat2x2<f32>(vec2<f32>(_e35.x, _e35.y), vec2<f32>(_e35.z, _e35.w)) * _e43) + _e45.xy);
    let _e48 = QC_1;
    Y1_ = _e48;
    if gh {
        let _e49 = SB_1;
        let _e54 = vec2<f32>(_e49.x, _e49.y);
        let _e55 = vec2<f32>(_e49.z, _e49.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e61 = (abs(_e54) + abs(_e55));
                let _e63 = (_e61.x != 0f);
                phi_315_ = _e63;
                if _e63 {
                    phi_315_ = (_e61.y != 0f);
                }
                let _e67 = phi_315_;
                if _e67 {
                    let _e71 = ((mat2x2<f32>(_e54, _e55) * _e47) + _e45.zw);
                    let _e72 = -(_e71);
                    let _e78 = (vec2<f32>(1f, 1f) / _e61).xyxy;
                    phi_378_ = (((vec4<f32>(_e71.x, _e71.y, _e72.x, _e72.y) * _e78) + _e78) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_378_ = _e45.zwzw;
                    break;
                }
            }
        }
        let _e83 = phi_378_;
        M0_ = _e83;
    }
    let _e85 = IB_1[0u];
    H1_ = bitcast<f32>(_e85);
    let _e88 = IB_1[1u];
    x3_ = _e88;
    let _e90 = IB_1[2u];
    A1_ = _e90;
    let _e92 = m.kf;
    let _e94 = m.lf;
    unnamed.gl_Position = vec4<f32>(((_e47.x * _e92) - 1f), ((_e47.y * _e94) - sign(_e94)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) XB: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) OB: vec4<f32>, @location(1) QC: vec2<f32>, @location(3) SB: vec4<f32>, @location(5) IB: vec4<u32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    XB_1 = XB;
    PC_1 = PC;
    OB_1 = OB;
    QC_1 = QC;
    SB_1 = SB;
    IB_1 = IB;
    main_1();
    let _e25 = Y1_;
    let _e26 = M0_;
    let _e27 = H1_;
    let _e28 = x3_;
    let _e29 = A1_;
    let _e30 = unnamed.gl_Position;
    return VertexOutput(_e25, _e26, _e27, _e28, _e29, _e30);
}
