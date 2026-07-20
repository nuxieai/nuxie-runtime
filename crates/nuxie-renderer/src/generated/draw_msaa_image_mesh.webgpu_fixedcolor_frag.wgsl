struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

@id(7) override bh: bool = true;

@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> E5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> H1_1: f32;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Fg: vec4<f32>;
var<private> H3_1: f32;
var<private> A1_1: u32;
@group(0) @binding(12)
var SD: texture_2d<f32>;

fn main_1() {
    var phi_191_: vec3<f32>;

    let _e17 = E5_1;
    let _e19 = m.md;
    let _e20 = textureSampleBias(IC, S5_, _e17, _e19);
    let _e21 = H1_1;
    let _e22 = (_e20 * _e21);
    let _e23 = _e22.xyz;
    let _e24 = gl_FragCoord_1;
    let _e26 = m.y3_;
    let _e28 = m.z3_;
    if bh {
        phi_191_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e24.x) + (0.00583715f * _e24.y))))) * _e26) + _e28)) + _e23);
    } else {
        phi_191_ = _e23;
    }
    let _e42 = phi_191_;
    let _e48 = vec4<f32>(_e42.x, _e22.y, _e22.z, _e22.w);
    let _e54 = vec4<f32>(_e48.x, _e42.y, _e48.z, _e48.w);
    Fg = vec4<f32>(_e54.x, _e54.y, _e42.z, _e54.w);
    return;
}

@fragment
fn main(@location(0) E5_: vec2<f32>, @location(3) @interpolate(flat, either) H1_: f32, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) H3_: f32, @location(4) @interpolate(flat, either) A1_: u32) -> @location(0) vec4<f32> {
    E5_1 = E5_;
    H1_1 = H1_;
    gl_FragCoord_1 = gl_FragCoord;
    H3_1 = H3_;
    A1_1 = A1_;
    main_1();
    let _e11 = Fg;
    return _e11;
}
