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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: f32,
    @location(3) @interpolate(flat, either) member_2: f32,
    @location(4) @interpolate(flat, either) member_3: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override fh: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_295_: f32;

    let _e26 = XB_1;
    let _e34 = PC_1;
    let _e36 = OB_1;
    let _e38 = ((mat2x2<f32>(vec2<f32>(_e26.x, _e26.y), vec2<f32>(_e26.z, _e26.w)) * _e34) + _e36.xy);
    let _e39 = QC_1;
    G5_ = _e39;
    if fh {
        let _e41 = IB_1[1u];
        let _e43 = m.d6_;
        if (_e41 == 0u) {
            phi_295_ = 0f;
        } else {
            phi_295_ = unpack2x16float(((_e41 + 1023u) * _e43)).x;
        }
        let _e50 = phi_295_;
        K3_ = _e50;
    }
    let _e52 = m.kf;
    let _e54 = m.lf;
    let _e62 = vec4<f32>(((_e38.x * _e52) - 1f), ((_e38.y * _e54) - sign(_e54)), 0f, 1f);
    let _e64 = IB_1[3u];
    let _e74 = IB_1[0u];
    H1_ = bitcast<f32>(_e74);
    let _e77 = IB_1[2u];
    A1_ = _e77;
    unnamed.gl_Position = vec4<f32>(_e62.x, _e62.y, (1f - (f32(_e64) * 0.000061035156f)), _e62.w);
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
    let _e21 = G5_;
    let _e22 = K3_;
    let _e23 = H1_;
    let _e24 = A1_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e21, _e22, _e23, _e24, _e25);
}
