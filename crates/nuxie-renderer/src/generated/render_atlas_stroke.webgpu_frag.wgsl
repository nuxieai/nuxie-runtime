struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Va: sampler;
var<private> Ii: f32;
var<private> S_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var I8_: sampler;
@group(1) @binding(13)
var S4_: sampler;

fn main_1() {
    let _e12 = S_1;
    let _e16 = textureSampleLevel(ZC, Va, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(ZC, Va, vec2<f32>((1f - _e12.y), 0f), 0f);
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
