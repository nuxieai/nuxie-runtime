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

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;
var<private> Fg: f32;
var<private> O_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: CC;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    let _e12 = O_1;
    let _e16 = textureSampleLevel(XC, Z9_, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(XC, Z9_, vec2<f32>((1f - _e12.y), 0f), 0f);
    Fg = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) O: vec4<f32>) -> @location(0) f32 {
    O_1 = O;
    main_1();
    let _e3 = Fg;
    return _e3;
}
