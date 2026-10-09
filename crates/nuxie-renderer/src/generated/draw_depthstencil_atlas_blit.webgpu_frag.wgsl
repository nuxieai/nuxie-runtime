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

@id(7) override dj: bool = true;
@id(6) override cj: bool = true;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(10)
var HD: texture_2d<f32>;
@group(3) @binding(10)
var Pa: sampler;
var<private> S2_1: vec2<f32>;
var<private> U0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
@group(0) @binding(12)
var KD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Fi: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> f4_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3006_: f32;
    var phi_3021_: vec4<f32>;
    var phi_3020_: vec4<f32>;
    var phi_1134_: bool;
    var phi_1149_: bool;
    var phi_3007_: f32;
    var phi_3016_: vec4<f32>;
    var phi_3023_: vec4<f32>;
    var phi_3024_: vec4<f32>;
    var phi_3025_: f32;
    var phi_3351_: vec4<f32>;
    var phi_3307_: i32;
    var phi_3463_: vec3<f32>;

    let _e58 = S2_1;
    let _e59 = textureSampleLevel(HD, Pa, _e58, 0f);
    let _e62 = P0_1;
    let _e63 = u32(_e62);
    let _e64 = U0_1;
    let _e65 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e68 = (Yi && (_e63 != 0u));
            if (_e65.w >= 0f) {
                phi_3020_ = _e65;
            } else {
                let _e72 = j.L8_;
                let _e74 = j.M8_;
                let _e77 = abs(_e65.z);
                let _e78 = floor(_e77);
                let _e81 = (((_e77 - _e78) * 8f) + 0.0009765625f);
                if (floor(_e81) == 2f) {
                    phi_3006_ = _e65.x;
                } else {
                    phi_3006_ = length(_e65.xy);
                }
                let _e90 = phi_3006_;
                let _e96 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e90, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e65.z < 0f))) + _e81), ((_e78 * _e72) + _e74)), 0f);
                phi_3021_ = _e96;
                if !(_e68) {
                    let _e102 = (_e96.xyz * _e96.w);
                    phi_3021_ = vec4<f32>(_e102.x, _e102.y, _e102.z, (_e96.w * (ceil(_e65.w) * -0.003921569f)));
                }
                let _e109 = phi_3021_;
                phi_3020_ = _e109;
            }
            let _e111 = phi_3020_;
            phi_1134_ = ej;
            if ej {
                phi_1134_ = (_e64.z < 0f);
            }
            let _e115 = phi_1134_;
            if _e115 {
                let _e117 = textureSampleLevel(TB, U4_, _e64.xy, 0f);
                phi_3024_ = _e117;
                break;
            }
            phi_1149_ = ej;
            if ej {
                phi_1149_ = (_e64.z > 0f);
            }
            let _e121 = phi_1149_;
            phi_3023_ = _e111;
            if _e121 {
                let _e125 = textureSampleLevel(TB, U4_, _e64.xy, (_e64.z - 1f));
                phi_3016_ = _e125;
                if _e68 {
                    if (_e125.w != 0f) {
                        phi_3007_ = (1f / _e125.w);
                    } else {
                        phi_3007_ = 0f;
                    }
                    let _e131 = phi_3007_;
                    let _e132 = (_e125.xyz * _e131);
                    phi_3016_ = vec4<f32>(_e132.x, _e132.y, _e132.z, _e125.w);
                }
                let _e138 = phi_3016_;
                phi_3023_ = (_e111 * _e138);
            }
            let _e141 = phi_3023_;
            phi_3024_ = _e141;
            break;
        }
    }
    let _e143 = phi_3024_;
    let _e144 = gl_FragCoord_1;
    let _e148 = textureLoad(KD, vec2<i32>(floor(_e144.xy)), 0i);
    let _e149 = _e143.xyz;
    local_2 = _e149;
    let _e150 = _e148.xyz;
    if (_e148.w != 0f) {
        phi_3025_ = (1f / _e148.w);
    } else {
        phi_3025_ = 0f;
    }
    let _e155 = phi_3025_;
    let _e156 = (_e150 * _e155);
    local = _e156;
    switch bitcast<i32>(_e63) {
        case 11: {
            let _e158 = local_2;
            local_1 = (_e158 * _e156);
            break;
        }
        case 1: {
            let _e160 = local_2;
            local_1 = ((_e160 + _e156) - (_e160 * _e156));
            break;
        }
        case 2: {
            let _e164 = local_2;
            let _e165 = (_e164 * _e156);
            local_1 = (select(_e165, (((_e164 + _e156) - _e165) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e156 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e172 = local_2;
            local_1 = min(_e172, _e156);
            break;
        }
        case 4: {
            let _e174 = local_2;
            local_1 = max(_e174, _e156);
            break;
        }
        case 5: {
            let _e177 = clamp(_e150, vec3<f32>(0f, 0f, 0f), _e148.www);
            let _e183 = vec4<f32>(_e177.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e189 = vec4<f32>(_e183.x, _e177.y, _e183.z, _e183.w);
            let _e196 = local_2;
            let _e199 = (clamp((vec3<f32>(1f, 1f, 1f) - _e196), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e148.w);
            let _e200 = vec4<f32>(_e189.x, _e189.y, _e177.z, _e189.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e200 / _e199)), sign(_e200), (_e199 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e206 = local_2;
            local_2 = clamp(_e206, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e209 = clamp(_e150, vec3<f32>(0f, 0f, 0f), _e148.www);
            let _e215 = vec4<f32>(_e209.x, _e148.y, _e148.z, _e148.w);
            let _e221 = vec4<f32>(_e215.x, _e209.y, _e215.z, _e215.w);
            phi_3351_ = vec4<f32>(_e221.x, _e221.y, _e209.z, _e221.w);
            if (_e148.w == 0f) {
                phi_3351_ = vec4<f32>(_e209.x, _e209.y, _e209.z, 1f);
            }
            let _e231 = phi_3351_;
            let _e235 = (vec3(_e231.w) - _e231.xyz);
            let _e236 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e235 / (_e236 * _e231.w))), sign(_e235), (_e236 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e244 = local_2;
            let _e245 = (_e244 * _e156);
            local_1 = (select(_e245, (((_e244 + _e156) - _e245) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e244 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3307_ = 0i;
            loop {
                let _e253 = phi_3307_;
                if (_e253 < 3i) {
                    let _e256 = local_2[_e253];
                    if (_e256 <= 0.5f) {
                        let _e259 = local[_e253];
                        local_1[_e253] = (1f - _e259);
                    } else {
                        let _e263 = local[_e253];
                        if (_e263 <= 0.25f) {
                            let _e265 = local[_e253];
                            let _e268 = local[_e253];
                            local_1[_e253] = ((((16f * _e265) - 12f) * _e268) + 3f);
                        } else {
                            let _e272 = local[_e253];
                            local_1[_e253] = (inverseSqrt(_e272) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3307_ = (_e253 + 1i);
                }
            }
            let _e277 = local_2;
            let _e281 = local_1;
            local_1 = (_e156 + ((_e156 * ((_e277 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e281));
            break;
        }
        case 9: {
            let _e284 = local_2;
            local_1 = abs((_e156 - _e284));
            break;
        }
        case 10: {
            let _e287 = local_2;
            local_1 = ((_e287 + _e156) - ((_e287 * 2f) * _e156));
            break;
        }
        case 12: {
            if cj {
                let _e292 = local_2;
                let _e293 = clamp(_e292, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e293;
                let _e308 = (_e293 - vec3(min(min(_e293.x, _e293.y), _e293.z)));
                let _e316 = (_e308 * ((max(max(_e156.x, _e156.y), _e156.z) - min(min(_e156.x, _e156.y), _e156.z)) / max(0.000062f, max(max(_e308.x, _e308.y), _e308.z))));
                let _e317 = dot(_e156, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e320 = (_e316 - vec3(dot(_e316, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e333 = (vec2<f32>(_e317, (1f - _e317)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e320.x, _e320.y), _e320.z)), max(max(_e320.x, _e320.y), _e320.z))));
                local_1 = ((_e320 * min(1f, min(_e333.x, _e333.y))) + vec3(_e317));
            }
            break;
        }
        case 13: {
            if cj {
                let _e341 = local_2;
                let _e342 = clamp(_e341, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e342;
                let _e357 = (_e156 - vec3(min(min(_e156.x, _e156.y), _e156.z)));
                let _e365 = (_e357 * ((max(max(_e342.x, _e342.y), _e342.z) - min(min(_e342.x, _e342.y), _e342.z)) / max(0.000062f, max(max(_e357.x, _e357.y), _e357.z))));
                let _e366 = dot(_e156, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e369 = (_e365 - vec3(dot(_e365, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e382 = (vec2<f32>(_e366, (1f - _e366)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e369.x, _e369.y), _e369.z)), max(max(_e369.x, _e369.y), _e369.z))));
                local_1 = ((_e369 * min(1f, min(_e382.x, _e382.y))) + vec3(_e366));
            }
            break;
        }
        case 14: {
            if cj {
                let _e390 = local_2;
                let _e391 = clamp(_e390, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e391;
                let _e392 = dot(_e156, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e395 = (_e391 - vec3(dot(_e391, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e408 = (vec2<f32>(_e392, (1f - _e392)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e395.x, _e395.y), _e395.z)), max(max(_e395.x, _e395.y), _e395.z))));
                local_1 = ((_e395 * min(1f, min(_e408.x, _e408.y))) + vec3(_e392));
            }
            break;
        }
        case 15: {
            if cj {
                let _e416 = local_2;
                let _e417 = clamp(_e416, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e417;
                let _e418 = dot(_e417, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e421 = (_e156 - vec3(dot(_e156, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e434 = (vec2<f32>(_e418, (1f - _e418)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e421.x, _e421.y), _e421.z)), max(max(_e421.x, _e421.y), _e421.z))));
                local_1 = ((_e421 * min(1f, min(_e434.x, _e434.y))) + vec3(_e418));
            }
            break;
        }
        default: {
        }
    }
    let _e442 = local_1;
    let _e446 = (mix(_e149, _e442, vec3(_e148.w)) * _e143.w);
    let _e452 = vec4<f32>(_e446.x, _e143.y, _e143.z, _e143.w);
    let _e458 = vec4<f32>(_e452.x, _e446.y, _e452.z, _e452.w);
    let _e465 = (vec4<f32>(_e458.x, _e458.y, _e446.z, _e458.w) * clamp(_e59.x, 0f, 1f));
    let _e466 = _e465.xyz;
    let _e468 = gl_FragCoord_1;
    let _e470 = j.F3_;
    let _e472 = j.G3_;
    if (dj && (_e465.w != 0f)) {
        phi_3463_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e468.x) + (0.00583715f * _e468.y))))) * _e470) + _e472)) + _e466);
    } else {
        phi_3463_ = _e466;
    }
    let _e488 = phi_3463_;
    let _e494 = vec4<f32>(_e488.x, _e465.y, _e465.z, _e465.w);
    let _e500 = vec4<f32>(_e494.x, _e488.y, _e494.z, _e494.w);
    Fi = vec4<f32>(_e500.x, _e500.y, _e488.z, _e500.w);
    return;
}

@fragment
fn main(@location(1) S2_: vec2<f32>, @location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) f4_: f32) -> @location(0) vec4<f32> {
    S2_1 = S2_;
    U0_1 = U0_;
    P0_1 = P0_;
    O0_1 = O0_;
    gl_FragCoord_1 = gl_FragCoord;
    f4_1 = f4_;
    main_1();
    let _e13 = Fi;
    return _e13;
}
