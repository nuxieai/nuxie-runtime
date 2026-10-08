struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

var<private> Ei: vec4<f32>;
var<private> p7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;

fn main_1() {
    let _e3 = p7_1;
    Ei = _e3;
    return;
}

@fragment
fn main(@location(0) p7_: vec4<f32>) -> @location(0) vec4<f32> {
    p7_1 = p7_;
    main_1();
    let _e3 = Ei;
    return _e3;
}
