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

struct mh {
    k2_: array<vec4<u32>>,
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
var<storage> LB: mh;
@group(0) @binding(3)
var<storage> WC: Ef;
@group(0) @binding(4)
var<storage> JB: Ff;
@group(0) @binding(5)
var<storage> ZC: nh;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_171_: i32;
    var phi_174_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = j.j8_[0u];
        phi_171_ = _e27;
    } else {
        let _e30 = j.j8_[2u];
        phi_171_ = _e30;
    }
    let _e32 = phi_171_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = j.j8_[1u];
        phi_174_ = _e37;
    } else {
        let _e40 = j.j8_[3u];
        phi_174_ = _e40;
    }
    let _e42 = phi_174_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = j.bg;
    let _e48 = j.cg;
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
