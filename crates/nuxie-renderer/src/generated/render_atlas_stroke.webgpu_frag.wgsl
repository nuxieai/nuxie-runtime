struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
}

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;
var<private> Tg: f32;
var<private> L_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: BC;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(3) @binding(8)
var Rb: sampler;
@group(1) @binding(13)
var V5_: sampler;

fn main_1() {
    let _e12 = L_1;
    let _e16 = textureSampleLevel(XC, Z9_, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(XC, Z9_, vec2<f32>((1f - _e12.y), 0f), 0f);
    Tg = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) L: vec4<f32>) -> @location(0) f32 {
    L_1 = L;
    main_1();
    let _e3 = Tg;
    return _e3;
}
