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

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var ag: sampler;
var<private> f2_1: vec2<f32>;
var<private> Uh: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;

fn main_1() {
    let _e6 = f2_1;
    let _e7 = textureSampleLevel(IC, ag, _e6, 0f);
    Uh = _e7;
    return;
}

@fragment
fn main(@location(0) f2_: vec2<f32>) -> @location(0) vec4<f32> {
    f2_1 = f2_;
    main_1();
    let _e3 = Uh;
    return _e3;
}
