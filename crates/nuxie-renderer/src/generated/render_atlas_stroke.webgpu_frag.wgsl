struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ea: sampler;
var<private> jh: f32;
var<private> M_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> l: BC;
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
    jh = ((1f - _e16.x) - _e22.x);
    return;
}

@fragment
fn main(@location(0) M: vec4<f32>) -> @location(0) f32 {
    M_1 = M;
    main_1();
    let _e3 = jh;
    return _e3;
}
