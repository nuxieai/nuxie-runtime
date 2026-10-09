struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct di {
    r2_: array<vec4<u32>>,
}

struct jg {
    r2_: array<vec2<u32>>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

struct ei {
    r2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> KB: di;
@group(0) @binding(3)
var<storage> VC: jg;
@group(0) @binding(4)
var<storage> JB: kg;
@group(0) @binding(5)
var<storage> BD: ei;
@group(3) @binding(9)
var ab: sampler;

fn main_1() {
    var phi_173_: i32;
    var phi_176_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = j.E8_[0u];
        phi_173_ = _e27;
    } else {
        let _e30 = j.E8_[2u];
        phi_173_ = _e30;
    }
    let _e32 = phi_173_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = j.E8_[1u];
        phi_176_ = _e37;
    } else {
        let _e40 = j.E8_[3u];
        phi_176_ = _e40;
    }
    let _e42 = phi_176_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = j.Hg;
    let _e48 = j.Ig;
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
