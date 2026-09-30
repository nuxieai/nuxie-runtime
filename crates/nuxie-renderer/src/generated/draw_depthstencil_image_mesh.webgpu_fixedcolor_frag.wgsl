struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

@id(7) override Lh: bool = true;

@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;
var<private> K5_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> J1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> lh: vec4<f32>;
var<private> O3_1: f32;
var<private> C1_1: u32;
@group(0) @binding(12)
var XD: texture_2d<f32>;

fn main_1() {
    var phi_206_: vec3<f32>;

    let _e18 = K5_1;
    let _e20 = j.Fd;
    let _e21 = textureSampleBias(GC, Y5_, _e18, _e20);
    let _e22 = J1_1;
    let _e23 = (_e21 * _e22);
    let _e24 = _e23.xyz;
    let _e26 = gl_FragCoord_1;
    let _e28 = j.F3_;
    let _e30 = j.G3_;
    if (Lh && (_e23.w != 0f)) {
        phi_206_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e26.x) + (0.00583715f * _e26.y))))) * _e28) + _e30)) + _e24);
    } else {
        phi_206_ = _e24;
    }
    let _e46 = phi_206_;
    let _e52 = vec4<f32>(_e46.x, _e23.y, _e23.z, _e23.w);
    let _e58 = vec4<f32>(_e52.x, _e46.y, _e52.z, _e52.w);
    lh = vec4<f32>(_e58.x, _e58.y, _e46.z, _e58.w);
    return;
}

@fragment
fn main(@location(0) K5_: vec2<f32>, @location(3) @interpolate(flat, either) J1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) O3_: f32, @location(4) @interpolate(flat, either) C1_: u32) -> @location(0) vec4<f32> {
    K5_1 = K5_;
    J1_1 = J1_;
    gl_FragCoord_1 = gl_FragCoord;
    O3_1 = O3_;
    C1_1 = C1_;
    main_1();
    let _e11 = lh;
    return _e11;
}
