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

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var da: sampler;
var<private> ph: f32;
var<private> O_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: TB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var W5_: sampler;

fn main_1() {
    let _e12 = O_1;
    let _e16 = textureSampleLevel(YC, da, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(YC, da, vec2<f32>((1f - _e12.y), 0f), 0f);
    ph = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) O: vec4<f32>) -> @location(0) f32 {
    O_1 = O;
    main_1();
    let _e3 = ph;
    return _e3;
}
