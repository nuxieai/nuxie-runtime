struct jg {
    r2_: array<vec2<u32>>,
}

struct m0Pe {
    r2_: array<u32>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

struct VB {
    xd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    F8_: vec4<i32>,
    zi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Di: f32,
    w6_: u32,
    i3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    wi: u32,
    Ba: u32,
    M8_: f32,
    N8_: f32,
}

struct n0Pe {
    r2_: array<u32>,
}

struct R4Pe {
    r2_: array<u32>,
}

@id(7) override ej: bool = true;
@id(6) override dj: bool = true;
@id(4) override bj: bool = true;
@id(0) override Xi: bool = true;
@id(1) override Yi: bool = true;
@id(2) override Zi: bool = true;

@group(0) @binding(3)
var<storage> VC: jg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var O8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Pe;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
var<private> l2_1: vec2<f32>;
var<private> r5_1: f32;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe;
var<private> S3_1: u32;
var<private> v5_1: vec4<f32>;
var<private> T1_1: vec4<f32>;
var<private> J1_1: u32;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6514_: f32;
    var phi_1789_: bool;
    var phi_5741_: f32;
    var phi_5740_: f32;
    var phi_5742_: f32;
    var phi_5745_: f32;
    var phi_5744_: f32;
    var phi_1826_: bool;
    var phi_5749_: f32;
    var phi_5746_: f32;
    var phi_5747_: f32;
    var phi_5748_: vec2<f32>;
    var phi_5772_: vec4<f32>;
    var phi_6479_: u32;
    var phi_6478_: u32;
    var phi_5770_: vec4<f32>;
    var phi_5779_: f32;
    var phi_6229_: i32;
    var phi_6297_: vec4<f32>;
    var phi_6463_: vec4<f32>;
    var phi_6476_: vec4<f32>;
    var phi_1481_: bool;
    var phi_6504_: u32;
    var phi_6534_: f32;
    var phi_8204_: f32;
    var phi_6566_: f32;
    var phi_6567_: f32;
    var phi_6568_: vec2<f32>;
    var phi_6603_: vec4<f32>;
    var phi_1549_: bool;
    var phi_6614_: f32;
    var phi_6615_: f32;
    var phi_7680_: i32;
    var phi_7836_: vec4<f32>;
    var phi_8219_: vec4<f32>;
    var phi_8232_: vec3<f32>;
    var phi_8234_: vec4<f32>;

    let _e91 = gl_FragCoord_1;
    let _e92 = _e91.xy;
    let _e95 = bitcast<vec2<u32>>(vec2<i32>(floor(_e92)));
    let _e97 = j.P6_;
    let _e126 = bitcast<i32>((((((_e95.y >> bitcast<u32>(5u)) * (((_e97 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e95.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e95.x & 28u) << bitcast<u32>(5u)) + ((_e95.y & 28u) << bitcast<u32>(2i)))) + (((_e95.y & 3u) << bitcast<u32>(2i)) + (_e95.x & 3u))));
    let _e127 = l2_1;
    let _e128 = textureSample(TB, U4_, _e127);
    let _e129 = r5_1;
    let _e130 = min(_e129, 1f);
    phi_6514_ = _e130;
    if Yi {
        let _e131 = V0_1;
        let _e134 = min(_e131.xy, _e131.zw);
        phi_6514_ = clamp(min(_e134.x, _e134.y), 0f, _e130);
    }
    let _e140 = phi_6514_;
    let _e143 = R4_.r2_[_e126];
    let _e145 = (_e143 >> bitcast<u32>(17u));
    let _e149 = ((f32((_e143 & 131071u)) * 0.00048828125f) + -32f);
    let _e152 = VC.r2_[_e145];
    phi_5740_ = _e149;
    if ((_e152.x & 768u) != 0u) {
        let _e156 = abs(_e149);
        phi_1789_ = bj;
        if bj {
            phi_1789_ = ((_e152.x & 512u) != 0u);
        }
        let _e160 = phi_1789_;
        phi_5741_ = _e156;
        if _e160 {
            phi_5741_ = (1f - abs(((fract((_e156 * 0.5f)) * 2f) + -1f)));
        }
        let _e168 = phi_5741_;
        phi_5740_ = _e168;
    }
    let _e170 = phi_5740_;
    let _e171 = clamp(_e170, 0f, 1f);
    phi_5744_ = _e171;
    if Xi {
        let _e173 = (_e152.x >> bitcast<u32>(16u));
        phi_5745_ = _e171;
        if (_e173 != 0u) {
            let _e177 = m0_.r2_[_e126];
            if (_e173 == (_e177 >> bitcast<u32>(16i))) {
                phi_5742_ = min(_e171, unpack2x16float(_e177).x);
            } else {
                phi_5742_ = 0f;
            }
            let _e185 = phi_5742_;
            phi_5745_ = _e185;
        }
        let _e187 = phi_5745_;
        phi_5744_ = _e187;
    }
    let _e189 = phi_5744_;
    phi_1826_ = Yi;
    if Yi {
        phi_1826_ = ((_e152.x & 1024u) != 0u);
    }
    let _e193 = phi_1826_;
    phi_5749_ = _e189;
    if _e193 {
        let _e194 = (_e145 * 8u);
        let _e198 = JB.r2_[(_e194 + 2u)];
        let _e209 = JB.r2_[(_e194 + 3u)];
        let _e214 = _e209.zw;
        let _e216 = ((abs(((mat2x2<f32>(vec2<f32>(_e198.x, _e198.y), vec2<f32>(_e198.z, _e198.w)) * _e92) + _e209.xy)) * _e214) - _e214);
        phi_5749_ = min(_e189, clamp((min(_e216.x, _e216.y) + 0.5f), 0f, 1f));
    }
    let _e224 = phi_5749_;
    let _e225 = (_e152.x & 15u);
    let _e228 = ((_e152.x >> bitcast<u32>(4i)) & 15u);
    let _e230 = (Zi && (_e228 != 0u));
    if (_e225 <= 1u) {
        let _e310 = (Xi && (_e225 == 0u));
        phi_6479_ = 0u;
        if _e310 {
            phi_6479_ = (_e152.y | pack2x16float(vec2<f32>(_e224, 0f)));
        }
        let _e315 = phi_6479_;
        phi_6478_ = _e315;
        phi_5770_ = select(unpack4x8unorm(_e152.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e310));
    } else {
        let _e232 = (_e145 * 8u);
        let _e235 = JB.r2_[_e232];
        let _e246 = JB.r2_[(_e232 + 1u)];
        let _e249 = ((mat2x2<f32>(vec2<f32>(_e235.x, _e235.y), vec2<f32>(_e235.z, _e235.w)) * _e92) + _e246.xy);
        let _e253 = f32(_e225);
        let _e255 = j.M8_;
        let _e257 = j.N8_;
        let _e262 = ((_e246.w * _e255) + _e257);
        switch bitcast<i32>(0u) {
            default: {
                if (_e253 == 4f) {
                    let _e279 = normalize(vec2<f32>(_e249.x, -(_e249.y)));
                    let _e282 = acos(clamp(_e279.x, -1f, 1f));
                    if (_e279.y >= 0f) {
                        phi_5747_ = _e282;
                    } else {
                        phi_5747_ = -(_e282);
                    }
                    let _e287 = phi_5747_;
                    phi_5748_ = vec2<f32>((_e287 * 0.15915507f), _e262);
                    break;
                } else {
                    if (_e253 == 2f) {
                        phi_5746_ = _e249.x;
                    } else {
                        phi_5746_ = length(_e249);
                    }
                    let _e269 = phi_5746_;
                    phi_5748_ = vec2<f32>(((clamp(_e269, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e246.z < 0f))) + ((max(0f, _e246.z) * 0.001953125f) + 0.0009765625f)), _e262);
                    break;
                }
            }
        }
        let _e291 = phi_5748_;
        let _e292 = textureSampleLevel(XC, O8_, _e291, 0f);
        phi_5772_ = _e292;
        if !(_e230) {
            let _e296 = (_e292.xyz * _e292.w);
            phi_5772_ = vec4<f32>(_e296.x, _e296.y, _e296.z, (_e292.w * abs(bitcast<f32>(_e152.y))));
        }
        let _e306 = phi_5772_;
        phi_6478_ = 0u;
        phi_5770_ = _e306;
    }
    let _e319 = phi_6478_;
    let _e321 = phi_5770_;
    phi_6476_ = _e321;
    if _e230 {
        phi_6463_ = _e321;
        if ((_e321.w * _e224) != 0f) {
            let _e327 = n0_.r2_[_e126];
            let _e328 = unpack4x8unorm(_e327);
            let _e329 = _e321.xyz;
            local_5 = _e329;
            let _e330 = _e328.xyz;
            if (_e328.w != 0f) {
                phi_5779_ = (1f / _e328.w);
            } else {
                phi_5779_ = 0f;
            }
            let _e335 = phi_5779_;
            let _e336 = (_e330 * _e335);
            local_3 = _e336;
            switch bitcast<i32>(_e228) {
                case 11: {
                    let _e620 = local_5;
                    local_4 = (_e620 * _e336);
                    break;
                }
                case 1: {
                    let _e616 = local_5;
                    local_4 = ((_e616 + _e336) - (_e616 * _e336));
                    break;
                }
                case 2: {
                    let _e608 = local_5;
                    let _e609 = (_e608 * _e336);
                    local_4 = (select(_e609, (((_e608 + _e336) - _e609) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e336 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e606 = local_5;
                    local_4 = min(_e606, _e336);
                    break;
                }
                case 4: {
                    let _e604 = local_5;
                    local_4 = max(_e604, _e336);
                    break;
                }
                case 5: {
                    let _e575 = clamp(_e330, vec3<f32>(0f, 0f, 0f), _e328.www);
                    let _e581 = vec4<f32>(_e575.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e587 = vec4<f32>(_e581.x, _e575.y, _e581.z, _e581.w);
                    let _e594 = local_5;
                    let _e597 = (clamp((vec3<f32>(1f, 1f, 1f) - _e594), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e328.w);
                    let _e598 = vec4<f32>(_e587.x, _e587.y, _e575.z, _e587.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e598 / _e597)), sign(_e598), (_e597 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e536 = local_5;
                    local_5 = clamp(_e536, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e539 = clamp(_e330, vec3<f32>(0f, 0f, 0f), _e328.www);
                    let _e545 = vec4<f32>(_e539.x, _e328.y, _e328.z, _e328.w);
                    let _e551 = vec4<f32>(_e545.x, _e539.y, _e545.z, _e545.w);
                    phi_6297_ = vec4<f32>(_e551.x, _e551.y, _e539.z, _e551.w);
                    if (_e328.w == 0f) {
                        phi_6297_ = vec4<f32>(_e539.x, _e539.y, _e539.z, 1f);
                    }
                    let _e561 = phi_6297_;
                    let _e565 = (vec3(_e561.w) - _e561.xyz);
                    let _e566 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e565 / (_e566 * _e561.w))), sign(_e565), (_e566 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e528 = local_5;
                    let _e529 = (_e528 * _e336);
                    local_4 = (select(_e529, (((_e528 + _e336) - _e529) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e528 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_6229_ = 0i;
                    loop {
                        let _e497 = phi_6229_;
                        if (_e497 < 3i) {
                            let _e500 = local_5[_e497];
                            if (_e500 <= 0.5f) {
                                let _e517 = local_3[_e497];
                                local_4[_e497] = (1f - _e517);
                            } else {
                                let _e503 = local_3[_e497];
                                if (_e503 <= 0.25f) {
                                    let _e509 = local_3[_e497];
                                    let _e512 = local_3[_e497];
                                    local_4[_e497] = ((((16f * _e509) - 12f) * _e512) + 3f);
                                } else {
                                    let _e505 = local_3[_e497];
                                    local_4[_e497] = (inverseSqrt(_e505) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_6229_ = (_e497 + 1i);
                        }
                    }
                    let _e521 = local_5;
                    let _e525 = local_4;
                    local_4 = (_e336 + ((_e336 * ((_e521 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e525));
                    break;
                }
                case 9: {
                    let _e493 = local_5;
                    local_4 = abs((_e336 - _e493));
                    break;
                }
                case 10: {
                    let _e488 = local_5;
                    local_4 = ((_e488 + _e336) - ((_e488 * 2f) * _e336));
                    break;
                }
                case 12: {
                    if dj {
                        let _e439 = local_5;
                        let _e440 = clamp(_e439, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e440;
                        let _e455 = (_e440 - vec3(min(min(_e440.x, _e440.y), _e440.z)));
                        let _e463 = (_e455 * ((max(max(_e336.x, _e336.y), _e336.z) - min(min(_e336.x, _e336.y), _e336.z)) / max(0.000062f, max(max(_e455.x, _e455.y), _e455.z))));
                        let _e464 = dot(_e336, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e467 = (_e463 - vec3(dot(_e463, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e480 = (vec2<f32>(_e464, (1f - _e464)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e467.x, _e467.y), _e467.z)), max(max(_e467.x, _e467.y), _e467.z))));
                        local_4 = ((_e467 * min(1f, min(_e480.x, _e480.y))) + vec3(_e464));
                    }
                    break;
                }
                case 13: {
                    if dj {
                        let _e390 = local_5;
                        let _e391 = clamp(_e390, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e391;
                        let _e406 = (_e336 - vec3(min(min(_e336.x, _e336.y), _e336.z)));
                        let _e414 = (_e406 * ((max(max(_e391.x, _e391.y), _e391.z) - min(min(_e391.x, _e391.y), _e391.z)) / max(0.000062f, max(max(_e406.x, _e406.y), _e406.z))));
                        let _e415 = dot(_e336, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e418 = (_e414 - vec3(dot(_e414, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e431 = (vec2<f32>(_e415, (1f - _e415)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e418.x, _e418.y), _e418.z)), max(max(_e418.x, _e418.y), _e418.z))));
                        local_4 = ((_e418 * min(1f, min(_e431.x, _e431.y))) + vec3(_e415));
                    }
                    break;
                }
                case 14: {
                    if dj {
                        let _e364 = local_5;
                        let _e365 = clamp(_e364, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e365;
                        let _e366 = dot(_e336, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e369 = (_e365 - vec3(dot(_e365, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e382 = (vec2<f32>(_e366, (1f - _e366)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e369.x, _e369.y), _e369.z)), max(max(_e369.x, _e369.y), _e369.z))));
                        local_4 = ((_e369 * min(1f, min(_e382.x, _e382.y))) + vec3(_e366));
                    }
                    break;
                }
                case 15: {
                    if dj {
                        let _e338 = local_5;
                        let _e339 = clamp(_e338, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e339;
                        let _e340 = dot(_e339, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e343 = (_e336 - vec3(dot(_e336, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e356 = (vec2<f32>(_e340, (1f - _e340)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e343.x, _e343.y), _e343.z)), max(max(_e343.x, _e343.y), _e343.z))));
                        local_4 = ((_e343 * min(1f, min(_e356.x, _e356.y))) + vec3(_e340));
                    }
                    break;
                }
                default: {
                }
            }
            let _e622 = local_4;
            let _e624 = mix(_e329, _e622, vec3(_e328.w));
            let _e630 = vec4<f32>(_e624.x, _e321.y, _e321.z, _e321.w);
            let _e636 = vec4<f32>(_e630.x, _e624.y, _e630.z, _e630.w);
            phi_6463_ = vec4<f32>(_e636.x, _e636.y, _e624.z, _e636.w);
        }
        let _e644 = phi_6463_;
        let _e647 = (_e644.xyz * _e644.w);
        let _e653 = vec4<f32>(_e647.x, _e644.y, _e644.z, _e644.w);
        let _e659 = vec4<f32>(_e653.x, _e647.y, _e653.z, _e653.w);
        phi_6476_ = vec4<f32>(_e659.x, _e659.y, _e647.z, _e659.w);
    }
    let _e667 = phi_6476_;
    let _e668 = (_e667 * _e224);
    phi_1481_ = Xi;
    if Xi {
        let _e669 = S3_1;
        phi_1481_ = (_e669 != 0u);
    }
    let _e672 = phi_1481_;
    phi_8204_ = _e140;
    if _e672 {
        if (_e319 != 0u) {
            phi_6504_ = _e319;
        } else {
            let _e676 = m0_.r2_[_e126];
            phi_6504_ = _e676;
        }
        let _e678 = phi_6504_;
        let _e679 = S3_1;
        if (_e679 == (_e678 >> bitcast<u32>(16i))) {
            phi_6534_ = min(_e140, unpack2x16float(_e678).x);
        } else {
            phi_6534_ = 0f;
        }
        let _e687 = phi_6534_;
        phi_8204_ = _e687;
    }
    let _e689 = phi_8204_;
    let _e691 = v5_1[3u];
    phi_6603_ = _e128;
    if (_e691 != 0f) {
        let _e693 = v5_1;
        let _e695 = j.M8_;
        let _e697 = j.N8_;
        let _e700 = abs(_e693.z);
        let _e701 = floor(_e700);
        let _e704 = (((_e700 - _e701) * 8f) + 0.0009765625f);
        let _e706 = ((_e701 * _e695) + _e697);
        let _e707 = floor(_e704);
        switch bitcast<i32>(0u) {
            default: {
                if (_e707 == 4f) {
                    let _e725 = normalize(vec2<f32>(_e693.x, -(_e693.y)));
                    let _e728 = acos(clamp(_e725.x, -1f, 1f));
                    if (_e725.y >= 0f) {
                        phi_6567_ = _e728;
                    } else {
                        phi_6567_ = -(_e728);
                    }
                    let _e733 = phi_6567_;
                    phi_6568_ = vec2<f32>((_e733 * 0.15915507f), _e706);
                    break;
                } else {
                    if (_e707 == 2f) {
                        phi_6566_ = _e693.x;
                    } else {
                        phi_6566_ = length(_e693.xy);
                    }
                    let _e715 = phi_6566_;
                    phi_6568_ = vec2<f32>(((clamp(_e715, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e693.z < 0f))) + _e704), _e706);
                    break;
                }
            }
        }
        let _e737 = phi_6568_;
        let _e738 = textureSampleLevel(XC, O8_, _e737, 0f);
        let _e741 = (_e738.xyz * _e738.w);
        let _e747 = vec4<f32>(_e741.x, _e738.y, _e738.z, _e738.w);
        let _e753 = vec4<f32>(_e747.x, _e741.y, _e747.z, _e747.w);
        phi_6603_ = (_e128 * vec4<f32>(_e753.x, _e753.y, _e741.z, _e753.w));
    }
    let _e762 = phi_6603_;
    let _e763 = T1_1;
    let _e764 = (_e762 * _e763);
    phi_1549_ = Zi;
    if Zi {
        let _e765 = J1_1;
        phi_1549_ = (_e765 != 0u);
    }
    let _e768 = phi_1549_;
    phi_8219_ = _e764;
    if _e768 {
        let _e771 = n0_.r2_[_e126];
        let _e776 = ((unpack4x8unorm(_e771) * (1f - _e668.w)) + _e668);
        if (_e764.w != 0f) {
            phi_6614_ = (1f / _e764.w);
        } else {
            phi_6614_ = 0f;
        }
        let _e782 = phi_6614_;
        let _e783 = (_e764.xyz * _e782);
        let _e784 = J1_1;
        local_2 = _e783;
        let _e785 = _e776.xyz;
        if (_e776.w != 0f) {
            phi_6615_ = (1f / _e776.w);
        } else {
            phi_6615_ = 0f;
        }
        let _e790 = phi_6615_;
        let _e791 = (_e785 * _e790);
        local = _e791;
        switch bitcast<i32>(_e784) {
            case 11: {
                let _e1075 = local_2;
                local_1 = (_e1075 * _e791);
                break;
            }
            case 1: {
                let _e1071 = local_2;
                local_1 = ((_e1071 + _e791) - (_e1071 * _e791));
                break;
            }
            case 2: {
                let _e1063 = local_2;
                let _e1064 = (_e1063 * _e791);
                local_1 = (select(_e1064, (((_e1063 + _e791) - _e1064) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e791 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e1061 = local_2;
                local_1 = min(_e1061, _e791);
                break;
            }
            case 4: {
                let _e1059 = local_2;
                local_1 = max(_e1059, _e791);
                break;
            }
            case 5: {
                let _e1030 = clamp(_e785, vec3<f32>(0f, 0f, 0f), _e776.www);
                let _e1036 = vec4<f32>(_e1030.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e1042 = vec4<f32>(_e1036.x, _e1030.y, _e1036.z, _e1036.w);
                let _e1049 = local_2;
                let _e1052 = (clamp((vec3<f32>(1f, 1f, 1f) - _e1049), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e776.w);
                let _e1053 = vec4<f32>(_e1042.x, _e1042.y, _e1030.z, _e1042.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e1053 / _e1052)), sign(_e1053), (_e1052 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e991 = local_2;
                local_2 = clamp(_e991, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e994 = clamp(_e785, vec3<f32>(0f, 0f, 0f), _e776.www);
                let _e1000 = vec4<f32>(_e994.x, _e776.y, _e776.z, _e776.w);
                let _e1006 = vec4<f32>(_e1000.x, _e994.y, _e1000.z, _e1000.w);
                phi_7836_ = vec4<f32>(_e1006.x, _e1006.y, _e994.z, _e1006.w);
                if (_e776.w == 0f) {
                    phi_7836_ = vec4<f32>(_e994.x, _e994.y, _e994.z, 1f);
                }
                let _e1016 = phi_7836_;
                let _e1020 = (vec3(_e1016.w) - _e1016.xyz);
                let _e1021 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e1020 / (_e1021 * _e1016.w))), sign(_e1020), (_e1021 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e983 = local_2;
                let _e984 = (_e983 * _e791);
                local_1 = (select(_e984, (((_e983 + _e791) - _e984) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e983 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7680_ = 0i;
                loop {
                    let _e952 = phi_7680_;
                    if (_e952 < 3i) {
                        let _e955 = local_2[_e952];
                        if (_e955 <= 0.5f) {
                            let _e972 = local[_e952];
                            local_1[_e952] = (1f - _e972);
                        } else {
                            let _e958 = local[_e952];
                            if (_e958 <= 0.25f) {
                                let _e964 = local[_e952];
                                let _e967 = local[_e952];
                                local_1[_e952] = ((((16f * _e964) - 12f) * _e967) + 3f);
                            } else {
                                let _e960 = local[_e952];
                                local_1[_e952] = (inverseSqrt(_e960) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7680_ = (_e952 + 1i);
                    }
                }
                let _e976 = local_2;
                let _e980 = local_1;
                local_1 = (_e791 + ((_e791 * ((_e976 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e980));
                break;
            }
            case 9: {
                let _e948 = local_2;
                local_1 = abs((_e791 - _e948));
                break;
            }
            case 10: {
                let _e943 = local_2;
                local_1 = ((_e943 + _e791) - ((_e943 * 2f) * _e791));
                break;
            }
            case 12: {
                if dj {
                    let _e894 = local_2;
                    let _e895 = clamp(_e894, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e895;
                    let _e910 = (_e895 - vec3(min(min(_e895.x, _e895.y), _e895.z)));
                    let _e918 = (_e910 * ((max(max(_e791.x, _e791.y), _e791.z) - min(min(_e791.x, _e791.y), _e791.z)) / max(0.000062f, max(max(_e910.x, _e910.y), _e910.z))));
                    let _e919 = dot(_e791, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e922 = (_e918 - vec3(dot(_e918, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e935 = (vec2<f32>(_e919, (1f - _e919)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e922.x, _e922.y), _e922.z)), max(max(_e922.x, _e922.y), _e922.z))));
                    local_1 = ((_e922 * min(1f, min(_e935.x, _e935.y))) + vec3(_e919));
                }
                break;
            }
            case 13: {
                if dj {
                    let _e845 = local_2;
                    let _e846 = clamp(_e845, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e846;
                    let _e861 = (_e791 - vec3(min(min(_e791.x, _e791.y), _e791.z)));
                    let _e869 = (_e861 * ((max(max(_e846.x, _e846.y), _e846.z) - min(min(_e846.x, _e846.y), _e846.z)) / max(0.000062f, max(max(_e861.x, _e861.y), _e861.z))));
                    let _e870 = dot(_e791, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e873 = (_e869 - vec3(dot(_e869, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e886 = (vec2<f32>(_e870, (1f - _e870)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e873.x, _e873.y), _e873.z)), max(max(_e873.x, _e873.y), _e873.z))));
                    local_1 = ((_e873 * min(1f, min(_e886.x, _e886.y))) + vec3(_e870));
                }
                break;
            }
            case 14: {
                if dj {
                    let _e819 = local_2;
                    let _e820 = clamp(_e819, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e820;
                    let _e821 = dot(_e791, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e824 = (_e820 - vec3(dot(_e820, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e837 = (vec2<f32>(_e821, (1f - _e821)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e824.x, _e824.y), _e824.z)), max(max(_e824.x, _e824.y), _e824.z))));
                    local_1 = ((_e824 * min(1f, min(_e837.x, _e837.y))) + vec3(_e821));
                }
                break;
            }
            case 15: {
                if dj {
                    let _e793 = local_2;
                    let _e794 = clamp(_e793, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e794;
                    let _e795 = dot(_e794, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e798 = (_e791 - vec3(dot(_e791, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e811 = (vec2<f32>(_e795, (1f - _e795)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e798.x, _e798.y), _e798.z)), max(max(_e798.x, _e798.y), _e798.z))));
                    local_1 = ((_e798 * min(1f, min(_e811.x, _e811.y))) + vec3(_e795));
                }
                break;
            }
            default: {
            }
        }
        let _e1077 = local_1;
        let _e1080 = (mix(_e783, _e1077, vec3(_e776.w)) * _e764.w);
        let _e1086 = vec4<f32>(_e1080.x, _e764.y, _e764.z, _e764.w);
        let _e1092 = vec4<f32>(_e1086.x, _e1080.y, _e1086.z, _e1086.w);
        phi_8219_ = vec4<f32>(_e1092.x, _e1092.y, _e1080.z, _e1092.w);
    }
    let _e1100 = phi_8219_;
    let _e1101 = (_e1100 * _e689);
    let _e1105 = ((_e668 * (1f - _e1101.w)) + _e1101);
    let _e1106 = _e1105.xyz;
    let _e1109 = j.F3_;
    let _e1111 = j.G3_;
    if (ej && (_e1105.w != 0f)) {
        phi_8232_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e91.x) + (0.00583715f * _e91.y))))) * _e1109) + _e1111)) + _e1106);
    } else {
        phi_8232_ = _e1106;
    }
    let _e1127 = phi_8232_;
    let _e1133 = vec4<f32>(_e1127.x, _e1105.y, _e1105.z, _e1105.w);
    let _e1139 = vec4<f32>(_e1133.x, _e1127.y, _e1133.z, _e1133.w);
    let _e1145 = vec4<f32>(_e1139.x, _e1139.y, _e1127.z, _e1139.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1127.x + _e1127.y) + _e1127.z) + _e1105.w) == 0f) {
                break;
            }
            let _e1151 = (1f - _e1105.w);
            phi_8234_ = _e1145;
            if (_e1151 != 0f) {
                let _e1155 = n0_.r2_[_e126];
                phi_8234_ = (_e1145 + (unpack4x8unorm(_e1155) * _e1151));
            }
            let _e1160 = phi_8234_;
            n0_.r2_[_e126] = pack4x8unorm(_e1160);
            break;
        }
    }
    if (_e319 != 0u) {
        m0_.r2_[_e126] = _e319;
    }
    R4_.r2_[_e126] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) l2_: vec2<f32>, @location(1) r5_: f32, @location(3) V0_: vec4<f32>, @location(5) @interpolate(flat, either) S3_: u32, @location(2) v5_: vec4<f32>, @location(4) @interpolate(flat, either) T1_: vec4<f32>, @location(6) @interpolate(flat, either) J1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    l2_1 = l2_;
    r5_1 = r5_;
    V0_1 = V0_;
    S3_1 = S3_;
    v5_1 = v5_;
    T1_1 = T1_;
    J1_1 = J1_;
    main_1();
}
