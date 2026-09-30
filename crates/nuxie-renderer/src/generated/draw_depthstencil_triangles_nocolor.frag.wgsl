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
@group(0) @binding(0)
var<uniform> l: BC;

fn main_1() {
    let _e8 = vec4<f32>(0f, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
    let _e13 = vec4<f32>(_e8.x, 0f, _e8.z, _e8.w);
    let _e18 = vec4<f32>(_e13.x, _e13.y, 0f, _e13.w);
    kh = vec4<f32>(_e18.x, _e18.y, _e18.z, 0f);
    return;
}

@fragment
fn main() -> @location(0) vec4<f32> {
    main_1();
    let _e1 = kh;
    return _e1;
}
