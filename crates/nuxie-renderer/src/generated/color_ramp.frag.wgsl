struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

var<private> Yi: vec4<f32>;
var<private> y7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: RB;

fn main_1() {
    let _e3 = y7_1;
    Yi = _e3;
    return;
}

@fragment
fn main(@location(0) y7_: vec4<f32>) -> @location(0) vec4<f32> {
    y7_1 = y7_;
    main_1();
    let _e3 = Yi;
    return _e3;
}
