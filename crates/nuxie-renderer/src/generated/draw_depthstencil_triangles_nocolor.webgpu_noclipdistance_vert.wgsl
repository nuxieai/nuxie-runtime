struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
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
var<uniform> m: BC;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    let _e13 = KB_1;
    let _e15 = m.of_;
    let _e17 = m.pf;
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
