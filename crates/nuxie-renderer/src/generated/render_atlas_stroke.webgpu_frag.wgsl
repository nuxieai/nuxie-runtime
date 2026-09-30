struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    f6_: u32,
    U2_: f32,
    Cd: f32,
    lf: u32,
    C3_: f32,
    D3_: f32,
    Dd: f32,
    Yg: u32,
}

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ea: sampler;
var<private> ih: f32;
var<private> M_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: BC;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var X5_: sampler;

fn main_1() {
    let _e12 = M_1;
    let _e16 = textureSampleLevel(YC, ea, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(YC, ea, vec2<f32>((1f - _e12.y), 0f), 0f);
    ih = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) M: vec4<f32>) -> @location(0) f32 {
    M_1 = M;
    main_1();
    let _e3 = ih;
    return _e3;
}
