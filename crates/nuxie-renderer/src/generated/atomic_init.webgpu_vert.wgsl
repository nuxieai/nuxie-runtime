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

struct nd {
    z2_: array<vec4<u32>>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

struct xi {
    z2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> j: RB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(6)
var WB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> JB: nd;
@group(0) @binding(3)
var<storage> KB: Cg;
@group(0) @binding(4)
var<storage> CD: xi;
@group(3) @binding(8)
var ib: sampler;

fn main_1() {
    var phi_169_: i32;
    var phi_172_: i32;

    let _e21 = gl_VertexIndex_1;
    if ((_e21 & 1i) == 0i) {
        let _e26 = j.M8_[0u];
        phi_169_ = _e26;
    } else {
        let _e29 = j.M8_[2u];
        phi_169_ = _e29;
    }
    let _e31 = phi_169_;
    if ((_e21 & 2i) == 0i) {
        let _e36 = j.M8_[1u];
        phi_172_ = _e36;
    } else {
        let _e39 = j.M8_[3u];
        phi_172_ = _e39;
    }
    let _e41 = phi_172_;
    let _e43 = vec2<f32>(vec2<i32>(_e31, _e41));
    let _e45 = j.ah;
    let _e47 = j.bh;
    unnamed.gl_Position = vec4<f32>(((_e43.x * _e45) - 1f), ((_e43.y * _e47) - sign(_e47)), 0f, 1f);
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
