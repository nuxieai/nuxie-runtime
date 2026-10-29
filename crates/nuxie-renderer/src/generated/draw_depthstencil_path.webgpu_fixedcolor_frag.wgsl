struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(16) override zj: bool = false;
@id(17) override Aj: bool = false;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

var<private> R0_1: f32;
var<private> M0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var Q8_: sampler;
var<private> N0_1: vec3<f32>;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Qi: vec4<f32>;
@group(3) @binding(8)
var fb: sampler;
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

    let _e48 = R0_1;
    let _e51 = (lj && (u32(_e48) != 0u));
    let _e53 = M0_1[3u];
    if (_e53 >= 0f) {
        let _e128 = M0_1;
        phi_1008_ = _e128;
    } else {
        let _e55 = M0_1;
        let _e57 = j.O8_;
        let _e59 = j.P8_;
        let _e62 = abs(_e55.z);
        let _e63 = floor(_e62);
        let _e66 = (((_e62 - _e63) * 8f) + 0.0009765625f);
        let _e68 = ((_e63 * _e57) + _e59);
        let _e69 = floor(_e66);
        let _e70 = _e55.xy;
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e69 == 4f)) {
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
        let _e97 = textureSampleLevel(YC, Q8_, _e96, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if zj {
                    phi_990_ = true;
                    break;
                }
                if Aj {
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
    phi_497_ = nj;
    if nj {
        let _e132 = N0_1[2u];
        phi_497_ = (_e132 > 0f);
    }
    let _e135 = phi_497_;
    phi_1011_ = _e130;
    if _e135 {
        let _e137 = N0_1[2u];
        let _e139 = N0_1;
        let _e141 = textureSampleLevel(RB, v4_, _e139.xy, (_e137 - 1f));
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
    let _e162 = j.I3_;
    let _e164 = j.J3_;
    if (rj && (_e157.w != 0f)) {
        phi_1012_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e160.x) + (0.00583715f * _e160.y))))) * _e162) + _e164)) + _e158);
    } else {
        phi_1012_ = _e158;
    }
    let _e180 = phi_1012_;
    let _e186 = vec4<f32>(_e180.x, _e157.y, _e157.z, _e157.w);
    let _e192 = vec4<f32>(_e186.x, _e180.y, _e186.z, _e186.w);
    phi_558_ = nj;
    if nj {
        let _e200 = N0_1[2u];
        phi_558_ = (_e200 < 0f);
    }
    let _e203 = phi_558_;
    phi_1043_ = vec4<f32>(_e192.x, _e192.y, _e180.z, _e192.w);
    if _e203 {
        let _e204 = N0_1;
        let _e206 = textureSampleLevel(RB, v4_, _e204.xy, 0f);
        let _e208 = N0_1[2u];
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
    Qi = _e226;
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) R0_: f32, @location(0) M0_: vec4<f32>, @location(2) N0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    R0_1 = R0_;
    M0_1 = M0_;
    N0_1 = N0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Qi;
    return _e9;
}
