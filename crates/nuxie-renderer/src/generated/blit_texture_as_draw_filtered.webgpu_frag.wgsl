struct SB {
    xc: f32,
    Hd: f32,
    Mf: f32,
    Nf: f32,
    r6_: u32,
    Rb: u32,
    yf: u32,
    zf: u32,
    X7_: vec4<i32>,
    jh: vec2<f32>,
    Id: vec2<f32>,
    f2_: u32,
    nh: f32,
    g6_: u32,
    W2_: f32,
    Jd: f32,
    sf: u32,
    F3_: f32,
    G3_: f32,
    Kd: f32,
    gh: u32,
    Qb: u32,
    dc: f32,
    ec: f32,
}

@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var Jf: sampler;
var<private> c2_1: vec2<f32>;
var<private> qh: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: SB;

fn main_1() {
    let _e6 = c2_1;
    let _e7 = textureSampleLevel(HC, Jf, _e6, 0f);
    qh = _e7;
    return;
}

@fragment
fn main(@location(0) c2_: vec2<f32>) -> @location(0) vec4<f32> {
    c2_1 = c2_;
    main_1();
    let _e3 = qh;
    return _e3;
}
