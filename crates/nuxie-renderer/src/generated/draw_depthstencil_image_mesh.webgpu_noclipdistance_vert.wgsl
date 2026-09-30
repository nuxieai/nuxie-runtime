struct SB {
    xc: f32,
    Hd: f32,
    Mf: f32,
    Nf: f32,
    r6_: u32,
    Rb: u32,
    yf: u32,
    zf: u32,
    X7_: vec4<i32>,
    jh: vec2<f32>,
    Id: vec2<f32>,
    f2_: u32,
    nh: f32,
    g6_: u32,
    W2_: f32,
    Jd: f32,
    sf: u32,
    F3_: f32,
    G3_: f32,
    Kd: f32,
    gh: u32,
    Qb: u32,
    dc: f32,
    ec: f32,
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
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Jh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> L5_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> O3_: f32;
var<private> YB_1: u32;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> RB_1: vec4<f32>;
var<private> LC_1: u32;
var<private> K1_: vec4<f32>;
var<private> XB_1: u32;
var<private> D1_: u32;
var<private> ZB_1: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_291_: f32;

    let _e27 = WB_1;
    let _e35 = OC_1;
    let _e37 = NB_1;
    let _e39 = ((mat2x2<f32>(vec2<f32>(_e27.x, _e27.y), vec2<f32>(_e27.z, _e27.w)) * _e35) + _e37.xy);
    let _e40 = PC_1;
    L5_ = _e40;
    if Jh {
        let _e41 = YB_1;
        let _e43 = j.g6_;
        if (_e41 == 0u) {
            phi_291_ = 0f;
        } else {
            phi_291_ = unpack2x16float(((_e41 + 1023u) * _e43)).x;
        }
        let _e50 = phi_291_;
        O3_ = _e50;
    }
    let _e52 = j.Mf;
    let _e54 = j.Nf;
    let _e62 = vec4<f32>(((_e39.x * _e52) - 1f), ((_e39.y * _e54) - sign(_e54)), 0f, 1f);
    let _e63 = LC_1;
    let _e72 = XB_1;
    K1_ = unpack4x8unorm(_e72);
    let _e74 = ZB_1;
    D1_ = _e74;
    unnamed.gl_Position = vec4<f32>(_e62.x, _e62.y, (1f - (f32(_e63) * 0.000061035156f)), _e62.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) WB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) PC: vec2<f32>, @location(6) YB: u32, @location(3) RB: vec4<f32>, @location(8) LC: u32, @location(5) XB: u32, @location(7) ZB: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    WB_1 = WB;
    OC_1 = OC;
    NB_1 = NB;
    PC_1 = PC;
    YB_1 = YB;
    RB_1 = RB;
    LC_1 = LC;
    XB_1 = XB;
    ZB_1 = ZB;
    main_1();
    let _e27 = L5_;
    let _e28 = O3_;
    let _e29 = K1_;
    let _e30 = D1_;
    let _e31 = unnamed.gl_Position;
    return VertexOutput(_e27, _e28, _e29, _e30, _e31);
}
