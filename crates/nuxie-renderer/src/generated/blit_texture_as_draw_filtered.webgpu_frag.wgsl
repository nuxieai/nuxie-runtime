struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var Ye: sampler;
var<private> X1_1: vec2<f32>;
var<private> Fg: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: CC;

fn main_1() {
    let _e6 = X1_1;
    let _e7 = textureSampleLevel(JC, Ye, _e6, 0f);
    Fg = _e7;
    return;
}

@fragment
fn main(@location(0) X1_: vec2<f32>) -> @location(0) vec4<f32> {
    X1_1 = X1_;
    main_1();
    let _e3 = Fg;
    return _e3;
}
