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

@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var Vf: sampler;
var<private> e2_1: vec2<f32>;
var<private> Jh: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;

fn main_1() {
    let _e6 = e2_1;
    let _e7 = textureSampleLevel(JC, Vf, _e6, 0f);
    Jh = _e7;
    return;
}

@fragment
fn main(@location(0) e2_: vec2<f32>) -> @location(0) vec4<f32> {
    e2_1 = e2_;
    main_1();
    let _e3 = Jh;
    return _e3;
}
