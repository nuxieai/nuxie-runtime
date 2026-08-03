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

@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var af: sampler;
var<private> X1_1: vec2<f32>;
var<private> Hg: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;

fn main_1() {
    let _e6 = X1_1;
    let _e7 = textureSampleLevel(JC, af, _e6, 0f);
    Hg = _e7;
    return;
}

@fragment
fn main(@location(0) X1_: vec2<f32>) -> @location(0) vec4<f32> {
    X1_1 = X1_;
    main_1();
    let _e3 = Hg;
    return _e3;
}
