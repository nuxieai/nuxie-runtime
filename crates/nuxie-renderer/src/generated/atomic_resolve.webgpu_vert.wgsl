struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct oh {
    k2_: array<vec4<u32>>,
}

struct Gf {
    k2_: array<vec2<u32>>,
}

struct Hf {
    k2_: array<vec4<f32>>,
}

struct ph {
    k2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> LB: oh;
@group(0) @binding(3)
var<storage> WC: Gf;
@group(0) @binding(4)
var<storage> JB: Hf;
@group(0) @binding(5)
var<storage> ZC: ph;
@group(3) @binding(9)
var xa: sampler;

fn main_1() {
    var phi_171_: i32;
    var phi_174_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = j.i8_[0u];
        phi_171_ = _e27;
    } else {
        let _e30 = j.i8_[2u];
        phi_171_ = _e30;
    }
    let _e32 = phi_171_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = j.i8_[1u];
        phi_174_ = _e37;
    } else {
        let _e40 = j.i8_[3u];
        phi_174_ = _e40;
    }
    let _e42 = phi_174_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = j.dg;
    let _e48 = j.eg;
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
