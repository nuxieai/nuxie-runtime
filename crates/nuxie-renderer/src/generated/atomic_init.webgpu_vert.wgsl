struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct jd {
    x2_: array<vec4<u32>>,
}

struct ug {
    x2_: array<vec4<f32>>,
}

struct pi {
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
var<storage> JB: jd;
@group(0) @binding(3)
var<storage> KB: ug;
@group(0) @binding(4)
var<storage> CD: pi;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_169_: i32;
    var phi_172_: i32;

    let _e21 = gl_VertexIndex_1;
    if ((_e21 & 1i) == 0i) {
        let _e26 = j.H8_[0u];
        phi_169_ = _e26;
    } else {
        let _e29 = j.H8_[2u];
        phi_169_ = _e29;
    }
    let _e31 = phi_169_;
    if ((_e21 & 2i) == 0i) {
        let _e36 = j.H8_[1u];
        phi_172_ = _e36;
    } else {
        let _e39 = j.H8_[3u];
        phi_172_ = _e39;
    }
    let _e41 = phi_172_;
    let _e43 = vec2<f32>(vec2<i32>(_e31, _e41));
    let _e45 = j.Rg;
    let _e47 = j.Sg;
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
