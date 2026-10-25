struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var db: sampler;
var<private> Ni: f32;
var<private> T_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
@group(1) @binding(13)
var W4_: sampler;

fn main_1() {
    let _e12 = T_1;
    let _e16 = textureSampleLevel(AD, db, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(AD, db, vec2<f32>((1f - _e12.y), 0f), 0f);
    Ni = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) T: vec4<f32>) -> @location(0) f32 {
    T_1 = T;
    main_1();
    let _e3 = Ni;
    return _e3;
}
