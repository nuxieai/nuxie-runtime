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

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: f32,
    @location(4) @interpolate(flat, either) member_3: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override jh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> G5_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> J3_: f32;
var<private> YB_1: u32;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> SB_1: vec4<f32>;
var<private> MC_1: u32;
var<private> I1_: f32;
var<private> XB_1: f32;
var<private> B1_: u32;
var<private> ZB_1: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_291_: f32;

    let _e27 = WB_1;
    let _e35 = OC_1;
    let _e37 = NB_1;
    let _e39 = ((mat2x2<f32>(vec2<f32>(_e27.x, _e27.y), vec2<f32>(_e27.z, _e27.w)) * _e35) + _e37.xy);
    let _e40 = PC_1;
    G5_ = _e40;
    if jh {
        let _e41 = YB_1;
        let _e43 = m.d6_;
        if (_e41 == 0u) {
            phi_291_ = 0f;
        } else {
            phi_291_ = unpack2x16float(((_e41 + 1023u) * _e43)).x;
        }
        let _e50 = phi_291_;
        J3_ = _e50;
    }
    let _e52 = m.of_;
    let _e54 = m.pf;
    let _e62 = vec4<f32>(((_e39.x * _e52) - 1f), ((_e39.y * _e54) - sign(_e54)), 0f, 1f);
    let _e63 = MC_1;
    let _e72 = XB_1;
    I1_ = _e72;
    let _e73 = ZB_1;
    B1_ = _e73;
    unnamed.gl_Position = vec4<f32>(_e62.x, _e62.y, (1f - (f32(_e63) * 0.000061035156f)), _e62.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) WB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) PC: vec2<f32>, @location(6) YB: u32, @location(3) SB: vec4<f32>, @location(8) MC: u32, @location(5) XB: f32, @location(7) ZB: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    WB_1 = WB;
    OC_1 = OC;
    NB_1 = NB;
    PC_1 = PC;
    YB_1 = YB;
    SB_1 = SB;
    MC_1 = MC;
    XB_1 = XB;
    ZB_1 = ZB;
    main_1();
    let _e27 = G5_;
    let _e28 = J3_;
    let _e29 = I1_;
    let _e30 = B1_;
    let _e31 = unnamed.gl_Position;
    return VertexOutput(_e27, _e28, _e29, _e30, _e31);
}
