struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    H8_: vec4<i32>,
    Fi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ji: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Ci: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ki {
    v2_: array<vec4<u32>>,
}

struct og {
    v2_: array<vec2<u32>>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct li {
    v2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(2)
var<storage> KB: ki;
@group(0) @binding(3)
var<storage> XC: og;
@group(0) @binding(4)
var<storage> JB: pg;
@group(0) @binding(5)
var<storage> DD: li;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    var phi_173_: i32;
    var phi_176_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = j.H8_[0u];
        phi_173_ = _e27;
    } else {
        let _e30 = j.H8_[2u];
        phi_173_ = _e30;
    }
    let _e32 = phi_173_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = j.H8_[1u];
        phi_176_ = _e37;
    } else {
        let _e40 = j.H8_[3u];
        phi_176_ = _e40;
    }
    let _e42 = phi_176_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = j.Mg;
    let _e48 = j.Ng;
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
