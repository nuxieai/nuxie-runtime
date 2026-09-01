struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

var<private> Pg: vec4<f32>;
var<private> T6_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;

fn main_1() {
    let _e3 = T6_1;
    Pg = _e3;
    return;
}

@fragment
fn main(@location(0) T6_: vec4<f32>) -> @location(0) vec4<f32> {
    T6_1 = T6_;
    main_1();
    let _e3 = Pg;
    return _e3;
}
