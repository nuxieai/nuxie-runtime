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
@id(2) override wj: bool = true;
@id(4) override yj: bool = true;

@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(1) @binding(12)
var x4_: sampler;
@group(0) @binding(9)
var ID: texture_2d<f32>;
@group(3) @binding(9)
var Xa: sampler;
var<private> W2_1: vec2<f32>;
var<private> O0_1: vec3<f32>;
var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Yi: vec4<f32>;
@group(3) @binding(8)
var ib: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
var<private> j4_1: f32;

fn main_1() {
    var phi_897_: f32;
    var phi_898_: f32;
    var phi_899_: vec2<f32>;
    var phi_916_: vec4<f32>;
    var phi_915_: vec4<f32>;
    var phi_597_: bool;
    var phi_612_: bool;
    var phi_900_: f32;
    var phi_911_: vec4<f32>;
    var phi_918_: vec4<f32>;
    var phi_919_: vec4<f32>;
    var phi_920_: vec3<f32>;

    let _e39 = W2_1;
    let _e40 = textureSampleLevel(ID, Xa, _e39, 0f);
    let _e43 = S0_1;
    let _e45 = O0_1;
    let _e46 = N0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e49 = (wj && (u32(_e43) != 0u));
            if (_e46.w >= 0f) {
                phi_915_ = _e46;
            } else {
                let _e53 = j.T8_;
                let _e55 = j.U8_;
                let _e58 = abs(_e46.z);
                let _e59 = floor(_e58);
                let _e62 = (((_e58 - _e59) * 8f) + 0.0009765625f);
                let _e64 = ((_e59 * _e53) + _e55);
                let _e65 = floor(_e62);
                let _e66 = _e46.xy;
                switch bitcast<i32>(0u) {
                    default: {
                        if (Dj && (_e65 == 4f)) {
                            let _e80 = normalize(_e66);
                            let _e83 = acos(clamp(_e80.x, -1f, 1f));
                            if (_e80.y >= 0f) {
                                phi_898_ = _e83;
                            } else {
                                phi_898_ = -(_e83);
                            }
                            let _e88 = phi_898_;
                            phi_899_ = vec2<f32>((_e88 * 0.15915494f), _e64);
                            break;
                        } else {
                            if (_e65 == 2f) {
                                phi_897_ = _e46.x;
                            } else {
                                phi_897_ = length(_e66);
                            }
                            let _e74 = phi_897_;
                            phi_899_ = vec2<f32>(((clamp(_e74, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e46.z < 0f))) + _e62), _e64);
                            break;
                        }
                    }
                }
                let _e92 = phi_899_;
                let _e93 = textureSampleLevel(YC, V8_, _e92, 0f);
                phi_916_ = _e93;
                if !(_e49) {
                    let _e99 = (_e93.xyz * _e93.w);
                    phi_916_ = vec4<f32>(_e99.x, _e99.y, _e99.z, (_e93.w * (ceil(_e46.w) * -0.003921569f)));
                }
                let _e106 = phi_916_;
                phi_915_ = _e106;
            }
            let _e108 = phi_915_;
            phi_597_ = yj;
            if yj {
                phi_597_ = (_e45.z < 0f);
            }
            let _e112 = phi_597_;
            if _e112 {
                let _e114 = textureSampleLevel(SB, x4_, _e45.xy, 0f);
                phi_919_ = _e114;
                break;
            }
            phi_612_ = yj;
            if yj {
                phi_612_ = (_e45.z > 0f);
            }
            let _e118 = phi_612_;
            phi_918_ = _e108;
            if _e118 {
                let _e122 = textureSampleLevel(SB, x4_, _e45.xy, (_e45.z - 1f));
                phi_911_ = _e122;
                if _e49 {
                    if (_e122.w != 0f) {
                        phi_900_ = (1f / _e122.w);
                    } else {
                        phi_900_ = 0f;
                    }
                    let _e128 = phi_900_;
                    let _e129 = (_e122.xyz * _e128);
                    phi_911_ = vec4<f32>(_e129.x, _e129.y, _e129.z, _e122.w);
                }
                let _e135 = phi_911_;
                phi_918_ = (_e108 * _e135);
            }
            let _e138 = phi_918_;
            phi_919_ = _e138;
            break;
        }
    }
    let _e140 = phi_919_;
    let _e141 = (_e140 * clamp(_e40.x, 0f, 1f));
    let _e142 = _e141.xyz;
    let _e144 = gl_FragCoord_1;
    let _e146 = j.K3_;
    let _e148 = j.L3_;
    if (Cj && (_e141.w != 0f)) {
        phi_920_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e144.x) + (0.00583715f * _e144.y))))) * _e146) + _e148)) + _e142);
    } else {
        phi_920_ = _e142;
    }
    let _e164 = phi_920_;
    let _e170 = vec4<f32>(_e164.x, _e141.y, _e141.z, _e141.w);
    let _e176 = vec4<f32>(_e170.x, _e164.y, _e170.z, _e170.w);
    Yi = vec4<f32>(_e176.x, _e176.y, _e164.z, _e176.w);
    return;
}

@fragment
fn main(@location(1) W2_: vec2<f32>, @location(9) O0_: vec3<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) j4_: f32) -> @location(0) vec4<f32> {
    W2_1 = W2_;
    O0_1 = O0_;
    S0_1 = S0_;
    N0_1 = N0_;
    gl_FragCoord_1 = gl_FragCoord;
    j4_1 = j4_;
    main_1();
    let _e13 = Yi;
    return _e13;
}
