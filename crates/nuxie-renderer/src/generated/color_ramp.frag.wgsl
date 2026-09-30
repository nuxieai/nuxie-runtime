struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

var<private> Jh: vec4<f32>;
var<private> g7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;

fn main_1() {
    let _e3 = g7_1;
    Jh = _e3;
    return;
}

@fragment
fn main(@location(0) g7_: vec4<f32>) -> @location(0) vec4<f32> {
    g7_1 = g7_;
    main_1();
    let _e3 = Jh;
    return _e3;
}
