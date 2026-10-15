struct gi {
    r2_: array<vec4<u32>>,
}

struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Bi: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Fi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    F3_: f32,
    G3_: f32,
    He: f32,
    yi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct lg {
    r2_: array<vec2<u32>>,
}

struct mg {
    r2_: array<vec4<f32>>,
}

struct hi {
    r2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(2)
var<storage> KB: gi;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> T2_: vec2<f32>;
var<private> G0_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(3)
var<storage> XC: lg;
@group(0) @binding(4)
var<storage> JB: mg;
@group(0) @binding(5)
var<storage> DD: hi;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    let _e24 = LB_1;
    let _e27 = (bitcast<u32>(_e24.z) & 65535u);
    let _e32 = KB.r2_[((_e27 * 4u) + 2u)];
    let _e35 = bitcast<vec3<f32>>(_e32.yzw);
    let _e41 = j.Bi;
    T2_ = (((_e24.xy * _e35.x) + _e35.yz) * _e41);
    G0_ = _e27;
    let _e44 = j.Jg;
    let _e46 = j.Kg;
    unnamed.gl_Position = vec4<f32>(((_e24.x * _e44) - 1f), ((_e24.y * _e46) - sign(_e46)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    LB_1 = LB;
    main_1();
    let _e12 = T2_;
    let _e13 = G0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
