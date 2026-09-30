enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
}

@id(0) override Hh: bool = true;
@id(1) override Ih: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
var<private> XB_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> J5_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> GC_1: vec4<f32>;
var<private> N3_: f32;
var<private> ZB_1: u32;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> RB_1: vec4<f32>;
var<private> LC_1: u32;
var<private> K1_: vec4<f32>;
var<private> YB_1: u32;
var<private> C1_: u32;
var<private> AC_1: u32;

fn main_1() {
    var phi_407_: f32;

    let _e36 = XB_1;
    let _e44 = PC_1;
    let _e46 = NB_1;
    let _e48 = ((mat2x2<f32>(vec2<f32>(_e36.x, _e36.y), vec2<f32>(_e36.z, _e36.w)) * _e44) + _e46.xy);
    let _e49 = QC_1;
    let _e50 = GC_1;
    J5_ = ((_e49 * _e50.zw) + _e50.xy);
    if Hh {
        let _e55 = ZB_1;
        let _e57 = j.c6_;
        if (_e55 == 0u) {
            phi_407_ = 0f;
        } else {
            phi_407_ = unpack2x16float(((_e55 + 1023u) * _e57)).x;
        }
        let _e64 = phi_407_;
        N3_ = _e64;
    }
    if Ih {
        let _e65 = RB_1;
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
    let _e103 = j.Hf;
    let _e105 = j.If;
    let _e113 = vec4<f32>(((_e48.x * _e103) - 1f), ((_e48.y * _e105) - sign(_e105)), 0f, 1f);
    let _e114 = LC_1;
    let _e126 = YB_1;
    K1_ = unpack4x8unorm(_e126);
    let _e128 = AC_1;
    C1_ = _e128;
    unnamed.gl_Position = vec4<f32>(_e113.x, _e113.y, ((f32(((_e114 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e113.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) XB: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) QC: vec2<f32>, @location(9) GC: vec4<f32>, @location(6) ZB: u32, @location(3) RB: vec4<f32>, @location(8) LC: u32, @location(5) YB: u32, @location(7) AC: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    XB_1 = XB;
    PC_1 = PC;
    NB_1 = NB;
    QC_1 = QC;
    GC_1 = GC;
    ZB_1 = ZB;
    RB_1 = RB;
    LC_1 = LC;
    YB_1 = YB;
    AC_1 = AC;
    main_1();
    let _e30 = unnamed.gl_Position;
    let _e31 = unnamed.gl_ClipDistance;
    let _e32 = J5_;
    let _e33 = N3_;
    let _e34 = K1_;
    let _e35 = C1_;
    return VertexOutput(_e30, _e31, _e32, _e33, _e34, _e35);
}
