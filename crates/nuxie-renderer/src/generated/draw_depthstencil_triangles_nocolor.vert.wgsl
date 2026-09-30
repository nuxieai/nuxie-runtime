struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
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
var<uniform> j: TB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    let _e16 = JB_1;
    let _e18 = j.Hf;
    let _e20 = j.If;
    let _e28 = vec4<f32>(((_e16.x * _e18) - 1f), ((_e16.y * _e20) - sign(_e20)), 0f, 1f);
    let _e30 = JB_1[2u];
    unnamed.gl_Position = vec4<f32>(_e28.x, _e28.y, ((f32((((bitcast<u32>(_e30) & 65535u) << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e28.w);
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
