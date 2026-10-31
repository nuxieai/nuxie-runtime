struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

@id(8) override Cj: bool = true;

@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(1) @binding(12)
var x4_: sampler;
var<private> h6_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> j: RB;
var<private> a2_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Yi: vec4<f32>;
var<private> j4_1: f32;
var<private> Q1_1: u32;

fn main_1() {
    var phi_207_: vec3<f32>;

    let _e17 = h6_1;
    let _e19 = j.We;
    let _e20 = textureSampleBias(SB, x4_, _e17, _e19);
    let _e21 = a2_1;
    let _e22 = (_e20 * _e21);
    let _e23 = _e22.xyz;
    let _e25 = gl_FragCoord_1;
    let _e27 = j.K3_;
    let _e29 = j.L3_;
    if (Cj && (_e22.w != 0f)) {
        phi_207_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e25.x) + (0.00583715f * _e25.y))))) * _e27) + _e29)) + _e23);
    } else {
        phi_207_ = _e23;
    }
    let _e45 = phi_207_;
    let _e51 = vec4<f32>(_e45.x, _e22.y, _e22.z, _e22.w);
    let _e57 = vec4<f32>(_e51.x, _e45.y, _e51.z, _e51.w);
    Yi = vec4<f32>(_e57.x, _e57.y, _e45.z, _e57.w);
    return;
}

@fragment
fn main(@location(0) h6_: vec2<f32>, @location(3) @interpolate(flat, either) a2_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) j4_: f32, @location(4) @interpolate(flat, either) Q1_: u32) -> @location(0) vec4<f32> {
    h6_1 = h6_;
    a2_1 = a2_;
    gl_FragCoord_1 = gl_FragCoord;
    j4_1 = j4_;
    Q1_1 = Q1_;
    main_1();
    let _e11 = Yi;
    return _e11;
}
