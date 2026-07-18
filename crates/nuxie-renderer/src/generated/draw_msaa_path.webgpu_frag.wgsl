struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

@id(7) override bh: bool = true;
@id(6) override ah: bool = true;
@id(2) override Wg: bool = true;

@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> e2_1: f32;
@group(0) @binding(12)
var SD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> Fg: vec4<f32>;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> U1_1: vec2<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_2808_: vec4<f32>;
    var phi_2805_: f32;
    var phi_2806_: f32;
    var phi_2810_: vec4<f32>;
    var phi_2801_: f32;
    var phi_2811_: vec4<f32>;
    var phi_2809_: vec4<f32>;
    var phi_2807_: vec4<f32>;
    var phi_2812_: f32;
    var phi_3107_: vec4<f32>;
    var phi_3067_: i32;
    var phi_3210_: vec3<f32>;

    let _e50 = f1_1;
    if (_e50.w >= 0f) {
        if Wg {
            phi_2808_ = vec4<f32>(_e50.x, _e50.y, _e50.z, _e50.w);
        } else {
            phi_2808_ = (_e50 * 1f);
        }
        let _e61 = phi_2808_;
        phi_2807_ = _e61;
    } else {
        if (_e50.w > -1f) {
            if (_e50.z > 0f) {
                phi_2805_ = _e50.x;
            } else {
                phi_2805_ = length(_e50.xy);
            }
            let _e69 = phi_2805_;
            let _e70 = clamp(_e69, 0f, 1f);
            let _e71 = abs(_e50.z);
            if (_e71 > 1f) {
                phi_2806_ = ((0.9980469f * _e70) + 0.0009765625f);
            } else {
                phi_2806_ = ((0.001953125f * _e70) + _e71);
            }
            let _e78 = phi_2806_;
            let _e81 = textureSampleLevel(KD, Jb, vec2<f32>(_e78, -(_e50.w)), 0f);
            let _e87 = vec4<f32>(_e81.x, _e81.y, _e81.z, _e81.w);
            if Wg {
                phi_2810_ = _e87;
            } else {
                let _e89 = (_e87.xyz * _e81.w);
                phi_2810_ = vec4<f32>(_e89.x, _e89.y, _e89.z, _e81.w);
            }
            let _e95 = phi_2810_;
            phi_2809_ = _e95;
        } else {
            let _e98 = textureSampleLevel(IC, S5_, _e50.xy, (-2f - _e50.w));
            if Wg {
                if (_e98.w != 0f) {
                    phi_2801_ = (1f / _e98.w);
                } else {
                    phi_2801_ = 0f;
                }
                let _e105 = phi_2801_;
                let _e106 = (_e98.xyz * _e105);
                phi_2811_ = vec4<f32>(_e106.x, _e106.y, _e106.z, (_e98.w * _e50.z));
            } else {
                phi_2811_ = (_e98 * _e50.z);
            }
            let _e114 = phi_2811_;
            phi_2809_ = _e114;
        }
        let _e116 = phi_2809_;
        phi_2807_ = _e116;
    }
    let _e118 = phi_2807_;
    let _e119 = e2_1;
    let _e121 = gl_FragCoord_1;
    let _e125 = textureLoad(SD, vec2<i32>(floor(_e121.xy)), 0i);
    let _e126 = _e118.xyz;
    local_2 = _e126;
    let _e127 = _e125.xyz;
    if (_e125.w != 0f) {
        phi_2812_ = (1f / _e125.w);
    } else {
        phi_2812_ = 0f;
    }
    let _e132 = phi_2812_;
    let _e133 = (_e127 * _e132);
    local = _e133;
    switch bitcast<i32>(u32(_e119)) {
        case 11: {
            let _e135 = local_2;
            local_1 = (_e135 * _e133);
            break;
        }
        case 1: {
            let _e137 = local_2;
            local_1 = ((_e137 + _e133) - (_e137 * _e133));
            break;
        }
        case 2: {
            let _e141 = local_2;
            let _e142 = (_e141 * _e133);
            local_1 = (select(_e142, (((_e141 + _e133) - _e142) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e133 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e149 = local_2;
            local_1 = min(_e149, _e133);
            break;
        }
        case 4: {
            let _e151 = local_2;
            local_1 = max(_e151, _e133);
            break;
        }
        case 5: {
            let _e154 = clamp(_e127, vec3<f32>(0f, 0f, 0f), _e125.www);
            let _e160 = vec4<f32>(_e154.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e166 = vec4<f32>(_e160.x, _e154.y, _e160.z, _e160.w);
            let _e173 = local_2;
            let _e176 = (clamp((vec3<f32>(1f, 1f, 1f) - _e173), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e125.w);
            let _e177 = vec4<f32>(_e166.x, _e166.y, _e154.z, _e166.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e177 / _e176)), sign(_e177), (_e176 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e183 = local_2;
            local_2 = clamp(_e183, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e186 = clamp(_e127, vec3<f32>(0f, 0f, 0f), _e125.www);
            let _e192 = vec4<f32>(_e186.x, _e125.y, _e125.z, _e125.w);
            let _e198 = vec4<f32>(_e192.x, _e186.y, _e192.z, _e192.w);
            phi_3107_ = vec4<f32>(_e198.x, _e198.y, _e186.z, _e198.w);
            if (_e125.w == 0f) {
                phi_3107_ = vec4<f32>(_e186.x, _e186.y, _e186.z, 1f);
            }
            let _e208 = phi_3107_;
            let _e212 = (vec3(_e208.w) - _e208.xyz);
            let _e213 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e212 / (_e213 * _e208.w))), sign(_e212), (_e213 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e221 = local_2;
            let _e222 = (_e221 * _e133);
            local_1 = (select(_e222, (((_e221 + _e133) - _e222) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e221 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3067_ = 0i;
            loop {
                let _e230 = phi_3067_;
                if (_e230 < 3i) {
                    let _e233 = local_2[_e230];
                    if (_e233 <= 0.5f) {
                        let _e236 = local[_e230];
                        local_1[_e230] = (1f - _e236);
                    } else {
                        let _e240 = local[_e230];
                        if (_e240 <= 0.25f) {
                            let _e242 = local[_e230];
                            let _e245 = local[_e230];
                            local_1[_e230] = ((((16f * _e242) - 12f) * _e245) + 3f);
                        } else {
                            let _e249 = local[_e230];
                            local_1[_e230] = (inverseSqrt(_e249) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3067_ = (_e230 + 1i);
                }
            }
            let _e254 = local_2;
            let _e258 = local_1;
            local_1 = (_e133 + ((_e133 * ((_e254 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e258));
            break;
        }
        case 9: {
            let _e261 = local_2;
            local_1 = abs((_e133 - _e261));
            break;
        }
        case 10: {
            let _e264 = local_2;
            local_1 = ((_e264 + _e133) - ((_e264 * 2f) * _e133));
            break;
        }
        case 12: {
            if ah {
                let _e269 = local_2;
                let _e270 = clamp(_e269, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e270;
                let _e285 = (_e270 - vec3(min(min(_e270.x, _e270.y), _e270.z)));
                let _e293 = (_e285 * ((max(max(_e133.x, _e133.y), _e133.z) - min(min(_e133.x, _e133.y), _e133.z)) / max(0.000062f, max(max(_e285.x, _e285.y), _e285.z))));
                let _e294 = dot(_e133, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e297 = (_e293 - vec3(dot(_e293, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e310 = (vec2<f32>(_e294, (1f - _e294)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e297.x, _e297.y), _e297.z)), max(max(_e297.x, _e297.y), _e297.z))));
                local_1 = ((_e297 * min(1f, min(_e310.x, _e310.y))) + vec3(_e294));
            }
            break;
        }
        case 13: {
            if ah {
                let _e318 = local_2;
                let _e319 = clamp(_e318, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e319;
                let _e334 = (_e133 - vec3(min(min(_e133.x, _e133.y), _e133.z)));
                let _e342 = (_e334 * ((max(max(_e319.x, _e319.y), _e319.z) - min(min(_e319.x, _e319.y), _e319.z)) / max(0.000062f, max(max(_e334.x, _e334.y), _e334.z))));
                let _e343 = dot(_e133, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e346 = (_e342 - vec3(dot(_e342, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e359 = (vec2<f32>(_e343, (1f - _e343)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e346.x, _e346.y), _e346.z)), max(max(_e346.x, _e346.y), _e346.z))));
                local_1 = ((_e346 * min(1f, min(_e359.x, _e359.y))) + vec3(_e343));
            }
            break;
        }
        case 14: {
            if ah {
                let _e367 = local_2;
                let _e368 = clamp(_e367, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e368;
                let _e369 = dot(_e133, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e372 = (_e368 - vec3(dot(_e368, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e385 = (vec2<f32>(_e369, (1f - _e369)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e372.x, _e372.y), _e372.z)), max(max(_e372.x, _e372.y), _e372.z))));
                local_1 = ((_e372 * min(1f, min(_e385.x, _e385.y))) + vec3(_e369));
            }
            break;
        }
        case 15: {
            if ah {
                let _e393 = local_2;
                let _e394 = clamp(_e393, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e394;
                let _e395 = dot(_e394, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e398 = (_e133 - vec3(dot(_e133, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e411 = (vec2<f32>(_e395, (1f - _e395)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e398.x, _e398.y), _e398.z)), max(max(_e398.x, _e398.y), _e398.z))));
                local_1 = ((_e398 * min(1f, min(_e411.x, _e411.y))) + vec3(_e395));
            }
            break;
        }
        default: {
        }
    }
    let _e419 = local_1;
    let _e421 = mix(_e126, _e419, vec3(_e125.w));
    let _e427 = vec4<f32>(_e421.x, _e118.y, _e118.z, _e118.w);
    let _e433 = vec4<f32>(_e427.x, _e421.y, _e427.z, _e427.w);
    let _e439 = vec4<f32>(_e433.x, _e433.y, _e421.z, _e433.w);
    let _e442 = (_e439.xyz * _e118.w);
    let _e448 = vec4<f32>(_e442.x, _e439.y, _e439.z, _e439.w);
    let _e454 = vec4<f32>(_e448.x, _e442.y, _e448.z, _e448.w);
    let _e460 = vec4<f32>(_e454.x, _e454.y, _e442.z, _e454.w);
    let _e461 = _e460.xyz;
    let _e462 = gl_FragCoord_1;
    let _e464 = m.y3_;
    let _e466 = m.z3_;
    if bh {
        phi_3210_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e462.x) + (0.00583715f * _e462.y))))) * _e464) + _e466)) + _e461);
    } else {
        phi_3210_ = _e461;
    }
    let _e480 = phi_3210_;
    let _e486 = vec4<f32>(_e480.x, _e460.y, _e460.z, _e460.w);
    let _e492 = vec4<f32>(_e486.x, _e480.y, _e486.z, _e486.w);
    Fg = vec4<f32>(_e492.x, _e492.y, _e480.z, _e492.w);
    return;
}

@fragment
fn main(@location(0) f1_: vec4<f32>, @location(6) @interpolate(flat) e2_: f32, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat) U1_: vec2<f32>) -> @location(0) vec4<f32> {
    f1_1 = f1_;
    e2_1 = e2_;
    gl_FragCoord_1 = gl_FragCoord;
    U1_1 = U1_;
    main_1();
    let _e9 = Fg;
    return _e9;
}
