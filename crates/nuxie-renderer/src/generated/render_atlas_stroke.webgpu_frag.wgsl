struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;
var<private> Uh: f32;
var<private> S_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(13)
var r5_: sampler;

fn main_1() {
    let _e12 = S_1;
    let _e16 = textureSampleLevel(YC, xa, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(YC, xa, vec2<f32>((1f - _e12.y), 0f), 0f);
    Uh = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) S: vec4<f32>) -> @location(0) f32 {
    S_1 = S;
    main_1();
    let _e3 = Uh;
    return _e3;
}
