struct Hg {
    g2_: array<vec4<u32>>,
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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct Ig {
    g2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) @interpolate(flat, either) member: f32,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(2)
var<storage> OB: Hg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> JB_1: vec3<f32>;
var<private> j1_: f32;
var<private> D0_: u32;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> CD: kf;
@group(0) @binding(4)
var<storage> PB: lf;
@group(0) @binding(5)
var<storage> HD: Ig;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    let _e23 = JB_1;
    let _e26 = (bitcast<u32>(_e23.z) & 65535u);
    let _e32 = (_e26 * 4u);
    let _e35 = OB.g2_[_e32];
    let _e36 = bitcast<vec4<f32>>(_e35);
    let _e47 = OB.g2_[(_e32 + 1u)];
    let _e51 = ((mat2x2<f32>(vec2<f32>(_e36.x, _e36.y), vec2<f32>(_e36.z, _e36.w)) * _e23.xy) + bitcast<vec2<f32>>(_e47.xy));
    j1_ = f32((bitcast<i32>(_e23.z) >> bitcast<u32>(16i)));
    D0_ = _e26;
    let _e53 = j.Hf;
    let _e55 = j.If;
    unnamed.gl_Position = vec4<f32>(((_e51.x * _e53) - 1f), ((_e51.y * _e55) - sign(_e55)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) JB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    JB_1 = JB;
    main_1();
    let _e12 = j1_;
    let _e13 = D0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
