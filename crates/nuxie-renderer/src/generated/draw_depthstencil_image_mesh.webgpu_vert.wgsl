enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

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

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
}

@id(0) override bi: bool = true;
@id(1) override ci: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
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

fn main_1() {
    var phi_406_: f32;

    let _e36 = YB_1;
    let _e44 = QC_1;
    let _e46 = PB_1;
    let _e48 = ((mat2x2<f32>(vec2<f32>(_e36.x, _e36.y), vec2<f32>(_e36.z, _e36.w)) * _e44) + _e46.xy);
    let _e49 = RC_1;
    let _e50 = HC_1;
    T5_ = ((_e49 * _e50.zw) + _e50.xy);
    if bi {
        let _e55 = AC_1;
        let _e57 = j.T4_;
        if (_e55 == 0u) {
            phi_406_ = 0f;
        } else {
            phi_406_ = unpack2x16float(((_e55 + 1023u) * _e57)).x;
        }
        let _e64 = phi_406_;
        Z3_ = _e64;
    }
    if ci {
        let _e65 = SB_1;
        if any((_e65 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e77 = ((mat2x2<f32>(vec2<f32>(_e65.x, _e65.y), vec2<f32>(_e65.z, _e65.w)) * _e48) + _e46.zw);
            unnamed.gl_ClipDistance[0i] = (_e77.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e77.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e77.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e77.y);
        } else {
            let _e93 = (_e46.z - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e93;
            unnamed.gl_ClipDistance[2i] = _e93;
            unnamed.gl_ClipDistance[1i] = _e93;
            unnamed.gl_ClipDistance[0i] = _e93;
        }
    }
    let _e103 = j.Yf;
    let _e105 = j.Zf;
    let _e113 = vec4<f32>(((_e48.x * _e103) - 1f), ((_e48.y * _e105) - sign(_e105)), 0f, 1f);
    let _e114 = MC_1;
    let _e126 = ZB_1;
    P1_ = unpack4x8unorm(_e126);
    let _e128 = BC_1;
    H1_ = _e128;
    unnamed.gl_Position = vec4<f32>(_e113.x, _e113.y, ((f32(((_e114 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e113.w);
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
    let _e30 = unnamed.gl_Position;
    let _e31 = unnamed.gl_ClipDistance;
    let _e32 = T5_;
    let _e33 = Z3_;
    let _e34 = P1_;
    let _e35 = H1_;
    return VertexOutput(_e30, _e31, _e32, _e33, _e34, _e35);
}
