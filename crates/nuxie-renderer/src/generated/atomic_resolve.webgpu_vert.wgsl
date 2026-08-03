struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Zf {
    c2_: array<vec4<u32>>,
}

struct He {
    c2_: array<vec2<u32>>,
}

struct Ie {
    c2_: array<vec4<f32>>,
}

struct ag {
    c2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> PB: Zf;
@group(0) @binding(3)
var<storage> AD: He;
@group(0) @binding(4)
var<storage> RB: Ie;
@group(0) @binding(5)
var<storage> ED: ag;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_170_: i32;
    var phi_173_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = n.R7_[0u];
        phi_170_ = _e27;
    } else {
        let _e30 = n.R7_[2u];
        phi_170_ = _e30;
    }
    let _e32 = phi_170_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = n.R7_[1u];
        phi_173_ = _e37;
    } else {
        let _e40 = n.R7_[3u];
        phi_173_ = _e40;
    }
    let _e42 = phi_173_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = n.df;
    let _e48 = n.ef;
    unnamed.gl_Position = vec4<f32>(((_e44.x * _e46) - 1f), ((_e44.y * _e48) - sign(_e48)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32) -> @builtin(position) vec4<f32> {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    main_1();
    let _e8 = unnamed.gl_Position;
    return _e8;
}
