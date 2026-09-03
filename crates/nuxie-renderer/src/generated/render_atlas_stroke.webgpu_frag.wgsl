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

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
var<private> Qg: f32;
var<private> O_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(3) @binding(8)
var Pb: sampler;
@group(1) @binding(13)
var V5_: sampler;

fn main_1() {
    let _e12 = O_1;
    let _e16 = textureSampleLevel(YC, aa, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(YC, aa, vec2<f32>((1f - _e12.y), 0f), 0f);
    Qg = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) O: vec4<f32>) -> @location(0) f32 {
    O_1 = O;
    main_1();
    let _e3 = Qg;
    return _e3;
}
