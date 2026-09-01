struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

@id(7) override lh: bool = true;

@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var U5_: sampler;
var<private> F5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> H1_1: f32;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Pg: vec4<f32>;
var<private> K3_1: f32;
var<private> A1_1: u32;
@group(0) @binding(12)
var UD: texture_2d<f32>;

fn main_1() {
    var phi_206_: vec3<f32>;

    let _e18 = F5_1;
    let _e20 = m.sd;
    let _e21 = textureSampleBias(JC, U5_, _e18, _e20);
    let _e22 = H1_1;
    let _e23 = (_e21 * _e22);
    let _e24 = _e23.xyz;
    let _e26 = gl_FragCoord_1;
    let _e28 = m.B3_;
    let _e30 = m.C3_;
    if (lh && (_e23.w != 0f)) {
        phi_206_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e26.x) + (0.00583715f * _e26.y))))) * _e28) + _e30)) + _e24);
    } else {
        phi_206_ = _e24;
    }
    let _e46 = phi_206_;
    let _e52 = vec4<f32>(_e46.x, _e23.y, _e23.z, _e23.w);
    let _e58 = vec4<f32>(_e52.x, _e46.y, _e52.z, _e52.w);
    Pg = vec4<f32>(_e58.x, _e58.y, _e46.z, _e58.w);
    return;
}

@fragment
fn main(@location(0) F5_: vec2<f32>, @location(3) @interpolate(flat, either) H1_: f32, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) K3_: f32, @location(4) @interpolate(flat, either) A1_: u32) -> @location(0) vec4<f32> {
    F5_1 = F5_;
    H1_1 = H1_;
    gl_FragCoord_1 = gl_FragCoord;
    K3_1 = K3_;
    A1_1 = A1_;
    main_1();
    let _e11 = Pg;
    return _e11;
}
