struct Eg {
    g2_: array<vec4<u32>>,
}

struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
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

struct Fg {
    g2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(2)
var<storage> OB: Eg;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> JB_1: vec3<f32>;
var<private> F2_: vec2<f32>;
var<private> D0_: u32;
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
var<storage> HD: Fg;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    let _e24 = JB_1;
    let _e27 = (bitcast<u32>(_e24.z) & 65535u);
    let _e32 = OB.g2_[((_e27 * 4u) + 2u)];
    let _e35 = bitcast<vec3<f32>>(_e32.yzw);
    let _e41 = j.eh;
    F2_ = (((_e24.xy * _e35.x) + _e35.yz) * _e41);
    D0_ = _e27;
    let _e44 = j.Hf;
    let _e46 = j.If;
    unnamed.gl_Position = vec4<f32>(((_e24.x * _e44) - 1f), ((_e24.y * _e46) - sign(_e46)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) JB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    JB_1 = JB;
    main_1();
    let _e12 = F2_;
    let _e13 = D0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
