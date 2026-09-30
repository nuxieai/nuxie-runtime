struct TB {
    uc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Ob: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    ih: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    mh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    fh: u32,
    Nb: u32,
    ac: f32,
    bc: f32,
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

@id(0) override Ih: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> XB_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> J5_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> GC_1: vec4<f32>;
var<private> O3_: f32;
var<private> ZB_1: u32;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> RB_1: vec4<f32>;
var<private> MC_1: u32;
var<private> K1_: vec4<f32>;
var<private> YB_1: u32;
var<private> D1_: u32;
var<private> AC_1: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_316_: f32;

    let _e31 = XB_1;
    let _e39 = PC_1;
    let _e41 = NB_1;
    let _e43 = ((mat2x2<f32>(vec2<f32>(_e31.x, _e31.y), vec2<f32>(_e31.z, _e31.w)) * _e39) + _e41.xy);
    let _e44 = QC_1;
    let _e45 = GC_1;
    J5_ = ((_e44 * _e45.zw) + _e45.xy);
    if Ih {
        let _e50 = ZB_1;
        let _e52 = j.c6_;
        if (_e50 == 0u) {
            phi_316_ = 0f;
        } else {
            phi_316_ = unpack2x16float(((_e50 + 1023u) * _e52)).x;
        }
        let _e59 = phi_316_;
        O3_ = _e59;
    }
    let _e61 = j.Hf;
    let _e63 = j.If;
    let _e71 = vec4<f32>(((_e43.x * _e61) - 1f), ((_e43.y * _e63) - sign(_e63)), 0f, 1f);
    let _e72 = MC_1;
    let _e84 = YB_1;
    K1_ = unpack4x8unorm(_e84);
    let _e86 = AC_1;
    D1_ = _e86;
    unnamed.gl_Position = vec4<f32>(_e71.x, _e71.y, ((f32(((_e72 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e71.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) XB: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) QC: vec2<f32>, @location(9) GC: vec4<f32>, @location(6) ZB: u32, @location(3) RB: vec4<f32>, @location(8) MC: u32, @location(5) YB: u32, @location(7) AC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    XB_1 = XB;
    PC_1 = PC;
    NB_1 = NB;
    QC_1 = QC;
    GC_1 = GC;
    ZB_1 = ZB;
    RB_1 = RB;
    MC_1 = MC;
    YB_1 = YB;
    AC_1 = AC;
    main_1();
    let _e29 = J5_;
    let _e30 = O3_;
    let _e31 = K1_;
    let _e32 = D1_;
    let _e33 = unnamed.gl_Position;
    return VertexOutput(_e29, _e30, _e31, _e32, _e33);
}
