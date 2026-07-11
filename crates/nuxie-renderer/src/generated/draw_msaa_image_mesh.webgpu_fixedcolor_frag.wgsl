struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct LC {
    r9_: vec4<f32>,
    c2_: vec2<f32>,
    x4_: f32,
    ki: f32,
    k2_: vec4<f32>,
    D2_: vec2<f32>,
    V0_: u32,
    n2_: u32,
    Z6_: u32,
}

@id(7) override Tg: bool = true;

@group(1) @binding(12) 
var AC: texture_2d<f32>;
@group(1) @binding(14) 
var R5_: sampler;
var<private> U0_1: vec2<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;
@group(0) @binding(2) 
var<uniform> A0_: LC;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> yg: vec4<f32>;
var<private> I3_1: f32;
@group(0) @binding(13) 
var LD: texture_2d<f32>;

fn main_1() {
    var phi_193_: vec3<f32>;

    let _e17 = U0_1;
    let _e19 = k.fd;
    let _e20 = textureSampleBias(AC, R5_, _e17, _e19);
    let _e22 = A0_.x4_;
    let _e23 = (_e20 * _e22);
    let _e24 = _e23.xyz;
    let _e25 = gl_FragCoord_1;
    let _e27 = k.y3_;
    let _e29 = k.z3_;
    if Tg {
        phi_193_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e25.x) + (0.00583715f * _e25.y))))) * _e27) + _e29)) + _e24);
    } else {
        phi_193_ = _e24;
    }
    let _e43 = phi_193_;
    let _e49 = vec4<f32>(_e43.x, _e23.y, _e23.z, _e23.w);
    let _e55 = vec4<f32>(_e49.x, _e43.y, _e49.z, _e49.w);
    yg = vec4<f32>(_e55.x, _e55.y, _e43.z, _e55.w);
    return;
}

@fragment 
fn main(@location(0) U0_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat) I3_: f32) -> @location(0) vec4<f32> {
    U0_1 = U0_;
    gl_FragCoord_1 = gl_FragCoord;
    I3_1 = I3_;
    main_1();
    let _e7 = yg;
    return _e7;
}
