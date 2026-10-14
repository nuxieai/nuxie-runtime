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
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe;
var<private> G0_1: u32;
@group(0) @binding(10)
var HD: texture_2d<f32>;
@group(3) @binding(10)
var Pa: sampler;
var<private> S2_1: vec2<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1610_: bool;
    var phi_3777_: f32;
    var phi_3776_: f32;
    var phi_3778_: f32;
    var phi_3781_: f32;
    var phi_3780_: f32;
    var phi_1647_: bool;
    var phi_3785_: f32;
    var phi_3782_: f32;
    var phi_3783_: f32;
    var phi_3784_: vec2<f32>;
    var phi_3805_: vec4<f32>;
    var phi_4471_: u32;
    var phi_4470_: u32;
    var phi_3803_: vec4<f32>;
    var phi_3812_: f32;
    var phi_4234_: i32;
    var phi_4298_: vec4<f32>;
    var phi_4455_: vec4<f32>;
    var phi_4468_: vec4<f32>;
    var phi_4495_: vec3<f32>;
    var phi_4497_: vec4<f32>;

    let _e85 = gl_FragCoord_1;
    let _e86 = _e85.xy;
    let _e89 = bitcast<vec2<u32>>(vec2<i32>(floor(_e86)));
    let _e91 = j.P6_;
    let _e120 = bitcast<i32>((((((_e89.y >> bitcast<u32>(5u)) * (((_e91 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e89.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e89.x & 28u) << bitcast<u32>(5u)) + ((_e89.y & 28u) << bitcast<u32>(2i)))) + (((_e89.y & 3u) << bitcast<u32>(2i)) + (_e89.x & 3u))));
    let _e123 = R4_.r2_[_e120];
    let _e125 = (_e123 >> bitcast<u32>(17u));
    let _e126 = G0_1;
    let _e130 = S2_1;
    let _e131 = textureSampleLevel(HD, Pa, _e130, 0f);
    R4_.r2_[_e120] = (((_e126 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e131.x, 0f, 1f) * 2048f)))));
    let _e142 = ((f32((_e123 & 131071u)) * 0.00048828125f) + -32f);
    let _e145 = VC.r2_[_e125];
    phi_3776_ = _e142;
    if ((_e145.x & 768u) != 0u) {
        let _e149 = abs(_e142);
        phi_1610_ = bj;
        if bj {
            phi_1610_ = ((_e145.x & 512u) != 0u);
        }
        let _e153 = phi_1610_;
        phi_3777_ = _e149;
        if _e153 {
            phi_3777_ = (1f - abs(((fract((_e149 * 0.5f)) * 2f) + -1f)));
        }
        let _e161 = phi_3777_;
        phi_3776_ = _e161;
    }
    let _e163 = phi_3776_;
    let _e164 = clamp(_e163, 0f, 1f);
    phi_3780_ = _e164;
    if Xi {
        let _e166 = (_e145.x >> bitcast<u32>(16u));
        phi_3781_ = _e164;
        if (_e166 != 0u) {
            let _e170 = m0_.r2_[_e120];
            if (_e166 == (_e170 >> bitcast<u32>(16i))) {
                phi_3778_ = min(_e164, unpack2x16float(_e170).x);
            } else {
                phi_3778_ = 0f;
            }
            let _e178 = phi_3778_;
            phi_3781_ = _e178;
        }
        let _e180 = phi_3781_;
        phi_3780_ = _e180;
    }
    let _e182 = phi_3780_;
    phi_1647_ = Yi;
    if Yi {
        phi_1647_ = ((_e145.x & 1024u) != 0u);
    }
    let _e186 = phi_1647_;
    phi_3785_ = _e182;
    if _e186 {
        let _e187 = (_e125 * 8u);
        let _e191 = JB.r2_[(_e187 + 2u)];
        let _e202 = JB.r2_[(_e187 + 3u)];
        let _e207 = _e202.zw;
        let _e209 = ((abs(((mat2x2<f32>(vec2<f32>(_e191.x, _e191.y), vec2<f32>(_e191.z, _e191.w)) * _e86) + _e202.xy)) * _e207) - _e207);
        phi_3785_ = min(_e182, clamp((min(_e209.x, _e209.y) + 0.5f), 0f, 1f));
    }
    let _e217 = phi_3785_;
    let _e218 = (_e145.x & 15u);
    let _e221 = ((_e145.x >> bitcast<u32>(4i)) & 15u);
    let _e223 = (Zi && (_e221 != 0u));
    if (_e218 <= 1u) {
        let _e303 = (Xi && (_e218 == 0u));
        phi_4471_ = 0u;
        if _e303 {
            phi_4471_ = (_e145.y | pack2x16float(vec2<f32>(_e217, 0f)));
        }
        let _e308 = phi_4471_;
        phi_4470_ = _e308;
        phi_3803_ = select(unpack4x8unorm(_e145.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e303));
    } else {
        let _e225 = (_e125 * 8u);
        let _e228 = JB.r2_[_e225];
        let _e239 = JB.r2_[(_e225 + 1u)];
        let _e242 = ((mat2x2<f32>(vec2<f32>(_e228.x, _e228.y), vec2<f32>(_e228.z, _e228.w)) * _e86) + _e239.xy);
        let _e246 = f32(_e218);
        let _e248 = j.M8_;
        let _e250 = j.N8_;
        let _e255 = ((_e239.w * _e248) + _e250);
        switch bitcast<i32>(0u) {
            default: {
                if (_e246 == 4f) {
                    let _e272 = normalize(vec2<f32>(_e242.x, -(_e242.y)));
                    let _e275 = acos(clamp(_e272.x, -1f, 1f));
                    if (_e272.y >= 0f) {
                        phi_3783_ = _e275;
                    } else {
                        phi_3783_ = -(_e275);
                    }
                    let _e280 = phi_3783_;
                    phi_3784_ = vec2<f32>((_e280 * 0.15915507f), _e255);
                    break;
                } else {
                    if (_e246 == 2f) {
                        phi_3782_ = _e242.x;
                    } else {
                        phi_3782_ = length(_e242);
                    }
                    let _e262 = phi_3782_;
                    phi_3784_ = vec2<f32>(((clamp(_e262, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e239.z < 0f))) + ((max(0f, _e239.z) * 0.001953125f) + 0.0009765625f)), _e255);
                    break;
                }
            }
        }
        let _e284 = phi_3784_;
        let _e285 = textureSampleLevel(XC, O8_, _e284, 0f);
        phi_3805_ = _e285;
        if !(_e223) {
            let _e289 = (_e285.xyz * _e285.w);
            phi_3805_ = vec4<f32>(_e289.x, _e289.y, _e289.z, (_e285.w * abs(bitcast<f32>(_e145.y))));
        }
        let _e299 = phi_3805_;
        phi_4470_ = 0u;
        phi_3803_ = _e299;
    }
    let _e312 = phi_4470_;
    let _e314 = phi_3803_;
    phi_4468_ = _e314;
    if _e223 {
        phi_4455_ = _e314;
        if ((_e314.w * _e217) != 0f) {
            let _e320 = n0_.r2_[_e120];
            let _e321 = unpack4x8unorm(_e320);
            let _e322 = _e314.xyz;
            local_2 = _e322;
            let _e323 = _e321.xyz;
            if (_e321.w != 0f) {
                phi_3812_ = (1f / _e321.w);
            } else {
                phi_3812_ = 0f;
            }
            let _e328 = phi_3812_;
            let _e329 = (_e323 * _e328);
            local = _e329;
            switch bitcast<i32>(_e221) {
                case 11: {
                    let _e613 = local_2;
                    local_1 = (_e613 * _e329);
                    break;
                }
                case 1: {
                    let _e609 = local_2;
                    local_1 = ((_e609 + _e329) - (_e609 * _e329));
                    break;
                }
                case 2: {
                    let _e601 = local_2;
                    let _e602 = (_e601 * _e329);
                    local_1 = (select(_e602, (((_e601 + _e329) - _e602) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e329 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e599 = local_2;
                    local_1 = min(_e599, _e329);
                    break;
                }
                case 4: {
                    let _e597 = local_2;
                    local_1 = max(_e597, _e329);
                    break;
                }
                case 5: {
                    let _e568 = clamp(_e323, vec3<f32>(0f, 0f, 0f), _e321.www);
                    let _e574 = vec4<f32>(_e568.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e580 = vec4<f32>(_e574.x, _e568.y, _e574.z, _e574.w);
                    let _e587 = local_2;
                    let _e590 = (clamp((vec3<f32>(1f, 1f, 1f) - _e587), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e321.w);
                    let _e591 = vec4<f32>(_e580.x, _e580.y, _e568.z, _e580.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e591 / _e590)), sign(_e591), (_e590 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e529 = local_2;
                    local_2 = clamp(_e529, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e532 = clamp(_e323, vec3<f32>(0f, 0f, 0f), _e321.www);
                    let _e538 = vec4<f32>(_e532.x, _e321.y, _e321.z, _e321.w);
                    let _e544 = vec4<f32>(_e538.x, _e532.y, _e538.z, _e538.w);
                    phi_4298_ = vec4<f32>(_e544.x, _e544.y, _e532.z, _e544.w);
                    if (_e321.w == 0f) {
                        phi_4298_ = vec4<f32>(_e532.x, _e532.y, _e532.z, 1f);
                    }
                    let _e554 = phi_4298_;
                    let _e558 = (vec3(_e554.w) - _e554.xyz);
                    let _e559 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e558 / (_e559 * _e554.w))), sign(_e558), (_e559 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e521 = local_2;
                    let _e522 = (_e521 * _e329);
                    local_1 = (select(_e522, (((_e521 + _e329) - _e522) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e521 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4234_ = 0i;
                    loop {
                        let _e490 = phi_4234_;
                        if (_e490 < 3i) {
                            let _e493 = local_2[_e490];
                            if (_e493 <= 0.5f) {
                                let _e510 = local[_e490];
                                local_1[_e490] = (1f - _e510);
                            } else {
                                let _e496 = local[_e490];
                                if (_e496 <= 0.25f) {
                                    let _e502 = local[_e490];
                                    let _e505 = local[_e490];
                                    local_1[_e490] = ((((16f * _e502) - 12f) * _e505) + 3f);
                                } else {
                                    let _e498 = local[_e490];
                                    local_1[_e490] = (inverseSqrt(_e498) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4234_ = (_e490 + 1i);
                        }
                    }
                    let _e514 = local_2;
                    let _e518 = local_1;
                    local_1 = (_e329 + ((_e329 * ((_e514 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e518));
                    break;
                }
                case 9: {
                    let _e486 = local_2;
                    local_1 = abs((_e329 - _e486));
                    break;
                }
                case 10: {
                    let _e481 = local_2;
                    local_1 = ((_e481 + _e329) - ((_e481 * 2f) * _e329));
                    break;
                }
                case 12: {
                    if dj {
                        let _e432 = local_2;
                        let _e433 = clamp(_e432, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e433;
                        let _e448 = (_e433 - vec3(min(min(_e433.x, _e433.y), _e433.z)));
                        let _e456 = (_e448 * ((max(max(_e329.x, _e329.y), _e329.z) - min(min(_e329.x, _e329.y), _e329.z)) / max(0.000062f, max(max(_e448.x, _e448.y), _e448.z))));
                        let _e457 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e460 = (_e456 - vec3(dot(_e456, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e473 = (vec2<f32>(_e457, (1f - _e457)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e460.x, _e460.y), _e460.z)), max(max(_e460.x, _e460.y), _e460.z))));
                        local_1 = ((_e460 * min(1f, min(_e473.x, _e473.y))) + vec3(_e457));
                    }
                    break;
                }
                case 13: {
                    if dj {
                        let _e383 = local_2;
                        let _e384 = clamp(_e383, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e384;
                        let _e399 = (_e329 - vec3(min(min(_e329.x, _e329.y), _e329.z)));
                        let _e407 = (_e399 * ((max(max(_e384.x, _e384.y), _e384.z) - min(min(_e384.x, _e384.y), _e384.z)) / max(0.000062f, max(max(_e399.x, _e399.y), _e399.z))));
                        let _e408 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e411 = (_e407 - vec3(dot(_e407, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e424 = (vec2<f32>(_e408, (1f - _e408)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e411.x, _e411.y), _e411.z)), max(max(_e411.x, _e411.y), _e411.z))));
                        local_1 = ((_e411 * min(1f, min(_e424.x, _e424.y))) + vec3(_e408));
                    }
                    break;
                }
                case 14: {
                    if dj {
                        let _e357 = local_2;
                        let _e358 = clamp(_e357, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e358;
                        let _e359 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e362 = (_e358 - vec3(dot(_e358, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e375 = (vec2<f32>(_e359, (1f - _e359)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e362.x, _e362.y), _e362.z)), max(max(_e362.x, _e362.y), _e362.z))));
                        local_1 = ((_e362 * min(1f, min(_e375.x, _e375.y))) + vec3(_e359));
                    }
                    break;
                }
                case 15: {
                    if dj {
                        let _e331 = local_2;
                        let _e332 = clamp(_e331, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e332;
                        let _e333 = dot(_e332, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e336 = (_e329 - vec3(dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e349 = (vec2<f32>(_e333, (1f - _e333)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e336.x, _e336.y), _e336.z)), max(max(_e336.x, _e336.y), _e336.z))));
                        local_1 = ((_e336 * min(1f, min(_e349.x, _e349.y))) + vec3(_e333));
                    }
                    break;
                }
                default: {
                }
            }
            let _e615 = local_1;
            let _e617 = mix(_e322, _e615, vec3(_e321.w));
            let _e623 = vec4<f32>(_e617.x, _e314.y, _e314.z, _e314.w);
            let _e629 = vec4<f32>(_e623.x, _e617.y, _e623.z, _e623.w);
            phi_4455_ = vec4<f32>(_e629.x, _e629.y, _e617.z, _e629.w);
        }
        let _e637 = phi_4455_;
        let _e640 = (_e637.xyz * _e637.w);
        let _e646 = vec4<f32>(_e640.x, _e637.y, _e637.z, _e637.w);
        let _e652 = vec4<f32>(_e646.x, _e640.y, _e646.z, _e646.w);
        phi_4468_ = vec4<f32>(_e652.x, _e652.y, _e640.z, _e652.w);
    }
    let _e660 = phi_4468_;
    let _e661 = (_e660 * _e217);
    let _e662 = _e661.xyz;
    let _e665 = j.F3_;
    let _e667 = j.G3_;
    if (ej && (_e661.w != 0f)) {
        phi_4495_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e85.x) + (0.00583715f * _e85.y))))) * _e665) + _e667)) + _e662);
    } else {
        phi_4495_ = _e662;
    }
    let _e683 = phi_4495_;
    let _e689 = vec4<f32>(_e683.x, _e661.y, _e661.z, _e661.w);
    let _e695 = vec4<f32>(_e689.x, _e683.y, _e689.z, _e689.w);
    let _e701 = vec4<f32>(_e695.x, _e695.y, _e683.z, _e695.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e683.x + _e683.y) + _e683.z) + _e661.w) == 0f) {
                break;
            }
            let _e707 = (1f - _e661.w);
            phi_4497_ = _e701;
            if (_e707 != 0f) {
                let _e711 = n0_.r2_[_e120];
                phi_4497_ = (_e701 + (unpack4x8unorm(_e711) * _e707));
            }
            let _e716 = phi_4497_;
            n0_.r2_[_e120] = pack4x8unorm(_e716);
            break;
        }
    }
    if (_e312 != 0u) {
        m0_.r2_[_e120] = _e312;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) S2_: vec2<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    S2_1 = S2_;
    main_1();
}
