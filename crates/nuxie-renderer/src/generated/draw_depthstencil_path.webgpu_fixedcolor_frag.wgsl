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
var<private> F1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Jh: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> l1_1: vec2<f32>;

fn main_1() {
    var phi_660_: f32;
    var phi_661_: f32;
    var phi_675_: vec4<f32>;
    var phi_674_: vec4<f32>;
    var phi_468_: bool;
    var phi_662_: f32;
    var phi_671_: vec4<f32>;
    var phi_677_: vec4<f32>;
    var phi_678_: vec3<f32>;

    let _e30 = Q0_1;
    let _e32 = F1_1;
    let _e33 = a1_1;
    let _e35 = (di && (u32(_e30) != 0u));
    if (_e33.w >= 0f) {
        phi_674_ = _e33;
    } else {
        let _e38 = -(_e33.w);
        let _e43 = j.wc;
        let _e46 = j.xc;
        if (_e33.z > 0f) {
            phi_660_ = _e33.x;
        } else {
            phi_660_ = length(_e33.xy);
        }
        let _e54 = phi_660_;
        let _e55 = clamp(_e54, 0f, 1f);
        let _e56 = abs(_e33.z);
        if (_e56 > 1f) {
            phi_661_ = ((0.9980469f * _e55) + 0.0009765625f);
        } else {
            phi_661_ = ((0.001953125f * _e55) + _e56);
        }
        let _e63 = phi_661_;
        let _e65 = textureSampleLevel(FD, ha, vec2<f32>(_e63, ((floor(_e38) * _e43) + _e46)), 0f);
        phi_675_ = _e65;
        if !(_e35) {
            let _e69 = (_e65.xyz * _e65.w);
            phi_675_ = vec4<f32>(_e69.x, _e69.y, _e69.z, (_e65.w * (fract(_e38) * 1.0039216f)));
        }
        let _e76 = phi_675_;
        phi_674_ = _e76;
    }
    let _e78 = phi_674_;
    phi_468_ = ji;
    if ji {
        phi_468_ = (_e32.z > 0f);
    }
    let _e82 = phi_468_;
    phi_677_ = _e78;
    if _e82 {
        let _e86 = textureSampleLevel(IC, f6_, _e32.xy, (_e32.z - 1f));
        phi_671_ = _e86;
        if _e35 {
            if (_e86.w != 0f) {
                phi_662_ = (1f / _e86.w);
            } else {
                phi_662_ = 0f;
            }
            let _e92 = phi_662_;
            let _e93 = (_e86.xyz * _e92);
            phi_671_ = vec4<f32>(_e93.x, _e93.y, _e93.z, _e86.w);
        }
        let _e99 = phi_671_;
        phi_677_ = (_e78 * _e99);
    }
    let _e102 = phi_677_;
    let _e103 = (_e102 * 1f);
    let _e104 = _e103.xyz;
    let _e106 = gl_FragCoord_1;
    let _e108 = j.M3_;
    let _e110 = j.N3_;
    if (ii && (_e103.w != 0f)) {
        phi_678_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e106.x) + (0.00583715f * _e106.y))))) * _e108) + _e110)) + _e104);
    } else {
        phi_678_ = _e104;
    }
    let _e126 = phi_678_;
    let _e132 = vec4<f32>(_e126.x, _e103.y, _e103.z, _e103.w);
    let _e138 = vec4<f32>(_e132.x, _e126.y, _e132.z, _e132.w);
    Jh = vec4<f32>(_e138.x, _e138.y, _e126.z, _e138.w);
    return;
}

@fragment
fn main(@location(9) F1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>) -> @location(0) vec4<f32> {
    F1_1 = F1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    gl_FragCoord_1 = gl_FragCoord;
    l1_1 = l1_;
    main_1();
    let _e11 = Jh;
    return _e11;
}
