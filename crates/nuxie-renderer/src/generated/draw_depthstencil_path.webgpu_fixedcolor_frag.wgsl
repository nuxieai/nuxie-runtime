struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

@id(8) override Cj: bool = true;
@id(9) override Dj: bool = true;
@id(16) override Kj: bool = false;
@id(17) override Lj: bool = false;
@id(2) override wj: bool = true;
@id(4) override yj: bool = true;

var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
var<private> O0_1: vec3<f32>;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(1) @binding(12)
var x4_: sampler;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Yi: vec4<f32>;
@group(3) @binding(8)
var ib: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;

fn main_1() {
    var phi_987_: f32;
    var phi_988_: f32;
    var phi_989_: vec2<f32>;
    var phi_990_: bool;
    var phi_992_: vec4<f32>;
    var phi_1009_: vec4<f32>;
    var phi_1008_: vec4<f32>;
    var phi_497_: bool;
    var phi_993_: f32;
    var phi_1005_: vec4<f32>;
    var phi_1011_: vec4<f32>;
    var phi_1012_: vec3<f32>;
    var phi_558_: bool;
    var phi_1028_: f32;
    var phi_1043_: vec4<f32>;

    let _e48 = S0_1;
    let _e51 = (wj && (u32(_e48) != 0u));
    let _e53 = N0_1[3u];
    if (_e53 >= 0f) {
        let _e128 = N0_1;
        phi_1008_ = _e128;
    } else {
        let _e55 = N0_1;
        let _e57 = j.T8_;
        let _e59 = j.U8_;
        let _e62 = abs(_e55.z);
        let _e63 = floor(_e62);
        let _e66 = (((_e62 - _e63) * 8f) + 0.0009765625f);
        let _e68 = ((_e63 * _e57) + _e59);
        let _e69 = floor(_e66);
        let _e70 = _e55.xy;
        switch bitcast<i32>(0u) {
            default: {
                if (Dj && (_e69 == 4f)) {
                    let _e84 = normalize(_e70);
                    let _e87 = acos(clamp(_e84.x, -1f, 1f));
                    if (_e84.y >= 0f) {
                        phi_988_ = _e87;
                    } else {
                        phi_988_ = -(_e87);
                    }
                    let _e92 = phi_988_;
                    phi_989_ = vec2<f32>((_e92 * 0.15915494f), _e68);
                    break;
                } else {
                    if (_e69 == 2f) {
                        phi_987_ = _e55.x;
                    } else {
                        phi_987_ = length(_e70);
                    }
                    let _e78 = phi_987_;
                    phi_989_ = vec2<f32>(((clamp(_e78, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e55.z < 0f))) + _e66), _e68);
                    break;
                }
            }
        }
        let _e96 = phi_989_;
        let _e97 = textureSampleLevel(YC, V8_, _e96, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if Kj {
                    phi_990_ = true;
                    break;
                }
                if Lj {
                    phi_990_ = true;
                    break;
                }
                phi_990_ = false;
                break;
            }
        }
        let _e100 = phi_990_;
        phi_992_ = _e97;
        if _e100 {
            phi_992_ = vec4<f32>(_e97.x, _e97.y, _e97.z, (_e97.w * ((fract(_e55.w) * -2f) + 1.5f)));
        }
        let _e113 = phi_992_;
        phi_1009_ = _e113;
        if !(_e51) {
            let _e117 = (_e113.xyz * _e113.w);
            phi_1009_ = vec4<f32>(_e117.x, _e117.y, _e117.z, (_e113.w * (ceil(_e55.w) * -0.003921569f)));
        }
        let _e127 = phi_1009_;
        phi_1008_ = _e127;
    }
    let _e130 = phi_1008_;
    phi_497_ = yj;
    if yj {
        let _e132 = O0_1[2u];
        phi_497_ = (_e132 > 0f);
    }
    let _e135 = phi_497_;
    phi_1011_ = _e130;
    if _e135 {
        let _e137 = O0_1[2u];
        let _e139 = O0_1;
        let _e141 = textureSampleLevel(SB, x4_, _e139.xy, (_e137 - 1f));
        phi_1005_ = _e141;
        if _e51 {
            if (_e141.w != 0f) {
                phi_993_ = (1f / _e141.w);
            } else {
                phi_993_ = 0f;
            }
            let _e147 = phi_993_;
            let _e148 = (_e141.xyz * _e147);
            phi_1005_ = vec4<f32>(_e148.x, _e148.y, _e148.z, _e141.w);
        }
        let _e154 = phi_1005_;
        phi_1011_ = (_e130 * _e154);
    }
    let _e157 = phi_1011_;
    let _e158 = _e157.xyz;
    let _e160 = gl_FragCoord_1;
    let _e162 = j.K3_;
    let _e164 = j.L3_;
    if (Cj && (_e157.w != 0f)) {
        phi_1012_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e160.x) + (0.00583715f * _e160.y))))) * _e162) + _e164)) + _e158);
    } else {
        phi_1012_ = _e158;
    }
    let _e180 = phi_1012_;
    let _e186 = vec4<f32>(_e180.x, _e157.y, _e157.z, _e157.w);
    let _e192 = vec4<f32>(_e186.x, _e180.y, _e186.z, _e186.w);
    phi_558_ = yj;
    if yj {
        let _e200 = O0_1[2u];
        phi_558_ = (_e200 < 0f);
    }
    let _e203 = phi_558_;
    phi_1043_ = vec4<f32>(_e192.x, _e192.y, _e180.z, _e192.w);
    if _e203 {
        let _e204 = O0_1;
        let _e206 = textureSampleLevel(SB, x4_, _e204.xy, 0f);
        let _e208 = O0_1[2u];
        let _e210 = u32((-1f - _e208));
        switch bitcast<i32>(0u) {
            default: {
                let _e213 = dot(_e206.xyz, vec3<f32>(0.3f, 0.59f, 0.11f));
                if (_e210 == 0u) {
                    phi_1028_ = _e206.w;
                    break;
                }
                if (_e210 == 1u) {
                    phi_1028_ = (1f - _e206.w);
                    break;
                }
                if (_e210 == 2u) {
                    phi_1028_ = _e213;
                    break;
                }
                phi_1028_ = (1f - _e213);
                break;
            }
        }
        let _e222 = phi_1028_;
        phi_1043_ = vec4(mix(1f, _e222, _e53));
    }
    let _e226 = phi_1043_;
    Yi = _e226;
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @location(2) O0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    S0_1 = S0_;
    N0_1 = N0_;
    O0_1 = O0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Yi;
    return _e9;
}
