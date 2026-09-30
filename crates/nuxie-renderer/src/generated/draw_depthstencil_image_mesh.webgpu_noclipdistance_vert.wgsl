struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
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

@id(0) override bi: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> QC_1: vec2<f32>;
var<private> PB_1: vec4<f32>;
var<private> T5_: vec2<f32>;
var<private> RC_1: vec2<f32>;
var<private> HC_1: vec4<f32>;
var<private> Z3_: f32;
var<private> AC_1: u32;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> SB_1: vec4<f32>;
var<private> MC_1: u32;
var<private> P1_: vec4<f32>;
var<private> ZB_1: u32;
var<private> H1_: u32;
var<private> BC_1: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_315_: f32;

    let _e31 = YB_1;
    let _e39 = QC_1;
    let _e41 = PB_1;
    let _e43 = ((mat2x2<f32>(vec2<f32>(_e31.x, _e31.y), vec2<f32>(_e31.z, _e31.w)) * _e39) + _e41.xy);
    let _e44 = RC_1;
    let _e45 = HC_1;
    T5_ = ((_e44 * _e45.zw) + _e45.xy);
    if bi {
        let _e50 = AC_1;
        let _e52 = j.T4_;
        if (_e50 == 0u) {
            phi_315_ = 0f;
        } else {
            phi_315_ = unpack2x16float(((_e50 + 1023u) * _e52)).x;
        }
        let _e59 = phi_315_;
        Z3_ = _e59;
    }
    let _e61 = j.Yf;
    let _e63 = j.Zf;
    let _e71 = vec4<f32>(((_e43.x * _e61) - 1f), ((_e43.y * _e63) - sign(_e63)), 0f, 1f);
    let _e72 = MC_1;
    let _e84 = ZB_1;
    P1_ = unpack4x8unorm(_e84);
    let _e86 = BC_1;
    H1_ = _e86;
    unnamed.gl_Position = vec4<f32>(_e71.x, _e71.y, ((f32(((_e72 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e71.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) YB: vec4<f32>, @location(0) QC: vec2<f32>, @location(4) PB: vec4<f32>, @location(1) RC: vec2<f32>, @location(9) HC: vec4<f32>, @location(6) AC: u32, @location(3) SB: vec4<f32>, @location(8) MC: u32, @location(5) ZB: u32, @location(7) BC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    YB_1 = YB;
    QC_1 = QC;
    PB_1 = PB;
    RC_1 = RC;
    HC_1 = HC;
    AC_1 = AC;
    SB_1 = SB;
    MC_1 = MC;
    ZB_1 = ZB;
    BC_1 = BC;
    main_1();
    let _e29 = T5_;
    let _e30 = Z3_;
    let _e31 = P1_;
    let _e32 = H1_;
    let _e33 = unnamed.gl_Position;
    return VertexOutput(_e29, _e30, _e31, _e32, _e33);
}
