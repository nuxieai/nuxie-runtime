struct jd {
    x2_: array<vec4<u32>>,
}

struct m0cf {
    x2_: array<u32>,
}

struct ug {
    x2_: array<vec4<f32>>,
}

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

struct o0cf {
    x2_: array<u32>,
}

struct W4cf {
    x2_: array<u32>,
}

@id(8) override rj: bool = true;
@id(7) override qj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

@group(0) @binding(2)
var<storage> JB: jd;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(3)
var<storage> KB: ug;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var Q8_: sampler;
@group(2) @binding(0)
var<storage, read_write> o0_: o0cf;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;
var<private> o2_1: vec2<f32>;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> V3_1: u32;
var<private> Y1_1: vec4<f32>;
var<private> N1_1: u32;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6291_: f32;
    var phi_1713_: bool;
    var phi_5518_: f32;
    var phi_5517_: f32;
    var phi_5519_: f32;
    var phi_5522_: f32;
    var phi_5521_: f32;
    var phi_1750_: bool;
    var phi_5526_: f32;
    var phi_5523_: f32;
    var phi_5524_: f32;
    var phi_5525_: vec2<f32>;
    var phi_5549_: vec4<f32>;
    var phi_6256_: u32;
    var phi_6255_: u32;
    var phi_5547_: vec4<f32>;
    var phi_5556_: f32;
    var phi_6006_: i32;
    var phi_6074_: vec4<f32>;
    var phi_6240_: vec4<f32>;
    var phi_6253_: vec4<f32>;
    var phi_1438_: bool;
    var phi_6281_: u32;
    var phi_6311_: f32;
    var phi_7774_: f32;
    var phi_1470_: bool;
    var phi_6349_: f32;
    var phi_6350_: f32;
    var phi_7303_: i32;
    var phi_7443_: vec4<f32>;
    var phi_7786_: vec4<f32>;
    var phi_7799_: vec3<f32>;
    var phi_7801_: vec4<f32>;

    let _e90 = gl_FragCoord_1;
    let _e91 = _e90.xy;
    let _e94 = bitcast<vec2<u32>>(vec2<i32>(floor(_e91)));
    let _e96 = j.R6_;
    let _e125 = bitcast<i32>((((((_e94.y >> bitcast<u32>(5u)) * (((_e96 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e94.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e94.x & 28u) << bitcast<u32>(5u)) + ((_e94.y & 28u) << bitcast<u32>(2i)))) + (((_e94.y & 3u) << bitcast<u32>(2i)) + (_e94.x & 3u))));
    let _e126 = o2_1;
    let _e127 = textureSample(RB, v4_, _e126);
    phi_6291_ = 1f;
    if kj {
        let _e128 = V0_1;
        let _e131 = min(_e128.xy, _e128.zw);
        phi_6291_ = clamp(min(_e131.x, _e131.y), 0f, 1f);
    }
    let _e137 = phi_6291_;
    let _e140 = W4_.x2_[_e125];
    let _e142 = (_e140 >> bitcast<u32>(17u));
    let _e146 = ((f32((_e140 & 131071u)) * 0.00048828125f) + -32f);
    let _e151 = JB.x2_[((_e142 * 4u) + 1u)];
    phi_5517_ = _e146;
    if ((_e151.z & 768u) != 0u) {
        let _e155 = abs(_e146);
        phi_1713_ = oj;
        if oj {
            phi_1713_ = ((_e151.z & 512u) != 0u);
        }
        let _e159 = phi_1713_;
        phi_5518_ = _e155;
        if _e159 {
            phi_5518_ = (1f - abs(((fract((_e155 * 0.5f)) * 2f) + -1f)));
        }
        let _e167 = phi_5518_;
        phi_5517_ = _e167;
    }
    let _e169 = phi_5517_;
    let _e170 = clamp(_e169, 0f, 1f);
    phi_5521_ = _e170;
    if jj {
        let _e172 = (_e151.z >> bitcast<u32>(16u));
        phi_5522_ = _e170;
        if (_e172 != 0u) {
            let _e176 = m0_.x2_[_e125];
            if (_e172 == (_e176 >> bitcast<u32>(16i))) {
                phi_5519_ = min(_e170, unpack2x16float(_e176).x);
            } else {
                phi_5519_ = 0f;
            }
            let _e184 = phi_5519_;
            phi_5522_ = _e184;
        }
        let _e186 = phi_5522_;
        phi_5521_ = _e186;
    }
    let _e188 = phi_5521_;
    phi_1750_ = kj;
    if kj {
        phi_1750_ = ((_e151.z & 1024u) != 0u);
    }
    let _e192 = phi_1750_;
    phi_5526_ = _e188;
    if _e192 {
        let _e193 = (_e142 * 8u);
        let _e197 = KB.x2_[(_e193 + 2u)];
        let _e208 = KB.x2_[(_e193 + 3u)];
        let _e213 = _e208.zw;
        let _e215 = ((abs(((mat2x2<f32>(vec2<f32>(_e197.x, _e197.y), vec2<f32>(_e197.z, _e197.w)) * _e91) + _e208.xy)) * _e213) - _e213);
        phi_5526_ = min(_e188, clamp((min(_e215.x, _e215.y) + 0.5f), 0f, 1f));
    }
    let _e223 = phi_5526_;
    let _e224 = (_e151.z & 15u);
    let _e227 = ((_e151.z >> bitcast<u32>(4i)) & 15u);
    let _e229 = (lj && (_e227 != 0u));
    if (_e224 <= 1u) {
        let _e306 = (jj && (_e224 == 0u));
        phi_6256_ = 0u;
        if _e306 {
            phi_6256_ = (_e151.w | pack2x16float(vec2<f32>(_e223, 0f)));
        }
        let _e311 = phi_6256_;
        phi_6255_ = _e311;
        phi_5547_ = select(unpack4x8unorm(_e151.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e306));
    } else {
        let _e231 = (_e142 * 8u);
        let _e234 = KB.x2_[_e231];
        let _e245 = KB.x2_[(_e231 + 1u)];
        let _e248 = ((mat2x2<f32>(vec2<f32>(_e234.x, _e234.y), vec2<f32>(_e234.z, _e234.w)) * _e91) + _e245.xy);
        let _e252 = f32(_e224);
        let _e254 = j.O8_;
        let _e256 = j.P8_;
        let _e261 = ((_e245.w * _e254) + _e256);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e252 == 4f)) {
                    let _e275 = normalize(_e248);
                    let _e278 = acos(clamp(_e275.x, -1f, 1f));
                    if (_e275.y >= 0f) {
                        phi_5524_ = _e278;
                    } else {
                        phi_5524_ = -(_e278);
                    }
                    let _e283 = phi_5524_;
                    phi_5525_ = vec2<f32>((_e283 * 0.15915494f), _e261);
                    break;
                } else {
                    if (_e252 == 2f) {
                        phi_5523_ = _e248.x;
                    } else {
                        phi_5523_ = length(_e248);
                    }
                    let _e269 = phi_5523_;
                    phi_5525_ = vec2<f32>(((clamp(_e269, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e245.z < 0f))) + ((max(0f, _e245.z) * 0.001953125f) + 0.0009765625f)), _e261);
                    break;
                }
            }
        }
        let _e287 = phi_5525_;
        let _e288 = textureSampleLevel(YC, Q8_, _e287, 0f);
        phi_5549_ = _e288;
        if !(_e229) {
            let _e292 = (_e288.xyz * _e288.w);
            phi_5549_ = vec4<f32>(_e292.x, _e292.y, _e292.z, (_e288.w * abs(bitcast<f32>(_e151.w))));
        }
        let _e302 = phi_5549_;
        phi_6255_ = 0u;
        phi_5547_ = _e302;
    }
    let _e315 = phi_6255_;
    let _e317 = phi_5547_;
    phi_6253_ = _e317;
    if _e229 {
        phi_6240_ = _e317;
        if ((_e317.w * _e223) != 0f) {
            let _e323 = o0_.x2_[_e125];
            let _e324 = unpack4x8unorm(_e323);
            let _e325 = _e317.xyz;
            local_5 = _e325;
            let _e326 = _e324.xyz;
            if (_e324.w != 0f) {
                phi_5556_ = (1f / _e324.w);
            } else {
                phi_5556_ = 0f;
            }
            let _e331 = phi_5556_;
            let _e332 = (_e326 * _e331);
            local_3 = _e332;
            switch bitcast<i32>(_e227) {
                case 11: {
                    let _e616 = local_5;
                    local_4 = (_e616 * _e332);
                    break;
                }
                case 1: {
                    let _e612 = local_5;
                    local_4 = ((_e612 + _e332) - (_e612 * _e332));
                    break;
                }
                case 2: {
                    let _e604 = local_5;
                    let _e605 = (_e604 * _e332);
                    local_4 = (select(_e605, (((_e604 + _e332) - _e605) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e332 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e602 = local_5;
                    local_4 = min(_e602, _e332);
                    break;
                }
                case 4: {
                    let _e600 = local_5;
                    local_4 = max(_e600, _e332);
                    break;
                }
                case 5: {
                    let _e571 = clamp(_e326, vec3<f32>(0f, 0f, 0f), _e324.www);
                    let _e577 = vec4<f32>(_e571.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e583 = vec4<f32>(_e577.x, _e571.y, _e577.z, _e577.w);
                    let _e590 = local_5;
                    let _e593 = (clamp((vec3<f32>(1f, 1f, 1f) - _e590), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e324.w);
                    let _e594 = vec4<f32>(_e583.x, _e583.y, _e571.z, _e583.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e594 / _e593)), sign(_e594), (_e593 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e532 = local_5;
                    local_5 = clamp(_e532, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e535 = clamp(_e326, vec3<f32>(0f, 0f, 0f), _e324.www);
                    let _e541 = vec4<f32>(_e535.x, _e324.y, _e324.z, _e324.w);
                    let _e547 = vec4<f32>(_e541.x, _e535.y, _e541.z, _e541.w);
                    phi_6074_ = vec4<f32>(_e547.x, _e547.y, _e535.z, _e547.w);
                    if (_e324.w == 0f) {
                        phi_6074_ = vec4<f32>(_e535.x, _e535.y, _e535.z, 1f);
                    }
                    let _e557 = phi_6074_;
                    let _e561 = (vec3(_e557.w) - _e557.xyz);
                    let _e562 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e561 / (_e562 * _e557.w))), sign(_e561), (_e562 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e524 = local_5;
                    let _e525 = (_e524 * _e332);
                    local_4 = (select(_e525, (((_e524 + _e332) - _e525) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e524 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_6006_ = 0i;
                    loop {
                        let _e493 = phi_6006_;
                        if (_e493 < 3i) {
                            let _e496 = local_5[_e493];
                            if (_e496 <= 0.5f) {
                                let _e513 = local_3[_e493];
                                local_4[_e493] = (1f - _e513);
                            } else {
                                let _e499 = local_3[_e493];
                                if (_e499 <= 0.25f) {
                                    let _e505 = local_3[_e493];
                                    let _e508 = local_3[_e493];
                                    local_4[_e493] = ((((16f * _e505) - 12f) * _e508) + 3f);
                                } else {
                                    let _e501 = local_3[_e493];
                                    local_4[_e493] = (inverseSqrt(_e501) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_6006_ = (_e493 + 1i);
                        }
                    }
                    let _e517 = local_5;
                    let _e521 = local_4;
                    local_4 = (_e332 + ((_e332 * ((_e517 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e521));
                    break;
                }
                case 9: {
                    let _e489 = local_5;
                    local_4 = abs((_e332 - _e489));
                    break;
                }
                case 10: {
                    let _e484 = local_5;
                    local_4 = ((_e484 + _e332) - ((_e484 * 2f) * _e332));
                    break;
                }
                case 12: {
                    if qj {
                        let _e435 = local_5;
                        let _e436 = clamp(_e435, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e436;
                        let _e451 = (_e436 - vec3(min(min(_e436.x, _e436.y), _e436.z)));
                        let _e459 = (_e451 * ((max(max(_e332.x, _e332.y), _e332.z) - min(min(_e332.x, _e332.y), _e332.z)) / max(0.000062f, max(max(_e451.x, _e451.y), _e451.z))));
                        let _e460 = dot(_e332, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e463 = (_e459 - vec3(dot(_e459, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e476 = (vec2<f32>(_e460, (1f - _e460)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e463.x, _e463.y), _e463.z)), max(max(_e463.x, _e463.y), _e463.z))));
                        local_4 = ((_e463 * min(1f, min(_e476.x, _e476.y))) + vec3(_e460));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e386 = local_5;
                        let _e387 = clamp(_e386, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e387;
                        let _e402 = (_e332 - vec3(min(min(_e332.x, _e332.y), _e332.z)));
                        let _e410 = (_e402 * ((max(max(_e387.x, _e387.y), _e387.z) - min(min(_e387.x, _e387.y), _e387.z)) / max(0.000062f, max(max(_e402.x, _e402.y), _e402.z))));
                        let _e411 = dot(_e332, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e414 = (_e410 - vec3(dot(_e410, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e427 = (vec2<f32>(_e411, (1f - _e411)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e414.x, _e414.y), _e414.z)), max(max(_e414.x, _e414.y), _e414.z))));
                        local_4 = ((_e414 * min(1f, min(_e427.x, _e427.y))) + vec3(_e411));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e360 = local_5;
                        let _e361 = clamp(_e360, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e361;
                        let _e362 = dot(_e332, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e365 = (_e361 - vec3(dot(_e361, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e378 = (vec2<f32>(_e362, (1f - _e362)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e365.x, _e365.y), _e365.z)), max(max(_e365.x, _e365.y), _e365.z))));
                        local_4 = ((_e365 * min(1f, min(_e378.x, _e378.y))) + vec3(_e362));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e334 = local_5;
                        let _e335 = clamp(_e334, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e335;
                        let _e336 = dot(_e335, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e339 = (_e332 - vec3(dot(_e332, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e352 = (vec2<f32>(_e336, (1f - _e336)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e339.x, _e339.y), _e339.z)), max(max(_e339.x, _e339.y), _e339.z))));
                        local_4 = ((_e339 * min(1f, min(_e352.x, _e352.y))) + vec3(_e336));
                    }
                    break;
                }
                default: {
                }
            }
            let _e618 = local_4;
            let _e620 = mix(_e325, _e618, vec3(_e324.w));
            let _e626 = vec4<f32>(_e620.x, _e317.y, _e317.z, _e317.w);
            let _e632 = vec4<f32>(_e626.x, _e620.y, _e626.z, _e626.w);
            phi_6240_ = vec4<f32>(_e632.x, _e632.y, _e620.z, _e632.w);
        }
        let _e640 = phi_6240_;
        let _e643 = (_e640.xyz * _e640.w);
        let _e649 = vec4<f32>(_e643.x, _e640.y, _e640.z, _e640.w);
        let _e655 = vec4<f32>(_e649.x, _e643.y, _e649.z, _e649.w);
        phi_6253_ = vec4<f32>(_e655.x, _e655.y, _e643.z, _e655.w);
    }
    let _e663 = phi_6253_;
    let _e664 = (_e663 * _e223);
    phi_1438_ = jj;
    if jj {
        let _e665 = V3_1;
        phi_1438_ = (_e665 != 0u);
    }
    let _e668 = phi_1438_;
    phi_7774_ = _e137;
    if _e668 {
        if (_e315 != 0u) {
            phi_6281_ = _e315;
        } else {
            let _e672 = m0_.x2_[_e125];
            phi_6281_ = _e672;
        }
        let _e674 = phi_6281_;
        let _e675 = V3_1;
        if (_e675 == (_e674 >> bitcast<u32>(16i))) {
            phi_6311_ = min(_e137, unpack2x16float(_e674).x);
        } else {
            phi_6311_ = 0f;
        }
        let _e683 = phi_6311_;
        phi_7774_ = _e683;
    }
    let _e685 = phi_7774_;
    let _e686 = Y1_1;
    let _e687 = (_e127 * _e686);
    phi_1470_ = lj;
    if lj {
        let _e688 = N1_1;
        phi_1470_ = (_e688 != 0u);
    }
    let _e691 = phi_1470_;
    phi_7786_ = _e687;
    if _e691 {
        let _e694 = o0_.x2_[_e125];
        let _e699 = ((unpack4x8unorm(_e694) * (1f - _e664.w)) + _e664);
        if (_e687.w != 0f) {
            phi_6349_ = (1f / _e687.w);
        } else {
            phi_6349_ = 0f;
        }
        let _e705 = phi_6349_;
        let _e706 = (_e687.xyz * _e705);
        let _e707 = N1_1;
        local_2 = _e706;
        let _e708 = _e699.xyz;
        if (_e699.w != 0f) {
            phi_6350_ = (1f / _e699.w);
        } else {
            phi_6350_ = 0f;
        }
        let _e713 = phi_6350_;
        let _e714 = (_e708 * _e713);
        local = _e714;
        switch bitcast<i32>(_e707) {
            case 11: {
                let _e998 = local_2;
                local_1 = (_e998 * _e714);
                break;
            }
            case 1: {
                let _e994 = local_2;
                local_1 = ((_e994 + _e714) - (_e994 * _e714));
                break;
            }
            case 2: {
                let _e986 = local_2;
                let _e987 = (_e986 * _e714);
                local_1 = (select(_e987, (((_e986 + _e714) - _e987) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e714 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e984 = local_2;
                local_1 = min(_e984, _e714);
                break;
            }
            case 4: {
                let _e982 = local_2;
                local_1 = max(_e982, _e714);
                break;
            }
            case 5: {
                let _e953 = clamp(_e708, vec3<f32>(0f, 0f, 0f), _e699.www);
                let _e959 = vec4<f32>(_e953.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e965 = vec4<f32>(_e959.x, _e953.y, _e959.z, _e959.w);
                let _e972 = local_2;
                let _e975 = (clamp((vec3<f32>(1f, 1f, 1f) - _e972), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e699.w);
                let _e976 = vec4<f32>(_e965.x, _e965.y, _e953.z, _e965.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e976 / _e975)), sign(_e976), (_e975 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e914 = local_2;
                local_2 = clamp(_e914, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e917 = clamp(_e708, vec3<f32>(0f, 0f, 0f), _e699.www);
                let _e923 = vec4<f32>(_e917.x, _e699.y, _e699.z, _e699.w);
                let _e929 = vec4<f32>(_e923.x, _e917.y, _e923.z, _e923.w);
                phi_7443_ = vec4<f32>(_e929.x, _e929.y, _e917.z, _e929.w);
                if (_e699.w == 0f) {
                    phi_7443_ = vec4<f32>(_e917.x, _e917.y, _e917.z, 1f);
                }
                let _e939 = phi_7443_;
                let _e943 = (vec3(_e939.w) - _e939.xyz);
                let _e944 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e943 / (_e944 * _e939.w))), sign(_e943), (_e944 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e906 = local_2;
                let _e907 = (_e906 * _e714);
                local_1 = (select(_e907, (((_e906 + _e714) - _e907) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e906 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7303_ = 0i;
                loop {
                    let _e875 = phi_7303_;
                    if (_e875 < 3i) {
                        let _e878 = local_2[_e875];
                        if (_e878 <= 0.5f) {
                            let _e895 = local[_e875];
                            local_1[_e875] = (1f - _e895);
                        } else {
                            let _e881 = local[_e875];
                            if (_e881 <= 0.25f) {
                                let _e887 = local[_e875];
                                let _e890 = local[_e875];
                                local_1[_e875] = ((((16f * _e887) - 12f) * _e890) + 3f);
                            } else {
                                let _e883 = local[_e875];
                                local_1[_e875] = (inverseSqrt(_e883) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7303_ = (_e875 + 1i);
                    }
                }
                let _e899 = local_2;
                let _e903 = local_1;
                local_1 = (_e714 + ((_e714 * ((_e899 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e903));
                break;
            }
            case 9: {
                let _e871 = local_2;
                local_1 = abs((_e714 - _e871));
                break;
            }
            case 10: {
                let _e866 = local_2;
                local_1 = ((_e866 + _e714) - ((_e866 * 2f) * _e714));
                break;
            }
            case 12: {
                if qj {
                    let _e817 = local_2;
                    let _e818 = clamp(_e817, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e818;
                    let _e833 = (_e818 - vec3(min(min(_e818.x, _e818.y), _e818.z)));
                    let _e841 = (_e833 * ((max(max(_e714.x, _e714.y), _e714.z) - min(min(_e714.x, _e714.y), _e714.z)) / max(0.000062f, max(max(_e833.x, _e833.y), _e833.z))));
                    let _e842 = dot(_e714, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e845 = (_e841 - vec3(dot(_e841, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e858 = (vec2<f32>(_e842, (1f - _e842)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e845.x, _e845.y), _e845.z)), max(max(_e845.x, _e845.y), _e845.z))));
                    local_1 = ((_e845 * min(1f, min(_e858.x, _e858.y))) + vec3(_e842));
                }
                break;
            }
            case 13: {
                if qj {
                    let _e768 = local_2;
                    let _e769 = clamp(_e768, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e769;
                    let _e784 = (_e714 - vec3(min(min(_e714.x, _e714.y), _e714.z)));
                    let _e792 = (_e784 * ((max(max(_e769.x, _e769.y), _e769.z) - min(min(_e769.x, _e769.y), _e769.z)) / max(0.000062f, max(max(_e784.x, _e784.y), _e784.z))));
                    let _e793 = dot(_e714, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e796 = (_e792 - vec3(dot(_e792, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e809 = (vec2<f32>(_e793, (1f - _e793)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e796.x, _e796.y), _e796.z)), max(max(_e796.x, _e796.y), _e796.z))));
                    local_1 = ((_e796 * min(1f, min(_e809.x, _e809.y))) + vec3(_e793));
                }
                break;
            }
            case 14: {
                if qj {
                    let _e742 = local_2;
                    let _e743 = clamp(_e742, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e743;
                    let _e744 = dot(_e714, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e747 = (_e743 - vec3(dot(_e743, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e760 = (vec2<f32>(_e744, (1f - _e744)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e747.x, _e747.y), _e747.z)), max(max(_e747.x, _e747.y), _e747.z))));
                    local_1 = ((_e747 * min(1f, min(_e760.x, _e760.y))) + vec3(_e744));
                }
                break;
            }
            case 15: {
                if qj {
                    let _e716 = local_2;
                    let _e717 = clamp(_e716, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e717;
                    let _e718 = dot(_e717, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e721 = (_e714 - vec3(dot(_e714, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e734 = (vec2<f32>(_e718, (1f - _e718)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e721.x, _e721.y), _e721.z)), max(max(_e721.x, _e721.y), _e721.z))));
                    local_1 = ((_e721 * min(1f, min(_e734.x, _e734.y))) + vec3(_e718));
                }
                break;
            }
            default: {
            }
        }
        let _e1000 = local_1;
        let _e1003 = (mix(_e706, _e1000, vec3(_e699.w)) * _e687.w);
        let _e1009 = vec4<f32>(_e1003.x, _e687.y, _e687.z, _e687.w);
        let _e1015 = vec4<f32>(_e1009.x, _e1003.y, _e1009.z, _e1009.w);
        phi_7786_ = vec4<f32>(_e1015.x, _e1015.y, _e1003.z, _e1015.w);
    }
    let _e1023 = phi_7786_;
    let _e1024 = (_e1023 * _e685);
    let _e1028 = ((_e664 * (1f - _e1024.w)) + _e1024);
    let _e1029 = _e1028.xyz;
    let _e1032 = j.I3_;
    let _e1034 = j.J3_;
    if (rj && (_e1028.w != 0f)) {
        phi_7799_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e90.x) + (0.00583715f * _e90.y))))) * _e1032) + _e1034)) + _e1029);
    } else {
        phi_7799_ = _e1029;
    }
    let _e1050 = phi_7799_;
    let _e1056 = vec4<f32>(_e1050.x, _e1028.y, _e1028.z, _e1028.w);
    let _e1062 = vec4<f32>(_e1056.x, _e1050.y, _e1056.z, _e1056.w);
    let _e1068 = vec4<f32>(_e1062.x, _e1062.y, _e1050.z, _e1062.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1050.x + _e1050.y) + _e1050.z) + _e1028.w) == 0f) {
                break;
            }
            phi_7801_ = _e1068;
            if (_e1028.w != 1f) {
                let _e1077 = o0_.x2_[_e125];
                phi_7801_ = (_e1068 + (unpack4x8unorm(_e1077) * (1f - _e1028.w)));
            }
            let _e1083 = phi_7801_;
            o0_.x2_[_e125] = pack4x8unorm(_e1083);
            break;
        }
    }
    if (_e315 != 0u) {
        m0_.x2_[_e125] = _e315;
    }
    W4_.x2_[_e125] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) o2_: vec2<f32>, @location(1) V0_: vec4<f32>, @location(4) @interpolate(flat, either) V3_: u32, @location(3) @interpolate(flat, either) Y1_: vec4<f32>, @location(5) @interpolate(flat, either) N1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    o2_1 = o2_;
    V0_1 = V0_;
    V3_1 = V3_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
}
