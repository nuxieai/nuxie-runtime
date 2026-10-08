struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

@id(7) override fj: bool = true;

@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
var<private> Z5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> U1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ii: vec4<f32>;
var<private> e4_1: f32;
var<private> K1_1: u32;
@group(0) @binding(12)
var KD: texture_2d<f32>;

fn main_1() {
    var phi_207_: vec3<f32>;

    let _e18 = Z5_1;
    let _e20 = j.Ee;
    let _e21 = textureSampleBias(TB, S4_, _e18, _e20);
    let _e22 = U1_1;
    let _e23 = (_e21 * _e22);
    let _e24 = _e23.xyz;
    let _e26 = gl_FragCoord_1;
    let _e28 = j.E3_;
    let _e30 = j.F3_;
    if (fj && (_e23.w != 0f)) {
        phi_207_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e26.x) + (0.00583715f * _e26.y))))) * _e28) + _e30)) + _e24);
    } else {
        phi_207_ = _e24;
    }
    let _e46 = phi_207_;
    let _e52 = vec4<f32>(_e46.x, _e23.y, _e23.z, _e23.w);
    let _e58 = vec4<f32>(_e52.x, _e46.y, _e52.z, _e52.w);
    Ii = vec4<f32>(_e58.x, _e58.y, _e46.z, _e58.w);
    return;
}

@fragment
fn main(@location(0) Z5_: vec2<f32>, @location(3) @interpolate(flat, either) U1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) e4_: f32, @location(4) @interpolate(flat, either) K1_: u32) -> @location(0) vec4<f32> {
    Z5_1 = Z5_;
    U1_1 = U1_;
    gl_FragCoord_1 = gl_FragCoord;
    e4_1 = e4_;
    K1_1 = K1_;
    main_1();
    let _e11 = Ii;
    return _e11;
}
