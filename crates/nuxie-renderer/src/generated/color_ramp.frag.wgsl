struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

var<private> oh: vec4<f32>;
var<private> W6_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: TB;

fn main_1() {
    let _e3 = W6_1;
    oh = _e3;
    return;
}

@fragment
fn main(@location(0) W6_: vec4<f32>) -> @location(0) vec4<f32> {
    W6_1 = W6_;
    main_1();
    let _e3 = oh;
    return _e3;
}
