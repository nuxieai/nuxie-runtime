struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Fg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Bg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Gg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    yg: u32,
}

@id(7) override fh: bool = true;

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> E5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> H1_1: f32;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Jg: vec4<f32>;
var<private> I3_1: f32;
var<private> A1_1: u32;
@group(0) @binding(12)
var SD: texture_2d<f32>;

fn main_1() {
    var phi_204_: vec3<f32>;

    let _e18 = E5_1;
    let _e20 = n.qd;
    let _e21 = textureSampleBias(IC, S5_, _e18, _e20);
    let _e22 = H1_1;
    let _e23 = (_e21 * _e22);
    let _e24 = _e23.xyz;
    let _e26 = gl_FragCoord_1;
    let _e28 = n.z3_;
    let _e30 = n.A3_;
    if (fh && (_e23.w != 0f)) {
        phi_204_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e26.x) + (0.00583715f * _e26.y))))) * _e28) + _e30)) + _e24);
    } else {
        phi_204_ = _e24;
    }
    let _e46 = phi_204_;
    let _e52 = vec4<f32>(_e46.x, _e23.y, _e23.z, _e23.w);
    let _e58 = vec4<f32>(_e52.x, _e46.y, _e52.z, _e52.w);
    Jg = vec4<f32>(_e58.x, _e58.y, _e46.z, _e58.w);
    return;
}

@fragment
fn main(@location(0) E5_: vec2<f32>, @location(3) @interpolate(flat, either) H1_: f32, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) I3_: f32, @location(4) @interpolate(flat, either) A1_: u32) -> @location(0) vec4<f32> {
    E5_1 = E5_;
    H1_1 = H1_;
    gl_FragCoord_1 = gl_FragCoord;
    I3_1 = I3_;
    A1_1 = A1_;
    main_1();
    let _e11 = Jg;
    return _e11;
}
