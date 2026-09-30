struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var fa: sampler;
var<private> kh: f32;
var<private> M_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> l: BC;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(3) @binding(8)
var O9_: sampler;
@group(1) @binding(13)
var V5_: sampler;

fn main_1() {
    let _e12 = M_1;
    let _e16 = textureSampleLevel(YC, fa, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(YC, fa, vec2<f32>((1f - _e12.y), 0f), 0f);
    kh = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) M: vec4<f32>) -> @location(0) f32 {
    M_1 = M;
    main_1();
    let _e3 = kh;
    return _e3;
}
