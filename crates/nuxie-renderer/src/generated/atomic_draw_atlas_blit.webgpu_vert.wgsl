struct pi {
    v2_: array<vec4<u32>>,
}

struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ug {
    v2_: array<vec2<u32>>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

struct qi {
    v2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(2)
var<storage> KB: pi;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> U2_: vec2<f32>;
var<private> I0_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(3)
var<storage> XC: ug;
@group(0) @binding(4)
var<storage> JB: vg;
@group(0) @binding(5)
var<storage> DD: qi;
@group(3) @binding(9)
var eb: sampler;

fn main_1() {
    let _e24 = LB_1;
    let _e27 = (bitcast<u32>(_e24.z) & 65535u);
    let _e32 = KB.v2_[((_e27 * 4u) + 2u)];
    let _e35 = bitcast<vec3<f32>>(_e32.yzw);
    let _e41 = j.Ki;
    U2_ = (((_e24.xy * _e35.x) + _e35.yz) * _e41);
    I0_ = _e27;
    let _e44 = j.Sg;
    let _e46 = j.Tg;
    unnamed.gl_Position = vec4<f32>(((_e24.x * _e44) - 1f), ((_e24.y * _e46) - sign(_e46)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    LB_1 = LB;
    main_1();
    let _e12 = U2_;
    let _e13 = I0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
