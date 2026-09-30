struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
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

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Hh: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> XB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> J5_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> O3_: f32;
var<private> ZB_1: u32;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> RB_1: vec4<f32>;
var<private> LC_1: u32;
var<private> K1_: vec4<f32>;
var<private> YB_1: u32;
var<private> D1_: u32;
var<private> AC_1: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_309_: f32;

    let _e30 = XB_1;
    let _e38 = OC_1;
    let _e40 = NB_1;
    let _e42 = ((mat2x2<f32>(vec2<f32>(_e30.x, _e30.y), vec2<f32>(_e30.z, _e30.w)) * _e38) + _e40.xy);
    let _e43 = PC_1;
    J5_ = _e43;
    if Hh {
        let _e44 = ZB_1;
        let _e46 = j.c6_;
        if (_e44 == 0u) {
            phi_309_ = 0f;
        } else {
            phi_309_ = unpack2x16float(((_e44 + 1023u) * _e46)).x;
        }
        let _e53 = phi_309_;
        O3_ = _e53;
    }
    let _e55 = j.Hf;
    let _e57 = j.If;
    let _e65 = vec4<f32>(((_e42.x * _e55) - 1f), ((_e42.y * _e57) - sign(_e57)), 0f, 1f);
    let _e66 = LC_1;
    let _e78 = YB_1;
    K1_ = unpack4x8unorm(_e78);
    let _e80 = AC_1;
    D1_ = _e80;
    unnamed.gl_Position = vec4<f32>(_e65.x, _e65.y, ((f32(((_e66 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e65.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) XB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) PC: vec2<f32>, @location(6) ZB: u32, @location(3) RB: vec4<f32>, @location(8) LC: u32, @location(5) YB: u32, @location(7) AC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    XB_1 = XB;
    OC_1 = OC;
    NB_1 = NB;
    PC_1 = PC;
    ZB_1 = ZB;
    RB_1 = RB;
    LC_1 = LC;
    YB_1 = YB;
    AC_1 = AC;
    main_1();
    let _e27 = J5_;
    let _e28 = O3_;
    let _e29 = K1_;
    let _e30 = D1_;
    let _e31 = unnamed.gl_Position;
    return VertexOutput(_e27, _e28, _e29, _e30, _e31);
}
