enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: f32,
    @location(4) @interpolate(flat, either) member_3: u32,
}

@id(0) override fh: bool = true;
@id(1) override gh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
var<private> XB_1: vec4<f32>;
var<private> PC_1: vec2<f32>;
var<private> OB_1: vec4<f32>;
var<private> G5_: vec2<f32>;
var<private> QC_1: vec2<f32>;
var<private> K3_: f32;
var<private> IB_1: vec4<u32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> SB_1: vec4<f32>;
var<private> H1_: f32;
var<private> A1_: u32;

fn main_1() {
    var phi_386_: f32;

    let _e31 = XB_1;
    let _e39 = PC_1;
    let _e41 = OB_1;
    let _e43 = ((mat2x2<f32>(vec2<f32>(_e31.x, _e31.y), vec2<f32>(_e31.z, _e31.w)) * _e39) + _e41.xy);
    let _e44 = QC_1;
    G5_ = _e44;
    if fh {
        let _e46 = IB_1[1u];
        let _e48 = m.d6_;
        if (_e46 == 0u) {
            phi_386_ = 0f;
        } else {
            phi_386_ = unpack2x16float(((_e46 + 1023u) * _e48)).x;
        }
        let _e55 = phi_386_;
        K3_ = _e55;
    }
    if gh {
        let _e56 = SB_1;
        if any((_e56 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e68 = ((mat2x2<f32>(vec2<f32>(_e56.x, _e56.y), vec2<f32>(_e56.z, _e56.w)) * _e43) + _e41.zw);
            unnamed.gl_ClipDistance[0i] = (_e68.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e68.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e68.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e68.y);
        } else {
            let _e84 = (_e41.z - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e84;
            unnamed.gl_ClipDistance[2i] = _e84;
            unnamed.gl_ClipDistance[1i] = _e84;
            unnamed.gl_ClipDistance[0i] = _e84;
        }
    }
    let _e94 = m.kf;
    let _e96 = m.lf;
    let _e104 = vec4<f32>(((_e43.x * _e94) - 1f), ((_e43.y * _e96) - sign(_e96)), 0f, 1f);
    let _e106 = IB_1[3u];
    let _e116 = IB_1[0u];
    H1_ = bitcast<f32>(_e116);
    let _e119 = IB_1[2u];
    A1_ = _e119;
    unnamed.gl_Position = vec4<f32>(_e104.x, _e104.y, (1f - (f32(_e106) * 0.000061035156f)), _e104.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) XB: vec4<f32>, @location(0) PC: vec2<f32>, @location(4) OB: vec4<f32>, @location(1) QC: vec2<f32>, @location(5) IB: vec4<u32>, @location(3) SB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    XB_1 = XB;
    PC_1 = PC;
    OB_1 = OB;
    QC_1 = QC;
    IB_1 = IB;
    SB_1 = SB;
    main_1();
    let _e22 = unnamed.gl_Position;
    let _e23 = unnamed.gl_ClipDistance;
    let _e24 = G5_;
    let _e25 = K3_;
    let _e26 = H1_;
    let _e27 = A1_;
    return VertexOutput(_e22, _e23, _e24, _e25, _e26, _e27);
}
