enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
}

@id(0) override jj: bool = true;
@id(1) override kj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
var<private> AC_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> PB_1: vec4<f32>;
var<private> e6_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> IC_1: vec4<f32>;
var<private> h4_: f32;
var<private> CC_1: u32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> UB_1: vec4<f32>;
var<private> MC_1: u32;
var<private> Y1_: vec4<f32>;
var<private> BC_1: u32;
var<private> N1_: u32;
var<private> DC_1: u32;

fn main_1() {
    var phi_409_: f32;

    let _e36 = AC_1;
    let _e44 = PC_1;
    let _e46 = PB_1;
    let _e48 = ((mat2x2<f32>(vec2<f32>(_e36.x, _e36.y), vec2<f32>(_e36.z, _e36.w)) * _e44) + _e46.xy);
    let _e49 = QC_1;
    let _e50 = IC_1;
    e6_ = ((_e49 * _e50.zw) + _e50.xy);
    if jj {
        let _e55 = CC_1;
        let _e57 = j.w6_;
        if (_e55 == 0u) {
            phi_409_ = 0f;
        } else {
            phi_409_ = unpack2x16float(((_e55 + 1023u) * _e57)).x;
        }
        let _e64 = phi_409_;
        h4_ = _e64;
    }
    if kj {
        let _e65 = UB_1;
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
    let _e103 = j.Rg;
    let _e105 = j.Sg;
    let _e113 = vec4<f32>(((_e48.x * _e103) - 1f), ((_e48.y * _e105) - sign(_e105)), 0f, 1f);
    let _e114 = MC_1;
    let _e126 = BC_1;
    Y1_ = unpack4x8unorm(_e126);
    let _e128 = DC_1;
    N1_ = _e128;
    unnamed.gl_Position = vec4<f32>(_e113.x, _e113.y, ((f32(((_e114 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e113.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) AC: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) PB: vec4<f32>, @location(1) QC: vec2<f32>, @location(9) IC: vec4<f32>, @location(6) CC: u32, @location(3) UB: vec4<f32>, @location(8) MC: u32, @location(5) BC: u32, @location(7) DC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    AC_1 = AC;
    PC_1 = PC;
    PB_1 = PB;
    QC_1 = QC;
    IC_1 = IC;
    CC_1 = CC;
    UB_1 = UB;
    MC_1 = MC;
    BC_1 = BC;
    DC_1 = DC;
    main_1();
    let _e30 = unnamed.gl_Position;
    let _e31 = unnamed.gl_ClipDistance;
    let _e32 = e6_;
    let _e33 = h4_;
    let _e34 = Y1_;
    let _e35 = N1_;
    return VertexOutput(_e30, _e31, _e32, _e33, _e34, _e35);
}
