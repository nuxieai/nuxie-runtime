struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ga: sampler;
var<private> lh: f32;
var<private> M_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: AC;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(13)
var Y5_: sampler;

fn main_1() {
    let _e12 = M_1;
    let _e16 = textureSampleLevel(XC, ga, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(XC, ga, vec2<f32>((1f - _e12.y), 0f), 0f);
    lh = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) M: vec4<f32>) -> @location(0) f32 {
    M_1 = M;
    main_1();
    let _e3 = lh;
    return _e3;
}
