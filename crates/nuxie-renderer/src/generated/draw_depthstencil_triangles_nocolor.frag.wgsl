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
@group(0) @binding(0)
var<uniform> j: TB;

fn main_1() {
    let _e8 = vec4<f32>(0f, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
    let _e13 = vec4<f32>(_e8.x, 0f, _e8.z, _e8.w);
    let _e18 = vec4<f32>(_e13.x, _e13.y, 0f, _e13.w);
    oh = vec4<f32>(_e18.x, _e18.y, _e18.z, 0f);
    return;
}

@fragment
fn main() -> @location(0) vec4<f32> {
    main_1();
    let _e1 = oh;
    return _e1;
}
