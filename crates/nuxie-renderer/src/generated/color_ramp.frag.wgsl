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

var<private> Fg: vec4<f32>;
var<private> R6_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: CC;

fn main_1() {
    let _e3 = R6_1;
    Fg = _e3;
    return;
}

@fragment
fn main(@location(0) R6_: vec4<f32>) -> @location(0) vec4<f32> {
    R6_1 = R6_;
    main_1();
    let _e3 = Fg;
    return _e3;
}
