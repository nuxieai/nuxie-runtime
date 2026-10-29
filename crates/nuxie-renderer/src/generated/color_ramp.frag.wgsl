struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

var<private> Qi: vec4<f32>;
var<private> w7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;

fn main_1() {
    let _e3 = w7_1;
    Qi = _e3;
    return;
}

@fragment
fn main(@location(0) w7_: vec4<f32>) -> @location(0) vec4<f32> {
    w7_1 = w7_;
    main_1();
    let _e3 = Qi;
    return _e3;
}
