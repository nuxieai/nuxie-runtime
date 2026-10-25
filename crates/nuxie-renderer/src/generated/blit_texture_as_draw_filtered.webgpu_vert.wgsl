struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

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

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

var<private> gl_VertexIndex_1: i32;
var<private> n2_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(0)
var<uniform> j: WB;

fn main_1() {
    let _e14 = gl_VertexIndex_1;
    let _e17 = select(1f, -1f, ((_e14 & 1i) == 0i));
    let _e20 = select(1f, -1f, ((_e14 & 2i) == 0i));
    n2_[0u] = ((_e17 * 0.5f) + 0.5f);
    n2_[1u] = ((_e20 * -0.5f) + 0.5f);
    unnamed.gl_Position = vec4<f32>(_e17, _e20, 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e6 = n2_;
    let _e7 = unnamed.gl_Position;
    return VertexOutput(_e6, _e7);
}
