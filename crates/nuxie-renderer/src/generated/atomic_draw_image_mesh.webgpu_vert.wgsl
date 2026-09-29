struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct lg {
    d2_: array<vec4<u32>>,
}

struct Re {
    d2_: array<vec2<u32>>,
}

struct Se {
    d2_: array<vec4<f32>>,
}

struct mg {
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

@id(1) override kh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> Y1_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> M0_: vec4<f32>;
var<private> SB_1: vec4<f32>;
var<private> I1_: f32;
var<private> XB_1: f32;
var<private> w3_: u32;
var<private> YB_1: u32;
var<private> B1_: u32;
var<private> ZB_1: u32;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> PB: lg;
@group(0) @binding(3)
var<storage> AD: Re;
@group(0) @binding(4)
var<storage> QB: Se;
@group(0) @binding(5)
var<storage> ED: mg;
@group(3) @binding(9)
var Z9_: sampler;
var<private> MC_1: u32;

fn main_1() {
    var phi_313_: bool;
    var phi_376_: vec4<f32>;

    let _e37 = WB_1;
    let _e45 = OC_1;
    let _e47 = NB_1;
    let _e49 = ((mat2x2<f32>(vec2<f32>(_e37.x, _e37.y), vec2<f32>(_e37.z, _e37.w)) * _e45) + _e47.xy);
    let _e50 = PC_1;
    Y1_ = _e50;
    if kh {
        let _e51 = SB_1;
        let _e56 = vec2<f32>(_e51.x, _e51.y);
        let _e57 = vec2<f32>(_e51.z, _e51.w);
        switch bitcast<i32>(0u) {
            default: {
                let _e63 = (abs(_e56) + abs(_e57));
                let _e65 = (_e63.x != 0f);
                phi_313_ = _e65;
                if _e65 {
                    phi_313_ = (_e63.y != 0f);
                }
                let _e69 = phi_313_;
                if _e69 {
                    let _e73 = ((mat2x2<f32>(_e56, _e57) * _e49) + _e47.zw);
                    let _e74 = -(_e73);
                    let _e80 = (vec2<f32>(1f, 1f) / _e63).xyxy;
                    phi_376_ = (((vec4<f32>(_e73.x, _e73.y, _e74.x, _e74.y) * _e80) + _e80) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_376_ = _e47.zwzw;
                    break;
                }
            }
        }
        let _e85 = phi_376_;
        M0_ = _e85;
    }
    let _e86 = XB_1;
    I1_ = _e86;
    let _e87 = YB_1;
    w3_ = _e87;
    let _e88 = ZB_1;
    B1_ = _e88;
    let _e90 = m.of_;
    let _e92 = m.pf;
    unnamed.gl_Position = vec4<f32>(((_e49.x * _e90) - 1f), ((_e49.y * _e92) - sign(_e92)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(2) WB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) PC: vec2<f32>, @location(3) SB: vec4<f32>, @location(5) XB: f32, @location(6) YB: u32, @location(7) ZB: u32, @location(8) MC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    WB_1 = WB;
    OC_1 = OC;
    NB_1 = NB;
    PC_1 = PC;
    SB_1 = SB;
    XB_1 = XB;
    YB_1 = YB;
    ZB_1 = ZB;
    MC_1 = MC;
    main_1();
    let _e31 = Y1_;
    let _e32 = M0_;
    let _e33 = I1_;
    let _e34 = w3_;
    let _e35 = B1_;
    let _e36 = unnamed.gl_Position;
    return VertexOutput(_e31, _e32, _e33, _e34, _e35, _e36);
}
