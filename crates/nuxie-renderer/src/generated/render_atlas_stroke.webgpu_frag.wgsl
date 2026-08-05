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

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
var<private> Jg: f32;
var<private> O_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(3) @binding(8)
var Mb: sampler;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    let _e12 = O_1;
    let _e16 = textureSampleLevel(XC, aa, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(XC, aa, vec2<f32>((1f - _e12.y), 0f), 0f);
    Jg = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) O: vec4<f32>) -> @location(0) f32 {
    O_1 = O;
    main_1();
    let _e3 = Jg;
    return _e3;
}
