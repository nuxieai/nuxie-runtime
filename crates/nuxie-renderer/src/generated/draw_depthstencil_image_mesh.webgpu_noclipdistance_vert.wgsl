struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
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

@id(0) override uj: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> AC_1: vec4<f32>;
var<private> QC_1: vec2<f32>;
var<private> QB_1: vec4<f32>;
var<private> h6_: vec2<f32>;
var<private> RC_1: vec2<f32>;
var<private> IC_1: vec4<f32>;
var<private> j4_: f32;
var<private> CC_1: u32;
@group(0) @binding(0)
var<uniform> j: RB;
var<private> VB_1: vec4<f32>;
var<private> NC_1: u32;
var<private> a2_: vec4<f32>;
var<private> BC_1: u32;
var<private> Q1_: u32;
var<private> DC_1: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_318_: f32;

    let _e31 = AC_1;
    let _e39 = QC_1;
    let _e41 = QB_1;
    let _e43 = ((mat2x2<f32>(vec2<f32>(_e31.x, _e31.y), vec2<f32>(_e31.z, _e31.w)) * _e39) + _e41.xy);
    let _e44 = RC_1;
    let _e45 = IC_1;
    h6_ = ((_e44 * _e45.zw) + _e45.xy);
    if uj {
        let _e50 = CC_1;
        let _e52 = j.z6_;
        if (_e50 == 0u) {
            phi_318_ = 0f;
        } else {
            phi_318_ = unpack2x16float(((_e50 + 1023u) * _e52)).x;
        }
        let _e59 = phi_318_;
        j4_ = _e59;
    }
    let _e61 = j.ah;
    let _e63 = j.bh;
    let _e71 = vec4<f32>(((_e43.x * _e61) - 1f), ((_e43.y * _e63) - sign(_e63)), 0f, 1f);
    let _e72 = NC_1;
    let _e84 = BC_1;
    a2_ = unpack4x8unorm(_e84);
    let _e86 = DC_1;
    Q1_ = _e86;
    unnamed.gl_Position = vec4<f32>(_e71.x, _e71.y, ((f32(((_e72 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e71.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) AC: vec4<f32>, @location(0) QC: vec2<f32>, @location(4) QB: vec4<f32>, @location(1) RC: vec2<f32>, @location(9) IC: vec4<f32>, @location(6) CC: u32, @location(3) VB: vec4<f32>, @location(8) NC: u32, @location(5) BC: u32, @location(7) DC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    AC_1 = AC;
    QC_1 = QC;
    QB_1 = QB;
    RC_1 = RC;
    IC_1 = IC;
    CC_1 = CC;
    VB_1 = VB;
    NC_1 = NC;
    BC_1 = BC;
    DC_1 = DC;
    main_1();
    let _e29 = h6_;
    let _e30 = j4_;
    let _e31 = a2_;
    let _e32 = Q1_;
    let _e33 = unnamed.gl_Position;
    return VertexOutput(_e29, _e30, _e31, _e32, _e33);
}
