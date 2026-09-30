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

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var Cf: sampler;
var<private> Z1_1: vec2<f32>;
var<private> jh: vec4<f32>;
@group(0) @binding(0)
var<uniform> l: BC;

fn main_1() {
    let _e6 = Z1_1;
    let _e7 = textureSampleLevel(IC, Cf, _e6, 0f);
    jh = _e7;
    return;
}

@fragment
fn main(@location(0) Z1_: vec2<f32>) -> @location(0) vec4<f32> {
    Z1_1 = Z1_;
    main_1();
    let _e3 = jh;
    return _e3;
}
