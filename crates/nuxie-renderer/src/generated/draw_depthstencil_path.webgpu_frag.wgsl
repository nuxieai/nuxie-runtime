struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

@id(8) override rj: bool = true;
@id(7) override qj: bool = true;
@id(9) override sj: bool = true;
@id(16) override zj: bool = false;
@id(17) override Aj: bool = false;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
var<private> O0_1: vec3<f32>;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;
@group(0) @binding(12)
var MD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ri: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3186_: f32;
    var phi_3187_: f32;
    var phi_3188_: vec2<f32>;
    var phi_3189_: bool;
    var phi_3191_: vec4<f32>;
    var phi_3208_: vec4<f32>;
    var phi_3207_: vec4<f32>;
    var phi_1039_: bool;
    var phi_3192_: f32;
    var phi_3204_: vec4<f32>;
    var phi_3210_: vec4<f32>;
    var phi_3211_: f32;
    var phi_3549_: i32;
    var phi_3601_: vec4<f32>;
    var phi_3731_: vec3<f32>;
    var phi_1134_: bool;
    var phi_3756_: f32;
    var phi_3781_: vec4<f32>;

    let _e67 = S0_1;
    let _e68 = u32(_e67);
    let _e70 = (lj && (_e68 != 0u));
    let _e72 = N0_1[3u];
    if (_e72 >= 0f) {
        let _e151 = N0_1;
        phi_3207_ = _e151;
    } else {
        let _e74 = N0_1;
        let _e76 = j.Q8_;
        let _e78 = j.R8_;
        let _e81 = abs(_e74.z);
        let _e82 = floor(_e81);
        let _e85 = (((_e81 - _e82) * 8f) + 0.0009765625f);
        let _e87 = ((_e82 * _e76) + _e78);
        let _e88 = floor(_e85);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e88 == 4f)) {
                    let _e107 = normalize(vec2<f32>(_e74.x, -(_e74.y)));
                    let _e110 = acos(clamp(_e107.x, -1f, 1f));
                    if (_e107.y >= 0f) {
                        phi_3187_ = _e110;
                    } else {
                        phi_3187_ = -(_e110);
                    }
                    let _e115 = phi_3187_;
                    phi_3188_ = vec2<f32>((_e115 * 0.15915494f), _e87);
                    break;
                } else {
                    if (_e88 == 2f) {
                        phi_3186_ = _e74.x;
                    } else {
                        phi_3186_ = length(_e74.xy);
                    }
                    let _e97 = phi_3186_;
                    phi_3188_ = vec2<f32>(((clamp(_e97, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e74.z < 0f))) + _e85), _e87);
                    break;
                }
            }
        }
        let _e119 = phi_3188_;
        let _e120 = textureSampleLevel(ZC, S8_, _e119, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if zj {
                    phi_3189_ = true;
                    break;
                }
                if Aj {
                    phi_3189_ = true;
                    break;
                }
                phi_3189_ = false;
                break;
            }
        }
        let _e123 = phi_3189_;
        phi_3191_ = _e120;
        if _e123 {
            phi_3191_ = vec4<f32>(_e120.x, _e120.y, _e120.z, (_e120.w * ((fract(_e74.w) * -2f) + 1.5f)));
        }
        let _e136 = phi_3191_;
        phi_3208_ = _e136;
        if !(_e70) {
            let _e140 = (_e136.xyz * _e136.w);
            phi_3208_ = vec4<f32>(_e140.x, _e140.y, _e140.z, (_e136.w * (ceil(_e74.w) * -0.003921569f)));
        }
        let _e150 = phi_3208_;
        phi_3207_ = _e150;
    }
    let _e153 = phi_3207_;
    phi_1039_ = nj;
    if nj {
        let _e155 = O0_1[2u];
        phi_1039_ = (_e155 > 0f);
    }
    let _e158 = phi_1039_;
    phi_3210_ = _e153;
    if _e158 {
        let _e160 = O0_1[2u];
        let _e162 = O0_1;
        let _e164 = textureSampleLevel(RB, v4_, _e162.xy, (_e160 - 1f));
        phi_3204_ = _e164;
        if _e70 {
            if (_e164.w != 0f) {
                phi_3192_ = (1f / _e164.w);
            } else {
                phi_3192_ = 0f;
            }
            let _e170 = phi_3192_;
            let _e171 = (_e164.xyz * _e170);
            phi_3204_ = vec4<f32>(_e171.x, _e171.y, _e171.z, _e164.w);
        }
        let _e177 = phi_3204_;
        phi_3210_ = (_e153 * _e177);
    }
    let _e180 = phi_3210_;
    let _e181 = gl_FragCoord_1;
    let _e185 = textureLoad(MD, vec2<i32>(floor(_e181.xy)), 0i);
    let _e186 = _e180.xyz;
    local_2 = _e186;
    let _e187 = _e185.xyz;
    if (_e185.w != 0f) {
        phi_3211_ = (1f / _e185.w);
    } else {
        phi_3211_ = 0f;
    }
    let _e192 = phi_3211_;
    let _e193 = (_e187 * _e192);
    local = _e193;
    switch bitcast<i32>(_e68) {
        case 11: {
            let _e477 = local_2;
            local_1 = (_e477 * _e193);
            break;
        }
        case 1: {
            let _e473 = local_2;
            local_1 = ((_e473 + _e193) - (_e473 * _e193));
            break;
        }
        case 2: {
            let _e465 = local_2;
            let _e466 = (_e465 * _e193);
            local_1 = (select(_e466, (((_e465 + _e193) - _e466) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e193 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e463 = local_2;
            local_1 = min(_e463, _e193);
            break;
        }
        case 4: {
            let _e461 = local_2;
            local_1 = max(_e461, _e193);
            break;
        }
        case 5: {
            let _e432 = clamp(_e187, vec3<f32>(0f, 0f, 0f), _e185.www);
            let _e438 = vec4<f32>(_e432.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e444 = vec4<f32>(_e438.x, _e432.y, _e438.z, _e438.w);
            let _e451 = local_2;
            let _e454 = (clamp((vec3<f32>(1f, 1f, 1f) - _e451), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e185.w);
            let _e455 = vec4<f32>(_e444.x, _e444.y, _e432.z, _e444.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e455 / _e454)), sign(_e455), (_e454 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e393 = local_2;
            local_2 = clamp(_e393, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e396 = clamp(_e187, vec3<f32>(0f, 0f, 0f), _e185.www);
            let _e402 = vec4<f32>(_e396.x, _e185.y, _e185.z, _e185.w);
            let _e408 = vec4<f32>(_e402.x, _e396.y, _e402.z, _e402.w);
            phi_3601_ = vec4<f32>(_e408.x, _e408.y, _e396.z, _e408.w);
            if (_e185.w == 0f) {
                phi_3601_ = vec4<f32>(_e396.x, _e396.y, _e396.z, 1f);
            }
            let _e418 = phi_3601_;
            let _e422 = (vec3(_e418.w) - _e418.xyz);
            let _e423 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e422 / (_e423 * _e418.w))), sign(_e422), (_e423 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e385 = local_2;
            let _e386 = (_e385 * _e193);
            local_1 = (select(_e386, (((_e385 + _e193) - _e386) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e385 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3549_ = 0i;
            loop {
                let _e354 = phi_3549_;
                if (_e354 < 3i) {
                    let _e357 = local_2[_e354];
                    if (_e357 <= 0.5f) {
                        let _e374 = local[_e354];
                        local_1[_e354] = (1f - _e374);
                    } else {
                        let _e360 = local[_e354];
                        if (_e360 <= 0.25f) {
                            let _e366 = local[_e354];
                            let _e369 = local[_e354];
                            local_1[_e354] = ((((16f * _e366) - 12f) * _e369) + 3f);
                        } else {
                            let _e362 = local[_e354];
                            local_1[_e354] = (inverseSqrt(_e362) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3549_ = (_e354 + 1i);
                }
            }
            let _e378 = local_2;
            let _e382 = local_1;
            local_1 = (_e193 + ((_e193 * ((_e378 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e382));
            break;
        }
        case 9: {
            let _e350 = local_2;
            local_1 = abs((_e193 - _e350));
            break;
        }
        case 10: {
            let _e345 = local_2;
            local_1 = ((_e345 + _e193) - ((_e345 * 2f) * _e193));
            break;
        }
        case 12: {
            if qj {
                let _e296 = local_2;
                let _e297 = clamp(_e296, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e297;
                let _e312 = (_e297 - vec3(min(min(_e297.x, _e297.y), _e297.z)));
                let _e320 = (_e312 * ((max(max(_e193.x, _e193.y), _e193.z) - min(min(_e193.x, _e193.y), _e193.z)) / max(0.000062f, max(max(_e312.x, _e312.y), _e312.z))));
                let _e321 = dot(_e193, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e324 = (_e320 - vec3(dot(_e320, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e337 = (vec2<f32>(_e321, (1f - _e321)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e324.x, _e324.y), _e324.z)), max(max(_e324.x, _e324.y), _e324.z))));
                local_1 = ((_e324 * min(1f, min(_e337.x, _e337.y))) + vec3(_e321));
            }
            break;
        }
        case 13: {
            if qj {
                let _e247 = local_2;
                let _e248 = clamp(_e247, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e248;
                let _e263 = (_e193 - vec3(min(min(_e193.x, _e193.y), _e193.z)));
                let _e271 = (_e263 * ((max(max(_e248.x, _e248.y), _e248.z) - min(min(_e248.x, _e248.y), _e248.z)) / max(0.000062f, max(max(_e263.x, _e263.y), _e263.z))));
                let _e272 = dot(_e193, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e275 = (_e271 - vec3(dot(_e271, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e288 = (vec2<f32>(_e272, (1f - _e272)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e275.x, _e275.y), _e275.z)), max(max(_e275.x, _e275.y), _e275.z))));
                local_1 = ((_e275 * min(1f, min(_e288.x, _e288.y))) + vec3(_e272));
            }
            break;
        }
        case 14: {
            if qj {
                let _e221 = local_2;
                let _e222 = clamp(_e221, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e222;
                let _e223 = dot(_e193, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e226 = (_e222 - vec3(dot(_e222, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e239 = (vec2<f32>(_e223, (1f - _e223)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e226.x, _e226.y), _e226.z)), max(max(_e226.x, _e226.y), _e226.z))));
                local_1 = ((_e226 * min(1f, min(_e239.x, _e239.y))) + vec3(_e223));
            }
            break;
        }
        case 15: {
            if qj {
                let _e195 = local_2;
                let _e196 = clamp(_e195, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e196;
                let _e197 = dot(_e196, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e200 = (_e193 - vec3(dot(_e193, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e213 = (vec2<f32>(_e197, (1f - _e197)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e200.x, _e200.y), _e200.z)), max(max(_e200.x, _e200.y), _e200.z))));
                local_1 = ((_e200 * min(1f, min(_e213.x, _e213.y))) + vec3(_e197));
            }
            break;
        }
        default: {
        }
    }
    let _e479 = local_1;
    let _e481 = mix(_e186, _e479, vec3(_e185.w));
    let _e487 = vec4<f32>(_e481.x, _e180.y, _e180.z, _e180.w);
    let _e493 = vec4<f32>(_e487.x, _e481.y, _e487.z, _e487.w);
    let _e499 = vec4<f32>(_e493.x, _e493.y, _e481.z, _e493.w);
    let _e502 = (_e499.xyz * _e180.w);
    let _e508 = vec4<f32>(_e502.x, _e499.y, _e499.z, _e499.w);
    let _e514 = vec4<f32>(_e508.x, _e502.y, _e508.z, _e508.w);
    let _e520 = vec4<f32>(_e514.x, _e514.y, _e502.z, _e514.w);
    let _e521 = _e520.xyz;
    let _e522 = gl_FragCoord_1;
    let _e524 = j.H3_;
    let _e526 = j.I3_;
    if (rj && (_e180.w != 0f)) {
        phi_3731_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e522.x) + (0.00583715f * _e522.y))))) * _e524) + _e526)) + _e521);
    } else {
        phi_3731_ = _e521;
    }
    let _e542 = phi_3731_;
    let _e548 = vec4<f32>(_e542.x, _e520.y, _e520.z, _e520.w);
    let _e554 = vec4<f32>(_e548.x, _e542.y, _e548.z, _e548.w);
    phi_1134_ = nj;
    if nj {
        let _e562 = O0_1[2u];
        phi_1134_ = (_e562 < 0f);
    }
    let _e565 = phi_1134_;
    phi_3781_ = vec4<f32>(_e554.x, _e554.y, _e542.z, _e554.w);
    if _e565 {
        let _e566 = O0_1;
        let _e568 = textureSampleLevel(RB, v4_, _e566.xy, 0f);
        let _e570 = O0_1[2u];
        let _e572 = u32((-1f - _e570));
        switch bitcast<i32>(0u) {
            default: {
                let _e575 = dot(_e568.xyz, vec3<f32>(0.3f, 0.59f, 0.11f));
                if (_e572 == 0u) {
                    phi_3756_ = _e568.w;
                    break;
                }
                if (_e572 == 1u) {
                    phi_3756_ = (1f - _e568.w);
                    break;
                }
                if (_e572 == 2u) {
                    phi_3756_ = _e575;
                    break;
                }
                phi_3756_ = (1f - _e575);
                break;
            }
        }
        let _e584 = phi_3756_;
        phi_3781_ = vec4(mix(1f, _e584, _e72));
    }
    let _e588 = phi_3781_;
    Ri = _e588;
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @location(2) O0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    S0_1 = S0_;
    N0_1 = N0_;
    O0_1 = O0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Ri;
    return _e9;
}
