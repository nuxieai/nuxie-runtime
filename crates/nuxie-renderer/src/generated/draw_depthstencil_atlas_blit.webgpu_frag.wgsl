struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Ci: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Gi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    G3_: f32,
    H3_: f32,
    He: f32,
    zi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

@id(7) override hj: bool = true;
@id(6) override gj: bool = true;
@id(2) override cj: bool = true;
@id(8) override ij: bool = true;

@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(10)
var JD: texture_2d<f32>;
@group(3) @binding(10)
var Sa: sampler;
var<private> T2_1: vec2<f32>;
var<private> U0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
@group(0) @binding(12)
var MD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ji: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> f4_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3106_: f32;
    var phi_3107_: f32;
    var phi_3108_: vec2<f32>;
    var phi_3125_: vec4<f32>;
    var phi_3124_: vec4<f32>;
    var phi_1185_: bool;
    var phi_1200_: bool;
    var phi_3109_: f32;
    var phi_3120_: vec4<f32>;
    var phi_3127_: vec4<f32>;
    var phi_3128_: vec4<f32>;
    var phi_3129_: f32;
    var phi_3467_: i32;
    var phi_3519_: vec4<f32>;
    var phi_3649_: vec3<f32>;

    let _e61 = T2_1;
    let _e62 = textureSampleLevel(JD, Sa, _e61, 0f);
    let _e65 = Q0_1;
    let _e66 = u32(_e65);
    let _e67 = U0_1;
    let _e68 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e71 = (cj && (_e66 != 0u));
            if (_e68.w >= 0f) {
                phi_3124_ = _e68;
            } else {
                let _e75 = j.O8_;
                let _e77 = j.P8_;
                let _e80 = abs(_e68.z);
                let _e81 = floor(_e80);
                let _e84 = (((_e80 - _e81) * 8f) + 0.0009765625f);
                let _e86 = ((_e81 * _e75) + _e77);
                let _e87 = floor(_e84);
                switch bitcast<i32>(0u) {
                    default: {
                        if (_e87 == 4f) {
                            let _e105 = normalize(vec2<f32>(_e68.x, -(_e68.y)));
                            let _e108 = acos(clamp(_e105.x, -1f, 1f));
                            if (_e105.y >= 0f) {
                                phi_3107_ = _e108;
                            } else {
                                phi_3107_ = -(_e108);
                            }
                            let _e113 = phi_3107_;
                            phi_3108_ = vec2<f32>((_e113 * 0.15915507f), _e86);
                            break;
                        } else {
                            if (_e87 == 2f) {
                                phi_3106_ = _e68.x;
                            } else {
                                phi_3106_ = length(_e68.xy);
                            }
                            let _e95 = phi_3106_;
                            phi_3108_ = vec2<f32>(((clamp(_e95, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e68.z < 0f))) + _e84), _e86);
                            break;
                        }
                    }
                }
                let _e117 = phi_3108_;
                let _e118 = textureSampleLevel(ZC, Q8_, _e117, 0f);
                phi_3125_ = _e118;
                if !(_e71) {
                    let _e124 = (_e118.xyz * _e118.w);
                    phi_3125_ = vec4<f32>(_e124.x, _e124.y, _e124.z, (_e118.w * (ceil(_e68.w) * -0.003921569f)));
                }
                let _e131 = phi_3125_;
                phi_3124_ = _e131;
            }
            let _e133 = phi_3124_;
            phi_1185_ = ij;
            if ij {
                phi_1185_ = (_e67.z < 0f);
            }
            let _e137 = phi_1185_;
            if _e137 {
                let _e139 = textureSampleLevel(TB, U4_, _e67.xy, 0f);
                phi_3128_ = _e139;
                break;
            }
            phi_1200_ = ij;
            if ij {
                phi_1200_ = (_e67.z > 0f);
            }
            let _e143 = phi_1200_;
            phi_3127_ = _e133;
            if _e143 {
                let _e147 = textureSampleLevel(TB, U4_, _e67.xy, (_e67.z - 1f));
                phi_3120_ = _e147;
                if _e71 {
                    if (_e147.w != 0f) {
                        phi_3109_ = (1f / _e147.w);
                    } else {
                        phi_3109_ = 0f;
                    }
                    let _e153 = phi_3109_;
                    let _e154 = (_e147.xyz * _e153);
                    phi_3120_ = vec4<f32>(_e154.x, _e154.y, _e154.z, _e147.w);
                }
                let _e160 = phi_3120_;
                phi_3127_ = (_e133 * _e160);
            }
            let _e163 = phi_3127_;
            phi_3128_ = _e163;
            break;
        }
    }
    let _e165 = phi_3128_;
    let _e166 = gl_FragCoord_1;
    let _e170 = textureLoad(MD, vec2<i32>(floor(_e166.xy)), 0i);
    let _e171 = _e165.xyz;
    local_2 = _e171;
    let _e172 = _e170.xyz;
    if (_e170.w != 0f) {
        phi_3129_ = (1f / _e170.w);
    } else {
        phi_3129_ = 0f;
    }
    let _e177 = phi_3129_;
    let _e178 = (_e172 * _e177);
    local = _e178;
    switch bitcast<i32>(_e66) {
        case 11: {
            let _e462 = local_2;
            local_1 = (_e462 * _e178);
            break;
        }
        case 1: {
            let _e458 = local_2;
            local_1 = ((_e458 + _e178) - (_e458 * _e178));
            break;
        }
        case 2: {
            let _e450 = local_2;
            let _e451 = (_e450 * _e178);
            local_1 = (select(_e451, (((_e450 + _e178) - _e451) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e178 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e448 = local_2;
            local_1 = min(_e448, _e178);
            break;
        }
        case 4: {
            let _e446 = local_2;
            local_1 = max(_e446, _e178);
            break;
        }
        case 5: {
            let _e417 = clamp(_e172, vec3<f32>(0f, 0f, 0f), _e170.www);
            let _e423 = vec4<f32>(_e417.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e429 = vec4<f32>(_e423.x, _e417.y, _e423.z, _e423.w);
            let _e436 = local_2;
            let _e439 = (clamp((vec3<f32>(1f, 1f, 1f) - _e436), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e170.w);
            let _e440 = vec4<f32>(_e429.x, _e429.y, _e417.z, _e429.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e440 / _e439)), sign(_e440), (_e439 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e378 = local_2;
            local_2 = clamp(_e378, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e381 = clamp(_e172, vec3<f32>(0f, 0f, 0f), _e170.www);
            let _e387 = vec4<f32>(_e381.x, _e170.y, _e170.z, _e170.w);
            let _e393 = vec4<f32>(_e387.x, _e381.y, _e387.z, _e387.w);
            phi_3519_ = vec4<f32>(_e393.x, _e393.y, _e381.z, _e393.w);
            if (_e170.w == 0f) {
                phi_3519_ = vec4<f32>(_e381.x, _e381.y, _e381.z, 1f);
            }
            let _e403 = phi_3519_;
            let _e407 = (vec3(_e403.w) - _e403.xyz);
            let _e408 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e407 / (_e408 * _e403.w))), sign(_e407), (_e408 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e370 = local_2;
            let _e371 = (_e370 * _e178);
            local_1 = (select(_e371, (((_e370 + _e178) - _e371) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e370 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3467_ = 0i;
            loop {
                let _e339 = phi_3467_;
                if (_e339 < 3i) {
                    let _e342 = local_2[_e339];
                    if (_e342 <= 0.5f) {
                        let _e359 = local[_e339];
                        local_1[_e339] = (1f - _e359);
                    } else {
                        let _e345 = local[_e339];
                        if (_e345 <= 0.25f) {
                            let _e351 = local[_e339];
                            let _e354 = local[_e339];
                            local_1[_e339] = ((((16f * _e351) - 12f) * _e354) + 3f);
                        } else {
                            let _e347 = local[_e339];
                            local_1[_e339] = (inverseSqrt(_e347) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3467_ = (_e339 + 1i);
                }
            }
            let _e363 = local_2;
            let _e367 = local_1;
            local_1 = (_e178 + ((_e178 * ((_e363 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e367));
            break;
        }
        case 9: {
            let _e335 = local_2;
            local_1 = abs((_e178 - _e335));
            break;
        }
        case 10: {
            let _e330 = local_2;
            local_1 = ((_e330 + _e178) - ((_e330 * 2f) * _e178));
            break;
        }
        case 12: {
            if gj {
                let _e281 = local_2;
                let _e282 = clamp(_e281, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e282;
                let _e297 = (_e282 - vec3(min(min(_e282.x, _e282.y), _e282.z)));
                let _e305 = (_e297 * ((max(max(_e178.x, _e178.y), _e178.z) - min(min(_e178.x, _e178.y), _e178.z)) / max(0.000062f, max(max(_e297.x, _e297.y), _e297.z))));
                let _e306 = dot(_e178, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e309 = (_e305 - vec3(dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e322 = (vec2<f32>(_e306, (1f - _e306)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e309.x, _e309.y), _e309.z)), max(max(_e309.x, _e309.y), _e309.z))));
                local_1 = ((_e309 * min(1f, min(_e322.x, _e322.y))) + vec3(_e306));
            }
            break;
        }
        case 13: {
            if gj {
                let _e232 = local_2;
                let _e233 = clamp(_e232, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e233;
                let _e248 = (_e178 - vec3(min(min(_e178.x, _e178.y), _e178.z)));
                let _e256 = (_e248 * ((max(max(_e233.x, _e233.y), _e233.z) - min(min(_e233.x, _e233.y), _e233.z)) / max(0.000062f, max(max(_e248.x, _e248.y), _e248.z))));
                let _e257 = dot(_e178, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e260 = (_e256 - vec3(dot(_e256, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e273 = (vec2<f32>(_e257, (1f - _e257)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e260.x, _e260.y), _e260.z)), max(max(_e260.x, _e260.y), _e260.z))));
                local_1 = ((_e260 * min(1f, min(_e273.x, _e273.y))) + vec3(_e257));
            }
            break;
        }
        case 14: {
            if gj {
                let _e206 = local_2;
                let _e207 = clamp(_e206, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e207;
                let _e208 = dot(_e178, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e211 = (_e207 - vec3(dot(_e207, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e224 = (vec2<f32>(_e208, (1f - _e208)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e211.x, _e211.y), _e211.z)), max(max(_e211.x, _e211.y), _e211.z))));
                local_1 = ((_e211 * min(1f, min(_e224.x, _e224.y))) + vec3(_e208));
            }
            break;
        }
        case 15: {
            if gj {
                let _e180 = local_2;
                let _e181 = clamp(_e180, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e181;
                let _e182 = dot(_e181, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e185 = (_e178 - vec3(dot(_e178, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e198 = (vec2<f32>(_e182, (1f - _e182)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e185.x, _e185.y), _e185.z)), max(max(_e185.x, _e185.y), _e185.z))));
                local_1 = ((_e185 * min(1f, min(_e198.x, _e198.y))) + vec3(_e182));
            }
            break;
        }
        default: {
        }
    }
    let _e464 = local_1;
    let _e468 = (mix(_e171, _e464, vec3(_e170.w)) * _e165.w);
    let _e474 = vec4<f32>(_e468.x, _e165.y, _e165.z, _e165.w);
    let _e480 = vec4<f32>(_e474.x, _e468.y, _e474.z, _e474.w);
    let _e487 = (vec4<f32>(_e480.x, _e480.y, _e468.z, _e480.w) * clamp(_e62.x, 0f, 1f));
    let _e488 = _e487.xyz;
    let _e490 = gl_FragCoord_1;
    let _e492 = j.G3_;
    let _e494 = j.H3_;
    if (hj && (_e487.w != 0f)) {
        phi_3649_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e490.x) + (0.00583715f * _e490.y))))) * _e492) + _e494)) + _e488);
    } else {
        phi_3649_ = _e488;
    }
    let _e510 = phi_3649_;
    let _e516 = vec4<f32>(_e510.x, _e487.y, _e487.z, _e487.w);
    let _e522 = vec4<f32>(_e516.x, _e510.y, _e516.z, _e516.w);
    Ji = vec4<f32>(_e522.x, _e522.y, _e510.z, _e522.w);
    return;
}

@fragment
fn main(@location(1) T2_: vec2<f32>, @location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) f4_: f32) -> @location(0) vec4<f32> {
    T2_1 = T2_;
    U0_1 = U0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    gl_FragCoord_1 = gl_FragCoord;
    f4_1 = f4_;
    main_1();
    let _e13 = Ji;
    return _e13;
}
