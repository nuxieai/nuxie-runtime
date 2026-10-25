struct og {
    v2_: array<vec2<u32>>,
}

struct m0Ue {
    v2_: array<u32>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    H8_: vec4<i32>,
    Fi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ji: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Ci: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct n0Ue {
    v2_: array<u32>,
}

struct T4Ue {
    v2_: array<u32>,
}

@id(6) override kj: bool = true;
@id(4) override ij: bool = true;
@id(0) override ej: bool = true;
@id(1) override fj: bool = true;
@id(2) override gj: bool = true;

@group(0) @binding(3)
var<storage> XC: og;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Ue;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Ue;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1397_: bool;
    var phi_3436_: f32;
    var phi_3435_: f32;
    var phi_3437_: f32;
    var phi_3440_: f32;
    var phi_3439_: f32;
    var phi_1434_: bool;
    var phi_3458_: f32;
    var phi_3441_: f32;
    var phi_3442_: f32;
    var phi_3443_: vec2<f32>;
    var phi_3455_: vec4<f32>;
    var phi_3453_: vec4<f32>;
    var phi_3463_: f32;
    var phi_3886_: i32;
    var phi_3950_: vec4<f32>;
    var phi_4107_: vec4<f32>;
    var phi_4120_: vec4<f32>;
    var phi_4121_: vec4<f32>;

    let _e74 = gl_FragCoord_1;
    let _e75 = _e74.xy;
    let _e78 = bitcast<vec2<u32>>(vec2<i32>(floor(_e75)));
    let _e80 = j.T6_;
    let _e109 = bitcast<i32>((((((_e78.y >> bitcast<u32>(5u)) * (((_e80 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e78.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e78.x & 28u) << bitcast<u32>(5u)) + ((_e78.y & 28u) << bitcast<u32>(2i)))) + (((_e78.y & 3u) << bitcast<u32>(2i)) + (_e78.x & 3u))));
    let _e112 = T4_.v2_[_e109];
    let _e116 = ((f32((_e112 & 131071u)) * 0.00048828125f) + -32f);
    let _e118 = (_e112 >> bitcast<u32>(17u));
    let _e121 = XC.v2_[_e118];
    phi_3435_ = _e116;
    if ((_e121.x & 768u) != 0u) {
        let _e125 = abs(_e116);
        phi_1397_ = ij;
        if ij {
            phi_1397_ = ((_e121.x & 512u) != 0u);
        }
        let _e129 = phi_1397_;
        phi_3436_ = _e125;
        if _e129 {
            phi_3436_ = (1f - abs(((fract((_e125 * 0.5f)) * 2f) + -1f)));
        }
        let _e137 = phi_3436_;
        phi_3435_ = _e137;
    }
    let _e139 = phi_3435_;
    let _e140 = clamp(_e139, 0f, 1f);
    phi_3439_ = _e140;
    if ej {
        let _e142 = (_e121.x >> bitcast<u32>(16u));
        phi_3440_ = _e140;
        if (_e142 != 0u) {
            let _e146 = m0_.v2_[_e109];
            if (_e142 == (_e146 >> bitcast<u32>(16i))) {
                phi_3437_ = min(_e140, unpack2x16float(_e146).x);
            } else {
                phi_3437_ = 0f;
            }
            let _e154 = phi_3437_;
            phi_3440_ = _e154;
        }
        let _e156 = phi_3440_;
        phi_3439_ = _e156;
    }
    let _e158 = phi_3439_;
    phi_1434_ = fj;
    if fj {
        phi_1434_ = ((_e121.x & 1024u) != 0u);
    }
    let _e162 = phi_1434_;
    phi_3458_ = _e158;
    if _e162 {
        let _e163 = (_e118 * 8u);
        let _e167 = JB.v2_[(_e163 + 2u)];
        let _e178 = JB.v2_[(_e163 + 3u)];
        let _e183 = _e178.zw;
        let _e185 = ((abs(((mat2x2<f32>(vec2<f32>(_e167.x, _e167.y), vec2<f32>(_e167.z, _e167.w)) * _e75) + _e178.xy)) * _e183) - _e183);
        phi_3458_ = min(_e158, clamp((min(_e185.x, _e185.y) + 0.5f), 0f, 1f));
    }
    let _e193 = phi_3458_;
    let _e194 = (_e121.x & 15u);
    let _e197 = ((_e121.x >> bitcast<u32>(4i)) & 15u);
    let _e199 = (gj && (_e197 != 0u));
    if (_e194 <= 1u) {
        phi_3453_ = select(unpack4x8unorm(_e121.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((ej && (_e194 == 0u))));
    } else {
        let _e201 = (_e118 * 8u);
        let _e204 = JB.v2_[_e201];
        let _e215 = JB.v2_[(_e201 + 1u)];
        let _e218 = ((mat2x2<f32>(vec2<f32>(_e204.x, _e204.y), vec2<f32>(_e204.z, _e204.w)) * _e75) + _e215.xy);
        let _e222 = f32(_e194);
        let _e224 = j.O8_;
        let _e226 = j.P8_;
        let _e231 = ((_e215.w * _e224) + _e226);
        switch bitcast<i32>(0u) {
            default: {
                if (_e222 == 4f) {
                    let _e248 = normalize(vec2<f32>(_e218.x, -(_e218.y)));
                    let _e251 = acos(clamp(_e248.x, -1f, 1f));
                    if (_e248.y >= 0f) {
                        phi_3442_ = _e251;
                    } else {
                        phi_3442_ = -(_e251);
                    }
                    let _e256 = phi_3442_;
                    phi_3443_ = vec2<f32>((_e256 * 0.15915507f), _e231);
                    break;
                } else {
                    if (_e222 == 2f) {
                        phi_3441_ = _e218.x;
                    } else {
                        phi_3441_ = length(_e218);
                    }
                    let _e238 = phi_3441_;
                    phi_3443_ = vec2<f32>(((clamp(_e238, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e215.z < 0f))) + ((max(0f, _e215.z) * 0.001953125f) + 0.0009765625f)), _e231);
                    break;
                }
            }
        }
        let _e260 = phi_3443_;
        let _e261 = textureSampleLevel(ZC, Q8_, _e260, 0f);
        phi_3455_ = _e261;
        if !(_e199) {
            let _e265 = (_e261.xyz * _e261.w);
            phi_3455_ = vec4<f32>(_e265.x, _e265.y, _e265.z, (_e261.w * abs(bitcast<f32>(_e121.y))));
        }
        let _e275 = phi_3455_;
        phi_3453_ = _e275;
    }
    let _e283 = phi_3453_;
    phi_4120_ = _e283;
    if _e199 {
        phi_4107_ = _e283;
        if ((_e283.w * _e193) != 0f) {
            let _e289 = n0_.v2_[_e109];
            let _e290 = unpack4x8unorm(_e289);
            let _e291 = _e283.xyz;
            local_2 = _e291;
            let _e292 = _e290.xyz;
            if (_e290.w != 0f) {
                phi_3463_ = (1f / _e290.w);
            } else {
                phi_3463_ = 0f;
            }
            let _e297 = phi_3463_;
            let _e298 = (_e292 * _e297);
            local = _e298;
            switch bitcast<i32>(_e197) {
                case 11: {
                    let _e582 = local_2;
                    local_1 = (_e582 * _e298);
                    break;
                }
                case 1: {
                    let _e578 = local_2;
                    local_1 = ((_e578 + _e298) - (_e578 * _e298));
                    break;
                }
                case 2: {
                    let _e570 = local_2;
                    let _e571 = (_e570 * _e298);
                    local_1 = (select(_e571, (((_e570 + _e298) - _e571) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e298 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e568 = local_2;
                    local_1 = min(_e568, _e298);
                    break;
                }
                case 4: {
                    let _e566 = local_2;
                    local_1 = max(_e566, _e298);
                    break;
                }
                case 5: {
                    let _e537 = clamp(_e292, vec3<f32>(0f, 0f, 0f), _e290.www);
                    let _e543 = vec4<f32>(_e537.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e549 = vec4<f32>(_e543.x, _e537.y, _e543.z, _e543.w);
                    let _e556 = local_2;
                    let _e559 = (clamp((vec3<f32>(1f, 1f, 1f) - _e556), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e290.w);
                    let _e560 = vec4<f32>(_e549.x, _e549.y, _e537.z, _e549.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e560 / _e559)), sign(_e560), (_e559 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e498 = local_2;
                    local_2 = clamp(_e498, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e501 = clamp(_e292, vec3<f32>(0f, 0f, 0f), _e290.www);
                    let _e507 = vec4<f32>(_e501.x, _e290.y, _e290.z, _e290.w);
                    let _e513 = vec4<f32>(_e507.x, _e501.y, _e507.z, _e507.w);
                    phi_3950_ = vec4<f32>(_e513.x, _e513.y, _e501.z, _e513.w);
                    if (_e290.w == 0f) {
                        phi_3950_ = vec4<f32>(_e501.x, _e501.y, _e501.z, 1f);
                    }
                    let _e523 = phi_3950_;
                    let _e527 = (vec3(_e523.w) - _e523.xyz);
                    let _e528 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e527 / (_e528 * _e523.w))), sign(_e527), (_e528 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e490 = local_2;
                    let _e491 = (_e490 * _e298);
                    local_1 = (select(_e491, (((_e490 + _e298) - _e491) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e490 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3886_ = 0i;
                    loop {
                        let _e459 = phi_3886_;
                        if (_e459 < 3i) {
                            let _e462 = local_2[_e459];
                            if (_e462 <= 0.5f) {
                                let _e479 = local[_e459];
                                local_1[_e459] = (1f - _e479);
                            } else {
                                let _e465 = local[_e459];
                                if (_e465 <= 0.25f) {
                                    let _e471 = local[_e459];
                                    let _e474 = local[_e459];
                                    local_1[_e459] = ((((16f * _e471) - 12f) * _e474) + 3f);
                                } else {
                                    let _e467 = local[_e459];
                                    local_1[_e459] = (inverseSqrt(_e467) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3886_ = (_e459 + 1i);
                        }
                    }
                    let _e483 = local_2;
                    let _e487 = local_1;
                    local_1 = (_e298 + ((_e298 * ((_e483 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e487));
                    break;
                }
                case 9: {
                    let _e455 = local_2;
                    local_1 = abs((_e298 - _e455));
                    break;
                }
                case 10: {
                    let _e450 = local_2;
                    local_1 = ((_e450 + _e298) - ((_e450 * 2f) * _e298));
                    break;
                }
                case 12: {
                    if kj {
                        let _e401 = local_2;
                        let _e402 = clamp(_e401, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e402;
                        let _e417 = (_e402 - vec3(min(min(_e402.x, _e402.y), _e402.z)));
                        let _e425 = (_e417 * ((max(max(_e298.x, _e298.y), _e298.z) - min(min(_e298.x, _e298.y), _e298.z)) / max(0.000062f, max(max(_e417.x, _e417.y), _e417.z))));
                        let _e426 = dot(_e298, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e429 = (_e425 - vec3(dot(_e425, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e442 = (vec2<f32>(_e426, (1f - _e426)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e429.x, _e429.y), _e429.z)), max(max(_e429.x, _e429.y), _e429.z))));
                        local_1 = ((_e429 * min(1f, min(_e442.x, _e442.y))) + vec3(_e426));
                    }
                    break;
                }
                case 13: {
                    if kj {
                        let _e352 = local_2;
                        let _e353 = clamp(_e352, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e353;
                        let _e368 = (_e298 - vec3(min(min(_e298.x, _e298.y), _e298.z)));
                        let _e376 = (_e368 * ((max(max(_e353.x, _e353.y), _e353.z) - min(min(_e353.x, _e353.y), _e353.z)) / max(0.000062f, max(max(_e368.x, _e368.y), _e368.z))));
                        let _e377 = dot(_e298, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e380 = (_e376 - vec3(dot(_e376, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e393 = (vec2<f32>(_e377, (1f - _e377)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e380.x, _e380.y), _e380.z)), max(max(_e380.x, _e380.y), _e380.z))));
                        local_1 = ((_e380 * min(1f, min(_e393.x, _e393.y))) + vec3(_e377));
                    }
                    break;
                }
                case 14: {
                    if kj {
                        let _e326 = local_2;
                        let _e327 = clamp(_e326, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e327;
                        let _e328 = dot(_e298, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e331 = (_e327 - vec3(dot(_e327, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e344 = (vec2<f32>(_e328, (1f - _e328)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e331.x, _e331.y), _e331.z)), max(max(_e331.x, _e331.y), _e331.z))));
                        local_1 = ((_e331 * min(1f, min(_e344.x, _e344.y))) + vec3(_e328));
                    }
                    break;
                }
                case 15: {
                    if kj {
                        let _e300 = local_2;
                        let _e301 = clamp(_e300, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e301;
                        let _e302 = dot(_e301, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e305 = (_e298 - vec3(dot(_e298, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e318 = (vec2<f32>(_e302, (1f - _e302)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e305.x, _e305.y), _e305.z)), max(max(_e305.x, _e305.y), _e305.z))));
                        local_1 = ((_e305 * min(1f, min(_e318.x, _e318.y))) + vec3(_e302));
                    }
                    break;
                }
                default: {
                }
            }
            let _e584 = local_1;
            let _e586 = mix(_e291, _e584, vec3(_e290.w));
            let _e592 = vec4<f32>(_e586.x, _e283.y, _e283.z, _e283.w);
            let _e598 = vec4<f32>(_e592.x, _e586.y, _e592.z, _e592.w);
            phi_4107_ = vec4<f32>(_e598.x, _e598.y, _e586.z, _e598.w);
        }
        let _e606 = phi_4107_;
        let _e609 = (_e606.xyz * _e606.w);
        let _e615 = vec4<f32>(_e609.x, _e606.y, _e606.z, _e606.w);
        let _e621 = vec4<f32>(_e615.x, _e609.y, _e615.z, _e615.w);
        phi_4120_ = vec4<f32>(_e621.x, _e621.y, _e609.z, _e621.w);
    }
    let _e629 = phi_4120_;
    let _e630 = (_e629 * _e193);
    let _e632 = (1f - _e630.w);
    phi_4121_ = _e630;
    if (_e632 != 0f) {
        let _e636 = n0_.v2_[_e109];
        phi_4121_ = (_e630 + (unpack4x8unorm(_e636) * _e632));
    }
    let _e641 = phi_4121_;
    Q1_ = _e641;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = Q1_;
    return _e3;
}
