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

@group(1) @binding(11)
var KC: texture_2d<f32>;
@group(1) @binding(13)
var ff: sampler;
var<private> X1_1: vec2<f32>;
var<private> Pg: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;

fn main_1() {
    let _e6 = X1_1;
    let _e7 = textureSampleLevel(KC, ff, _e6, 0f);
    Pg = _e7;
    return;
}

@fragment
fn main(@location(0) X1_: vec2<f32>) -> @location(0) vec4<f32> {
    X1_1 = X1_;
    main_1();
    let _e3 = Pg;
    return _e3;
}
