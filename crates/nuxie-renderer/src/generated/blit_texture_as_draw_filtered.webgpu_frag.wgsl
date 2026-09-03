struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

@group(1) @binding(11)
var KC: texture_2d<f32>;
@group(1) @binding(13)
var gf: sampler;
var<private> Y1_1: vec2<f32>;
var<private> Qg: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;

fn main_1() {
    let _e6 = Y1_1;
    let _e7 = textureSampleLevel(KC, gf, _e6, 0f);
    Qg = _e7;
    return;
}

@fragment
fn main(@location(0) Y1_: vec2<f32>) -> @location(0) vec4<f32> {
    Y1_1 = Y1_;
    main_1();
    let _e3 = Qg;
    return _e3;
}
