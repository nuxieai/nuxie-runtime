struct dh {
    j2_: array<vec4<u32>>,
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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Bf {
    j2_: array<vec2<u32>>,
}

struct Cf {
    j2_: array<vec4<f32>>,
}

struct eh {
    j2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) @interpolate(flat, either) member: f32,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(2)
var<storage> LB: dh;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> m1_: f32;
var<private> F0_: u32;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> XC: Bf;
@group(0) @binding(4)
var<storage> JB: Cf;
@group(0) @binding(5)
var<storage> AD: eh;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    let _e23 = MB_1;
    let _e26 = (bitcast<u32>(_e23.z) & 65535u);
    let _e32 = (_e26 * 4u);
    let _e35 = LB.j2_[_e32];
    let _e36 = bitcast<vec4<f32>>(_e35);
    let _e47 = LB.j2_[(_e32 + 1u)];
    let _e51 = ((mat2x2<f32>(vec2<f32>(_e36.x, _e36.y), vec2<f32>(_e36.z, _e36.w)) * _e23.xy) + bitcast<vec2<f32>>(_e47.xy));
    m1_ = f32((bitcast<i32>(_e23.z) >> bitcast<u32>(16i)));
    F0_ = _e26;
    let _e53 = j.Yf;
    let _e55 = j.Zf;
    unnamed.gl_Position = vec4<f32>(((_e51.x * _e53) - 1f), ((_e51.y * _e55) - sign(_e55)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    MB_1 = MB;
    main_1();
    let _e12 = m1_;
    let _e13 = F0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
