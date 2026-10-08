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
@group(0) @binding(0)
var<uniform> j: VB;

fn main_1() {
    let _e8 = vec4<f32>(0f, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
    let _e13 = vec4<f32>(_e8.x, 0f, _e8.z, _e8.w);
    let _e18 = vec4<f32>(_e13.x, _e13.y, 0f, _e13.w);
    Ii = vec4<f32>(_e18.x, _e18.y, _e18.z, 0f);
    return;
}

@fragment
fn main() -> @location(0) vec4<f32> {
    main_1();
    let _e1 = Ii;
    return _e1;
}
