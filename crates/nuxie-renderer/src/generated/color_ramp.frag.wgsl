struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

var<private> Ri: vec4<f32>;
var<private> z7_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;

fn main_1() {
    let _e3 = z7_1;
    Ri = _e3;
    return;
}

@fragment
fn main(@location(0) z7_: vec4<f32>) -> @location(0) vec4<f32> {
    z7_1 = z7_;
    main_1();
    let _e3 = Ri;
    return _e3;
}
