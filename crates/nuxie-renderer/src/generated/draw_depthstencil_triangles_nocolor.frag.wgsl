struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

var<private> Yi: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: RB;

fn main_1() {
    let _e8 = vec4<f32>(0f, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
    let _e13 = vec4<f32>(_e8.x, 0f, _e8.z, _e8.w);
    let _e18 = vec4<f32>(_e13.x, _e13.y, 0f, _e13.w);
    Yi = vec4<f32>(_e18.x, _e18.y, _e18.z, 0f);
    return;
}

@fragment
fn main() -> @location(0) vec4<f32> {
    main_1();
    let _e1 = Yi;
    return _e1;
}
