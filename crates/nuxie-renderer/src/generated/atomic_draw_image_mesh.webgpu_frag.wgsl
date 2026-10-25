struct ug {
    v2_: array<vec2<u32>>,
}

struct m0cf {
    v2_: array<u32>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

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

struct n0cf {
    v2_: array<u32>,
}

struct W4cf {
    v2_: array<u32>,
}

@id(8) override rj: bool = true;
@id(7) override qj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

@group(0) @binding(3)
var<storage> XC: ug;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(4)
var<storage> JB: vg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0cf;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;
var<private> n2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> V3_1: u32;
var<private> Y1_1: vec4<f32>;
var<private> N1_1: u32;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6283_: f32;
    var phi_1705_: bool;
    var phi_5510_: f32;
    var phi_5509_: f32;
    var phi_5511_: f32;
    var phi_5514_: f32;
    var phi_5513_: f32;
    var phi_1742_: bool;
    var phi_5518_: f32;
    var phi_5515_: f32;
    var phi_5516_: f32;
    var phi_5517_: vec2<f32>;
    var phi_5541_: vec4<f32>;
    var phi_6248_: u32;
    var phi_6247_: u32;
    var phi_5539_: vec4<f32>;
    var phi_5548_: f32;
    var phi_5998_: i32;
    var phi_6066_: vec4<f32>;
    var phi_6232_: vec4<f32>;
    var phi_6245_: vec4<f32>;
    var phi_1433_: bool;
    var phi_6273_: u32;
    var phi_6303_: f32;
    var phi_7766_: f32;
    var phi_1465_: bool;
    var phi_6341_: f32;
    var phi_6342_: f32;
    var phi_7295_: i32;
    var phi_7435_: vec4<f32>;
    var phi_7778_: vec4<f32>;
    var phi_7791_: vec3<f32>;
    var phi_7793_: vec4<f32>;

    let _e89 = gl_FragCoord_1;
    let _e90 = _e89.xy;
    let _e93 = bitcast<vec2<u32>>(vec2<i32>(floor(_e90)));
    let _e95 = j.T6_;
    let _e124 = bitcast<i32>((((((_e93.y >> bitcast<u32>(5u)) * (((_e95 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e93.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e93.x & 28u) << bitcast<u32>(5u)) + ((_e93.y & 28u) << bitcast<u32>(2i)))) + (((_e93.y & 3u) << bitcast<u32>(2i)) + (_e93.x & 3u))));
    let _e125 = n2_1;
    let _e126 = textureSample(RB, v4_, _e125);
    phi_6283_ = 1f;
    if kj {
        let _e127 = W0_1;
        let _e130 = min(_e127.xy, _e127.zw);
        phi_6283_ = clamp(min(_e130.x, _e130.y), 0f, 1f);
    }
    let _e136 = phi_6283_;
    let _e139 = W4_.v2_[_e124];
    let _e141 = (_e139 >> bitcast<u32>(17u));
    let _e145 = ((f32((_e139 & 131071u)) * 0.00048828125f) + -32f);
    let _e148 = XC.v2_[_e141];
    phi_5509_ = _e145;
    if ((_e148.x & 768u) != 0u) {
        let _e152 = abs(_e145);
        phi_1705_ = oj;
        if oj {
            phi_1705_ = ((_e148.x & 512u) != 0u);
        }
        let _e156 = phi_1705_;
        phi_5510_ = _e152;
        if _e156 {
            phi_5510_ = (1f - abs(((fract((_e152 * 0.5f)) * 2f) + -1f)));
        }
        let _e164 = phi_5510_;
        phi_5509_ = _e164;
    }
    let _e166 = phi_5509_;
    let _e167 = clamp(_e166, 0f, 1f);
    phi_5513_ = _e167;
    if jj {
        let _e169 = (_e148.x >> bitcast<u32>(16u));
        phi_5514_ = _e167;
        if (_e169 != 0u) {
            let _e173 = m0_.v2_[_e124];
            if (_e169 == (_e173 >> bitcast<u32>(16i))) {
                phi_5511_ = min(_e167, unpack2x16float(_e173).x);
            } else {
                phi_5511_ = 0f;
            }
            let _e181 = phi_5511_;
            phi_5514_ = _e181;
        }
        let _e183 = phi_5514_;
        phi_5513_ = _e183;
    }
    let _e185 = phi_5513_;
    phi_1742_ = kj;
    if kj {
        phi_1742_ = ((_e148.x & 1024u) != 0u);
    }
    let _e189 = phi_1742_;
    phi_5518_ = _e185;
    if _e189 {
        let _e190 = (_e141 * 8u);
        let _e194 = JB.v2_[(_e190 + 2u)];
        let _e205 = JB.v2_[(_e190 + 3u)];
        let _e210 = _e205.zw;
        let _e212 = ((abs(((mat2x2<f32>(vec2<f32>(_e194.x, _e194.y), vec2<f32>(_e194.z, _e194.w)) * _e90) + _e205.xy)) * _e210) - _e210);
        phi_5518_ = min(_e185, clamp((min(_e212.x, _e212.y) + 0.5f), 0f, 1f));
    }
    let _e220 = phi_5518_;
    let _e221 = (_e148.x & 15u);
    let _e224 = ((_e148.x >> bitcast<u32>(4i)) & 15u);
    let _e226 = (lj && (_e224 != 0u));
    if (_e221 <= 1u) {
        let _e303 = (jj && (_e221 == 0u));
        phi_6248_ = 0u;
        if _e303 {
            phi_6248_ = (_e148.y | pack2x16float(vec2<f32>(_e220, 0f)));
        }
        let _e308 = phi_6248_;
        phi_6247_ = _e308;
        phi_5539_ = select(unpack4x8unorm(_e148.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e303));
    } else {
        let _e228 = (_e141 * 8u);
        let _e231 = JB.v2_[_e228];
        let _e242 = JB.v2_[(_e228 + 1u)];
        let _e245 = ((mat2x2<f32>(vec2<f32>(_e231.x, _e231.y), vec2<f32>(_e231.z, _e231.w)) * _e90) + _e242.xy);
        let _e249 = f32(_e221);
        let _e251 = j.Q8_;
        let _e253 = j.R8_;
        let _e258 = ((_e242.w * _e251) + _e253);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e249 == 4f)) {
                    let _e272 = normalize(_e245);
                    let _e275 = acos(clamp(_e272.x, -1f, 1f));
                    if (_e272.y >= 0f) {
                        phi_5516_ = _e275;
                    } else {
                        phi_5516_ = -(_e275);
                    }
                    let _e280 = phi_5516_;
                    phi_5517_ = vec2<f32>((_e280 * 0.15915494f), _e258);
                    break;
                } else {
                    if (_e249 == 2f) {
                        phi_5515_ = _e245.x;
                    } else {
                        phi_5515_ = length(_e245);
                    }
                    let _e266 = phi_5515_;
                    phi_5517_ = vec2<f32>(((clamp(_e266, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e242.z < 0f))) + ((max(0f, _e242.z) * 0.001953125f) + 0.0009765625f)), _e258);
                    break;
                }
            }
        }
        let _e284 = phi_5517_;
        let _e285 = textureSampleLevel(ZC, S8_, _e284, 0f);
        phi_5541_ = _e285;
        if !(_e226) {
            let _e289 = (_e285.xyz * _e285.w);
            phi_5541_ = vec4<f32>(_e289.x, _e289.y, _e289.z, (_e285.w * abs(bitcast<f32>(_e148.y))));
        }
        let _e299 = phi_5541_;
        phi_6247_ = 0u;
        phi_5539_ = _e299;
    }
    let _e312 = phi_6247_;
    let _e314 = phi_5539_;
    phi_6245_ = _e314;
    if _e226 {
        phi_6232_ = _e314;
        if ((_e314.w * _e220) != 0f) {
            let _e320 = n0_.v2_[_e124];
            let _e321 = unpack4x8unorm(_e320);
            let _e322 = _e314.xyz;
            local_5 = _e322;
            let _e323 = _e321.xyz;
            if (_e321.w != 0f) {
                phi_5548_ = (1f / _e321.w);
            } else {
                phi_5548_ = 0f;
            }
            let _e328 = phi_5548_;
            let _e329 = (_e323 * _e328);
            local_3 = _e329;
            switch bitcast<i32>(_e224) {
                case 11: {
                    let _e613 = local_5;
                    local_4 = (_e613 * _e329);
                    break;
                }
                case 1: {
                    let _e609 = local_5;
                    local_4 = ((_e609 + _e329) - (_e609 * _e329));
                    break;
                }
                case 2: {
                    let _e601 = local_5;
                    let _e602 = (_e601 * _e329);
                    local_4 = (select(_e602, (((_e601 + _e329) - _e602) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e329 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e599 = local_5;
                    local_4 = min(_e599, _e329);
                    break;
                }
                case 4: {
                    let _e597 = local_5;
                    local_4 = max(_e597, _e329);
                    break;
                }
                case 5: {
                    let _e568 = clamp(_e323, vec3<f32>(0f, 0f, 0f), _e321.www);
                    let _e574 = vec4<f32>(_e568.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e580 = vec4<f32>(_e574.x, _e568.y, _e574.z, _e574.w);
                    let _e587 = local_5;
                    let _e590 = (clamp((vec3<f32>(1f, 1f, 1f) - _e587), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e321.w);
                    let _e591 = vec4<f32>(_e580.x, _e580.y, _e568.z, _e580.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e591 / _e590)), sign(_e591), (_e590 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e529 = local_5;
                    local_5 = clamp(_e529, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e532 = clamp(_e323, vec3<f32>(0f, 0f, 0f), _e321.www);
                    let _e538 = vec4<f32>(_e532.x, _e321.y, _e321.z, _e321.w);
                    let _e544 = vec4<f32>(_e538.x, _e532.y, _e538.z, _e538.w);
                    phi_6066_ = vec4<f32>(_e544.x, _e544.y, _e532.z, _e544.w);
                    if (_e321.w == 0f) {
                        phi_6066_ = vec4<f32>(_e532.x, _e532.y, _e532.z, 1f);
                    }
                    let _e554 = phi_6066_;
                    let _e558 = (vec3(_e554.w) - _e554.xyz);
                    let _e559 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e558 / (_e559 * _e554.w))), sign(_e558), (_e559 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e521 = local_5;
                    let _e522 = (_e521 * _e329);
                    local_4 = (select(_e522, (((_e521 + _e329) - _e522) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e521 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_5998_ = 0i;
                    loop {
                        let _e490 = phi_5998_;
                        if (_e490 < 3i) {
                            let _e493 = local_5[_e490];
                            if (_e493 <= 0.5f) {
                                let _e510 = local_3[_e490];
                                local_4[_e490] = (1f - _e510);
                            } else {
                                let _e496 = local_3[_e490];
                                if (_e496 <= 0.25f) {
                                    let _e502 = local_3[_e490];
                                    let _e505 = local_3[_e490];
                                    local_4[_e490] = ((((16f * _e502) - 12f) * _e505) + 3f);
                                } else {
                                    let _e498 = local_3[_e490];
                                    local_4[_e490] = (inverseSqrt(_e498) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_5998_ = (_e490 + 1i);
                        }
                    }
                    let _e514 = local_5;
                    let _e518 = local_4;
                    local_4 = (_e329 + ((_e329 * ((_e514 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e518));
                    break;
                }
                case 9: {
                    let _e486 = local_5;
                    local_4 = abs((_e329 - _e486));
                    break;
                }
                case 10: {
                    let _e481 = local_5;
                    local_4 = ((_e481 + _e329) - ((_e481 * 2f) * _e329));
                    break;
                }
                case 12: {
                    if qj {
                        let _e432 = local_5;
                        let _e433 = clamp(_e432, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e433;
                        let _e448 = (_e433 - vec3(min(min(_e433.x, _e433.y), _e433.z)));
                        let _e456 = (_e448 * ((max(max(_e329.x, _e329.y), _e329.z) - min(min(_e329.x, _e329.y), _e329.z)) / max(0.000062f, max(max(_e448.x, _e448.y), _e448.z))));
                        let _e457 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e460 = (_e456 - vec3(dot(_e456, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e473 = (vec2<f32>(_e457, (1f - _e457)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e460.x, _e460.y), _e460.z)), max(max(_e460.x, _e460.y), _e460.z))));
                        local_4 = ((_e460 * min(1f, min(_e473.x, _e473.y))) + vec3(_e457));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e383 = local_5;
                        let _e384 = clamp(_e383, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e384;
                        let _e399 = (_e329 - vec3(min(min(_e329.x, _e329.y), _e329.z)));
                        let _e407 = (_e399 * ((max(max(_e384.x, _e384.y), _e384.z) - min(min(_e384.x, _e384.y), _e384.z)) / max(0.000062f, max(max(_e399.x, _e399.y), _e399.z))));
                        let _e408 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e411 = (_e407 - vec3(dot(_e407, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e424 = (vec2<f32>(_e408, (1f - _e408)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e411.x, _e411.y), _e411.z)), max(max(_e411.x, _e411.y), _e411.z))));
                        local_4 = ((_e411 * min(1f, min(_e424.x, _e424.y))) + vec3(_e408));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e357 = local_5;
                        let _e358 = clamp(_e357, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e358;
                        let _e359 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e362 = (_e358 - vec3(dot(_e358, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e375 = (vec2<f32>(_e359, (1f - _e359)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e362.x, _e362.y), _e362.z)), max(max(_e362.x, _e362.y), _e362.z))));
                        local_4 = ((_e362 * min(1f, min(_e375.x, _e375.y))) + vec3(_e359));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e331 = local_5;
                        let _e332 = clamp(_e331, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e332;
                        let _e333 = dot(_e332, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e336 = (_e329 - vec3(dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e349 = (vec2<f32>(_e333, (1f - _e333)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e336.x, _e336.y), _e336.z)), max(max(_e336.x, _e336.y), _e336.z))));
                        local_4 = ((_e336 * min(1f, min(_e349.x, _e349.y))) + vec3(_e333));
                    }
                    break;
                }
                default: {
                }
            }
            let _e615 = local_4;
            let _e617 = mix(_e322, _e615, vec3(_e321.w));
            let _e623 = vec4<f32>(_e617.x, _e314.y, _e314.z, _e314.w);
            let _e629 = vec4<f32>(_e623.x, _e617.y, _e623.z, _e623.w);
            phi_6232_ = vec4<f32>(_e629.x, _e629.y, _e617.z, _e629.w);
        }
        let _e637 = phi_6232_;
        let _e640 = (_e637.xyz * _e637.w);
        let _e646 = vec4<f32>(_e640.x, _e637.y, _e637.z, _e637.w);
        let _e652 = vec4<f32>(_e646.x, _e640.y, _e646.z, _e646.w);
        phi_6245_ = vec4<f32>(_e652.x, _e652.y, _e640.z, _e652.w);
    }
    let _e660 = phi_6245_;
    let _e661 = (_e660 * _e220);
    phi_1433_ = jj;
    if jj {
        let _e662 = V3_1;
        phi_1433_ = (_e662 != 0u);
    }
    let _e665 = phi_1433_;
    phi_7766_ = _e136;
    if _e665 {
        if (_e312 != 0u) {
            phi_6273_ = _e312;
        } else {
            let _e669 = m0_.v2_[_e124];
            phi_6273_ = _e669;
        }
        let _e671 = phi_6273_;
        let _e672 = V3_1;
        if (_e672 == (_e671 >> bitcast<u32>(16i))) {
            phi_6303_ = min(_e136, unpack2x16float(_e671).x);
        } else {
            phi_6303_ = 0f;
        }
        let _e680 = phi_6303_;
        phi_7766_ = _e680;
    }
    let _e682 = phi_7766_;
    let _e683 = Y1_1;
    let _e684 = (_e126 * _e683);
    phi_1465_ = lj;
    if lj {
        let _e685 = N1_1;
        phi_1465_ = (_e685 != 0u);
    }
    let _e688 = phi_1465_;
    phi_7778_ = _e684;
    if _e688 {
        let _e691 = n0_.v2_[_e124];
        let _e696 = ((unpack4x8unorm(_e691) * (1f - _e661.w)) + _e661);
        if (_e684.w != 0f) {
            phi_6341_ = (1f / _e684.w);
        } else {
            phi_6341_ = 0f;
        }
        let _e702 = phi_6341_;
        let _e703 = (_e684.xyz * _e702);
        let _e704 = N1_1;
        local_2 = _e703;
        let _e705 = _e696.xyz;
        if (_e696.w != 0f) {
            phi_6342_ = (1f / _e696.w);
        } else {
            phi_6342_ = 0f;
        }
        let _e710 = phi_6342_;
        let _e711 = (_e705 * _e710);
        local = _e711;
        switch bitcast<i32>(_e704) {
            case 11: {
                let _e995 = local_2;
                local_1 = (_e995 * _e711);
                break;
            }
            case 1: {
                let _e991 = local_2;
                local_1 = ((_e991 + _e711) - (_e991 * _e711));
                break;
            }
            case 2: {
                let _e983 = local_2;
                let _e984 = (_e983 * _e711);
                local_1 = (select(_e984, (((_e983 + _e711) - _e984) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e711 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e981 = local_2;
                local_1 = min(_e981, _e711);
                break;
            }
            case 4: {
                let _e979 = local_2;
                local_1 = max(_e979, _e711);
                break;
            }
            case 5: {
                let _e950 = clamp(_e705, vec3<f32>(0f, 0f, 0f), _e696.www);
                let _e956 = vec4<f32>(_e950.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e962 = vec4<f32>(_e956.x, _e950.y, _e956.z, _e956.w);
                let _e969 = local_2;
                let _e972 = (clamp((vec3<f32>(1f, 1f, 1f) - _e969), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e696.w);
                let _e973 = vec4<f32>(_e962.x, _e962.y, _e950.z, _e962.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e973 / _e972)), sign(_e973), (_e972 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e911 = local_2;
                local_2 = clamp(_e911, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e914 = clamp(_e705, vec3<f32>(0f, 0f, 0f), _e696.www);
                let _e920 = vec4<f32>(_e914.x, _e696.y, _e696.z, _e696.w);
                let _e926 = vec4<f32>(_e920.x, _e914.y, _e920.z, _e920.w);
                phi_7435_ = vec4<f32>(_e926.x, _e926.y, _e914.z, _e926.w);
                if (_e696.w == 0f) {
                    phi_7435_ = vec4<f32>(_e914.x, _e914.y, _e914.z, 1f);
                }
                let _e936 = phi_7435_;
                let _e940 = (vec3(_e936.w) - _e936.xyz);
                let _e941 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e940 / (_e941 * _e936.w))), sign(_e940), (_e941 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e903 = local_2;
                let _e904 = (_e903 * _e711);
                local_1 = (select(_e904, (((_e903 + _e711) - _e904) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e903 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7295_ = 0i;
                loop {
                    let _e872 = phi_7295_;
                    if (_e872 < 3i) {
                        let _e875 = local_2[_e872];
                        if (_e875 <= 0.5f) {
                            let _e892 = local[_e872];
                            local_1[_e872] = (1f - _e892);
                        } else {
                            let _e878 = local[_e872];
                            if (_e878 <= 0.25f) {
                                let _e884 = local[_e872];
                                let _e887 = local[_e872];
                                local_1[_e872] = ((((16f * _e884) - 12f) * _e887) + 3f);
                            } else {
                                let _e880 = local[_e872];
                                local_1[_e872] = (inverseSqrt(_e880) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7295_ = (_e872 + 1i);
                    }
                }
                let _e896 = local_2;
                let _e900 = local_1;
                local_1 = (_e711 + ((_e711 * ((_e896 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e900));
                break;
            }
            case 9: {
                let _e868 = local_2;
                local_1 = abs((_e711 - _e868));
                break;
            }
            case 10: {
                let _e863 = local_2;
                local_1 = ((_e863 + _e711) - ((_e863 * 2f) * _e711));
                break;
            }
            case 12: {
                if qj {
                    let _e814 = local_2;
                    let _e815 = clamp(_e814, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e815;
                    let _e830 = (_e815 - vec3(min(min(_e815.x, _e815.y), _e815.z)));
                    let _e838 = (_e830 * ((max(max(_e711.x, _e711.y), _e711.z) - min(min(_e711.x, _e711.y), _e711.z)) / max(0.000062f, max(max(_e830.x, _e830.y), _e830.z))));
                    let _e839 = dot(_e711, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e842 = (_e838 - vec3(dot(_e838, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e855 = (vec2<f32>(_e839, (1f - _e839)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e842.x, _e842.y), _e842.z)), max(max(_e842.x, _e842.y), _e842.z))));
                    local_1 = ((_e842 * min(1f, min(_e855.x, _e855.y))) + vec3(_e839));
                }
                break;
            }
            case 13: {
                if qj {
                    let _e765 = local_2;
                    let _e766 = clamp(_e765, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e766;
                    let _e781 = (_e711 - vec3(min(min(_e711.x, _e711.y), _e711.z)));
                    let _e789 = (_e781 * ((max(max(_e766.x, _e766.y), _e766.z) - min(min(_e766.x, _e766.y), _e766.z)) / max(0.000062f, max(max(_e781.x, _e781.y), _e781.z))));
                    let _e790 = dot(_e711, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e793 = (_e789 - vec3(dot(_e789, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e806 = (vec2<f32>(_e790, (1f - _e790)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e793.x, _e793.y), _e793.z)), max(max(_e793.x, _e793.y), _e793.z))));
                    local_1 = ((_e793 * min(1f, min(_e806.x, _e806.y))) + vec3(_e790));
                }
                break;
            }
            case 14: {
                if qj {
                    let _e739 = local_2;
                    let _e740 = clamp(_e739, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e740;
                    let _e741 = dot(_e711, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e744 = (_e740 - vec3(dot(_e740, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e757 = (vec2<f32>(_e741, (1f - _e741)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e744.x, _e744.y), _e744.z)), max(max(_e744.x, _e744.y), _e744.z))));
                    local_1 = ((_e744 * min(1f, min(_e757.x, _e757.y))) + vec3(_e741));
                }
                break;
            }
            case 15: {
                if qj {
                    let _e713 = local_2;
                    let _e714 = clamp(_e713, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e714;
                    let _e715 = dot(_e714, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e718 = (_e711 - vec3(dot(_e711, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e731 = (vec2<f32>(_e715, (1f - _e715)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e718.x, _e718.y), _e718.z)), max(max(_e718.x, _e718.y), _e718.z))));
                    local_1 = ((_e718 * min(1f, min(_e731.x, _e731.y))) + vec3(_e715));
                }
                break;
            }
            default: {
            }
        }
        let _e997 = local_1;
        let _e1000 = (mix(_e703, _e997, vec3(_e696.w)) * _e684.w);
        let _e1006 = vec4<f32>(_e1000.x, _e684.y, _e684.z, _e684.w);
        let _e1012 = vec4<f32>(_e1006.x, _e1000.y, _e1006.z, _e1006.w);
        phi_7778_ = vec4<f32>(_e1012.x, _e1012.y, _e1000.z, _e1012.w);
    }
    let _e1020 = phi_7778_;
    let _e1021 = (_e1020 * _e682);
    let _e1025 = ((_e661 * (1f - _e1021.w)) + _e1021);
    let _e1026 = _e1025.xyz;
    let _e1029 = j.H3_;
    let _e1031 = j.I3_;
    if (rj && (_e1025.w != 0f)) {
        phi_7791_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e89.x) + (0.00583715f * _e89.y))))) * _e1029) + _e1031)) + _e1026);
    } else {
        phi_7791_ = _e1026;
    }
    let _e1047 = phi_7791_;
    let _e1053 = vec4<f32>(_e1047.x, _e1025.y, _e1025.z, _e1025.w);
    let _e1059 = vec4<f32>(_e1053.x, _e1047.y, _e1053.z, _e1053.w);
    let _e1065 = vec4<f32>(_e1059.x, _e1059.y, _e1047.z, _e1059.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1047.x + _e1047.y) + _e1047.z) + _e1025.w) == 0f) {
                break;
            }
            phi_7793_ = _e1065;
            if (_e1025.w != 1f) {
                let _e1074 = n0_.v2_[_e124];
                phi_7793_ = (_e1065 + (unpack4x8unorm(_e1074) * (1f - _e1025.w)));
            }
            let _e1080 = phi_7793_;
            n0_.v2_[_e124] = pack4x8unorm(_e1080);
            break;
        }
    }
    if (_e312 != 0u) {
        m0_.v2_[_e124] = _e312;
    }
    W4_.v2_[_e124] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) n2_: vec2<f32>, @location(1) W0_: vec4<f32>, @location(4) @interpolate(flat, either) V3_: u32, @location(3) @interpolate(flat, either) Y1_: vec4<f32>, @location(5) @interpolate(flat, either) N1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    n2_1 = n2_;
    W0_1 = W0_;
    V3_1 = V3_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
}
