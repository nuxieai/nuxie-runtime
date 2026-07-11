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

var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    let _e13 = KB_1;
    let _e15 = k.Xe;
    let _e17 = k.Ye;
    let _e25 = vec4<f32>(((_e13.x * _e15) - 1f), ((_e13.y * _e17) - sign(_e17)), 0f, 1f);
    let _e27 = KB_1[2u];
    unnamed.gl_Position = vec4<f32>(_e25.x, _e25.y, (1f - (f32((bitcast<u32>(_e27) & 65535u)) * 0.000061035156f)), _e25.w);
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> @builtin(position) vec4<f32> {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e7 = unnamed.gl_Position;
    return _e7;
}
