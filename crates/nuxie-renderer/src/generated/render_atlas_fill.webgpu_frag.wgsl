struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;
var<private> Fi: f32;
var<private> S_1: vec4<f32>;
var<private> gl_FrontFacing_1: bool;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    var phi_419_: f32;
    var phi_423_: f32;
    var phi_424_: f32;

    let _e25 = S_1;
    let _e26 = gl_FrontFacing_1;
    let _e29 = max(_e25.w, 0f);
    if (_e25.z >= 0f) {
        let _e32 = textureSampleLevel(YC, ab, vec2<f32>(_e29, 0f), 0f);
        phi_419_ = _e32.x;
    } else {
        phi_419_ = 0f;
    }
    let _e35 = phi_419_;
    phi_423_ = _e35;
    if (abs(_e25.z) < 1000f) {
        let _e42 = (-2f - _e25.y);
        let _e44 = ((_e42 - _e29) * 0.5984134f);
        let _e47 = (vec4(_e29) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e44));
        let _e53 = ((_e47 * -(_e25.z)) + vec4(((_e42 * _e25.z) + (abs(_e25.x) - 0.25f))));
        let _e56 = textureSampleLevel(YC, ab, vec2<f32>(_e53.x, 0f), 0f);
        let _e59 = textureSampleLevel(YC, ab, vec2<f32>(_e53.y, 0f), 0f);
        let _e62 = textureSampleLevel(YC, ab, vec2<f32>(_e53.z, 0f), 0f);
        let _e65 = textureSampleLevel(YC, ab, vec2<f32>(_e53.w, 0f), 0f);
        let _e71 = (_e47 * 5.0959306f);
        phi_423_ = (_e35 + (dot(vec4<f32>(_e56.x, _e59.x, _e62.x, _e65.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e71) * (_e71 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e44));
    }
    let _e80 = phi_423_;
    let _e83 = (_e80 * sign(_e25.x));
    phi_424_ = _e83;
    if !(_e26) {
        phi_424_ = -(_e83);
    }
    let _e87 = phi_424_;
    Fi = _e87;
    return;
}

@fragment
fn main(@location(0) S: vec4<f32>, @builtin(front_facing) gl_FrontFacing: bool) -> @location(0) f32 {
    S_1 = S;
    gl_FrontFacing_1 = gl_FrontFacing;
    main_1();
    let _e5 = Fi;
    return _e5;
}
