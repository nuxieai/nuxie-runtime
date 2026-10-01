struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var wa: sampler;
var<private> Sh: f32;
var<private> S_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(13)
var r5_: sampler;

fn main_1() {
    let _e12 = S_1;
    let _e16 = textureSampleLevel(YC, wa, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(YC, wa, vec2<f32>((1f - _e12.y), 0f), 0f);
    Sh = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) S: vec4<f32>) -> @location(0) f32 {
    S_1 = S;
    main_1();
    let _e3 = Sh;
    return _e3;
}
