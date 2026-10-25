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

@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var Jg: sampler;
var<private> n2_1: vec2<f32>;
var<private> Mi: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;

fn main_1() {
    let _e6 = n2_1;
    let _e7 = textureSampleLevel(JC, Jg, _e6, 0f);
    Mi = _e7;
    return;
}

@fragment
fn main(@location(0) n2_: vec2<f32>) -> @location(0) vec4<f32> {
    n2_1 = n2_;
    main_1();
    let _e3 = Mi;
    return _e3;
}
