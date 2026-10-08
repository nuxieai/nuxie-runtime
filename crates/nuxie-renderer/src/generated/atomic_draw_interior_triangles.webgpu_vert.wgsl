struct ci {
    v2_: array<vec4<u32>>,
}

struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct gg {
    v2_: array<vec2<u32>>,
}

struct hg {
    v2_: array<vec4<f32>>,
}

struct di {
    v2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) @interpolate(flat, either) member: f32,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(2)
var<storage> KB: ci;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> o1_: f32;
var<private> G0_: u32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> WC: gg;
@group(0) @binding(4)
var<storage> JB: hg;
@group(0) @binding(5)
var<storage> BD: di;
@group(3) @binding(9)
var Ta: sampler;

fn main_1() {
    let _e23 = LB_1;
    let _e26 = (bitcast<u32>(_e23.z) & 65535u);
    let _e32 = (_e26 * 4u);
    let _e35 = KB.v2_[_e32];
    let _e36 = bitcast<vec4<f32>>(_e35);
    let _e47 = KB.v2_[(_e32 + 1u)];
    let _e51 = ((mat2x2<f32>(vec2<f32>(_e36.x, _e36.y), vec2<f32>(_e36.z, _e36.w)) * _e23.xy) + bitcast<vec2<f32>>(_e47.xy));
    o1_ = f32((bitcast<i32>(_e23.z) >> bitcast<u32>(16i)));
    G0_ = _e26;
    let _e53 = j.Dg;
    let _e55 = j.Eg;
    unnamed.gl_Position = vec4<f32>(((_e51.x * _e53) - 1f), ((_e51.y * _e55) - sign(_e55)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    LB_1 = LB;
    main_1();
    let _e12 = o1_;
    let _e13 = G0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
