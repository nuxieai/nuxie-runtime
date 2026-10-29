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
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var Q8_: sampler;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;
@group(0) @binding(9)
var ID: texture_2d<f32>;
@group(3) @binding(9)
var Ua: sampler;
var<private> W2_1: vec2<f32>;
var<private> N0_1: vec3<f32>;
var<private> R0_1: f32;
var<private> M0_1: vec4<f32>;
@group(0) @binding(11)
var LD: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Qi: vec4<f32>;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
var<private> h4_1: f32;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3097_: f32;
    var phi_3098_: f32;
    var phi_3099_: vec2<f32>;
    var phi_3116_: vec4<f32>;
    var phi_3115_: vec4<f32>;
    var phi_1182_: bool;
    var phi_1197_: bool;
    var phi_3100_: f32;
    var phi_3111_: vec4<f32>;
    var phi_3118_: vec4<f32>;
    var phi_3119_: vec4<f32>;
    var phi_3120_: f32;
    var phi_3458_: i32;
    var phi_3510_: vec4<f32>;
    var phi_3640_: vec3<f32>;

    let _e62 = W2_1;
    let _e63 = textureSampleLevel(ID, Ua, _e62, 0f);
    let _e66 = R0_1;
    let _e67 = u32(_e66);
    let _e68 = N0_1;
    let _e69 = M0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e72 = (lj && (_e67 != 0u));
            if (_e69.w >= 0f) {
                phi_3115_ = _e69;
            } else {
                let _e76 = j.O8_;
                let _e78 = j.P8_;
                let _e81 = abs(_e69.z);
                let _e82 = floor(_e81);
                let _e85 = (((_e81 - _e82) * 8f) + 0.0009765625f);
                let _e87 = ((_e82 * _e76) + _e78);
                let _e88 = floor(_e85);
                let _e89 = _e69.xy;
                switch bitcast<i32>(0u) {
                    default: {
                        if (sj && (_e88 == 4f)) {
                            let _e103 = normalize(_e89);
                            let _e106 = acos(clamp(_e103.x, -1f, 1f));
                            if (_e103.y >= 0f) {
                                phi_3098_ = _e106;
                            } else {
                                phi_3098_ = -(_e106);
                            }
                            let _e111 = phi_3098_;
                            phi_3099_ = vec2<f32>((_e111 * 0.15915494f), _e87);
                            break;
                        } else {
                            if (_e88 == 2f) {
                                phi_3097_ = _e69.x;
                            } else {
                                phi_3097_ = length(_e89);
                            }
                            let _e97 = phi_3097_;
                            phi_3099_ = vec2<f32>(((clamp(_e97, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e69.z < 0f))) + _e85), _e87);
                            break;
                        }
                    }
                }
                let _e115 = phi_3099_;
                let _e116 = textureSampleLevel(YC, Q8_, _e115, 0f);
                phi_3116_ = _e116;
                if !(_e72) {
                    let _e122 = (_e116.xyz * _e116.w);
                    phi_3116_ = vec4<f32>(_e122.x, _e122.y, _e122.z, (_e116.w * (ceil(_e69.w) * -0.003921569f)));
                }
                let _e129 = phi_3116_;
                phi_3115_ = _e129;
            }
            let _e131 = phi_3115_;
            phi_1182_ = nj;
            if nj {
                phi_1182_ = (_e68.z < 0f);
            }
            let _e135 = phi_1182_;
            if _e135 {
                let _e137 = textureSampleLevel(RB, v4_, _e68.xy, 0f);
                phi_3119_ = _e137;
                break;
            }
            phi_1197_ = nj;
            if nj {
                phi_1197_ = (_e68.z > 0f);
            }
            let _e141 = phi_1197_;
            phi_3118_ = _e131;
            if _e141 {
                let _e145 = textureSampleLevel(RB, v4_, _e68.xy, (_e68.z - 1f));
                phi_3111_ = _e145;
                if _e72 {
                    if (_e145.w != 0f) {
                        phi_3100_ = (1f / _e145.w);
                    } else {
                        phi_3100_ = 0f;
                    }
                    let _e151 = phi_3100_;
                    let _e152 = (_e145.xyz * _e151);
                    phi_3111_ = vec4<f32>(_e152.x, _e152.y, _e152.z, _e145.w);
                }
                let _e158 = phi_3111_;
                phi_3118_ = (_e131 * _e158);
            }
            let _e161 = phi_3118_;
            phi_3119_ = _e161;
            break;
        }
    }
    let _e163 = phi_3119_;
    let _e164 = gl_FragCoord_1;
    let _e168 = textureLoad(LD, vec2<i32>(floor(_e164.xy)), 0i);
    let _e169 = _e163.xyz;
    local_2 = _e169;
    let _e170 = _e168.xyz;
    if (_e168.w != 0f) {
        phi_3120_ = (1f / _e168.w);
    } else {
        phi_3120_ = 0f;
    }
    let _e175 = phi_3120_;
    let _e176 = (_e170 * _e175);
    local = _e176;
    switch bitcast<i32>(_e67) {
        case 11: {
            let _e460 = local_2;
            local_1 = (_e460 * _e176);
            break;
        }
        case 1: {
            let _e456 = local_2;
            local_1 = ((_e456 + _e176) - (_e456 * _e176));
            break;
        }
        case 2: {
            let _e448 = local_2;
            let _e449 = (_e448 * _e176);
            local_1 = (select(_e449, (((_e448 + _e176) - _e449) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e176 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 3: {
            let _e446 = local_2;
            local_1 = min(_e446, _e176);
            break;
        }
        case 4: {
            let _e444 = local_2;
            local_1 = max(_e444, _e176);
            break;
        }
        case 5: {
            let _e415 = clamp(_e170, vec3<f32>(0f, 0f, 0f), _e168.www);
            let _e421 = vec4<f32>(_e415.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e427 = vec4<f32>(_e421.x, _e415.y, _e421.z, _e421.w);
            let _e434 = local_2;
            let _e437 = (clamp((vec3<f32>(1f, 1f, 1f) - _e434), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e168.w);
            let _e438 = vec4<f32>(_e427.x, _e427.y, _e415.z, _e427.w).xyz;
            local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e438 / _e437)), sign(_e438), (_e437 == vec3<f32>(0f, 0f, 0f)));
            break;
        }
        case 6: {
            let _e376 = local_2;
            local_2 = clamp(_e376, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
            let _e379 = clamp(_e170, vec3<f32>(0f, 0f, 0f), _e168.www);
            let _e385 = vec4<f32>(_e379.x, _e168.y, _e168.z, _e168.w);
            let _e391 = vec4<f32>(_e385.x, _e379.y, _e385.z, _e385.w);
            phi_3510_ = vec4<f32>(_e391.x, _e391.y, _e379.z, _e391.w);
            if (_e168.w == 0f) {
                phi_3510_ = vec4<f32>(_e379.x, _e379.y, _e379.z, 1f);
            }
            let _e401 = phi_3510_;
            let _e405 = (vec3(_e401.w) - _e401.xyz);
            let _e406 = local_2;
            local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e405 / (_e406 * _e401.w))), sign(_e405), (_e406 == vec3<f32>(0f, 0f, 0f))));
            break;
        }
        case 7: {
            let _e368 = local_2;
            let _e369 = (_e368 * _e176);
            local_1 = (select(_e369, (((_e368 + _e176) - _e369) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e368 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
            break;
        }
        case 8: {
            phi_3458_ = 0i;
            loop {
                let _e337 = phi_3458_;
                if (_e337 < 3i) {
                    let _e340 = local_2[_e337];
                    if (_e340 <= 0.5f) {
                        let _e357 = local[_e337];
                        local_1[_e337] = (1f - _e357);
                    } else {
                        let _e343 = local[_e337];
                        if (_e343 <= 0.25f) {
                            let _e349 = local[_e337];
                            let _e352 = local[_e337];
                            local_1[_e337] = ((((16f * _e349) - 12f) * _e352) + 3f);
                        } else {
                            let _e345 = local[_e337];
                            local_1[_e337] = (inverseSqrt(_e345) - 1f);
                        }
                    }
                    continue;
                } else {
                    break;
                }
                continuing {
                    phi_3458_ = (_e337 + 1i);
                }
            }
            let _e361 = local_2;
            let _e365 = local_1;
            local_1 = (_e176 + ((_e176 * ((_e361 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e365));
            break;
        }
        case 9: {
            let _e333 = local_2;
            local_1 = abs((_e176 - _e333));
            break;
        }
        case 10: {
            let _e328 = local_2;
            local_1 = ((_e328 + _e176) - ((_e328 * 2f) * _e176));
            break;
        }
        case 12: {
            if qj {
                let _e279 = local_2;
                let _e280 = clamp(_e279, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e280;
                let _e295 = (_e280 - vec3(min(min(_e280.x, _e280.y), _e280.z)));
                let _e303 = (_e295 * ((max(max(_e176.x, _e176.y), _e176.z) - min(min(_e176.x, _e176.y), _e176.z)) / max(0.000062f, max(max(_e295.x, _e295.y), _e295.z))));
                let _e304 = dot(_e176, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e307 = (_e303 - vec3(dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e320 = (vec2<f32>(_e304, (1f - _e304)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e307.x, _e307.y), _e307.z)), max(max(_e307.x, _e307.y), _e307.z))));
                local_1 = ((_e307 * min(1f, min(_e320.x, _e320.y))) + vec3(_e304));
            }
            break;
        }
        case 13: {
            if qj {
                let _e230 = local_2;
                let _e231 = clamp(_e230, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e231;
                let _e246 = (_e176 - vec3(min(min(_e176.x, _e176.y), _e176.z)));
                let _e254 = (_e246 * ((max(max(_e231.x, _e231.y), _e231.z) - min(min(_e231.x, _e231.y), _e231.z)) / max(0.000062f, max(max(_e246.x, _e246.y), _e246.z))));
                let _e255 = dot(_e176, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e258 = (_e254 - vec3(dot(_e254, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e271 = (vec2<f32>(_e255, (1f - _e255)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e258.x, _e258.y), _e258.z)), max(max(_e258.x, _e258.y), _e258.z))));
                local_1 = ((_e258 * min(1f, min(_e271.x, _e271.y))) + vec3(_e255));
            }
            break;
        }
        case 14: {
            if qj {
                let _e204 = local_2;
                let _e205 = clamp(_e204, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e205;
                let _e206 = dot(_e176, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e209 = (_e205 - vec3(dot(_e205, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e222 = (vec2<f32>(_e206, (1f - _e206)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e209.x, _e209.y), _e209.z)), max(max(_e209.x, _e209.y), _e209.z))));
                local_1 = ((_e209 * min(1f, min(_e222.x, _e222.y))) + vec3(_e206));
            }
            break;
        }
        case 15: {
            if qj {
                let _e178 = local_2;
                let _e179 = clamp(_e178, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                local_2 = _e179;
                let _e180 = dot(_e179, vec3<f32>(0.3f, 0.59f, 0.11f));
                let _e183 = (_e176 - vec3(dot(_e176, vec3<f32>(0.3f, 0.59f, 0.11f))));
                let _e196 = (vec2<f32>(_e180, (1f - _e180)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e183.x, _e183.y), _e183.z)), max(max(_e183.x, _e183.y), _e183.z))));
                local_1 = ((_e183 * min(1f, min(_e196.x, _e196.y))) + vec3(_e180));
            }
            break;
        }
        default: {
        }
    }
    let _e462 = local_1;
    let _e466 = (mix(_e169, _e462, vec3(_e168.w)) * _e163.w);
    let _e472 = vec4<f32>(_e466.x, _e163.y, _e163.z, _e163.w);
    let _e478 = vec4<f32>(_e472.x, _e466.y, _e472.z, _e472.w);
    let _e485 = (vec4<f32>(_e478.x, _e478.y, _e466.z, _e478.w) * clamp(_e63.x, 0f, 1f));
    let _e486 = _e485.xyz;
    let _e488 = gl_FragCoord_1;
    let _e490 = j.I3_;
    let _e492 = j.J3_;
    if (rj && (_e485.w != 0f)) {
        phi_3640_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e488.x) + (0.00583715f * _e488.y))))) * _e490) + _e492)) + _e486);
    } else {
        phi_3640_ = _e486;
    }
    let _e508 = phi_3640_;
    let _e514 = vec4<f32>(_e508.x, _e485.y, _e485.z, _e485.w);
    let _e520 = vec4<f32>(_e514.x, _e508.y, _e514.z, _e514.w);
    Qi = vec4<f32>(_e520.x, _e520.y, _e508.z, _e520.w);
    return;
}

@fragment
fn main(@location(1) W2_: vec2<f32>, @location(9) N0_: vec3<f32>, @location(6) @interpolate(flat, either) R0_: f32, @location(0) M0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) h4_: f32) -> @location(0) vec4<f32> {
    W2_1 = W2_;
    N0_1 = N0_;
    R0_1 = R0_;
    M0_1 = M0_;
    gl_FragCoord_1 = gl_FragCoord;
    h4_1 = h4_;
    main_1();
    let _e13 = Qi;
    return _e13;
}
