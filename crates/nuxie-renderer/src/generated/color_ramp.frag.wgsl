struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

var<private> kh: vec4<f32>;
var<private> W6_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> l: BC;

fn main_1() {
    let _e3 = W6_1;
    kh = _e3;
    return;
}

@fragment
fn main(@location(0) W6_: vec4<f32>) -> @location(0) vec4<f32> {
    W6_1 = W6_;
    main_1();
    let _e3 = kh;
    return _e3;
}
