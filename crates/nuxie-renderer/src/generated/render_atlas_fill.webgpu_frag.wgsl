struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
var<private> Qg: f32;
var<private> O_1: vec4<f32>;
var<private> gl_FrontFacing_1: bool;
@group(0) @binding(0)
var<uniform> m: DC;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(3) @binding(8)
var Pb: sampler;
@group(1) @binding(13)
var V5_: sampler;

fn main_1() {
    var phi_419_: f32;
    var phi_423_: f32;
    var phi_424_: f32;

    let _e25 = O_1;
    let _e26 = gl_FrontFacing_1;
    let _e29 = max(_e25.w, 0f);
    if (_e25.z >= 0f) {
        let _e32 = textureSampleLevel(YC, aa, vec2<f32>(_e29, 0f), 0f);
        phi_419_ = _e32.x;
    } else {
        phi_419_ = 0f;
    }
    let _e35 = phi_419_;
    phi_423_ = _e35;
    if (abs(_e25.z) < 1000f) {
        let _e42 = (-2f - _e25.y);
        let _e44 = ((_e42 - _e29) * 0.5984134f);
        let _e47 = (vec4(_e29) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e44));
        let _e53 = ((_e47 * -(_e25.z)) + vec4(((_e42 * _e25.z) + (abs(_e25.x) - 0.25f))));
        let _e56 = textureSampleLevel(YC, aa, vec2<f32>(_e53.x, 0f), 0f);
        let _e59 = textureSampleLevel(YC, aa, vec2<f32>(_e53.y, 0f), 0f);
        let _e62 = textureSampleLevel(YC, aa, vec2<f32>(_e53.z, 0f), 0f);
        let _e65 = textureSampleLevel(YC, aa, vec2<f32>(_e53.w, 0f), 0f);
        let _e71 = (_e47 * 5.0959306f);
        phi_423_ = (_e35 + (dot(vec4<f32>(_e56.x, _e59.x, _e62.x, _e65.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e71) * (_e71 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e44));
    }
    let _e80 = phi_423_;
    let _e83 = (_e80 * sign(_e25.x));
    phi_424_ = _e83;
    if !(_e26) {
        phi_424_ = -(_e83);
    }
    let _e87 = phi_424_;
    Qg = _e87;
    return;
}

@fragment
fn main(@location(0) O: vec4<f32>, @builtin(front_facing) gl_FrontFacing: bool) -> @location(0) f32 {
    O_1 = O;
    gl_FrontFacing_1 = gl_FrontFacing;
    main_1();
    let _e5 = Qg;
    return _e5;
}
