struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct VertexOutput {
    @location(0) member: vec2<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

var<private> gl_VertexIndex_1: i32;
var<private> X1_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(0)
var<uniform> m: DC;

fn main_1() {
    let _e14 = gl_VertexIndex_1;
    let _e17 = select(1f, -1f, ((_e14 & 1i) == 0i));
    let _e20 = select(1f, -1f, ((_e14 & 2i) == 0i));
    X1_[0u] = ((_e17 * 0.5f) + 0.5f);
    X1_[1u] = ((_e20 * -0.5f) + 0.5f);
    unnamed.gl_Position = vec4<f32>(_e17, _e20, 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e6 = X1_;
    let _e7 = unnamed.gl_Position;
    return VertexOutput(_e6, _e7);
}
