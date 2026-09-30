enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct SB {
    xc: f32,
    Hd: f32,
    Mf: f32,
    Nf: f32,
    r6_: u32,
    Rb: u32,
    yf: u32,
    zf: u32,
    X7_: vec4<i32>,
    jh: vec2<f32>,
    Id: vec2<f32>,
    f2_: u32,
    nh: f32,
    g6_: u32,
    W2_: f32,
    Jd: f32,
    sf: u32,
    F3_: f32,
    G3_: f32,
    Kd: f32,
    gh: u32,
    Qb: u32,
    dc: f32,
    ec: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: vec4<f32>,
    @location(4) @interpolate(flat, either) member_3: u32,
}

@id(0) override Jh: bool = true;
@id(1) override Kh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> L5_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> O3_: f32;
var<private> YB_1: u32;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> RB_1: vec4<f32>;
var<private> LC_1: u32;
var<private> K1_: vec4<f32>;
var<private> XB_1: u32;
var<private> D1_: u32;
var<private> ZB_1: u32;

fn main_1() {
    var phi_382_: f32;

    let _e32 = WB_1;
    let _e40 = OC_1;
    let _e42 = NB_1;
    let _e44 = ((mat2x2<f32>(vec2<f32>(_e32.x, _e32.y), vec2<f32>(_e32.z, _e32.w)) * _e40) + _e42.xy);
    let _e45 = PC_1;
    L5_ = _e45;
    if Jh {
        let _e46 = YB_1;
        let _e48 = j.g6_;
        if (_e46 == 0u) {
            phi_382_ = 0f;
        } else {
            phi_382_ = unpack2x16float(((_e46 + 1023u) * _e48)).x;
        }
        let _e55 = phi_382_;
        O3_ = _e55;
    }
    if Kh {
        let _e56 = RB_1;
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
    let _e94 = j.Mf;
    let _e96 = j.Nf;
    let _e104 = vec4<f32>(((_e44.x * _e94) - 1f), ((_e44.y * _e96) - sign(_e96)), 0f, 1f);
    let _e105 = LC_1;
    let _e114 = XB_1;
    K1_ = unpack4x8unorm(_e114);
    let _e116 = ZB_1;
    D1_ = _e116;
    unnamed.gl_Position = vec4<f32>(_e104.x, _e104.y, (1f - (f32(_e105) * 0.000061035156f)), _e104.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) WB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) PC: vec2<f32>, @location(6) YB: u32, @location(3) RB: vec4<f32>, @location(8) LC: u32, @location(5) XB: u32, @location(7) ZB: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    WB_1 = WB;
    OC_1 = OC;
    NB_1 = NB;
    PC_1 = PC;
    YB_1 = YB;
    RB_1 = RB;
    LC_1 = LC;
    XB_1 = XB;
    ZB_1 = ZB;
    main_1();
    let _e28 = unnamed.gl_Position;
    let _e29 = unnamed.gl_ClipDistance;
    let _e30 = L5_;
    let _e31 = O3_;
    let _e32 = K1_;
    let _e33 = D1_;
    return VertexOutput(_e28, _e29, _e30, _e31, _e32, _e33);
}
