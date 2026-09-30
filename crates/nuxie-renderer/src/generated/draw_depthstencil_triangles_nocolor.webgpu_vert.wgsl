struct SB {
    yc: f32,
    Id: f32,
    Nf: f32,
    Of: f32,
    r6_: u32,
    Sb: u32,
    zf: u32,
    Af: u32,
    X7_: vec4<i32>,
    kh: vec2<f32>,
    Jd: vec2<f32>,
    f2_: u32,
    oh: f32,
    g6_: u32,
    W2_: f32,
    Kd: f32,
    tf: u32,
    F3_: f32,
    G3_: f32,
    Ld: f32,
    hh: u32,
    Rb: u32,
    ec: f32,
    fc: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

var<private> gl_VertexIndex_1: i32;
var<private> JB_1: vec3<f32>;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    let _e13 = JB_1;
    let _e15 = j.Nf;
    let _e17 = j.Of;
    let _e25 = vec4<f32>(((_e13.x * _e15) - 1f), ((_e13.y * _e17) - sign(_e17)), 0f, 1f);
    let _e27 = JB_1[2u];
    unnamed.gl_Position = vec4<f32>(_e25.x, _e25.y, (1f - (f32((bitcast<u32>(_e27) & 65535u)) * 0.000061035156f)), _e25.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) JB: vec3<f32>) -> @builtin(position) vec4<f32> {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    JB_1 = JB;
    main_1();
    let _e7 = unnamed.gl_Position;
    return _e7;
}
