enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

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

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
}

@id(0) override Ui: bool = true;
@id(1) override Vi: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
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

fn main_1() {
    var phi_407_: f32;

    let _e36 = ZB_1;
    let _e44 = PC_1;
    let _e46 = PB_1;
    let _e48 = ((mat2x2<f32>(vec2<f32>(_e36.x, _e36.y), vec2<f32>(_e36.z, _e36.w)) * _e44) + _e46.xy);
    let _e49 = QC_1;
    let _e50 = HC_1;
    Z5_ = ((_e49 * _e50.zw) + _e50.xy);
    if Ui {
        let _e55 = BC_1;
        let _e57 = j.p6_;
        if (_e55 == 0u) {
            phi_407_ = 0f;
        } else {
            phi_407_ = unpack2x16float(((_e55 + 1023u) * _e57)).x;
        }
        let _e64 = phi_407_;
        e4_ = _e64;
    }
    if Vi {
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
    let _e103 = j.Dg;
    let _e105 = j.Eg;
    let _e113 = vec4<f32>(((_e48.x * _e103) - 1f), ((_e48.y * _e105) - sign(_e105)), 0f, 1f);
    let _e114 = LC_1;
    let _e126 = AC_1;
    U1_ = unpack4x8unorm(_e126);
    let _e128 = CC_1;
    K1_ = _e128;
    unnamed.gl_Position = vec4<f32>(_e113.x, _e113.y, ((f32(((_e114 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e113.w);
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
    let _e30 = unnamed.gl_Position;
    let _e31 = unnamed.gl_ClipDistance;
    let _e32 = Z5_;
    let _e33 = e4_;
    let _e34 = U1_;
    let _e35 = K1_;
    return VertexOutput(_e30, _e31, _e32, _e33, _e34, _e35);
}
