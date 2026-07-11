struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

var<private> yg: vec4<f32>;
var<private> R6_1: vec4<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;

fn main_1() {
    let _e3 = R6_1;
    yg = _e3;
    return;
}

@fragment 
fn main(@location(0) R6_: vec4<f32>) -> @location(0) vec4<f32> {
    R6_1 = R6_;
    main_1();
    let _e3 = yg;
    return _e3;
}
