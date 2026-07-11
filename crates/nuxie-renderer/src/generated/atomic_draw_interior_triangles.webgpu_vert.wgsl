struct Rf {
    X1_: array<vec4<u32>>,
}

struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Be {
    X1_: array<vec2<u32>>,
}

struct Ce {
    X1_: array<vec4<f32>>,
}

struct Sf {
    X1_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) @interpolate(flat) member: f32,
    @location(1) @interpolate(flat) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(3) 
var<storage> MB: Rf;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> j1_: f32;
var<private> z0_: u32;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8) 
var DC: texture_2d<u32>;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(0) @binding(4) 
var<storage> TC: Be;
@group(0) @binding(5) 
var<storage> PB: Ce;
@group(0) @binding(6) 
var<storage> XC: Sf;
@group(3) @binding(10) 
var T9_: sampler;

fn main_1() {
    let _e23 = KB_1;
    let _e26 = (bitcast<u32>(_e23.z) & 65535u);
    let _e32 = (_e26 * 4u);
    let _e35 = MB.X1_[_e32];
    let _e36 = bitcast<vec4<f32>>(_e35);
    let _e47 = MB.X1_[(_e32 + 1u)];
    let _e51 = ((mat2x2<f32>(vec2<f32>(_e36.x, _e36.y), vec2<f32>(_e36.z, _e36.w)) * _e23.xy) + bitcast<vec2<f32>>(_e47.xy));
    j1_ = f32((bitcast<i32>(_e23.z) >> bitcast<u32>(16i)));
    z0_ = _e26;
    let _e53 = k.Xe;
    let _e55 = k.Ye;
    unnamed.gl_Position = vec4<f32>(((_e51.x * _e53) - 1f), ((_e51.y * _e55) - sign(_e55)), 0f, 1f);
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    KB_1 = KB;
    main_1();
    let _e12 = j1_;
    let _e13 = z0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
