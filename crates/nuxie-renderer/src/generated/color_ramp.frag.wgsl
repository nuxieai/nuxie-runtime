struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Bi: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Fi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    F3_: f32,
    G3_: f32,
    He: f32,
    yi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

var<private> Ii: vec4<f32>;
var<private> y7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;

fn main_1() {
    let _e3 = y7_1;
    Ii = _e3;
    return;
}

@fragment
fn main(@location(0) y7_: vec4<f32>) -> @location(0) vec4<f32> {
    y7_1 = y7_;
    main_1();
    let _e3 = Ii;
    return _e3;
}
