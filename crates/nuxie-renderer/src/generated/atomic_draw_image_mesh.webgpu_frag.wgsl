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
    var phi_6295_: f32;
    var phi_1710_: bool;
    var phi_5522_: f32;
    var phi_5521_: f32;
    var phi_5523_: f32;
    var phi_5526_: f32;
    var phi_5525_: f32;
    var phi_1747_: bool;
    var phi_5530_: f32;
    var phi_5527_: f32;
    var phi_5528_: f32;
    var phi_5529_: vec2<f32>;
    var phi_5553_: vec4<f32>;
    var phi_6260_: u32;
    var phi_6259_: u32;
    var phi_5551_: vec4<f32>;
    var phi_5560_: f32;
    var phi_6010_: i32;
    var phi_6078_: vec4<f32>;
    var phi_6244_: vec4<f32>;
    var phi_6257_: vec4<f32>;
    var phi_1438_: bool;
    var phi_6285_: u32;
    var phi_6315_: f32;
    var phi_7778_: f32;
    var phi_1470_: bool;
    var phi_6353_: f32;
    var phi_6354_: f32;
    var phi_7307_: i32;
    var phi_7447_: vec4<f32>;
    var phi_7790_: vec4<f32>;
    var phi_7803_: vec3<f32>;
    var phi_7805_: vec4<f32>;

    let _e89 = gl_FragCoord_1;
    let _e90 = _e89.xy;
    let _e93 = bitcast<vec2<u32>>(vec2<i32>(floor(_e90)));
    let _e95 = j.T6_;
    let _e124 = bitcast<i32>((((((_e93.y >> bitcast<u32>(5u)) * (((_e95 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e93.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e93.x & 28u) << bitcast<u32>(5u)) + ((_e93.y & 28u) << bitcast<u32>(2i)))) + (((_e93.y & 3u) << bitcast<u32>(2i)) + (_e93.x & 3u))));
    let _e125 = n2_1;
    let _e126 = textureSample(RB, v4_, _e125);
    phi_6295_ = 1f;
    if kj {
        let _e127 = W0_1;
        let _e130 = min(_e127.xy, _e127.zw);
        phi_6295_ = clamp(min(_e130.x, _e130.y), 0f, 1f);
    }
    let _e136 = phi_6295_;
    let _e139 = W4_.v2_[_e124];
    let _e141 = (_e139 >> bitcast<u32>(17u));
    let _e145 = ((f32((_e139 & 131071u)) * 0.00048828125f) + -32f);
    let _e148 = XC.v2_[_e141];
    phi_5521_ = _e145;
    if ((_e148.x & 768u) != 0u) {
        let _e152 = abs(_e145);
        phi_1710_ = oj;
        if oj {
            phi_1710_ = ((_e148.x & 512u) != 0u);
        }
        let _e156 = phi_1710_;
        phi_5522_ = _e152;
        if _e156 {
            phi_5522_ = (1f - abs(((fract((_e152 * 0.5f)) * 2f) + -1f)));
        }
        let _e164 = phi_5522_;
        phi_5521_ = _e164;
    }
    let _e166 = phi_5521_;
    let _e167 = clamp(_e166, 0f, 1f);
    phi_5525_ = _e167;
    if jj {
        let _e169 = (_e148.x >> bitcast<u32>(16u));
        phi_5526_ = _e167;
        if (_e169 != 0u) {
            let _e173 = m0_.v2_[_e124];
            if (_e169 == (_e173 >> bitcast<u32>(16i))) {
                phi_5523_ = min(_e167, unpack2x16float(_e173).x);
            } else {
                phi_5523_ = 0f;
            }
            let _e181 = phi_5523_;
            phi_5526_ = _e181;
        }
        let _e183 = phi_5526_;
        phi_5525_ = _e183;
    }
    let _e185 = phi_5525_;
    phi_1747_ = kj;
    if kj {
        phi_1747_ = ((_e148.x & 1024u) != 0u);
    }
    let _e189 = phi_1747_;
    phi_5530_ = _e185;
    if _e189 {
        let _e190 = (_e141 * 8u);
        let _e194 = JB.v2_[(_e190 + 2u)];
        let _e205 = JB.v2_[(_e190 + 3u)];
        let _e210 = _e205.zw;
        let _e212 = ((abs(((mat2x2<f32>(vec2<f32>(_e194.x, _e194.y), vec2<f32>(_e194.z, _e194.w)) * _e90) + _e205.xy)) * _e210) - _e210);
        phi_5530_ = min(_e185, clamp((min(_e212.x, _e212.y) + 0.5f), 0f, 1f));
    }
    let _e220 = phi_5530_;
    let _e221 = (_e148.x & 15u);
    let _e224 = ((_e148.x >> bitcast<u32>(4i)) & 15u);
    let _e226 = (lj && (_e224 != 0u));
    if (_e221 <= 1u) {
        let _e307 = (jj && (_e221 == 0u));
        phi_6260_ = 0u;
        if _e307 {
            phi_6260_ = (_e148.y | pack2x16float(vec2<f32>(_e220, 0f)));
        }
        let _e312 = phi_6260_;
        phi_6259_ = _e312;
        phi_5551_ = select(unpack4x8unorm(_e148.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e307));
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
                    let _e276 = normalize(vec2<f32>(_e245.x, -(_e245.y)));
                    let _e279 = acos(clamp(_e276.x, -1f, 1f));
                    if (_e276.y >= 0f) {
                        phi_5528_ = _e279;
                    } else {
                        phi_5528_ = -(_e279);
                    }
                    let _e284 = phi_5528_;
                    phi_5529_ = vec2<f32>((_e284 * 0.15915494f), _e258);
                    break;
                } else {
                    if (_e249 == 2f) {
                        phi_5527_ = _e245.x;
                    } else {
                        phi_5527_ = length(_e245);
                    }
                    let _e266 = phi_5527_;
                    phi_5529_ = vec2<f32>(((clamp(_e266, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e242.z < 0f))) + ((max(0f, _e242.z) * 0.001953125f) + 0.0009765625f)), _e258);
                    break;
                }
            }
        }
        let _e288 = phi_5529_;
        let _e289 = textureSampleLevel(ZC, S8_, _e288, 0f);
        phi_5553_ = _e289;
        if !(_e226) {
            let _e293 = (_e289.xyz * _e289.w);
            phi_5553_ = vec4<f32>(_e293.x, _e293.y, _e293.z, (_e289.w * abs(bitcast<f32>(_e148.y))));
        }
        let _e303 = phi_5553_;
        phi_6259_ = 0u;
        phi_5551_ = _e303;
    }
    let _e316 = phi_6259_;
    let _e318 = phi_5551_;
    phi_6257_ = _e318;
    if _e226 {
        phi_6244_ = _e318;
        if ((_e318.w * _e220) != 0f) {
            let _e324 = n0_.v2_[_e124];
            let _e325 = unpack4x8unorm(_e324);
            let _e326 = _e318.xyz;
            local_5 = _e326;
            let _e327 = _e325.xyz;
            if (_e325.w != 0f) {
                phi_5560_ = (1f / _e325.w);
            } else {
                phi_5560_ = 0f;
            }
            let _e332 = phi_5560_;
            let _e333 = (_e327 * _e332);
            local_3 = _e333;
            switch bitcast<i32>(_e224) {
                case 11: {
                    let _e617 = local_5;
                    local_4 = (_e617 * _e333);
                    break;
                }
                case 1: {
                    let _e613 = local_5;
                    local_4 = ((_e613 + _e333) - (_e613 * _e333));
                    break;
                }
                case 2: {
                    let _e605 = local_5;
                    let _e606 = (_e605 * _e333);
                    local_4 = (select(_e606, (((_e605 + _e333) - _e606) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e333 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e603 = local_5;
                    local_4 = min(_e603, _e333);
                    break;
                }
                case 4: {
                    let _e601 = local_5;
                    local_4 = max(_e601, _e333);
                    break;
                }
                case 5: {
                    let _e572 = clamp(_e327, vec3<f32>(0f, 0f, 0f), _e325.www);
                    let _e578 = vec4<f32>(_e572.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e584 = vec4<f32>(_e578.x, _e572.y, _e578.z, _e578.w);
                    let _e591 = local_5;
                    let _e594 = (clamp((vec3<f32>(1f, 1f, 1f) - _e591), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e325.w);
                    let _e595 = vec4<f32>(_e584.x, _e584.y, _e572.z, _e584.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e595 / _e594)), sign(_e595), (_e594 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e533 = local_5;
                    local_5 = clamp(_e533, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e536 = clamp(_e327, vec3<f32>(0f, 0f, 0f), _e325.www);
                    let _e542 = vec4<f32>(_e536.x, _e325.y, _e325.z, _e325.w);
                    let _e548 = vec4<f32>(_e542.x, _e536.y, _e542.z, _e542.w);
                    phi_6078_ = vec4<f32>(_e548.x, _e548.y, _e536.z, _e548.w);
                    if (_e325.w == 0f) {
                        phi_6078_ = vec4<f32>(_e536.x, _e536.y, _e536.z, 1f);
                    }
                    let _e558 = phi_6078_;
                    let _e562 = (vec3(_e558.w) - _e558.xyz);
                    let _e563 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e562 / (_e563 * _e558.w))), sign(_e562), (_e563 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e525 = local_5;
                    let _e526 = (_e525 * _e333);
                    local_4 = (select(_e526, (((_e525 + _e333) - _e526) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e525 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_6010_ = 0i;
                    loop {
                        let _e494 = phi_6010_;
                        if (_e494 < 3i) {
                            let _e497 = local_5[_e494];
                            if (_e497 <= 0.5f) {
                                let _e514 = local_3[_e494];
                                local_4[_e494] = (1f - _e514);
                            } else {
                                let _e500 = local_3[_e494];
                                if (_e500 <= 0.25f) {
                                    let _e506 = local_3[_e494];
                                    let _e509 = local_3[_e494];
                                    local_4[_e494] = ((((16f * _e506) - 12f) * _e509) + 3f);
                                } else {
                                    let _e502 = local_3[_e494];
                                    local_4[_e494] = (inverseSqrt(_e502) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_6010_ = (_e494 + 1i);
                        }
                    }
                    let _e518 = local_5;
                    let _e522 = local_4;
                    local_4 = (_e333 + ((_e333 * ((_e518 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e522));
                    break;
                }
                case 9: {
                    let _e490 = local_5;
                    local_4 = abs((_e333 - _e490));
                    break;
                }
                case 10: {
                    let _e485 = local_5;
                    local_4 = ((_e485 + _e333) - ((_e485 * 2f) * _e333));
                    break;
                }
                case 12: {
                    if qj {
                        let _e436 = local_5;
                        let _e437 = clamp(_e436, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e437;
                        let _e452 = (_e437 - vec3(min(min(_e437.x, _e437.y), _e437.z)));
                        let _e460 = (_e452 * ((max(max(_e333.x, _e333.y), _e333.z) - min(min(_e333.x, _e333.y), _e333.z)) / max(0.000062f, max(max(_e452.x, _e452.y), _e452.z))));
                        let _e461 = dot(_e333, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e464 = (_e460 - vec3(dot(_e460, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e477 = (vec2<f32>(_e461, (1f - _e461)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e464.x, _e464.y), _e464.z)), max(max(_e464.x, _e464.y), _e464.z))));
                        local_4 = ((_e464 * min(1f, min(_e477.x, _e477.y))) + vec3(_e461));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e387 = local_5;
                        let _e388 = clamp(_e387, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e388;
                        let _e403 = (_e333 - vec3(min(min(_e333.x, _e333.y), _e333.z)));
                        let _e411 = (_e403 * ((max(max(_e388.x, _e388.y), _e388.z) - min(min(_e388.x, _e388.y), _e388.z)) / max(0.000062f, max(max(_e403.x, _e403.y), _e403.z))));
                        let _e412 = dot(_e333, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e415 = (_e411 - vec3(dot(_e411, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e428 = (vec2<f32>(_e412, (1f - _e412)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e415.x, _e415.y), _e415.z)), max(max(_e415.x, _e415.y), _e415.z))));
                        local_4 = ((_e415 * min(1f, min(_e428.x, _e428.y))) + vec3(_e412));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e361 = local_5;
                        let _e362 = clamp(_e361, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e362;
                        let _e363 = dot(_e333, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e366 = (_e362 - vec3(dot(_e362, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e379 = (vec2<f32>(_e363, (1f - _e363)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e366.x, _e366.y), _e366.z)), max(max(_e366.x, _e366.y), _e366.z))));
                        local_4 = ((_e366 * min(1f, min(_e379.x, _e379.y))) + vec3(_e363));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e335 = local_5;
                        let _e336 = clamp(_e335, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e336;
                        let _e337 = dot(_e336, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e340 = (_e333 - vec3(dot(_e333, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e353 = (vec2<f32>(_e337, (1f - _e337)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e340.x, _e340.y), _e340.z)), max(max(_e340.x, _e340.y), _e340.z))));
                        local_4 = ((_e340 * min(1f, min(_e353.x, _e353.y))) + vec3(_e337));
                    }
                    break;
                }
                default: {
                }
            }
            let _e619 = local_4;
            let _e621 = mix(_e326, _e619, vec3(_e325.w));
            let _e627 = vec4<f32>(_e621.x, _e318.y, _e318.z, _e318.w);
            let _e633 = vec4<f32>(_e627.x, _e621.y, _e627.z, _e627.w);
            phi_6244_ = vec4<f32>(_e633.x, _e633.y, _e621.z, _e633.w);
        }
        let _e641 = phi_6244_;
        let _e644 = (_e641.xyz * _e641.w);
        let _e650 = vec4<f32>(_e644.x, _e641.y, _e641.z, _e641.w);
        let _e656 = vec4<f32>(_e650.x, _e644.y, _e650.z, _e650.w);
        phi_6257_ = vec4<f32>(_e656.x, _e656.y, _e644.z, _e656.w);
    }
    let _e664 = phi_6257_;
    let _e665 = (_e664 * _e220);
    phi_1438_ = jj;
    if jj {
        let _e666 = V3_1;
        phi_1438_ = (_e666 != 0u);
    }
    let _e669 = phi_1438_;
    phi_7778_ = _e136;
    if _e669 {
        if (_e316 != 0u) {
            phi_6285_ = _e316;
        } else {
            let _e673 = m0_.v2_[_e124];
            phi_6285_ = _e673;
        }
        let _e675 = phi_6285_;
        let _e676 = V3_1;
        if (_e676 == (_e675 >> bitcast<u32>(16i))) {
            phi_6315_ = min(_e136, unpack2x16float(_e675).x);
        } else {
            phi_6315_ = 0f;
        }
        let _e684 = phi_6315_;
        phi_7778_ = _e684;
    }
    let _e686 = phi_7778_;
    let _e687 = Y1_1;
    let _e688 = (_e126 * _e687);
    phi_1470_ = lj;
    if lj {
        let _e689 = N1_1;
        phi_1470_ = (_e689 != 0u);
    }
    let _e692 = phi_1470_;
    phi_7790_ = _e688;
    if _e692 {
        let _e695 = n0_.v2_[_e124];
        let _e700 = ((unpack4x8unorm(_e695) * (1f - _e665.w)) + _e665);
        if (_e688.w != 0f) {
            phi_6353_ = (1f / _e688.w);
        } else {
            phi_6353_ = 0f;
        }
        let _e706 = phi_6353_;
        let _e707 = (_e688.xyz * _e706);
        let _e708 = N1_1;
        local_2 = _e707;
        let _e709 = _e700.xyz;
        if (_e700.w != 0f) {
            phi_6354_ = (1f / _e700.w);
        } else {
            phi_6354_ = 0f;
        }
        let _e714 = phi_6354_;
        let _e715 = (_e709 * _e714);
        local = _e715;
        switch bitcast<i32>(_e708) {
            case 11: {
                let _e999 = local_2;
                local_1 = (_e999 * _e715);
                break;
            }
            case 1: {
                let _e995 = local_2;
                local_1 = ((_e995 + _e715) - (_e995 * _e715));
                break;
            }
            case 2: {
                let _e987 = local_2;
                let _e988 = (_e987 * _e715);
                local_1 = (select(_e988, (((_e987 + _e715) - _e988) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e715 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e985 = local_2;
                local_1 = min(_e985, _e715);
                break;
            }
            case 4: {
                let _e983 = local_2;
                local_1 = max(_e983, _e715);
                break;
            }
            case 5: {
                let _e954 = clamp(_e709, vec3<f32>(0f, 0f, 0f), _e700.www);
                let _e960 = vec4<f32>(_e954.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e966 = vec4<f32>(_e960.x, _e954.y, _e960.z, _e960.w);
                let _e973 = local_2;
                let _e976 = (clamp((vec3<f32>(1f, 1f, 1f) - _e973), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e700.w);
                let _e977 = vec4<f32>(_e966.x, _e966.y, _e954.z, _e966.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e977 / _e976)), sign(_e977), (_e976 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e915 = local_2;
                local_2 = clamp(_e915, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e918 = clamp(_e709, vec3<f32>(0f, 0f, 0f), _e700.www);
                let _e924 = vec4<f32>(_e918.x, _e700.y, _e700.z, _e700.w);
                let _e930 = vec4<f32>(_e924.x, _e918.y, _e924.z, _e924.w);
                phi_7447_ = vec4<f32>(_e930.x, _e930.y, _e918.z, _e930.w);
                if (_e700.w == 0f) {
                    phi_7447_ = vec4<f32>(_e918.x, _e918.y, _e918.z, 1f);
                }
                let _e940 = phi_7447_;
                let _e944 = (vec3(_e940.w) - _e940.xyz);
                let _e945 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e944 / (_e945 * _e940.w))), sign(_e944), (_e945 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e907 = local_2;
                let _e908 = (_e907 * _e715);
                local_1 = (select(_e908, (((_e907 + _e715) - _e908) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e907 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7307_ = 0i;
                loop {
                    let _e876 = phi_7307_;
                    if (_e876 < 3i) {
                        let _e879 = local_2[_e876];
                        if (_e879 <= 0.5f) {
                            let _e896 = local[_e876];
                            local_1[_e876] = (1f - _e896);
                        } else {
                            let _e882 = local[_e876];
                            if (_e882 <= 0.25f) {
                                let _e888 = local[_e876];
                                let _e891 = local[_e876];
                                local_1[_e876] = ((((16f * _e888) - 12f) * _e891) + 3f);
                            } else {
                                let _e884 = local[_e876];
                                local_1[_e876] = (inverseSqrt(_e884) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7307_ = (_e876 + 1i);
                    }
                }
                let _e900 = local_2;
                let _e904 = local_1;
                local_1 = (_e715 + ((_e715 * ((_e900 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e904));
                break;
            }
            case 9: {
                let _e872 = local_2;
                local_1 = abs((_e715 - _e872));
                break;
            }
            case 10: {
                let _e867 = local_2;
                local_1 = ((_e867 + _e715) - ((_e867 * 2f) * _e715));
                break;
            }
            case 12: {
                if qj {
                    let _e818 = local_2;
                    let _e819 = clamp(_e818, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e819;
                    let _e834 = (_e819 - vec3(min(min(_e819.x, _e819.y), _e819.z)));
                    let _e842 = (_e834 * ((max(max(_e715.x, _e715.y), _e715.z) - min(min(_e715.x, _e715.y), _e715.z)) / max(0.000062f, max(max(_e834.x, _e834.y), _e834.z))));
                    let _e843 = dot(_e715, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e846 = (_e842 - vec3(dot(_e842, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e859 = (vec2<f32>(_e843, (1f - _e843)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e846.x, _e846.y), _e846.z)), max(max(_e846.x, _e846.y), _e846.z))));
                    local_1 = ((_e846 * min(1f, min(_e859.x, _e859.y))) + vec3(_e843));
                }
                break;
            }
            case 13: {
                if qj {
                    let _e769 = local_2;
                    let _e770 = clamp(_e769, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e770;
                    let _e785 = (_e715 - vec3(min(min(_e715.x, _e715.y), _e715.z)));
                    let _e793 = (_e785 * ((max(max(_e770.x, _e770.y), _e770.z) - min(min(_e770.x, _e770.y), _e770.z)) / max(0.000062f, max(max(_e785.x, _e785.y), _e785.z))));
                    let _e794 = dot(_e715, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e797 = (_e793 - vec3(dot(_e793, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e810 = (vec2<f32>(_e794, (1f - _e794)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e797.x, _e797.y), _e797.z)), max(max(_e797.x, _e797.y), _e797.z))));
                    local_1 = ((_e797 * min(1f, min(_e810.x, _e810.y))) + vec3(_e794));
                }
                break;
            }
            case 14: {
                if qj {
                    let _e743 = local_2;
                    let _e744 = clamp(_e743, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e744;
                    let _e745 = dot(_e715, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e748 = (_e744 - vec3(dot(_e744, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e761 = (vec2<f32>(_e745, (1f - _e745)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e748.x, _e748.y), _e748.z)), max(max(_e748.x, _e748.y), _e748.z))));
                    local_1 = ((_e748 * min(1f, min(_e761.x, _e761.y))) + vec3(_e745));
                }
                break;
            }
            case 15: {
                if qj {
                    let _e717 = local_2;
                    let _e718 = clamp(_e717, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e718;
                    let _e719 = dot(_e718, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e722 = (_e715 - vec3(dot(_e715, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e735 = (vec2<f32>(_e719, (1f - _e719)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e722.x, _e722.y), _e722.z)), max(max(_e722.x, _e722.y), _e722.z))));
                    local_1 = ((_e722 * min(1f, min(_e735.x, _e735.y))) + vec3(_e719));
                }
                break;
            }
            default: {
            }
        }
        let _e1001 = local_1;
        let _e1004 = (mix(_e707, _e1001, vec3(_e700.w)) * _e688.w);
        let _e1010 = vec4<f32>(_e1004.x, _e688.y, _e688.z, _e688.w);
        let _e1016 = vec4<f32>(_e1010.x, _e1004.y, _e1010.z, _e1010.w);
        phi_7790_ = vec4<f32>(_e1016.x, _e1016.y, _e1004.z, _e1016.w);
    }
    let _e1024 = phi_7790_;
    let _e1025 = (_e1024 * _e686);
    let _e1029 = ((_e665 * (1f - _e1025.w)) + _e1025);
    let _e1030 = _e1029.xyz;
    let _e1033 = j.H3_;
    let _e1035 = j.I3_;
    if (rj && (_e1029.w != 0f)) {
        phi_7803_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e89.x) + (0.00583715f * _e89.y))))) * _e1033) + _e1035)) + _e1030);
    } else {
        phi_7803_ = _e1030;
    }
    let _e1051 = phi_7803_;
    let _e1057 = vec4<f32>(_e1051.x, _e1029.y, _e1029.z, _e1029.w);
    let _e1063 = vec4<f32>(_e1057.x, _e1051.y, _e1057.z, _e1057.w);
    let _e1069 = vec4<f32>(_e1063.x, _e1063.y, _e1051.z, _e1063.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1051.x + _e1051.y) + _e1051.z) + _e1029.w) == 0f) {
                break;
            }
            phi_7805_ = _e1069;
            if (_e1029.w != 1f) {
                let _e1078 = n0_.v2_[_e124];
                phi_7805_ = (_e1069 + (unpack4x8unorm(_e1078) * (1f - _e1029.w)));
            }
            let _e1084 = phi_7805_;
            n0_.v2_[_e124] = pack4x8unorm(_e1084);
            break;
        }
    }
    if (_e316 != 0u) {
        m0_.v2_[_e124] = _e316;
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
