struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Fg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Bg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Gg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    yg: u32,
}

@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var cf: sampler;
var<private> X1_1: vec2<f32>;
var<private> Jg: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;

fn main_1() {
    let _e6 = X1_1;
    let _e7 = textureSampleLevel(JC, cf, _e6, 0f);
    Jg = _e7;
    return;
}

@fragment
fn main(@location(0) X1_: vec2<f32>) -> @location(0) vec4<f32> {
    X1_1 = X1_;
    main_1();
    let _e3 = Jg;
    return _e3;
}
