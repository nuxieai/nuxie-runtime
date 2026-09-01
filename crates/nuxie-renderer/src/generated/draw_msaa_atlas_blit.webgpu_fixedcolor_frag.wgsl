struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

@id(7) override lh: bool = true;
@id(2) override gh: bool = true;
@id(8) override mh: bool = true;

@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Ob: sampler;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var U5_: sampler;
@group(0) @binding(10)
var CD: texture_2d<f32>;
@group(3) @binding(10)
var Q9_: sampler;
var<private> D2_1: vec2<f32>;
var<private> f1_1: vec4<f32>;
var<private> A2_1: vec3<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> Pg: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> K3_1: f32;
var<private> e2_1: f32;

fn main_1() {
    var phi_616_: vec4<f32>;
    var phi_600_: f32;
    var phi_601_: f32;
    var phi_617_: vec4<f32>;
    var phi_615_: vec4<f32>;
    var phi_461_: bool;
    var phi_602_: f32;
    var phi_612_: vec4<f32>;
    var phi_619_: vec4<f32>;
    var phi_620_: vec3<f32>;

    let _e29 = D2_1;
    let _e30 = textureSampleLevel(CD, Q9_, _e29, 0f);
    let _e32 = clamp(_e30.x, 0f, 1f);
    let _e33 = f1_1;
    let _e34 = A2_1;
    if (_e33.w >= 0f) {
        if gh {
            phi_616_ = vec4<f32>(_e33.x, _e33.y, _e33.z, (_e33.w * _e32));
        } else {
            phi_616_ = (_e33 * _e32);
        }
        let _e46 = phi_616_;
        phi_615_ = _e46;
    } else {
        if (_e33.z > 0f) {
            phi_600_ = _e33.x;
        } else {
            phi_600_ = length(_e33.xy);
        }
        let _e53 = phi_600_;
        let _e54 = clamp(_e53, 0f, 1f);
        let _e55 = abs(_e33.z);
        if (_e55 > 1f) {
            phi_601_ = ((0.9980469f * _e54) + 0.0009765625f);
        } else {
            phi_601_ = ((0.001953125f * _e54) + _e55);
        }
        let _e62 = phi_601_;
        let _e65 = textureSampleLevel(MD, Ob, vec2<f32>(_e62, -(_e33.w)), 0f);
        let _e67 = (_e65.w * _e32);
        let _e72 = vec4<f32>(_e65.x, _e65.y, _e65.z, _e67);
        if gh {
            phi_617_ = _e72;
        } else {
            let _e74 = (_e72.xyz * _e67);
            phi_617_ = vec4<f32>(_e74.x, _e74.y, _e74.z, _e67);
        }
        let _e80 = phi_617_;
        phi_615_ = _e80;
    }
    let _e82 = phi_615_;
    phi_461_ = mh;
    if mh {
        phi_461_ = (_e34.z > 0f);
    }
    let _e86 = phi_461_;
    phi_619_ = _e82;
    if _e86 {
        let _e90 = textureSampleLevel(JC, U5_, _e34.xy, (_e34.z - 1f));
        phi_612_ = _e90;
        if gh {
            if (_e90.w != 0f) {
                phi_602_ = (1f / _e90.w);
            } else {
                phi_602_ = 0f;
            }
            let _e96 = phi_602_;
            let _e97 = (_e90.xyz * _e96);
            phi_612_ = vec4<f32>(_e97.x, _e97.y, _e97.z, _e90.w);
        }
        let _e103 = phi_612_;
        phi_619_ = (_e82 * _e103);
    }
    let _e106 = phi_619_;
    let _e107 = _e106.xyz;
    let _e109 = gl_FragCoord_1;
    let _e111 = m.B3_;
    let _e113 = m.C3_;
    if (lh && (_e106.w != 0f)) {
        phi_620_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e109.x) + (0.00583715f * _e109.y))))) * _e111) + _e113)) + _e107);
    } else {
        phi_620_ = _e107;
    }
    let _e129 = phi_620_;
    let _e135 = vec4<f32>(_e129.x, _e106.y, _e106.z, _e106.w);
    let _e141 = vec4<f32>(_e135.x, _e129.y, _e135.z, _e135.w);
    Pg = vec4<f32>(_e141.x, _e141.y, _e129.z, _e141.w);
    return;
}

@fragment
fn main(@location(1) D2_: vec2<f32>, @location(0) f1_: vec4<f32>, @location(9) A2_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) K3_: f32, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    D2_1 = D2_;
    f1_1 = f1_;
    A2_1 = A2_;
    gl_FragCoord_1 = gl_FragCoord;
    K3_1 = K3_;
    e2_1 = e2_;
    main_1();
    let _e13 = Pg;
    return _e13;
}
