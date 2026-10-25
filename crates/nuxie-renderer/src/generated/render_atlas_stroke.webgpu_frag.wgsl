struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var eb: sampler;
var<private> Ri: f32;
var<private> T_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(1) @binding(13)
var v4_: sampler;

fn main_1() {
    let _e12 = T_1;
    let _e16 = textureSampleLevel(AD, eb, vec2<f32>((3f + _e12.x), 0f), 0f);
    let _e22 = textureSampleLevel(AD, eb, vec2<f32>((1f - _e12.y), 0f), 0f);
    Ri = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) T: vec4<f32>) -> @location(0) f32 {
    T_1 = T;
    main_1();
    let _e3 = Ri;
    return _e3;
}
