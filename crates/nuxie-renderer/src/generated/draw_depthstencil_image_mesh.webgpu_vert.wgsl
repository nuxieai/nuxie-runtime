enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: f32,
    @location(4) @interpolate(flat, either) member_3: u32,
}

@id(0) override jh: bool = true;
@id(1) override kh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> G5_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> J3_: f32;
var<private> YB_1: u32;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> SB_1: vec4<f32>;
var<private> MC_1: u32;
var<private> I1_: f32;
var<private> XB_1: f32;
var<private> B1_: u32;
var<private> ZB_1: u32;

fn main_1() {
    var phi_382_: f32;

    let _e32 = WB_1;
    let _e40 = OC_1;
    let _e42 = NB_1;
    let _e44 = ((mat2x2<f32>(vec2<f32>(_e32.x, _e32.y), vec2<f32>(_e32.z, _e32.w)) * _e40) + _e42.xy);
    let _e45 = PC_1;
    G5_ = _e45;
    if jh {
        let _e46 = YB_1;
        let _e48 = m.d6_;
        if (_e46 == 0u) {
            phi_382_ = 0f;
        } else {
            phi_382_ = unpack2x16float(((_e46 + 1023u) * _e48)).x;
        }
        let _e55 = phi_382_;
        J3_ = _e55;
    }
    if kh {
        let _e56 = SB_1;
        if any((_e56 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e68 = ((mat2x2<f32>(vec2<f32>(_e56.x, _e56.y), vec2<f32>(_e56.z, _e56.w)) * _e44) + _e42.zw);
            unnamed.gl_ClipDistance[0i] = (_e68.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e68.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e68.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e68.y);
        } else {
            let _e84 = (_e42.z - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e84;
            unnamed.gl_ClipDistance[2i] = _e84;
            unnamed.gl_ClipDistance[1i] = _e84;
            unnamed.gl_ClipDistance[0i] = _e84;
        }
    }
    let _e94 = m.of_;
    let _e96 = m.pf;
    let _e104 = vec4<f32>(((_e44.x * _e94) - 1f), ((_e44.y * _e96) - sign(_e96)), 0f, 1f);
    let _e105 = MC_1;
    let _e114 = XB_1;
    I1_ = _e114;
    let _e115 = ZB_1;
    B1_ = _e115;
    unnamed.gl_Position = vec4<f32>(_e104.x, _e104.y, (1f - (f32(_e105) * 0.000061035156f)), _e104.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) WB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) PC: vec2<f32>, @location(6) YB: u32, @location(3) SB: vec4<f32>, @location(8) MC: u32, @location(5) XB: f32, @location(7) ZB: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    WB_1 = WB;
    OC_1 = OC;
    NB_1 = NB;
    PC_1 = PC;
    YB_1 = YB;
    SB_1 = SB;
    MC_1 = MC;
    XB_1 = XB;
    ZB_1 = ZB;
    main_1();
    let _e28 = unnamed.gl_Position;
    let _e29 = unnamed.gl_ClipDistance;
    let _e30 = G5_;
    let _e31 = J3_;
    let _e32 = I1_;
    let _e33 = B1_;
    return VertexOutput(_e28, _e29, _e30, _e31, _e32, _e33);
}
