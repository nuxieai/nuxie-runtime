struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

var<private> Ii: vec4<f32>;
var<private> p7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;

fn main_1() {
    let _e3 = p7_1;
    Ii = _e3;
    return;
}

@fragment
fn main(@location(0) p7_: vec4<f32>) -> @location(0) vec4<f32> {
    p7_1 = p7_;
    main_1();
    let _e3 = Ii;
    return _e3;
}
