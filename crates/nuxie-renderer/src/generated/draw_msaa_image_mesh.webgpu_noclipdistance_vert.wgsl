struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat) member_1: f32,
    @location(3) @interpolate(flat) member_2: f32,
    @location(4) @interpolate(flat) member_3: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Ug: bool = true;

var<private> gl_VertexIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> OC_1: vec2<f32>;
var<private> NB_1: vec4<f32>;
var<private> E5_: vec2<f32>;
var<private> PC_1: vec2<f32>;
var<private> H3_: f32;
var<private> IB_1: vec4<u32>;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> QB_1: vec4<f32>;
var<private> H1_: f32;
var<private> A1_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    var phi_293_: f32;

    let _e26 = WB_1;
    let _e34 = OC_1;
    let _e36 = NB_1;
    let _e38 = ((mat2x2<f32>(vec2<f32>(_e26.x, _e26.y), vec2<f32>(_e26.z, _e26.w)) * _e34) + _e36.xy);
    let _e39 = PC_1;
    E5_ = _e39;
    if Ug {
        let _e41 = IB_1[1u];
        let _e43 = m.Z5_;
        if (_e41 == 0u) {
            phi_293_ = 0f;
        } else {
            phi_293_ = unpack2x16float(((_e41 + 1023u) * _e43)).x;
        }
        let _e50 = phi_293_;
        H3_ = _e50;
    }
    let _e52 = m.bf;
    let _e54 = m.cf;
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
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(2) WB: vec4<f32>, @location(0) OC: vec2<f32>, @location(4) NB: vec4<f32>, @location(1) PC: vec2<f32>, @location(5) IB: vec4<u32>, @location(3) QB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    WB_1 = WB;
    OC_1 = OC;
    NB_1 = NB;
    PC_1 = PC;
    IB_1 = IB;
    QB_1 = QB;
    main_1();
    let _e21 = E5_;
    let _e22 = H3_;
    let _e23 = H1_;
    let _e24 = A1_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e21, _e22, _e23, _e24, _e25);
}
