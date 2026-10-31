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

struct kd {
    x2_: array<vec4<u32>>,
}

struct vg {
    x2_: array<vec4<f32>>,
}

struct qi {
    x2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> JB: kd;
@group(0) @binding(3)
var<storage> KB: vg;
@group(0) @binding(4)
var<storage> CD: qi;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_169_: i32;
    var phi_172_: i32;

    let _e21 = gl_VertexIndex_1;
    if ((_e21 & 1i) == 0i) {
        let _e26 = j.I8_[0u];
        phi_169_ = _e26;
    } else {
        let _e29 = j.I8_[2u];
        phi_169_ = _e29;
    }
    let _e31 = phi_169_;
    if ((_e21 & 2i) == 0i) {
        let _e36 = j.I8_[1u];
        phi_172_ = _e36;
    } else {
        let _e39 = j.I8_[3u];
        phi_172_ = _e39;
    }
    let _e41 = phi_172_;
    let _e43 = vec2<f32>(vec2<i32>(_e31, _e41));
    let _e45 = j.Sg;
    let _e47 = j.Tg;
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
