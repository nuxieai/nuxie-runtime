struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());

fn main_1() {
    let _e16 = LB_1;
    let _e18 = j.Gg;
    let _e20 = j.Hg;
    let _e28 = vec4<f32>(((_e16.x * _e18) - 1f), ((_e16.y * _e20) - sign(_e20)), 0f, 1f);
    let _e30 = LB_1[2u];
    unnamed.gl_Position = vec4<f32>(_e28.x, _e28.y, ((f32((((bitcast<u32>(_e30) & 65535u) << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e28.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> @builtin(position) vec4<f32> {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e7 = unnamed.gl_Position;
    return _e7;
}
