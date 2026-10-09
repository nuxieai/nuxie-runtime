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

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var Eg: sampler;
var<private> l2_1: vec2<f32>;
var<private> Fi: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;

fn main_1() {
    let _e6 = l2_1;
    let _e7 = textureSampleLevel(IC, Eg, _e6, 0f);
    Fi = _e7;
    return;
}

@fragment
fn main(@location(0) l2_: vec2<f32>) -> @location(0) vec4<f32> {
    l2_1 = l2_;
    main_1();
    let _e3 = Fi;
    return _e3;
}
