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
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct n0Ue {
    v2_: array<u32>,
}

struct T4Ue {
    v2_: array<u32>,
}

struct T4Ue_1 {
    v2_: array<atomic<u32>>,
}

@id(8) override nj: bool = true;
@id(7) override mj: bool = true;
@id(9) override oj: bool = true;
@id(5) override kj: bool = true;
@id(0) override fj: bool = true;
@id(1) override gj: bool = true;
@id(2) override hj: bool = true;
@id(3) override ij: bool = true;

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var db: sampler;
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
var R8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Ue;
var<private> T_1: vec4<f32>;
var<private> G0_1: u32;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue_1;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1595_: bool;
    var phi_1608_: bool;
    var phi_4315_: f32;
    var phi_4323_: f32;
    var phi_4331_: f32;
    var phi_4330_: f32;
    var phi_2113_: bool;
    var phi_4334_: f32;
    var phi_4333_: f32;
    var phi_4335_: f32;
    var phi_4338_: f32;
    var phi_4337_: f32;
    var phi_2150_: bool;
    var phi_4342_: f32;
    var phi_4339_: f32;
    var phi_4340_: f32;
    var phi_4341_: vec2<f32>;
    var phi_4374_: vec4<f32>;
    var phi_5286_: u32;
    var phi_5285_: u32;
    var phi_4372_: vec4<f32>;
    var phi_4381_: f32;
    var phi_4971_: i32;
    var phi_5059_: vec4<f32>;
    var phi_5270_: vec4<f32>;
    var phi_5283_: vec4<f32>;
    var phi_5317_: u32;
    var phi_5310_: vec4<f32>;
    var phi_5312_: vec3<f32>;
    var phi_5314_: vec4<f32>;

    let _e100 = gl_FragCoord_1;
    let _e101 = _e100.xy;
    let _e104 = bitcast<vec2<u32>>(vec2<i32>(floor(_e101)));
    let _e106 = j.T6_;
    let _e135 = bitcast<i32>((((((_e104.y >> bitcast<u32>(5u)) * (((_e106 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e104.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e104.x & 28u) << bitcast<u32>(5u)) + ((_e104.y & 28u) << bitcast<u32>(2i)))) + (((_e104.y & 3u) << bitcast<u32>(2i)) + (_e104.x & 3u))));
    phi_1595_ = ij;
    if ij {
        let _e136 = T_1;
        phi_1595_ = (_e136.x < -1.5f);
    }
    let _e140 = phi_1595_;
    if _e140 {
        let _e213 = T_1;
        let _e217 = textureSampleLevel(AD, db, vec2<f32>((3f + _e213.x), 0f), 0f);
        let _e223 = textureSampleLevel(AD, db, vec2<f32>((1f - _e213.y), 0f), 0f);
        phi_4330_ = ((1f - _e217.x) - _e223.x);
    } else {
        phi_1608_ = ij;
        if ij {
            let _e141 = T_1;
            phi_1608_ = (_e141.y < -1.5f);
        }
        let _e145 = phi_1608_;
        if _e145 {
            let _e153 = T_1;
            let _e156 = max(_e153.w, 0f);
            if (_e153.z >= 0f) {
                let _e159 = textureSampleLevel(AD, db, vec2<f32>(_e156, 0f), 0f);
                phi_4315_ = _e159.x;
            } else {
                phi_4315_ = 0f;
            }
            let _e162 = phi_4315_;
            phi_4323_ = _e162;
            if (abs(_e153.z) < 1000f) {
                let _e169 = (-2f - _e153.y);
                let _e171 = ((_e169 - _e156) * 0.5984134f);
                let _e174 = (vec4(_e156) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e171));
                let _e180 = ((_e174 * -(_e153.z)) + vec4(((_e169 * _e153.z) + (abs(_e153.x) - 0.25f))));
                let _e183 = textureSampleLevel(AD, db, vec2<f32>(_e180.x, 0f), 0f);
                let _e186 = textureSampleLevel(AD, db, vec2<f32>(_e180.y, 0f), 0f);
                let _e189 = textureSampleLevel(AD, db, vec2<f32>(_e180.z, 0f), 0f);
                let _e192 = textureSampleLevel(AD, db, vec2<f32>(_e180.w, 0f), 0f);
                let _e198 = (_e174 * 5.0959306f);
                phi_4323_ = (_e162 + (dot(vec4<f32>(_e183.x, _e186.x, _e189.x, _e192.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e198) * (_e198 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e171));
            }
            let _e207 = phi_4323_;
            phi_4331_ = (_e207 * sign(_e153.x));
        } else {
            let _e147 = T_1[0u];
            let _e149 = T_1[1u];
            phi_4331_ = min(min(_e147, abs(_e149)), 1f);
        }
        let _e212 = phi_4331_;
        phi_4330_ = _e212;
    }
    let _e227 = phi_4330_;
    let _e231 = u32(round(((_e227 * 2048f) + 65536f)));
    let _e232 = G0_1;
    let _e235 = ((_e232 << bitcast<u32>(17u)) | _e231);
    let _e238 = atomicMax((&T4_.v2_[_e135]), _e235);
    let _e240 = (_e238 >> bitcast<u32>(17u));
    if (_e240 == _e232) {
        let _e766 = T_1;
        if (_e766.y < 0f) {
            let _e773 = atomicAdd((&T4_.v2_[_e135]), ((_e231 + (_e238 - max(_e235, _e238))) - 65536u));
        }
        phi_5317_ = 0u;
        phi_5310_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e245 = ((f32((_e238 & 131071u)) * 0.00048828125f) + -32f);
        let _e248 = XC.v2_[_e240];
        phi_4333_ = _e245;
        if ((_e248.x & 768u) != 0u) {
            let _e252 = abs(_e245);
            phi_2113_ = kj;
            if kj {
                phi_2113_ = ((_e248.x & 512u) != 0u);
            }
            let _e256 = phi_2113_;
            phi_4334_ = _e252;
            if _e256 {
                phi_4334_ = (1f - abs(((fract((_e252 * 0.5f)) * 2f) + -1f)));
            }
            let _e264 = phi_4334_;
            phi_4333_ = _e264;
        }
        let _e266 = phi_4333_;
        let _e267 = clamp(_e266, 0f, 1f);
        phi_4337_ = _e267;
        if fj {
            let _e269 = (_e248.x >> bitcast<u32>(16u));
            phi_4338_ = _e267;
            if (_e269 != 0u) {
                let _e273 = m0_.v2_[_e135];
                if (_e269 == (_e273 >> bitcast<u32>(16i))) {
                    phi_4335_ = min(_e267, unpack2x16float(_e273).x);
                } else {
                    phi_4335_ = 0f;
                }
                let _e281 = phi_4335_;
                phi_4338_ = _e281;
            }
            let _e283 = phi_4338_;
            phi_4337_ = _e283;
        }
        let _e285 = phi_4337_;
        phi_2150_ = gj;
        if gj {
            phi_2150_ = ((_e248.x & 1024u) != 0u);
        }
        let _e289 = phi_2150_;
        phi_4342_ = _e285;
        if _e289 {
            let _e290 = (_e240 * 8u);
            let _e294 = JB.v2_[(_e290 + 2u)];
            let _e305 = JB.v2_[(_e290 + 3u)];
            let _e310 = _e305.zw;
            let _e312 = ((abs(((mat2x2<f32>(vec2<f32>(_e294.x, _e294.y), vec2<f32>(_e294.z, _e294.w)) * _e101) + _e305.xy)) * _e310) - _e310);
            phi_4342_ = min(_e285, clamp((min(_e312.x, _e312.y) + 0.5f), 0f, 1f));
        }
        let _e320 = phi_4342_;
        let _e321 = (_e248.x & 15u);
        let _e324 = ((_e248.x >> bitcast<u32>(4i)) & 15u);
        let _e326 = (hj && (_e324 != 0u));
        if (_e321 <= 1u) {
            let _e407 = (fj && (_e321 == 0u));
            phi_5286_ = 0u;
            if _e407 {
                phi_5286_ = (_e248.y | pack2x16float(vec2<f32>(_e320, 0f)));
            }
            let _e412 = phi_5286_;
            phi_5285_ = _e412;
            phi_4372_ = select(unpack4x8unorm(_e248.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e407));
        } else {
            let _e328 = (_e240 * 8u);
            let _e331 = JB.v2_[_e328];
            let _e342 = JB.v2_[(_e328 + 1u)];
            let _e345 = ((mat2x2<f32>(vec2<f32>(_e331.x, _e331.y), vec2<f32>(_e331.z, _e331.w)) * _e101) + _e342.xy);
            let _e349 = f32(_e321);
            let _e351 = j.P8_;
            let _e353 = j.Q8_;
            let _e358 = ((_e342.w * _e351) + _e353);
            switch bitcast<i32>(0u) {
                default: {
                    if (oj && (_e349 == 4f)) {
                        let _e376 = normalize(vec2<f32>(_e345.x, -(_e345.y)));
                        let _e379 = acos(clamp(_e376.x, -1f, 1f));
                        if (_e376.y >= 0f) {
                            phi_4340_ = _e379;
                        } else {
                            phi_4340_ = -(_e379);
                        }
                        let _e384 = phi_4340_;
                        phi_4341_ = vec2<f32>((_e384 * 0.15915494f), _e358);
                        break;
                    } else {
                        if (_e349 == 2f) {
                            phi_4339_ = _e345.x;
                        } else {
                            phi_4339_ = length(_e345);
                        }
                        let _e366 = phi_4339_;
                        phi_4341_ = vec2<f32>(((clamp(_e366, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e342.z < 0f))) + ((max(0f, _e342.z) * 0.001953125f) + 0.0009765625f)), _e358);
                        break;
                    }
                }
            }
            let _e388 = phi_4341_;
            let _e389 = textureSampleLevel(ZC, R8_, _e388, 0f);
            phi_4374_ = _e389;
            if !(_e326) {
                let _e393 = (_e389.xyz * _e389.w);
                phi_4374_ = vec4<f32>(_e393.x, _e393.y, _e393.z, (_e389.w * abs(bitcast<f32>(_e248.y))));
            }
            let _e403 = phi_4374_;
            phi_5285_ = 0u;
            phi_4372_ = _e403;
        }
        let _e416 = phi_5285_;
        let _e418 = phi_4372_;
        phi_5283_ = _e418;
        if _e326 {
            phi_5270_ = _e418;
            if ((_e418.w * _e320) != 0f) {
                let _e424 = n0_.v2_[_e135];
                let _e425 = unpack4x8unorm(_e424);
                let _e426 = _e418.xyz;
                local_2 = _e426;
                let _e427 = _e425.xyz;
                if (_e425.w != 0f) {
                    phi_4381_ = (1f / _e425.w);
                } else {
                    phi_4381_ = 0f;
                }
                let _e432 = phi_4381_;
                let _e433 = (_e427 * _e432);
                local = _e433;
                switch bitcast<i32>(_e324) {
                    case 11: {
                        let _e717 = local_2;
                        local_1 = (_e717 * _e433);
                        break;
                    }
                    case 1: {
                        let _e713 = local_2;
                        local_1 = ((_e713 + _e433) - (_e713 * _e433));
                        break;
                    }
                    case 2: {
                        let _e705 = local_2;
                        let _e706 = (_e705 * _e433);
                        local_1 = (select(_e706, (((_e705 + _e433) - _e706) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e433 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e703 = local_2;
                        local_1 = min(_e703, _e433);
                        break;
                    }
                    case 4: {
                        let _e701 = local_2;
                        local_1 = max(_e701, _e433);
                        break;
                    }
                    case 5: {
                        let _e672 = clamp(_e427, vec3<f32>(0f, 0f, 0f), _e425.www);
                        let _e678 = vec4<f32>(_e672.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e684 = vec4<f32>(_e678.x, _e672.y, _e678.z, _e678.w);
                        let _e691 = local_2;
                        let _e694 = (clamp((vec3<f32>(1f, 1f, 1f) - _e691), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e425.w);
                        let _e695 = vec4<f32>(_e684.x, _e684.y, _e672.z, _e684.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e695 / _e694)), sign(_e695), (_e694 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e633 = local_2;
                        local_2 = clamp(_e633, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e636 = clamp(_e427, vec3<f32>(0f, 0f, 0f), _e425.www);
                        let _e642 = vec4<f32>(_e636.x, _e425.y, _e425.z, _e425.w);
                        let _e648 = vec4<f32>(_e642.x, _e636.y, _e642.z, _e642.w);
                        phi_5059_ = vec4<f32>(_e648.x, _e648.y, _e636.z, _e648.w);
                        if (_e425.w == 0f) {
                            phi_5059_ = vec4<f32>(_e636.x, _e636.y, _e636.z, 1f);
                        }
                        let _e658 = phi_5059_;
                        let _e662 = (vec3(_e658.w) - _e658.xyz);
                        let _e663 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e662 / (_e663 * _e658.w))), sign(_e662), (_e663 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e625 = local_2;
                        let _e626 = (_e625 * _e433);
                        local_1 = (select(_e626, (((_e625 + _e433) - _e626) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e625 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4971_ = 0i;
                        loop {
                            let _e594 = phi_4971_;
                            if (_e594 < 3i) {
                                let _e597 = local_2[_e594];
                                if (_e597 <= 0.5f) {
                                    let _e614 = local[_e594];
                                    local_1[_e594] = (1f - _e614);
                                } else {
                                    let _e600 = local[_e594];
                                    if (_e600 <= 0.25f) {
                                        let _e606 = local[_e594];
                                        let _e609 = local[_e594];
                                        local_1[_e594] = ((((16f * _e606) - 12f) * _e609) + 3f);
                                    } else {
                                        let _e602 = local[_e594];
                                        local_1[_e594] = (inverseSqrt(_e602) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4971_ = (_e594 + 1i);
                            }
                        }
                        let _e618 = local_2;
                        let _e622 = local_1;
                        local_1 = (_e433 + ((_e433 * ((_e618 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e622));
                        break;
                    }
                    case 9: {
                        let _e590 = local_2;
                        local_1 = abs((_e433 - _e590));
                        break;
                    }
                    case 10: {
                        let _e585 = local_2;
                        local_1 = ((_e585 + _e433) - ((_e585 * 2f) * _e433));
                        break;
                    }
                    case 12: {
                        if mj {
                            let _e536 = local_2;
                            let _e537 = clamp(_e536, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e537;
                            let _e552 = (_e537 - vec3(min(min(_e537.x, _e537.y), _e537.z)));
                            let _e560 = (_e552 * ((max(max(_e433.x, _e433.y), _e433.z) - min(min(_e433.x, _e433.y), _e433.z)) / max(0.000062f, max(max(_e552.x, _e552.y), _e552.z))));
                            let _e561 = dot(_e433, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e564 = (_e560 - vec3(dot(_e560, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e577 = (vec2<f32>(_e561, (1f - _e561)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e564.x, _e564.y), _e564.z)), max(max(_e564.x, _e564.y), _e564.z))));
                            local_1 = ((_e564 * min(1f, min(_e577.x, _e577.y))) + vec3(_e561));
                        }
                        break;
                    }
                    case 13: {
                        if mj {
                            let _e487 = local_2;
                            let _e488 = clamp(_e487, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e488;
                            let _e503 = (_e433 - vec3(min(min(_e433.x, _e433.y), _e433.z)));
                            let _e511 = (_e503 * ((max(max(_e488.x, _e488.y), _e488.z) - min(min(_e488.x, _e488.y), _e488.z)) / max(0.000062f, max(max(_e503.x, _e503.y), _e503.z))));
                            let _e512 = dot(_e433, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e515 = (_e511 - vec3(dot(_e511, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e528 = (vec2<f32>(_e512, (1f - _e512)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e515.x, _e515.y), _e515.z)), max(max(_e515.x, _e515.y), _e515.z))));
                            local_1 = ((_e515 * min(1f, min(_e528.x, _e528.y))) + vec3(_e512));
                        }
                        break;
                    }
                    case 14: {
                        if mj {
                            let _e461 = local_2;
                            let _e462 = clamp(_e461, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e462;
                            let _e463 = dot(_e433, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e466 = (_e462 - vec3(dot(_e462, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e479 = (vec2<f32>(_e463, (1f - _e463)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e466.x, _e466.y), _e466.z)), max(max(_e466.x, _e466.y), _e466.z))));
                            local_1 = ((_e466 * min(1f, min(_e479.x, _e479.y))) + vec3(_e463));
                        }
                        break;
                    }
                    case 15: {
                        if mj {
                            let _e435 = local_2;
                            let _e436 = clamp(_e435, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e436;
                            let _e437 = dot(_e436, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e440 = (_e433 - vec3(dot(_e433, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e453 = (vec2<f32>(_e437, (1f - _e437)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e440.x, _e440.y), _e440.z)), max(max(_e440.x, _e440.y), _e440.z))));
                            local_1 = ((_e440 * min(1f, min(_e453.x, _e453.y))) + vec3(_e437));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e719 = local_1;
                let _e721 = mix(_e426, _e719, vec3(_e425.w));
                let _e727 = vec4<f32>(_e721.x, _e418.y, _e418.z, _e418.w);
                let _e733 = vec4<f32>(_e727.x, _e721.y, _e727.z, _e727.w);
                phi_5270_ = vec4<f32>(_e733.x, _e733.y, _e721.z, _e733.w);
            }
            let _e741 = phi_5270_;
            let _e744 = (_e741.xyz * _e741.w);
            let _e750 = vec4<f32>(_e744.x, _e741.y, _e741.z, _e741.w);
            let _e756 = vec4<f32>(_e750.x, _e744.y, _e750.z, _e750.w);
            phi_5283_ = vec4<f32>(_e756.x, _e756.y, _e744.z, _e756.w);
        }
        let _e764 = phi_5283_;
        phi_5317_ = _e416;
        phi_5310_ = (_e764 * _e320);
    }
    let _e775 = phi_5317_;
    let _e777 = phi_5310_;
    let _e778 = _e777.xyz;
    let _e781 = j.G3_;
    let _e783 = j.H3_;
    if (nj && (_e777.w != 0f)) {
        phi_5312_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e100.x) + (0.00583715f * _e100.y))))) * _e781) + _e783)) + _e778);
    } else {
        phi_5312_ = _e778;
    }
    let _e799 = phi_5312_;
    let _e805 = vec4<f32>(_e799.x, _e777.y, _e777.z, _e777.w);
    let _e811 = vec4<f32>(_e805.x, _e799.y, _e805.z, _e805.w);
    let _e817 = vec4<f32>(_e811.x, _e811.y, _e799.z, _e811.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e799.x + _e799.y) + _e799.z) + _e777.w) == 0f) {
                break;
            }
            phi_5314_ = _e817;
            if (_e777.w != 1f) {
                let _e826 = n0_.v2_[_e135];
                phi_5314_ = (_e817 + (unpack4x8unorm(_e826) * (1f - _e777.w)));
            }
            let _e832 = phi_5314_;
            n0_.v2_[_e135] = pack4x8unorm(_e832);
            break;
        }
    }
    if (_e775 != 0u) {
        m0_.v2_[_e135] = _e775;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) T: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    T_1 = T;
    G0_1 = G0_;
    main_1();
}
