struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Xf {
    c2_: array<vec4<u32>>,
}

struct Fe {
    c2_: array<vec2<u32>>,
}

struct Ge {
    c2_: array<vec4<f32>>,
}

struct Yf {
    c2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> PB: Xf;
@group(0) @binding(3)
var<storage> AD: Fe;
@group(0) @binding(4)
var<storage> RB: Ge;
@group(0) @binding(5)
var<storage> ED: Yf;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_170_: i32;
    var phi_173_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = m.Q7_[0u];
        phi_170_ = _e27;
    } else {
        let _e30 = m.Q7_[2u];
        phi_170_ = _e30;
    }
    let _e32 = phi_170_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = m.Q7_[1u];
        phi_173_ = _e37;
    } else {
        let _e40 = m.Q7_[3u];
        phi_173_ = _e40;
    }
    let _e42 = phi_173_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = m.bf;
    let _e48 = m.cf;
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
