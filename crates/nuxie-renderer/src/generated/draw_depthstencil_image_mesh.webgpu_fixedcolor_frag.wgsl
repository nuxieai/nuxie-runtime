struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

@id(7) override ii: bool = true;

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var f6_: sampler;
var<private> T5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> P1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Jh: vec4<f32>;
var<private> Z3_1: f32;
var<private> H1_1: u32;
@group(0) @binding(12)
var YD: texture_2d<f32>;

fn main_1() {
    var phi_205_: vec3<f32>;

    let _e18 = T5_1;
    let _e20 = j.Vd;
    let _e21 = textureSampleBias(IC, f6_, _e18, _e20);
    let _e22 = P1_1;
    let _e23 = (_e21 * _e22);
    let _e24 = _e23.xyz;
    let _e26 = gl_FragCoord_1;
    let _e28 = j.M3_;
    let _e30 = j.N3_;
    if (ii && (_e23.w != 0f)) {
        phi_205_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e26.x) + (0.00583715f * _e26.y))))) * _e28) + _e30)) + _e24);
    } else {
        phi_205_ = _e24;
    }
    let _e46 = phi_205_;
    let _e52 = vec4<f32>(_e46.x, _e23.y, _e23.z, _e23.w);
    let _e58 = vec4<f32>(_e52.x, _e46.y, _e52.z, _e52.w);
    Jh = vec4<f32>(_e58.x, _e58.y, _e46.z, _e58.w);
    return;
}

@fragment
fn main(@location(0) T5_: vec2<f32>, @location(3) @interpolate(flat, either) P1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) Z3_: f32, @location(4) @interpolate(flat, either) H1_: u32) -> @location(0) vec4<f32> {
    T5_1 = T5_;
    P1_1 = P1_;
    gl_FragCoord_1 = gl_FragCoord;
    Z3_1 = Z3_;
    H1_1 = H1_;
    main_1();
    let _e11 = Jh;
    return _e11;
}
