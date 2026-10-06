struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

var<private> gl_VertexIndex_1: i32;
var<private> MB_1: vec3<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    let _e16 = MB_1;
    let _e18 = j.dg;
    let _e20 = j.eg;
    let _e28 = vec4<f32>(((_e16.x * _e18) - 1f), ((_e16.y * _e20) - sign(_e20)), 0f, 1f);
    let _e30 = MB_1[2u];
    unnamed.gl_Position = vec4<f32>(_e28.x, _e28.y, ((f32((((bitcast<u32>(_e30) & 65535u) << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e28.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) MB: vec3<f32>) -> @builtin(position) vec4<f32> {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    MB_1 = MB;
    main_1();
    let _e7 = unnamed.gl_Position;
    return _e7;
}
