struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

@id(7) override Oh: bool = true;

@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
var<private> J5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> K1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> oh: vec4<f32>;
var<private> O3_1: f32;
var<private> D1_1: u32;
@group(0) @binding(12)
var XD: texture_2d<f32>;

fn main_1() {
    var phi_206_: vec3<f32>;

    let _e18 = J5_1;
    let _e20 = j.Ed;
    let _e21 = textureSampleBias(GC, W5_, _e18, _e20);
    let _e22 = K1_1;
    let _e23 = (_e21 * _e22);
    let _e24 = _e23.xyz;
    let _e26 = gl_FragCoord_1;
    let _e28 = j.F3_;
    let _e30 = j.G3_;
    if (Oh && (_e23.w != 0f)) {
        phi_206_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e26.x) + (0.00583715f * _e26.y))))) * _e28) + _e30)) + _e24);
    } else {
        phi_206_ = _e24;
    }
    let _e46 = phi_206_;
    let _e52 = vec4<f32>(_e46.x, _e23.y, _e23.z, _e23.w);
    let _e58 = vec4<f32>(_e52.x, _e46.y, _e52.z, _e52.w);
    oh = vec4<f32>(_e58.x, _e58.y, _e46.z, _e58.w);
    return;
}

@fragment
fn main(@location(0) J5_: vec2<f32>, @location(3) @interpolate(flat, either) K1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) O3_: f32, @location(4) @interpolate(flat, either) D1_: u32) -> @location(0) vec4<f32> {
    J5_1 = J5_;
    K1_1 = K1_;
    gl_FragCoord_1 = gl_FragCoord;
    O3_1 = O3_;
    D1_1 = D1_;
    main_1();
    let _e11 = oh;
    return _e11;
}
