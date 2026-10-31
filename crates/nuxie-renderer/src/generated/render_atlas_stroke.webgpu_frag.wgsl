struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
}

@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var fb: sampler;
var<private> Ri: f32;
var<private> S_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(3) @binding(7)
var R8_: sampler;
@group(1) @binding(12)
var v4_: sampler;

fn main_1() {
    let _e12 = S_1;
    let _e16 = textureSampleLevel(ZC, fb, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(ZC, fb, vec2<f32>((1f - _e12.y), 0f), 0f);
    Ri = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) S: vec4<f32>) -> @location(0) f32 {
    S_1 = S;
    main_1();
    let _e3 = Ri;
    return _e3;
}
