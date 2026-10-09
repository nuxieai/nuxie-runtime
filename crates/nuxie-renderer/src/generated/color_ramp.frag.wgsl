struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

var<private> Fi: vec4<f32>;
var<private> v7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;

fn main_1() {
    let _e3 = v7_1;
    Fi = _e3;
    return;
}

@fragment
fn main(@location(0) v7_: vec4<f32>) -> @location(0) vec4<f32> {
    v7_1 = v7_;
    main_1();
    let _e3 = Fi;
    return _e3;
}
