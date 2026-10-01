struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
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

@id(0) override ki: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> PB_1: vec4<f32>;
var<private> V5_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> HC_1: vec4<f32>;
var<private> Y3_: f32;
var<private> AC_1: u32;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> SB_1: vec4<f32>;
var<private> LC_1: u32;
var<private> Q1_: vec4<f32>;
var<private> ZB_1: u32;
var<private> H1_: u32;
var<private> BC_1: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_315_: f32;

    let _e31 = YB_1;
    let _e39 = PC_1;
    let _e41 = PB_1;
    let _e43 = ((mat2x2<f32>(vec2<f32>(_e31.x, _e31.y), vec2<f32>(_e31.z, _e31.w)) * _e39) + _e41.xy);
    let _e44 = QC_1;
    let _e45 = HC_1;
    V5_ = ((_e44 * _e45.zw) + _e45.xy);
    if ki {
        let _e50 = AC_1;
        let _e52 = j.T4_;
        if (_e50 == 0u) {
            phi_315_ = 0f;
        } else {
            phi_315_ = unpack2x16float(((_e50 + 1023u) * _e52)).x;
        }
        let _e59 = phi_315_;
        Y3_ = _e59;
    }
    let _e61 = j.bg;
    let _e63 = j.cg;
    let _e71 = vec4<f32>(((_e43.x * _e61) - 1f), ((_e43.y * _e63) - sign(_e63)), 0f, 1f);
    let _e72 = LC_1;
    let _e84 = ZB_1;
    Q1_ = unpack4x8unorm(_e84);
    let _e86 = BC_1;
    H1_ = _e86;
    unnamed.gl_Position = vec4<f32>(_e71.x, _e71.y, ((f32(((_e72 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e71.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) YB: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) PB: vec4<f32>, @location(1) QC: vec2<f32>, @location(9) HC: vec4<f32>, @location(6) AC: u32, @location(3) SB: vec4<f32>, @location(8) LC: u32, @location(5) ZB: u32, @location(7) BC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    YB_1 = YB;
    PC_1 = PC;
    PB_1 = PB;
    QC_1 = QC;
    HC_1 = HC;
    AC_1 = AC;
    SB_1 = SB;
    LC_1 = LC;
    ZB_1 = ZB;
    BC_1 = BC;
    main_1();
    let _e29 = V5_;
    let _e30 = Y3_;
    let _e31 = Q1_;
    let _e32 = H1_;
    let _e33 = unnamed.gl_Position;
    return VertexOutput(_e29, _e30, _e31, _e32, _e33);
}
