struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

@id(7) override mh: bool = true;
@id(6) override lh: bool = true;
@id(2) override hh: bool = true;
@id(8) override nh: bool = true;

@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Pb: sampler;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;
@group(0) @binding(10)
var CD: texture_2d<f32>;
@group(3) @binding(10)
var Q9_: sampler;
var<private> D2_1: vec2<f32>;
var<private> f1_1: vec4<f32>;
var<private> A2_1: vec3<f32>;
var<private> f2_1: f32;
@group(0) @binding(12)
var UD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> Qg: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> K3_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2841_: vec4<f32>;
    var phi_2825_: f32;
    var phi_2826_: f32;
    var phi_2842_: vec4<f32>;
    var phi_2840_: vec4<f32>;
    var phi_1065_: bool;
    var phi_2827_: f32;
    var phi_2837_: vec4<f32>;
    var phi_2844_: vec4<f32>;
    var phi_2845_: f32;
    var phi_3172_: vec4<f32>;
    var phi_3128_: i32;
    var phi_3284_: vec3<f32>;

    let _e53 = D2_1;
    let _e54 = textureSampleLevel(CD, Q9_, _e53, 0f);
    let _e56 = clamp(_e54.x, 0f, 1f);
    let _e57 = f1_1;
    let _e58 = A2_1;
    if (_e57.w >= 0f) {
        if hh {
            phi_2841_ = vec4<f32>(_e57.x, _e57.y, _e57.z, (_e57.w * _e56));
        } else {
            phi_2841_ = (_e57 * _e56);
        }
        let _e70 = phi_2841_;
        phi_2840_ = _e70;
    } else {
        if (_e57.z > 0f) {
            phi_2825_ = _e57.x;
        } else {
            phi_2825_ = length(_e57.xy);
        }
        let _e77 = phi_2825_;
        let _e78 = clamp(_e77, 0f, 1f);
        let _e79 = abs(_e57.z);
        if (_e79 > 1f) {
            phi_2826_ = ((0.9980469f * _e78) + 0.0009765625f);
        } else {
            phi_2826_ = ((0.001953125f * _e78) + _e79);
        }
        let _e86 = phi_2826_;
        let _e89 = textureSampleLevel(MD, Pb, vec2<f32>(_e86, -(_e57.w)), 0f);
        let _e91 = (_e89.w * _e56);
        let _e96 = vec4<f32>(_e89.x, _e89.y, _e89.z, _e91);
        if hh {
            phi_2842_ = _e96;
        } else {
            let _e98 = (_e96.xyz * _e91);
            phi_2842_ = vec4<f32>(_e98.x, _e98.y, _e98.z, _e91);
        }
        let _e104 = phi_2842_;
        phi_2840_ = _e104;
    }
    let _e106 = phi_2840_;
    phi_1065_ = nh;
    if nh {
        phi_1065_ = (_e58.z > 0f);
    }
    let _e110 = phi_1065_;
    phi_2844_ = _e106;
    if _e110 {
        let _e114 = textureSampleLevel(JC, V5_, _e58.xy, (_e58.z - 1f));
        phi_2837_ = _e114;
        if hh {
            if (_e114.w != 0f) {
                phi_2827_ = (1f / _e114.w);
            } else {
                phi_2827_ = 0f;
            }
            let _e120 = phi_2827_;
            let _e121 = (_e114.xyz * _e120);
            phi_2837_ = vec4<f32>(_e121.x, _e121.y, _e121.z, _e114.w);
        }
        let _e127 = phi_2837_;
        phi_2844_ = (_e106 * _e127);
    }
    let _e130 = phi_2844_;
    let _e131 = f2_1;
    let _e133 = gl_FragCoord_1;
    let _e137 = textureLoad(UD, vec2<i32>(floor(_e133.xy)), 0i);
    let _e138 = _e130.xyz;
    local_2 = _e138;
    let _e139 = _e137.xyz;
    if (_e137.w != 0f) {
        phi_2845_ = (1f / _e137.w);
    } else {
        phi_2845_ = 0f;
    }
    let _e144 = phi_2845_;
    let _e145 = (_e139 * _e144);
    local = _e145;
    switch bitcast<i32>(u32(_e131)) {
        case 11: {
            let _e147 = local_2;
            local_1 = (_e147 * _e145);
            break;
        }
        case 1: {
            let _e149 = local_2;
            local_1 = ((_e149 + _e145) - (_e149 * _e145));
            break;
        }
        case 2: {
            let _e153 = local_2;
            let _e154 = (_e153 * _e145);
            local_1 = (select(_e154, (((_e153 + _e145) - _e154) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e145 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e161 = local_2;
            local_1 = min(_e161, _e145);
            break;
        }
        case 4: {
            let _e163 = local_2;
            local_1 = max(_e163, _e145);
            break;
        }
        case 5: {
            let _e166 = clamp(_e139, vec3<f32>(0f, 0f, 0f), _e137.www);
            let _e172 = vec4<f32>(_e166.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e178 = vec4<f32>(_e172.x, _e166.y, _e172.z, _e172.w);
            let _e185 = local_2;
            let _e188 = (clamp((vec3<f32>(1f, 1f, 1f) - _e185), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e137.w);
            let _e189 = vec4<f32>(_e178.x, _e178.y, _e166.z, _e178.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e189 / _e188)), sign(_e189), (_e188 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e195 = local_2;
            local_2 = clamp(_e195, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e198 = clamp(_e139, vec3<f32>(0f, 0f, 0f), _e137.www);
            let _e204 = vec4<f32>(_e198.x, _e137.y, _e137.z, _e137.w);
            let _e210 = vec4<f32>(_e204.x, _e198.y, _e204.z, _e204.w);
            phi_3172_ = vec4<f32>(_e210.x, _e210.y, _e198.z, _e210.w);
            if (_e137.w == 0f) {
                phi_3172_ = vec4<f32>(_e198.x, _e198.y, _e198.z, 1f);
            }
            let _e220 = phi_3172_;
            let _e224 = (vec3(_e220.w) - _e220.xyz);
            let _e225 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e224 / (_e225 * _e220.w))), sign(_e224), (_e225 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e233 = local_2;
            let _e234 = (_e233 * _e145);
            local_1 = (select(_e234, (((_e233 + _e145) - _e234) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e233 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3128_ = 0i;
            loop {
                let _e242 = phi_3128_;
                if (_e242 < 3i) {
                    let _e245 = local_2[_e242];
                    if (_e245 <= 0.5f) {
                        let _e248 = local[_e242];
                        local_1[_e242] = (1f - _e248);
                    } else {
                        let _e252 = local[_e242];
                        if (_e252 <= 0.25f) {
                            let _e254 = local[_e242];
                            let _e257 = local[_e242];
                            local_1[_e242] = ((((16f * _e254) - 12f) * _e257) + 3f);
                        } else {
                            let _e261 = local[_e242];
                            local_1[_e242] = (inverseSqrt(_e261) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3128_ = (_e242 + 1i);
                }
            }
            let _e266 = local_2;
            let _e270 = local_1;
            local_1 = (_e145 + ((_e145 * ((_e266 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e270));
            break;
        }
        case 9: {
            let _e273 = local_2;
            local_1 = abs((_e145 - _e273));
            break;
        }
        case 10: {
            let _e276 = local_2;
            local_1 = ((_e276 + _e145) - ((_e276 * 2f) * _e145));
            break;
        }
        case 12: {
            if lh {
                let _e281 = local_2;
                let _e282 = clamp(_e281, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e282;
                let _e297 = (_e282 - vec3(min(min(_e282.x, _e282.y), _e282.z)));
                let _e305 = (_e297 * ((max(max(_e145.x, _e145.y), _e145.z) - min(min(_e145.x, _e145.y), _e145.z)) / max(0.000062f, max(max(_e297.x, _e297.y), _e297.z))));
                let _e306 = dot(_e145, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e309 = (_e305 - vec3(dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e322 = (vec2<f32>(_e306, (1f - _e306)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e309.x, _e309.y), _e309.z)), max(max(_e309.x, _e309.y), _e309.z))));
                local_1 = ((_e309 * min(1f, min(_e322.x, _e322.y))) + vec3(_e306));
            }
            break;
        }
        case 13: {
            if lh {
                let _e330 = local_2;
                let _e331 = clamp(_e330, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e331;
                let _e346 = (_e145 - vec3(min(min(_e145.x, _e145.y), _e145.z)));
                let _e354 = (_e346 * ((max(max(_e331.x, _e331.y), _e331.z) - min(min(_e331.x, _e331.y), _e331.z)) / max(0.000062f, max(max(_e346.x, _e346.y), _e346.z))));
                let _e355 = dot(_e145, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e358 = (_e354 - vec3(dot(_e354, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e371 = (vec2<f32>(_e355, (1f - _e355)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e358.x, _e358.y), _e358.z)), max(max(_e358.x, _e358.y), _e358.z))));
                local_1 = ((_e358 * min(1f, min(_e371.x, _e371.y))) + vec3(_e355));
            }
            break;
        }
        case 14: {
            if lh {
                let _e379 = local_2;
                let _e380 = clamp(_e379, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e380;
                let _e381 = dot(_e145, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e384 = (_e380 - vec3(dot(_e380, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e397 = (vec2<f32>(_e381, (1f - _e381)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e384.x, _e384.y), _e384.z)), max(max(_e384.x, _e384.y), _e384.z))));
                local_1 = ((_e384 * min(1f, min(_e397.x, _e397.y))) + vec3(_e381));
            }
            break;
        }
        case 15: {
            if lh {
                let _e405 = local_2;
                let _e406 = clamp(_e405, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e406;
                let _e407 = dot(_e406, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e410 = (_e145 - vec3(dot(_e145, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e423 = (vec2<f32>(_e407, (1f - _e407)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e410.x, _e410.y), _e410.z)), max(max(_e410.x, _e410.y), _e410.z))));
                local_1 = ((_e410 * min(1f, min(_e423.x, _e423.y))) + vec3(_e407));
            }
            break;
        }
        default: {
        }
    }
    let _e431 = local_1;
    let _e433 = mix(_e138, _e431, vec3(_e137.w));
    let _e439 = vec4<f32>(_e433.x, _e130.y, _e130.z, _e130.w);
    let _e445 = vec4<f32>(_e439.x, _e433.y, _e439.z, _e439.w);
    let _e451 = vec4<f32>(_e445.x, _e445.y, _e433.z, _e445.w);
    let _e454 = (_e451.xyz * _e130.w);
    let _e460 = vec4<f32>(_e454.x, _e451.y, _e451.z, _e451.w);
    let _e466 = vec4<f32>(_e460.x, _e454.y, _e460.z, _e460.w);
    let _e472 = vec4<f32>(_e466.x, _e466.y, _e454.z, _e466.w);
    let _e473 = _e472.xyz;
    let _e474 = gl_FragCoord_1;
    let _e476 = m.B3_;
    let _e478 = m.C3_;
    if (mh && (_e130.w != 0f)) {
        phi_3284_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e474.x) + (0.00583715f * _e474.y))))) * _e476) + _e478)) + _e473);
    } else {
        phi_3284_ = _e473;
    }
    let _e494 = phi_3284_;
    let _e500 = vec4<f32>(_e494.x, _e472.y, _e472.z, _e472.w);
    let _e506 = vec4<f32>(_e500.x, _e494.y, _e500.z, _e500.w);
    Qg = vec4<f32>(_e506.x, _e506.y, _e494.z, _e506.w);
    return;
}

@fragment
fn main(@location(1) D2_: vec2<f32>, @location(0) f1_: vec4<f32>, @location(9) A2_: vec3<f32>, @location(6) @interpolate(flat, either) f2_: f32, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) K3_: f32) -> @location(0) vec4<f32> {
    D2_1 = D2_;
    f1_1 = f1_;
    A2_1 = A2_;
    f2_1 = f2_;
    gl_FragCoord_1 = gl_FragCoord;
    K3_1 = K3_;
    main_1();
    let _e13 = Qg;
    return _e13;
}
