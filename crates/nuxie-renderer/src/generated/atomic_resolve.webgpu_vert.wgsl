struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ig {
    d2_: array<vec4<u32>>,
}

struct Ne {
    d2_: array<vec2<u32>>,
}

struct Oe {
    d2_: array<vec4<f32>>,
}

struct jg {
    d2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> QB: ig;
@group(0) @binding(3)
var<storage> BD: Ne;
@group(0) @binding(4)
var<storage> RB: Oe;
@group(0) @binding(5)
var<storage> FD: jg;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_172_: i32;
    var phi_175_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = m.U7_[0u];
        phi_172_ = _e27;
    } else {
        let _e30 = m.U7_[2u];
        phi_172_ = _e30;
    }
    let _e32 = phi_172_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = m.U7_[1u];
        phi_175_ = _e37;
    } else {
        let _e40 = m.U7_[3u];
        phi_175_ = _e40;
    }
    let _e42 = phi_175_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = m.kf;
    let _e48 = m.lf;
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
