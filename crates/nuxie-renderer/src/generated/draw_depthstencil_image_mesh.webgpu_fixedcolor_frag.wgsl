struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

@id(7) override ri: bool = true;

@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;
var<private> V5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> Q1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Sh: vec4<f32>;
var<private> Y3_1: f32;
var<private> H1_1: u32;
@group(0) @binding(12)
var XD: texture_2d<f32>;

fn main_1() {
    var phi_205_: vec3<f32>;

    let _e18 = V5_1;
    let _e20 = j.Vd;
    let _e21 = textureSampleBias(CC, r5_, _e18, _e20);
    let _e22 = Q1_1;
    let _e23 = (_e21 * _e22);
    let _e24 = _e23.xyz;
    let _e26 = gl_FragCoord_1;
    let _e28 = j.L3_;
    let _e30 = j.M3_;
    if (ri && (_e23.w != 0f)) {
        phi_205_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e26.x) + (0.00583715f * _e26.y))))) * _e28) + _e30)) + _e24);
    } else {
        phi_205_ = _e24;
    }
    let _e46 = phi_205_;
    let _e52 = vec4<f32>(_e46.x, _e23.y, _e23.z, _e23.w);
    let _e58 = vec4<f32>(_e52.x, _e46.y, _e52.z, _e52.w);
    Sh = vec4<f32>(_e58.x, _e58.y, _e46.z, _e58.w);
    return;
}

@fragment
fn main(@location(0) V5_: vec2<f32>, @location(3) @interpolate(flat, either) Q1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) Y3_: f32, @location(4) @interpolate(flat, either) H1_: u32) -> @location(0) vec4<f32> {
    V5_1 = V5_;
    Q1_1 = Q1_;
    gl_FragCoord_1 = gl_FragCoord;
    Y3_1 = Y3_;
    H1_1 = H1_;
    main_1();
    let _e11 = Sh;
    return _e11;
}
