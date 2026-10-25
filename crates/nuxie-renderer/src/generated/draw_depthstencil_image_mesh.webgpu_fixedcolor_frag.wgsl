struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

@id(8) override nj: bool = true;

@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;
var<private> g6_1: vec2<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> Y1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ni: vec4<f32>;
var<private> h4_1: f32;
var<private> N1_1: u32;

fn main_1() {
    var phi_207_: vec3<f32>;

    let _e17 = g6_1;
    let _e19 = j.Je;
    let _e20 = textureSampleBias(UB, W4_, _e17, _e19);
    let _e21 = Y1_1;
    let _e22 = (_e20 * _e21);
    let _e23 = _e22.xyz;
    let _e25 = gl_FragCoord_1;
    let _e27 = j.G3_;
    let _e29 = j.H3_;
    if (nj && (_e22.w != 0f)) {
        phi_207_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e25.x) + (0.00583715f * _e25.y))))) * _e27) + _e29)) + _e23);
    } else {
        phi_207_ = _e23;
    }
    let _e45 = phi_207_;
    let _e51 = vec4<f32>(_e45.x, _e22.y, _e22.z, _e22.w);
    let _e57 = vec4<f32>(_e51.x, _e45.y, _e51.z, _e51.w);
    Ni = vec4<f32>(_e57.x, _e57.y, _e45.z, _e57.w);
    return;
}

@fragment
fn main(@location(0) g6_: vec2<f32>, @location(3) @interpolate(flat, either) Y1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) h4_: f32, @location(4) @interpolate(flat, either) N1_: u32) -> @location(0) vec4<f32> {
    g6_1 = g6_;
    Y1_1 = Y1_;
    gl_FragCoord_1 = gl_FragCoord;
    h4_1 = h4_;
    N1_1 = N1_;
    main_1();
    let _e11 = Ni;
    return _e11;
}
