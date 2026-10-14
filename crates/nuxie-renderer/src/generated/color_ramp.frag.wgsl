struct VB {
    xd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    F8_: vec4<i32>,
    zi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Di: f32,
    w6_: u32,
    i3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    wi: u32,
    Ba: u32,
    M8_: f32,
    N8_: f32,
}

var<private> Gi: vec4<f32>;
var<private> w7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;

fn main_1() {
    let _e3 = w7_1;
    Gi = _e3;
    return;
}

@fragment
fn main(@location(0) w7_: vec4<f32>) -> @location(0) vec4<f32> {
    w7_1 = w7_;
    main_1();
    let _e3 = Gi;
    return _e3;
}
