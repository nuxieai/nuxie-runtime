struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Bi: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Fi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    F3_: f32,
    G3_: f32,
    He: f32,
    yi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var db: sampler;
var<private> Ii: f32;
var<private> S_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    let _e12 = S_1;
    let _e16 = textureSampleLevel(AD, db, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(AD, db, vec2<f32>((1f - _e12.y), 0f), 0f);
    Ii = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) S: vec4<f32>) -> @location(0) f32 {
    S_1 = S;
    main_1();
    let _e3 = Ii;
    return _e3;
}
