struct UB {
    Rc: f32,
    Ud: f32,
    cg: f32,
    dg: f32,
    B6_: u32,
    Y9_: u32,
    Of: u32,
    Pf: u32,
    k8_: vec4<i32>,
    Mh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Qh: f32,
    U4_: u32,
    c3_: f32,
    Wd: f32,
    If: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Jh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;
var<private> Th: f32;
var<private> S_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(1) @binding(11)
var DC: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(13)
var v5_: sampler;

fn main_1() {
    let _e12 = S_1;
    let _e16 = textureSampleLevel(ZC, xa, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(ZC, xa, vec2<f32>((1f - _e12.y), 0f), 0f);
    Th = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) S: vec4<f32>) -> @location(0) f32 {
    S_1 = S;
    main_1();
    let _e3 = Th;
    return _e3;
}
