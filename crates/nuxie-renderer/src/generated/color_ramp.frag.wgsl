struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
}

var<private> Tg: vec4<f32>;
var<private> V6_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: BC;

fn main_1() {
    let _e3 = V6_1;
    Tg = _e3;
    return;
}

@fragment
fn main(@location(0) V6_: vec4<f32>) -> @location(0) vec4<f32> {
    V6_1 = V6_;
    main_1();
    let _e3 = Tg;
    return _e3;
}
