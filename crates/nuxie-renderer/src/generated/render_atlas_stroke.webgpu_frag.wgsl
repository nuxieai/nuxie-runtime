struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
var<private> Hg: f32;
var<private> O_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    let _e12 = O_1;
    let _e16 = textureSampleLevel(XC, aa, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(XC, aa, vec2<f32>((1f - _e12.y), 0f), 0f);
    Hg = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) O: vec4<f32>) -> @location(0) f32 {
    O_1 = O;
    main_1();
    let _e3 = Hg;
    return _e3;
}
