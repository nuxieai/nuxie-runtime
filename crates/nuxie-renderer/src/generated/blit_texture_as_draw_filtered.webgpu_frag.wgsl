struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var Ef: sampler;
var<private> c2_1: vec2<f32>;
var<private> lh: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: SB;

fn main_1() {
    let _e6 = c2_1;
    let _e7 = textureSampleLevel(HC, Ef, _e6, 0f);
    lh = _e7;
    return;
}

@fragment
fn main(@location(0) c2_: vec2<f32>) -> @location(0) vec4<f32> {
    c2_1 = c2_;
    main_1();
    let _e3 = lh;
    return _e3;
}
