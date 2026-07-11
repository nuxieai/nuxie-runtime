struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Rf {
    X1_: array<vec4<u32>>,
}

struct Be {
    X1_: array<vec2<u32>>,
}

struct Ce {
    X1_: array<vec4<f32>>,
}

struct Sf {
    X1_: array<vec4<u32>>,
}

var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8) 
var DC: texture_2d<u32>;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(0) @binding(3) 
var<storage> MB: Rf;
@group(0) @binding(4) 
var<storage> TC: Be;
@group(0) @binding(5) 
var<storage> PB: Ce;
@group(0) @binding(6) 
var<storage> XC: Sf;
@group(3) @binding(10) 
var T9_: sampler;

fn main_1() {
    var phi_170_: i32;
    var phi_173_: i32;

    let _e22 = gl_VertexIndex_1;
    if ((_e22 & 1i) == 0i) {
        let _e27 = k.U7_[0u];
        phi_170_ = _e27;
    } else {
        let _e30 = k.U7_[2u];
        phi_170_ = _e30;
    }
    let _e32 = phi_170_;
    if ((_e22 & 2i) == 0i) {
        let _e37 = k.U7_[1u];
        phi_173_ = _e37;
    } else {
        let _e40 = k.U7_[3u];
        phi_173_ = _e40;
    }
    let _e42 = phi_173_;
    let _e44 = vec2<f32>(vec2<i32>(_e32, _e42));
    let _e46 = k.Xe;
    let _e48 = k.Ye;
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
