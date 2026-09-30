enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

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

fn main_1() {
    var phi_400_: f32;

    let _e35 = XB_1;
    let _e43 = OC_1;
    let _e45 = NB_1;
    let _e47 = ((mat2x2<f32>(vec2<f32>(_e35.x, _e35.y), vec2<f32>(_e35.z, _e35.w)) * _e43) + _e45.xy);
    let _e48 = PC_1;
    J5_ = _e48;
    if Hh {
        let _e49 = ZB_1;
        let _e51 = j.c6_;
        if (_e49 == 0u) {
            phi_400_ = 0f;
        } else {
            phi_400_ = unpack2x16float(((_e49 + 1023u) * _e51)).x;
        }
        let _e58 = phi_400_;
        O3_ = _e58;
    }
    if Ih {
        let _e59 = RB_1;
        if any((_e59 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e71 = ((mat2x2<f32>(vec2<f32>(_e59.x, _e59.y), vec2<f32>(_e59.z, _e59.w)) * _e47) + _e45.zw);
            unnamed.gl_ClipDistance[0i] = (_e71.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e71.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e71.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e71.y);
        } else {
            let _e87 = (_e45.z - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e87;
            unnamed.gl_ClipDistance[2i] = _e87;
            unnamed.gl_ClipDistance[1i] = _e87;
            unnamed.gl_ClipDistance[0i] = _e87;
        }
    }
    let _e97 = j.Hf;
    let _e99 = j.If;
    let _e107 = vec4<f32>(((_e47.x * _e97) - 1f), ((_e47.y * _e99) - sign(_e99)), 0f, 1f);
    let _e108 = LC_1;
    let _e120 = YB_1;
    K1_ = unpack4x8unorm(_e120);
    let _e122 = AC_1;
    D1_ = _e122;
    unnamed.gl_Position = vec4<f32>(_e107.x, _e107.y, ((f32(((_e108 << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e107.w);
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
    let _e28 = unnamed.gl_Position;
    let _e29 = unnamed.gl_ClipDistance;
    let _e30 = J5_;
    let _e31 = O3_;
    let _e32 = K1_;
    let _e33 = D1_;
    return VertexOutput(_e28, _e29, _e30, _e31, _e32, _e33);
}
