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
@id(7) override qj: bool = true;
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
@group(0) @binding(11)
var LD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Qi: vec4<f32>;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3174_: f32;
    var phi_3175_: f32;
    var phi_3176_: vec2<f32>;
    var phi_3177_: bool;
    var phi_3179_: vec4<f32>;
    var phi_3196_: vec4<f32>;
    var phi_3195_: vec4<f32>;
    var phi_1034_: bool;
    var phi_3180_: f32;
    var phi_3192_: vec4<f32>;
    var phi_3198_: vec4<f32>;
    var phi_3199_: f32;
    var phi_3537_: i32;
    var phi_3589_: vec4<f32>;
    var phi_3719_: vec3<f32>;
    var phi_1129_: bool;
    var phi_3744_: f32;
    var phi_3769_: vec4<f32>;

    let _e67 = R0_1;
    let _e68 = u32(_e67);
    let _e70 = (lj && (_e68 != 0u));
    let _e72 = M0_1[3u];
    if (_e72 >= 0f) {
        let _e147 = M0_1;
        phi_3195_ = _e147;
    } else {
        let _e74 = M0_1;
        let _e76 = j.O8_;
        let _e78 = j.P8_;
        let _e81 = abs(_e74.z);
        let _e82 = floor(_e81);
        let _e85 = (((_e81 - _e82) * 8f) + 0.0009765625f);
        let _e87 = ((_e82 * _e76) + _e78);
        let _e88 = floor(_e85);
        let _e89 = _e74.xy;
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e88 == 4f)) {
                    let _e103 = normalize(_e89);
                    let _e106 = acos(clamp(_e103.x, -1f, 1f));
                    if (_e103.y >= 0f) {
                        phi_3175_ = _e106;
                    } else {
                        phi_3175_ = -(_e106);
                    }
                    let _e111 = phi_3175_;
                    phi_3176_ = vec2<f32>((_e111 * 0.15915494f), _e87);
                    break;
                } else {
                    if (_e88 == 2f) {
                        phi_3174_ = _e74.x;
                    } else {
                        phi_3174_ = length(_e89);
                    }
                    let _e97 = phi_3174_;
                    phi_3176_ = vec2<f32>(((clamp(_e97, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e74.z < 0f))) + _e85), _e87);
                    break;
                }
            }
        }
        let _e115 = phi_3176_;
        let _e116 = textureSampleLevel(YC, Q8_, _e115, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if zj {
                    phi_3177_ = true;
                    break;
                }
                if Aj {
                    phi_3177_ = true;
                    break;
                }
                phi_3177_ = false;
                break;
            }
        }
        let _e119 = phi_3177_;
        phi_3179_ = _e116;
        if _e119 {
            phi_3179_ = vec4<f32>(_e116.x, _e116.y, _e116.z, (_e116.w * ((fract(_e74.w) * -2f) + 1.5f)));
        }
        let _e132 = phi_3179_;
        phi_3196_ = _e132;
        if !(_e70) {
            let _e136 = (_e132.xyz * _e132.w);
            phi_3196_ = vec4<f32>(_e136.x, _e136.y, _e136.z, (_e132.w * (ceil(_e74.w) * -0.003921569f)));
        }
        let _e146 = phi_3196_;
        phi_3195_ = _e146;
    }
    let _e149 = phi_3195_;
    phi_1034_ = nj;
    if nj {
        let _e151 = N0_1[2u];
        phi_1034_ = (_e151 > 0f);
    }
    let _e154 = phi_1034_;
    phi_3198_ = _e149;
    if _e154 {
        let _e156 = N0_1[2u];
        let _e158 = N0_1;
        let _e160 = textureSampleLevel(RB, v4_, _e158.xy, (_e156 - 1f));
        phi_3192_ = _e160;
        if _e70 {
            if (_e160.w != 0f) {
                phi_3180_ = (1f / _e160.w);
            } else {
                phi_3180_ = 0f;
            }
            let _e166 = phi_3180_;
            let _e167 = (_e160.xyz * _e166);
            phi_3192_ = vec4<f32>(_e167.x, _e167.y, _e167.z, _e160.w);
        }
        let _e173 = phi_3192_;
        phi_3198_ = (_e149 * _e173);
    }
    let _e176 = phi_3198_;
    let _e177 = gl_FragCoord_1;
    let _e181 = textureLoad(LD, vec2<i32>(floor(_e177.xy)), 0i);
    let _e182 = _e176.xyz;
    local_2 = _e182;
    let _e183 = _e181.xyz;
    if (_e181.w != 0f) {
        phi_3199_ = (1f / _e181.w);
    } else {
        phi_3199_ = 0f;
    }
    let _e188 = phi_3199_;
    let _e189 = (_e183 * _e188);
    local = _e189;
    switch bitcast<i32>(_e68) {
        case 11: {
            let _e473 = local_2;
            local_1 = (_e473 * _e189);
            break;
        }
        case 1: {
            let _e469 = local_2;
            local_1 = ((_e469 + _e189) - (_e469 * _e189));
            break;
        }
        case 2: {
            let _e461 = local_2;
            let _e462 = (_e461 * _e189);
            local_1 = (select(_e462, (((_e461 + _e189) - _e462) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e189 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e459 = local_2;
            local_1 = min(_e459, _e189);
            break;
        }
        case 4: {
            let _e457 = local_2;
            local_1 = max(_e457, _e189);
            break;
        }
        case 5: {
            let _e428 = clamp(_e183, vec3<f32>(0f, 0f, 0f), _e181.www);
            let _e434 = vec4<f32>(_e428.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e440 = vec4<f32>(_e434.x, _e428.y, _e434.z, _e434.w);
            let _e447 = local_2;
            let _e450 = (clamp((vec3<f32>(1f, 1f, 1f) - _e447), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e181.w);
            let _e451 = vec4<f32>(_e440.x, _e440.y, _e428.z, _e440.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e451 / _e450)), sign(_e451), (_e450 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e389 = local_2;
            local_2 = clamp(_e389, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e392 = clamp(_e183, vec3<f32>(0f, 0f, 0f), _e181.www);
            let _e398 = vec4<f32>(_e392.x, _e181.y, _e181.z, _e181.w);
            let _e404 = vec4<f32>(_e398.x, _e392.y, _e398.z, _e398.w);
            phi_3589_ = vec4<f32>(_e404.x, _e404.y, _e392.z, _e404.w);
            if (_e181.w == 0f) {
                phi_3589_ = vec4<f32>(_e392.x, _e392.y, _e392.z, 1f);
            }
            let _e414 = phi_3589_;
            let _e418 = (vec3(_e414.w) - _e414.xyz);
            let _e419 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e418 / (_e419 * _e414.w))), sign(_e418), (_e419 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e381 = local_2;
            let _e382 = (_e381 * _e189);
            local_1 = (select(_e382, (((_e381 + _e189) - _e382) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e381 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3537_ = 0i;
            loop {
                let _e350 = phi_3537_;
                if (_e350 < 3i) {
                    let _e353 = local_2[_e350];
                    if (_e353 <= 0.5f) {
                        let _e370 = local[_e350];
                        local_1[_e350] = (1f - _e370);
                    } else {
                        let _e356 = local[_e350];
                        if (_e356 <= 0.25f) {
                            let _e362 = local[_e350];
                            let _e365 = local[_e350];
                            local_1[_e350] = ((((16f * _e362) - 12f) * _e365) + 3f);
                        } else {
                            let _e358 = local[_e350];
                            local_1[_e350] = (inverseSqrt(_e358) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3537_ = (_e350 + 1i);
                }
            }
            let _e374 = local_2;
            let _e378 = local_1;
            local_1 = (_e189 + ((_e189 * ((_e374 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e378));
            break;
        }
        case 9: {
            let _e346 = local_2;
            local_1 = abs((_e189 - _e346));
            break;
        }
        case 10: {
            let _e341 = local_2;
            local_1 = ((_e341 + _e189) - ((_e341 * 2f) * _e189));
            break;
        }
        case 12: {
            if qj {
                let _e292 = local_2;
                let _e293 = clamp(_e292, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e293;
                let _e308 = (_e293 - vec3(min(min(_e293.x, _e293.y), _e293.z)));
                let _e316 = (_e308 * ((max(max(_e189.x, _e189.y), _e189.z) - min(min(_e189.x, _e189.y), _e189.z)) / max(0.000062f, max(max(_e308.x, _e308.y), _e308.z))));
                let _e317 = dot(_e189, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e320 = (_e316 - vec3(dot(_e316, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e333 = (vec2<f32>(_e317, (1f - _e317)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e320.x, _e320.y), _e320.z)), max(max(_e320.x, _e320.y), _e320.z))));
                local_1 = ((_e320 * min(1f, min(_e333.x, _e333.y))) + vec3(_e317));
            }
            break;
        }
        case 13: {
            if qj {
                let _e243 = local_2;
                let _e244 = clamp(_e243, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e244;
                let _e259 = (_e189 - vec3(min(min(_e189.x, _e189.y), _e189.z)));
                let _e267 = (_e259 * ((max(max(_e244.x, _e244.y), _e244.z) - min(min(_e244.x, _e244.y), _e244.z)) / max(0.000062f, max(max(_e259.x, _e259.y), _e259.z))));
                let _e268 = dot(_e189, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e271 = (_e267 - vec3(dot(_e267, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e284 = (vec2<f32>(_e268, (1f - _e268)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e271.x, _e271.y), _e271.z)), max(max(_e271.x, _e271.y), _e271.z))));
                local_1 = ((_e271 * min(1f, min(_e284.x, _e284.y))) + vec3(_e268));
            }
            break;
        }
        case 14: {
            if qj {
                let _e217 = local_2;
                let _e218 = clamp(_e217, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e218;
                let _e219 = dot(_e189, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e222 = (_e218 - vec3(dot(_e218, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e235 = (vec2<f32>(_e219, (1f - _e219)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e222.x, _e222.y), _e222.z)), max(max(_e222.x, _e222.y), _e222.z))));
                local_1 = ((_e222 * min(1f, min(_e235.x, _e235.y))) + vec3(_e219));
            }
            break;
        }
        case 15: {
            if qj {
                let _e191 = local_2;
                let _e192 = clamp(_e191, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e192;
                let _e193 = dot(_e192, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e196 = (_e189 - vec3(dot(_e189, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e209 = (vec2<f32>(_e193, (1f - _e193)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e196.x, _e196.y), _e196.z)), max(max(_e196.x, _e196.y), _e196.z))));
                local_1 = ((_e196 * min(1f, min(_e209.x, _e209.y))) + vec3(_e193));
            }
            break;
        }
        default: {
        }
    }
    let _e475 = local_1;
    let _e477 = mix(_e182, _e475, vec3(_e181.w));
    let _e483 = vec4<f32>(_e477.x, _e176.y, _e176.z, _e176.w);
    let _e489 = vec4<f32>(_e483.x, _e477.y, _e483.z, _e483.w);
    let _e495 = vec4<f32>(_e489.x, _e489.y, _e477.z, _e489.w);
    let _e498 = (_e495.xyz * _e176.w);
    let _e504 = vec4<f32>(_e498.x, _e495.y, _e495.z, _e495.w);
    let _e510 = vec4<f32>(_e504.x, _e498.y, _e504.z, _e504.w);
    let _e516 = vec4<f32>(_e510.x, _e510.y, _e498.z, _e510.w);
    let _e517 = _e516.xyz;
    let _e518 = gl_FragCoord_1;
    let _e520 = j.I3_;
    let _e522 = j.J3_;
    if (rj && (_e176.w != 0f)) {
        phi_3719_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e518.x) + (0.00583715f * _e518.y))))) * _e520) + _e522)) + _e517);
    } else {
        phi_3719_ = _e517;
    }
    let _e538 = phi_3719_;
    let _e544 = vec4<f32>(_e538.x, _e516.y, _e516.z, _e516.w);
    let _e550 = vec4<f32>(_e544.x, _e538.y, _e544.z, _e544.w);
    phi_1129_ = nj;
    if nj {
        let _e558 = N0_1[2u];
        phi_1129_ = (_e558 < 0f);
    }
    let _e561 = phi_1129_;
    phi_3769_ = vec4<f32>(_e550.x, _e550.y, _e538.z, _e550.w);
    if _e561 {
        let _e562 = N0_1;
        let _e564 = textureSampleLevel(RB, v4_, _e562.xy, 0f);
        let _e566 = N0_1[2u];
        let _e568 = u32((-1f - _e566));
        switch bitcast<i32>(0u) {
            default: {
                let _e571 = dot(_e564.xyz, vec3<f32>(0.3f, 0.59f, 0.11f));
                if (_e568 == 0u) {
                    phi_3744_ = _e564.w;
                    break;
                }
                if (_e568 == 1u) {
                    phi_3744_ = (1f - _e564.w);
                    break;
                }
                if (_e568 == 2u) {
                    phi_3744_ = _e571;
                    break;
                }
                phi_3744_ = (1f - _e571);
                break;
            }
        }
        let _e580 = phi_3744_;
        phi_3769_ = vec4(mix(1f, _e580, _e72));
    }
    let _e584 = phi_3769_;
    Qi = _e584;
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
