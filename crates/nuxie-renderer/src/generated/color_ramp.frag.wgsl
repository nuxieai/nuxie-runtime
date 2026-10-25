struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

var<private> Ni: vec4<f32>;
var<private> z7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;

fn main_1() {
    let _e3 = z7_1;
    Ni = _e3;
    return;
}

@fragment
fn main(@location(0) z7_: vec4<f32>) -> @location(0) vec4<f32> {
    z7_1 = z7_;
    main_1();
    let _e3 = Ni;
    return _e3;
}
