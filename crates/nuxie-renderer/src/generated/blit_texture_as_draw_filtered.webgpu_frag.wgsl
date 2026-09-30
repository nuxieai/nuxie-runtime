struct TB {
    uc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Ob: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    ih: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    mh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    fh: u32,
    Nb: u32,
    ac: f32,
    bc: f32,
}

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var Ef: sampler;
var<private> c2_1: vec2<f32>;
var<private> ph: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: TB;

fn main_1() {
    let _e6 = c2_1;
    let _e7 = textureSampleLevel(IC, Ef, _e6, 0f);
    ph = _e7;
    return;
}

@fragment
fn main(@location(0) c2_: vec2<f32>) -> @location(0) vec4<f32> {
    c2_1 = c2_;
    main_1();
    let _e3 = ph;
    return _e3;
}
