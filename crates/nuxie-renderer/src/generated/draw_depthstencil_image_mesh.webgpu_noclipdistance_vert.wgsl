struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
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

@id(0) override Ui: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> ZB_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> PB_1: vec4<f32>;
var<private> Z5_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> HC_1: vec4<f32>;
var<private> e4_: f32;
var<private> BC_1: u32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> SB_1: vec4<f32>;
var<private> LC_1: u32;
var<private> U1_: vec4<f32>;
var<private> AC_1: u32;
var<private> K1_: u32;
var<private> CC_1: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_316_: f32;

    let _e31 = ZB_1;
    let _e39 = PC_1;
    let _e41 = PB_1;
    let _e43 = ((mat2x2<f32>(vec2<f32>(_e31.x, _e31.y), vec2<f32>(_e31.z, _e31.w)) * _e39) + _e41.xy);
    let _e44 = QC_1;
    let _e45 = HC_1;
    Z5_ = ((_e44 * _e45.zw) + _e45.xy);
    if Ui {
        let _e50 = BC_1;
        let _e52 = j.p6_;
        if (_e50 == 0u) {
            phi_316_ = 0f;
        } else {
            phi_316_ = unpack2x16float(((_e50 + 1023u) * _e52)).x;
        }
        let _e59 = phi_316_;
        e4_ = _e59;
    }
    let _e61 = j.Dg;
    let _e63 = j.Eg;
    let _e71 = vec4<f32>(((_e43.x * _e61) - 1f), ((_e43.y * _e63) - sign(_e63)), 0f, 1f);
    let _e72 = LC_1;
    let _e84 = AC_1;
    U1_ = unpack4x8unorm(_e84);
    let _e86 = CC_1;
    K1_ = _e86;
    unnamed.gl_Position = vec4<f32>(_e71.x, _e71.y, ((f32(((_e72 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e71.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) ZB: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) PB: vec4<f32>, @location(1) QC: vec2<f32>, @location(9) HC: vec4<f32>, @location(6) BC: u32, @location(3) SB: vec4<f32>, @location(8) LC: u32, @location(5) AC: u32, @location(7) CC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    ZB_1 = ZB;
    PC_1 = PC;
    PB_1 = PB;
    QC_1 = QC;
    HC_1 = HC;
    BC_1 = BC;
    SB_1 = SB;
    LC_1 = LC;
    AC_1 = AC;
    CC_1 = CC;
    main_1();
    let _e29 = Z5_;
    let _e30 = e4_;
    let _e31 = U1_;
    let _e32 = K1_;
    let _e33 = unnamed.gl_Position;
    return VertexOutput(_e29, _e30, _e31, _e32, _e33);
}
