struct SB {
    yc: f32,
    Id: f32,
    Nf: f32,
    Of: f32,
    r6_: u32,
    Sb: u32,
    zf: u32,
    Af: u32,
    X7_: vec4<i32>,
    kh: vec2<f32>,
    Jd: vec2<f32>,
    f2_: u32,
    oh: f32,
    g6_: u32,
    W2_: f32,
    Kd: f32,
    tf: u32,
    F3_: f32,
    G3_: f32,
    Ld: f32,
    hh: u32,
    Rb: u32,
    ec: f32,
    fc: f32,
}

var<private> rh: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: SB;

fn main_1() {
    let _e8 = vec4<f32>(0f, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
    let _e13 = vec4<f32>(_e8.x, 0f, _e8.z, _e8.w);
    let _e18 = vec4<f32>(_e13.x, _e13.y, 0f, _e13.w);
    rh = vec4<f32>(_e18.x, _e18.y, _e18.z, 0f);
    return;
}

@fragment
fn main() -> @location(0) vec4<f32> {
    main_1();
    let _e1 = rh;
    return _e1;
}
