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

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Ta: sampler;
var<private> Ei: f32;
var<private> S_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
@group(1) @binding(13)
var S4_: sampler;

fn main_1() {
    let _e12 = S_1;
    let _e16 = textureSampleLevel(ZC, Ta, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(ZC, Ta, vec2<f32>((1f - _e12.y), 0f), 0f);
    Ei = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) S: vec4<f32>) -> @location(0) f32 {
    S_1 = S;
    main_1();
    let _e3 = Ei;
    return _e3;
}
