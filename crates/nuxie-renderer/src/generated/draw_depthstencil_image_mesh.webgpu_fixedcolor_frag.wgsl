struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
}

@id(8) override sj: bool = true;

@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;
var<private> e6_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> Y1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ri: vec4<f32>;
var<private> h4_1: f32;
var<private> O1_1: u32;

fn main_1() {
    var phi_207_: vec3<f32>;

    let _e17 = e6_1;
    let _e19 = j.Se;
    let _e20 = textureSampleBias(RB, v4_, _e17, _e19);
    let _e21 = Y1_1;
    let _e22 = (_e20 * _e21);
    let _e23 = _e22.xyz;
    let _e25 = gl_FragCoord_1;
    let _e27 = j.I3_;
    let _e29 = j.J3_;
    if (sj && (_e22.w != 0f)) {
        phi_207_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e25.x) + (0.00583715f * _e25.y))))) * _e27) + _e29)) + _e23);
    } else {
        phi_207_ = _e23;
    }
    let _e45 = phi_207_;
    let _e51 = vec4<f32>(_e45.x, _e22.y, _e22.z, _e22.w);
    let _e57 = vec4<f32>(_e51.x, _e45.y, _e51.z, _e51.w);
    Ri = vec4<f32>(_e57.x, _e57.y, _e45.z, _e57.w);
    return;
}

@fragment
fn main(@location(0) e6_: vec2<f32>, @location(3) @interpolate(flat, either) Y1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) h4_: f32, @location(4) @interpolate(flat, either) O1_: u32) -> @location(0) vec4<f32> {
    e6_1 = e6_;
    Y1_1 = Y1_;
    gl_FragCoord_1 = gl_FragCoord;
    h4_1 = h4_;
    O1_1 = O1_;
    main_1();
    let _e11 = Ri;
    return _e11;
}
