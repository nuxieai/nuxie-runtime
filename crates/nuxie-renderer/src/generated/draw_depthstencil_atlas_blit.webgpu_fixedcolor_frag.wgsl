struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

@id(7) override ii: bool = true;
@id(2) override di: bool = true;
@id(8) override ji: bool = true;

@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var f6_: sampler;
@group(0) @binding(10)
var GD: texture_2d<f32>;
@group(3) @binding(10)
var ma: sampler;
var<private> K2_1: vec2<f32>;
var<private> F1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Jh: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> Z3_1: f32;

fn main_1() {
    var phi_686_: f32;
    var phi_687_: f32;
    var phi_701_: vec4<f32>;
    var phi_700_: vec4<f32>;
    var phi_494_: bool;
    var phi_688_: f32;
    var phi_697_: vec4<f32>;
    var phi_703_: vec4<f32>;
    var phi_704_: vec3<f32>;

    let _e33 = K2_1;
    let _e34 = textureSampleLevel(GD, ma, _e33, 0f);
    let _e37 = Q0_1;
    let _e39 = F1_1;
    let _e40 = a1_1;
    let _e42 = (di && (u32(_e37) != 0u));
    if (_e40.w >= 0f) {
        phi_700_ = _e40;
    } else {
        let _e45 = -(_e40.w);
        let _e50 = j.wc;
        let _e53 = j.xc;
        if (_e40.z > 0f) {
            phi_686_ = _e40.x;
        } else {
            phi_686_ = length(_e40.xy);
        }
        let _e61 = phi_686_;
        let _e62 = clamp(_e61, 0f, 1f);
        let _e63 = abs(_e40.z);
        if (_e63 > 1f) {
            phi_687_ = ((0.9980469f * _e62) + 0.0009765625f);
        } else {
            phi_687_ = ((0.001953125f * _e62) + _e63);
        }
        let _e70 = phi_687_;
        let _e72 = textureSampleLevel(FD, ha, vec2<f32>(_e70, ((floor(_e45) * _e50) + _e53)), 0f);
        phi_701_ = _e72;
        if !(_e42) {
            let _e76 = (_e72.xyz * _e72.w);
            phi_701_ = vec4<f32>(_e76.x, _e76.y, _e76.z, (_e72.w * (fract(_e45) * 1.0039216f)));
        }
        let _e83 = phi_701_;
        phi_700_ = _e83;
    }
    let _e85 = phi_700_;
    phi_494_ = ji;
    if ji {
        phi_494_ = (_e39.z > 0f);
    }
    let _e89 = phi_494_;
    phi_703_ = _e85;
    if _e89 {
        let _e93 = textureSampleLevel(IC, f6_, _e39.xy, (_e39.z - 1f));
        phi_697_ = _e93;
        if _e42 {
            if (_e93.w != 0f) {
                phi_688_ = (1f / _e93.w);
            } else {
                phi_688_ = 0f;
            }
            let _e99 = phi_688_;
            let _e100 = (_e93.xyz * _e99);
            phi_697_ = vec4<f32>(_e100.x, _e100.y, _e100.z, _e93.w);
        }
        let _e106 = phi_697_;
        phi_703_ = (_e85 * _e106);
    }
    let _e109 = phi_703_;
    let _e110 = (_e109 * clamp(_e34.x, 0f, 1f));
    let _e111 = _e110.xyz;
    let _e113 = gl_FragCoord_1;
    let _e115 = j.M3_;
    let _e117 = j.N3_;
    if (ii && (_e110.w != 0f)) {
        phi_704_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e113.x) + (0.00583715f * _e113.y))))) * _e115) + _e117)) + _e111);
    } else {
        phi_704_ = _e111;
    }
    let _e133 = phi_704_;
    let _e139 = vec4<f32>(_e133.x, _e110.y, _e110.z, _e110.w);
    let _e145 = vec4<f32>(_e139.x, _e133.y, _e139.z, _e139.w);
    Jh = vec4<f32>(_e145.x, _e145.y, _e133.z, _e145.w);
    return;
}

@fragment
fn main(@location(1) K2_: vec2<f32>, @location(9) F1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) Z3_: f32) -> @location(0) vec4<f32> {
    K2_1 = K2_;
    F1_1 = F1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    gl_FragCoord_1 = gl_FragCoord;
    Z3_1 = Z3_;
    main_1();
    let _e13 = Jh;
    return _e13;
}
