struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var Yf: sampler;
var<private> f2_1: vec2<f32>;
var<private> Sh: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;

fn main_1() {
    let _e6 = f2_1;
    let _e7 = textureSampleLevel(IC, Yf, _e6, 0f);
    Sh = _e7;
    return;
}

@fragment
fn main(@location(0) f2_: vec2<f32>) -> @location(0) vec4<f32> {
    f2_1 = f2_;
    main_1();
    let _e3 = Sh;
    return _e3;
}
