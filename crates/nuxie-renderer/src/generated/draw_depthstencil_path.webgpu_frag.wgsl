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
@id(6) override hi: bool = true;
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
@group(0) @binding(12)
var YD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Jh: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> l1_1: vec2<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2868_: f32;
    var phi_2869_: f32;
    var phi_2883_: vec4<f32>;
    var phi_2882_: vec4<f32>;
    var phi_1061_: bool;
    var phi_2870_: f32;
    var phi_2879_: vec4<f32>;
    var phi_2885_: vec4<f32>;
    var phi_2886_: f32;
    var phi_3181_: vec4<f32>;
    var phi_3141_: i32;
    var phi_3284_: vec3<f32>;

    let _e54 = Q0_1;
    let _e55 = u32(_e54);
    let _e56 = F1_1;
    let _e57 = a1_1;
    let _e59 = (di && (_e55 != 0u));
    if (_e57.w >= 0f) {
        phi_2882_ = _e57;
    } else {
        let _e62 = -(_e57.w);
        let _e67 = j.wc;
        let _e70 = j.xc;
        if (_e57.z > 0f) {
            phi_2868_ = _e57.x;
        } else {
            phi_2868_ = length(_e57.xy);
        }
        let _e78 = phi_2868_;
        let _e79 = clamp(_e78, 0f, 1f);
        let _e80 = abs(_e57.z);
        if (_e80 > 1f) {
            phi_2869_ = ((0.9980469f * _e79) + 0.0009765625f);
        } else {
            phi_2869_ = ((0.001953125f * _e79) + _e80);
        }
        let _e87 = phi_2869_;
        let _e89 = textureSampleLevel(FD, ha, vec2<f32>(_e87, ((floor(_e62) * _e67) + _e70)), 0f);
        phi_2883_ = _e89;
        if !(_e59) {
            let _e93 = (_e89.xyz * _e89.w);
            phi_2883_ = vec4<f32>(_e93.x, _e93.y, _e93.z, (_e89.w * (fract(_e62) * 1.0039216f)));
        }
        let _e100 = phi_2883_;
        phi_2882_ = _e100;
    }
    let _e102 = phi_2882_;
    phi_1061_ = ji;
    if ji {
        phi_1061_ = (_e56.z > 0f);
    }
    let _e106 = phi_1061_;
    phi_2885_ = _e102;
    if _e106 {
        let _e110 = textureSampleLevel(IC, f6_, _e56.xy, (_e56.z - 1f));
        phi_2879_ = _e110;
        if _e59 {
            if (_e110.w != 0f) {
                phi_2870_ = (1f / _e110.w);
            } else {
                phi_2870_ = 0f;
            }
            let _e116 = phi_2870_;
            let _e117 = (_e110.xyz * _e116);
            phi_2879_ = vec4<f32>(_e117.x, _e117.y, _e117.z, _e110.w);
        }
        let _e123 = phi_2879_;
        phi_2885_ = (_e102 * _e123);
    }
    let _e126 = phi_2885_;
    let _e127 = gl_FragCoord_1;
    let _e131 = textureLoad(YD, vec2<i32>(floor(_e127.xy)), 0i);
    let _e132 = _e126.xyz;
    local_2 = _e132;
    let _e133 = _e131.xyz;
    if (_e131.w != 0f) {
        phi_2886_ = (1f / _e131.w);
    } else {
        phi_2886_ = 0f;
    }
    let _e138 = phi_2886_;
    let _e139 = (_e133 * _e138);
    local = _e139;
    switch bitcast<i32>(_e55) {
        case 11: {
            let _e141 = local_2;
            local_1 = (_e141 * _e139);
            break;
        }
        case 1: {
            let _e143 = local_2;
            local_1 = ((_e143 + _e139) - (_e143 * _e139));
            break;
        }
        case 2: {
            let _e147 = local_2;
            let _e148 = (_e147 * _e139);
            local_1 = (select(_e148, (((_e147 + _e139) - _e148) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e139 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e155 = local_2;
            local_1 = min(_e155, _e139);
            break;
        }
        case 4: {
            let _e157 = local_2;
            local_1 = max(_e157, _e139);
            break;
        }
        case 5: {
            let _e160 = clamp(_e133, vec3<f32>(0f, 0f, 0f), _e131.www);
            let _e166 = vec4<f32>(_e160.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e172 = vec4<f32>(_e166.x, _e160.y, _e166.z, _e166.w);
            let _e179 = local_2;
            let _e182 = (clamp((vec3<f32>(1f, 1f, 1f) - _e179), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e131.w);
            let _e183 = vec4<f32>(_e172.x, _e172.y, _e160.z, _e172.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e183 / _e182)), sign(_e183), (_e182 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e189 = local_2;
            local_2 = clamp(_e189, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e192 = clamp(_e133, vec3<f32>(0f, 0f, 0f), _e131.www);
            let _e198 = vec4<f32>(_e192.x, _e131.y, _e131.z, _e131.w);
            let _e204 = vec4<f32>(_e198.x, _e192.y, _e198.z, _e198.w);
            phi_3181_ = vec4<f32>(_e204.x, _e204.y, _e192.z, _e204.w);
            if (_e131.w == 0f) {
                phi_3181_ = vec4<f32>(_e192.x, _e192.y, _e192.z, 1f);
            }
            let _e214 = phi_3181_;
            let _e218 = (vec3(_e214.w) - _e214.xyz);
            let _e219 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e218 / (_e219 * _e214.w))), sign(_e218), (_e219 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e227 = local_2;
            let _e228 = (_e227 * _e139);
            local_1 = (select(_e228, (((_e227 + _e139) - _e228) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e227 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3141_ = 0i;
            loop {
                let _e236 = phi_3141_;
                if (_e236 < 3i) {
                    let _e239 = local_2[_e236];
                    if (_e239 <= 0.5f) {
                        let _e242 = local[_e236];
                        local_1[_e236] = (1f - _e242);
                    } else {
                        let _e246 = local[_e236];
                        if (_e246 <= 0.25f) {
                            let _e248 = local[_e236];
                            let _e251 = local[_e236];
                            local_1[_e236] = ((((16f * _e248) - 12f) * _e251) + 3f);
                        } else {
                            let _e255 = local[_e236];
                            local_1[_e236] = (inverseSqrt(_e255) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3141_ = (_e236 + 1i);
                }
            }
            let _e260 = local_2;
            let _e264 = local_1;
            local_1 = (_e139 + ((_e139 * ((_e260 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e264));
            break;
        }
        case 9: {
            let _e267 = local_2;
            local_1 = abs((_e139 - _e267));
            break;
        }
        case 10: {
            let _e270 = local_2;
            local_1 = ((_e270 + _e139) - ((_e270 * 2f) * _e139));
            break;
        }
        case 12: {
            if hi {
                let _e275 = local_2;
                let _e276 = clamp(_e275, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e276;
                let _e291 = (_e276 - vec3(min(min(_e276.x, _e276.y), _e276.z)));
                let _e299 = (_e291 * ((max(max(_e139.x, _e139.y), _e139.z) - min(min(_e139.x, _e139.y), _e139.z)) / max(0.000062f, max(max(_e291.x, _e291.y), _e291.z))));
                let _e300 = dot(_e139, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e303 = (_e299 - vec3(dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e316 = (vec2<f32>(_e300, (1f - _e300)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e303.x, _e303.y), _e303.z)), max(max(_e303.x, _e303.y), _e303.z))));
                local_1 = ((_e303 * min(1f, min(_e316.x, _e316.y))) + vec3(_e300));
            }
            break;
        }
        case 13: {
            if hi {
                let _e324 = local_2;
                let _e325 = clamp(_e324, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e325;
                let _e340 = (_e139 - vec3(min(min(_e139.x, _e139.y), _e139.z)));
                let _e348 = (_e340 * ((max(max(_e325.x, _e325.y), _e325.z) - min(min(_e325.x, _e325.y), _e325.z)) / max(0.000062f, max(max(_e340.x, _e340.y), _e340.z))));
                let _e349 = dot(_e139, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e352 = (_e348 - vec3(dot(_e348, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e365 = (vec2<f32>(_e349, (1f - _e349)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e352.x, _e352.y), _e352.z)), max(max(_e352.x, _e352.y), _e352.z))));
                local_1 = ((_e352 * min(1f, min(_e365.x, _e365.y))) + vec3(_e349));
            }
            break;
        }
        case 14: {
            if hi {
                let _e373 = local_2;
                let _e374 = clamp(_e373, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e374;
                let _e375 = dot(_e139, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e378 = (_e374 - vec3(dot(_e374, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e391 = (vec2<f32>(_e375, (1f - _e375)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e378.x, _e378.y), _e378.z)), max(max(_e378.x, _e378.y), _e378.z))));
                local_1 = ((_e378 * min(1f, min(_e391.x, _e391.y))) + vec3(_e375));
            }
            break;
        }
        case 15: {
            if hi {
                let _e399 = local_2;
                let _e400 = clamp(_e399, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e400;
                let _e401 = dot(_e400, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e404 = (_e139 - vec3(dot(_e139, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e417 = (vec2<f32>(_e401, (1f - _e401)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e404.x, _e404.y), _e404.z)), max(max(_e404.x, _e404.y), _e404.z))));
                local_1 = ((_e404 * min(1f, min(_e417.x, _e417.y))) + vec3(_e401));
            }
            break;
        }
        default: {
        }
    }
    let _e425 = local_1;
    let _e429 = (mix(_e132, _e425, vec3(_e131.w)) * _e126.w);
    let _e435 = vec4<f32>(_e429.x, _e126.y, _e126.z, _e126.w);
    let _e441 = vec4<f32>(_e435.x, _e429.y, _e435.z, _e435.w);
    let _e448 = (vec4<f32>(_e441.x, _e441.y, _e429.z, _e441.w) * 1f);
    let _e449 = _e448.xyz;
    let _e451 = gl_FragCoord_1;
    let _e453 = j.M3_;
    let _e455 = j.N3_;
    if (ii && (_e448.w != 0f)) {
        phi_3284_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e451.x) + (0.00583715f * _e451.y))))) * _e453) + _e455)) + _e449);
    } else {
        phi_3284_ = _e449;
    }
    let _e471 = phi_3284_;
    let _e477 = vec4<f32>(_e471.x, _e448.y, _e448.z, _e448.w);
    let _e483 = vec4<f32>(_e477.x, _e471.y, _e477.z, _e477.w);
    Jh = vec4<f32>(_e483.x, _e483.y, _e471.z, _e483.w);
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
