struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    f6_: u32,
    U2_: f32,
    Cd: f32,
    lf: u32,
    C3_: f32,
    D3_: f32,
    Dd: f32,
    Yg: u32,
}

var<private> ih: vec4<f32>;
var<private> W6_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: BC;

fn main_1() {
    let _e3 = W6_1;
    ih = _e3;
    return;
}

@fragment
fn main(@location(0) W6_: vec4<f32>) -> @location(0) vec4<f32> {
    W6_1 = W6_;
    main_1();
    let _e3 = ih;
    return _e3;
}
