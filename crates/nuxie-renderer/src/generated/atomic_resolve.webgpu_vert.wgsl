struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ci {
    v2_: array<vec4<u32>>,
}

struct gg {
    v2_: array<vec2<u32>>,
}

struct hg {
    v2_: array<vec4<f32>>,
}

struct di {
    v2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> KB: ci;
@group(0) @binding(3)
var<storage> WC: gg;
@group(0) @binding(4)
var<storage> JB: hg;
@group(0) @binding(5)
var<storage> BD: di;
@group(3) @binding(9)
var Ta: sampler;

fn main_1() {
    var phi_172_: i32;
    var phi_175_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = j.B8_[0u];
        phi_172_ = _e27;
    } else {
        let _e30 = j.B8_[2u];
        phi_172_ = _e30;
    }
    let _e32 = phi_172_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = j.B8_[1u];
        phi_175_ = _e37;
    } else {
        let _e40 = j.B8_[3u];
        phi_175_ = _e40;
    }
    let _e42 = phi_175_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = j.Dg;
    let _e48 = j.Eg;
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
