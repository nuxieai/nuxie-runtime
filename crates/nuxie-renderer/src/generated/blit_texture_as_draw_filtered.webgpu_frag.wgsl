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

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var lf: sampler;
var<private> Y1_1: vec2<f32>;
var<private> Tg: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: BC;

fn main_1() {
    let _e6 = Y1_1;
    let _e7 = textureSampleLevel(IC, lf, _e6, 0f);
    Tg = _e7;
    return;
}

@fragment
fn main(@location(0) Y1_: vec2<f32>) -> @location(0) vec4<f32> {
    Y1_1 = Y1_;
    main_1();
    let _e3 = Tg;
    return _e3;
}
