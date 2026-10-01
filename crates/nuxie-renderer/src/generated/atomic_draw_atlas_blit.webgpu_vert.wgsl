struct mh {
    k2_: array<vec4<u32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
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

struct Ef {
    k2_: array<vec2<u32>>,
}

struct Ff {
    k2_: array<vec4<f32>>,
}

struct nh {
    k2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(2)
var<storage> LB: mh;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> J2_: vec2<f32>;
var<private> F0_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> WC: Ef;
@group(0) @binding(4)
var<storage> JB: Ff;
@group(0) @binding(5)
var<storage> ZC: nh;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    let _e24 = MB_1;
    let _e27 = (bitcast<u32>(_e24.z) & 65535u);
    let _e32 = LB.k2_[((_e27 * 4u) + 2u)];
    let _e35 = bitcast<vec3<f32>>(_e32.yzw);
    let _e41 = j.Lh;
    J2_ = (((_e24.xy * _e35.x) + _e35.yz) * _e41);
    F0_ = _e27;
    let _e44 = j.bg;
    let _e46 = j.cg;
    unnamed.gl_Position = vec4<f32>(((_e24.x * _e44) - 1f), ((_e24.y * _e46) - sign(_e46)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    MB_1 = MB;
    main_1();
    let _e12 = J2_;
    let _e13 = F0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
