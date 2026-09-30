struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Cg {
    e2_: array<vec4<u32>>,
}

struct jf {
    e2_: array<vec2<u32>>,
}

struct kf {
    e2_: array<vec4<f32>>,
}

struct Dg {
    e2_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(2)
var<storage> PB: Cg;
@group(0) @binding(3)
var<storage> DD: jf;
@group(0) @binding(4)
var<storage> QB: kf;
@group(0) @binding(5)
var<storage> ID: Dg;
@group(3) @binding(9)
var ea: sampler;

fn main_1() {
    var phi_172_: i32;
    var phi_175_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = l.V7_[0u];
        phi_172_ = _e27;
    } else {
        let _e30 = l.V7_[2u];
        phi_172_ = _e30;
    }
    let _e32 = phi_172_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = l.V7_[1u];
        phi_175_ = _e37;
    } else {
        let _e40 = l.V7_[3u];
        phi_175_ = _e40;
    }
    let _e42 = phi_175_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = l.Ff;
    let _e48 = l.Gf;
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
