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

var<private> Jg: vec4<f32>;
var<private> R6_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;

fn main_1() {
    let _e3 = R6_1;
    Jg = _e3;
    return;
}

@fragment
fn main(@location(0) R6_: vec4<f32>) -> @location(0) vec4<f32> {
    R6_1 = R6_;
    main_1();
    let _e3 = Jg;
    return _e3;
}
