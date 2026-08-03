struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

var<private> Hg: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;

fn main_1() {
    let _e8 = vec4<f32>(0f, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
    let _e13 = vec4<f32>(_e8.x, 0f, _e8.z, _e8.w);
    let _e18 = vec4<f32>(_e13.x, _e13.y, 0f, _e13.w);
    Hg = vec4<f32>(_e18.x, _e18.y, _e18.z, 0f);
    return;
}

@fragment
fn main() -> @location(0) vec4<f32> {
    main_1();
    let _e1 = Hg;
    return _e1;
}
