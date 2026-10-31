struct nd {
    z2_: array<vec4<u32>>,
}

struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

struct xi {
    z2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(0) @interpolate(flat, either) member: f32,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(2)
var<storage> JB: nd;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> r1_: f32;
var<private> J0_: u32;
@group(0) @binding(0)
var<uniform> j: RB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(6)
var WB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> KB: Cg;
@group(0) @binding(4)
var<storage> CD: xi;
@group(3) @binding(8)
var ib: sampler;

fn main_1() {
    let _e22 = MB_1;
    let _e25 = (bitcast<u32>(_e22.z) & 65535u);
    let _e26 = (_e25 * 4u);
    let _e30 = JB.z2_[(_e26 + 1u)];
    let _e38 = JB.z2_[_e26];
    let _e39 = bitcast<vec4<f32>>(_e38);
    let _e50 = ((mat2x2<f32>(vec2<f32>(_e39.x, _e39.y), vec2<f32>(_e39.z, _e39.w)) * _e22.xy) + bitcast<vec2<f32>>(_e30.xy));
    r1_ = f32((bitcast<i32>(_e22.z) >> bitcast<u32>(16i)));
    J0_ = _e25;
    let _e52 = j.ah;
    let _e54 = j.bh;
    unnamed.gl_Position = vec4<f32>(((_e50.x * _e52) - 1f), ((_e50.y * _e54) - sign(_e54)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    MB_1 = MB;
    main_1();
    let _e12 = r1_;
    let _e13 = J0_;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14);
}
