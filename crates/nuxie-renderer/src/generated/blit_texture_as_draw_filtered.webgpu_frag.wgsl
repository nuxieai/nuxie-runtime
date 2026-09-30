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

@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var Zf: sampler;
var<private> f2_1: vec2<f32>;
var<private> Th: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;

fn main_1() {
    let _e6 = f2_1;
    let _e7 = textureSampleLevel(JC, Zf, _e6, 0f);
    Th = _e7;
    return;
}

@fragment
fn main(@location(0) f2_: vec2<f32>) -> @location(0) vec4<f32> {
    f2_1 = f2_;
    main_1();
    let _e3 = Th;
    return _e3;
}
