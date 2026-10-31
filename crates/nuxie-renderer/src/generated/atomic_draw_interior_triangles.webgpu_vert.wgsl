struct kd {
    x2_: array<vec4<u32>>,
}

struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct vg {
    x2_: array<vec4<f32>>,
}

struct qi {
    x2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) @interpolate(flat, either) member: f32,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(2)
var<storage> JB: kd;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> o1_: f32;
var<private> I0_: u32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> KB: vg;
@group(0) @binding(4)
var<storage> CD: qi;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    let _e22 = LB_1;
    let _e25 = (bitcast<u32>(_e22.z) & 65535u);
    let _e26 = (_e25 * 4u);
    let _e30 = JB.x2_[(_e26 + 1u)];
    let _e38 = JB.x2_[_e26];
    let _e39 = bitcast<vec4<f32>>(_e38);
    let _e50 = ((mat2x2<f32>(vec2<f32>(_e39.x, _e39.y), vec2<f32>(_e39.z, _e39.w)) * _e22.xy) + bitcast<vec2<f32>>(_e30.xy));
    o1_ = f32((bitcast<i32>(_e22.z) >> bitcast<u32>(16i)));
    I0_ = _e25;
    let _e52 = j.Sg;
    let _e54 = j.Tg;
    unnamed.gl_Position = vec4<f32>(((_e50.x * _e52) - 1f), ((_e50.y * _e54) - sign(_e54)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    LB_1 = LB;
    main_1();
    let _e12 = o1_;
    let _e13 = I0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
