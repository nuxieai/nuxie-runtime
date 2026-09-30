struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

@id(7) override Kh: bool = true;

@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;
var<private> G5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> H1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> kh: vec4<f32>;
var<private> L3_1: f32;
var<private> A1_1: u32;
@group(0) @binding(12)
var YD: texture_2d<f32>;

fn main_1() {
    var phi_206_: vec3<f32>;

    let _e18 = G5_1;
    let _e20 = l.Ed;
    let _e21 = textureSampleBias(HC, V5_, _e18, _e20);
    let _e22 = H1_1;
    let _e23 = (_e21 * _e22);
    let _e24 = _e23.xyz;
    let _e26 = gl_FragCoord_1;
    let _e28 = l.C3_;
    let _e30 = l.D3_;
    if (Kh && (_e23.w != 0f)) {
        phi_206_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e26.x) + (0.00583715f * _e26.y))))) * _e28) + _e30)) + _e24);
    } else {
        phi_206_ = _e24;
    }
    let _e46 = phi_206_;
    let _e52 = vec4<f32>(_e46.x, _e23.y, _e23.z, _e23.w);
    let _e58 = vec4<f32>(_e52.x, _e46.y, _e52.z, _e52.w);
    kh = vec4<f32>(_e58.x, _e58.y, _e46.z, _e58.w);
    return;
}

@fragment
fn main(@location(0) G5_: vec2<f32>, @location(3) @interpolate(flat, either) H1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) L3_: f32, @location(4) @interpolate(flat, either) A1_: u32) -> @location(0) vec4<f32> {
    G5_1 = G5_;
    H1_1 = H1_;
    gl_FragCoord_1 = gl_FragCoord;
    L3_1 = L3_;
    A1_1 = A1_;
    main_1();
    let _e11 = kh;
    return _e11;
}
