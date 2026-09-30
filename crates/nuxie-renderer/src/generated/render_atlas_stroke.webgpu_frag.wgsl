struct SB {
    yc: f32,
    Id: f32,
    Nf: f32,
    Of: f32,
    r6_: u32,
    Sb: u32,
    zf: u32,
    Af: u32,
    X7_: vec4<i32>,
    kh: vec2<f32>,
    Jd: vec2<f32>,
    f2_: u32,
    oh: f32,
    g6_: u32,
    W2_: f32,
    Kd: f32,
    tf: u32,
    F3_: f32,
    G3_: f32,
    Ld: f32,
    hh: u32,
    Rb: u32,
    ec: f32,
    fc: f32,
}

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ha: sampler;
var<private> rh: f32;
var<private> O_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: SB;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(13)
var Z5_: sampler;

fn main_1() {
    let _e12 = O_1;
    let _e16 = textureSampleLevel(XC, ha, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(XC, ha, vec2<f32>((1f - _e12.y), 0f), 0f);
    rh = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) O: vec4<f32>) -> @location(0) f32 {
    O_1 = O;
    main_1();
    let _e3 = rh;
    return _e3;
}
