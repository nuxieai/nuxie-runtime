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

@id(7) override mh: bool = true;

@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;
var<private> G5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> H1_1: f32;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Qg: vec4<f32>;
var<private> K3_1: f32;
var<private> A1_1: u32;
@group(0) @binding(12)
var UD: texture_2d<f32>;

fn main_1() {
    var phi_206_: vec3<f32>;

    let _e18 = G5_1;
    let _e20 = m.td;
    let _e21 = textureSampleBias(JC, V5_, _e18, _e20);
    let _e22 = H1_1;
    let _e23 = (_e21 * _e22);
    let _e24 = _e23.xyz;
    let _e26 = gl_FragCoord_1;
    let _e28 = m.B3_;
    let _e30 = m.C3_;
    if (mh && (_e23.w != 0f)) {
        phi_206_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e26.x) + (0.00583715f * _e26.y))))) * _e28) + _e30)) + _e24);
    } else {
        phi_206_ = _e24;
    }
    let _e46 = phi_206_;
    let _e52 = vec4<f32>(_e46.x, _e23.y, _e23.z, _e23.w);
    let _e58 = vec4<f32>(_e52.x, _e46.y, _e52.z, _e52.w);
    Qg = vec4<f32>(_e58.x, _e58.y, _e46.z, _e58.w);
    return;
}

@fragment
fn main(@location(0) G5_: vec2<f32>, @location(3) @interpolate(flat, either) H1_: f32, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) K3_: f32, @location(4) @interpolate(flat, either) A1_: u32) -> @location(0) vec4<f32> {
    G5_1 = G5_;
    H1_1 = H1_;
    gl_FragCoord_1 = gl_FragCoord;
    K3_1 = K3_;
    A1_1 = A1_;
    main_1();
    let _e11 = Qg;
    return _e11;
}
