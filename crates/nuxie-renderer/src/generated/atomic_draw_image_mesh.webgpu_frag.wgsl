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
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct n0Pe {
    r2_: array<u32>,
}

struct R4Pe {
    r2_: array<u32>,
}

@id(7) override dj: bool = true;
@id(6) override cj: bool = true;
@id(4) override aj: bool = true;
@id(0) override Wi: bool = true;
@id(1) override Xi: bool = true;
@id(2) override Yi: bool = true;

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
var N8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Pe;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
var<private> l2_1: vec2<f32>;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe;
var<private> S3_1: u32;
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
    var phi_6087_: f32;
    var phi_1647_: bool;
    var phi_5402_: f32;
    var phi_5401_: f32;
    var phi_5403_: f32;
    var phi_5406_: f32;
    var phi_5405_: f32;
    var phi_1684_: bool;
    var phi_5408_: f32;
    var phi_6054_: u32;
    var phi_5407_: f32;
    var phi_5431_: vec4<f32>;
    var phi_6053_: u32;
    var phi_5429_: vec4<f32>;
    var phi_5436_: f32;
    var phi_5890_: vec4<f32>;
    var phi_5830_: i32;
    var phi_6038_: vec4<f32>;
    var phi_6051_: vec4<f32>;
    var phi_1380_: bool;
    var phi_6077_: u32;
    var phi_6105_: f32;
    var phi_7484_: f32;
    var phi_1412_: bool;
    var phi_6141_: f32;
    var phi_6142_: f32;
    var phi_7171_: vec4<f32>;
    var phi_7039_: i32;
    var phi_7496_: vec4<f32>;
    var phi_7509_: vec3<f32>;
    var phi_7511_: vec4<f32>;

    let _e86 = gl_FragCoord_1;
    let _e87 = _e86.xy;
    let _e90 = bitcast<vec2<u32>>(vec2<i32>(floor(_e87)));
    let _e92 = j.P6_;
    let _e121 = bitcast<i32>((((((_e90.y >> bitcast<u32>(5u)) * (((_e92 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e90.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e90.x & 28u) << bitcast<u32>(5u)) + ((_e90.y & 28u) << bitcast<u32>(2i)))) + (((_e90.y & 3u) << bitcast<u32>(2i)) + (_e90.x & 3u))));
    let _e122 = l2_1;
    let _e123 = textureSample(TB, U4_, _e122);
    phi_6087_ = 1f;
    if Xi {
        let _e124 = V0_1;
        let _e127 = min(_e124.xy, _e124.zw);
        phi_6087_ = clamp(min(_e127.x, _e127.y), 0f, 1f);
    }
    let _e133 = phi_6087_;
    let _e136 = R4_.r2_[_e121];
    let _e138 = (_e136 >> bitcast<u32>(17u));
    let _e142 = ((f32((_e136 & 131071u)) * 0.00048828125f) + -32f);
    let _e145 = VC.r2_[_e138];
    phi_5401_ = _e142;
    if ((_e145.x & 768u) != 0u) {
        let _e149 = abs(_e142);
        phi_1647_ = aj;
        if aj {
            phi_1647_ = ((_e145.x & 512u) != 0u);
        }
        let _e153 = phi_1647_;
        phi_5402_ = _e149;
        if _e153 {
            phi_5402_ = (1f - abs(((fract((_e149 * 0.5f)) * 2f) + -1f)));
        }
        let _e161 = phi_5402_;
        phi_5401_ = _e161;
    }
    let _e163 = phi_5401_;
    let _e164 = clamp(_e163, 0f, 1f);
    phi_5405_ = _e164;
    if Wi {
        let _e166 = (_e145.x >> bitcast<u32>(16u));
        phi_5406_ = _e164;
        if (_e166 != 0u) {
            let _e170 = m0_.r2_[_e121];
            if (_e166 == (_e170 >> bitcast<u32>(16i))) {
                phi_5403_ = min(_e164, unpack2x16float(_e170).x);
            } else {
                phi_5403_ = 0f;
            }
            let _e178 = phi_5403_;
            phi_5406_ = _e178;
        }
        let _e180 = phi_5406_;
        phi_5405_ = _e180;
    }
    let _e182 = phi_5405_;
    phi_1684_ = Xi;
    if Xi {
        phi_1684_ = ((_e145.x & 1024u) != 0u);
    }
    let _e186 = phi_1684_;
    phi_5408_ = _e182;
    if _e186 {
        let _e187 = (_e138 * 8u);
        let _e191 = JB.r2_[(_e187 + 2u)];
        let _e202 = JB.r2_[(_e187 + 3u)];
        let _e207 = _e202.zw;
        let _e209 = ((abs(((mat2x2<f32>(vec2<f32>(_e191.x, _e191.y), vec2<f32>(_e191.z, _e191.w)) * _e87) + _e202.xy)) * _e207) - _e207);
        phi_5408_ = min(_e182, clamp((min(_e209.x, _e209.y) + 0.5f), 0f, 1f));
    }
    let _e217 = phi_5408_;
    let _e218 = (_e145.x & 15u);
    let _e221 = ((_e145.x >> bitcast<u32>(4i)) & 15u);
    let _e223 = (Yi && (_e221 != 0u));
    if (_e218 <= 1u) {
        let _e228 = (Wi && (_e218 == 0u));
        phi_6054_ = 0u;
        if _e228 {
            phi_6054_ = (_e145.y | pack2x16float(vec2<f32>(_e217, 0f)));
        }
        let _e233 = phi_6054_;
        phi_6053_ = _e233;
        phi_5429_ = select(unpack4x8unorm(_e145.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e228));
    } else {
        let _e236 = (_e138 * 8u);
        let _e239 = JB.r2_[_e236];
        let _e250 = JB.r2_[(_e236 + 1u)];
        let _e253 = ((mat2x2<f32>(vec2<f32>(_e239.x, _e239.y), vec2<f32>(_e239.z, _e239.w)) * _e87) + _e250.xy);
        let _e259 = j.L8_;
        let _e261 = j.M8_;
        if (f32(_e218) == 2f) {
            phi_5407_ = _e253.x;
        } else {
            phi_5407_ = length(_e253);
        }
        let _e271 = phi_5407_;
        let _e277 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e271, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e250.z < 0f))) + ((max(0f, _e250.z) * 0.001953125f) + 0.0009765625f)), ((_e250.w * _e259) + _e261)), 0f);
        phi_5431_ = _e277;
        if !(_e223) {
            let _e281 = (_e277.xyz * _e277.w);
            phi_5431_ = vec4<f32>(_e281.x, _e281.y, _e281.z, (_e277.w * abs(bitcast<f32>(_e145.y))));
        }
        let _e291 = phi_5431_;
        phi_6053_ = 0u;
        phi_5429_ = _e291;
    }
    let _e293 = phi_6053_;
    let _e295 = phi_5429_;
    phi_6051_ = _e295;
    if _e223 {
        phi_6038_ = _e295;
        if ((_e295.w * _e217) != 0f) {
            let _e301 = n0_.r2_[_e121];
            let _e302 = unpack4x8unorm(_e301);
            let _e303 = _e295.xyz;
            local_5 = _e303;
            let _e304 = _e302.xyz;
            if (_e302.w != 0f) {
                phi_5436_ = (1f / _e302.w);
            } else {
                phi_5436_ = 0f;
            }
            let _e309 = phi_5436_;
            let _e310 = (_e304 * _e309);
            local_3 = _e310;
            switch bitcast<i32>(_e221) {
                case 11: {
                    let _e312 = local_5;
                    local_4 = (_e312 * _e310);
                    break;
                }
                case 1: {
                    let _e314 = local_5;
                    local_4 = ((_e314 + _e310) - (_e314 * _e310));
                    break;
                }
                case 2: {
                    let _e318 = local_5;
                    let _e319 = (_e318 * _e310);
                    local_4 = (select(_e319, (((_e318 + _e310) - _e319) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e310 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e326 = local_5;
                    local_4 = min(_e326, _e310);
                    break;
                }
                case 4: {
                    let _e328 = local_5;
                    local_4 = max(_e328, _e310);
                    break;
                }
                case 5: {
                    let _e331 = clamp(_e304, vec3<f32>(0f, 0f, 0f), _e302.www);
                    let _e337 = vec4<f32>(_e331.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e343 = vec4<f32>(_e337.x, _e331.y, _e337.z, _e337.w);
                    let _e350 = local_5;
                    let _e353 = (clamp((vec3<f32>(1f, 1f, 1f) - _e350), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e302.w);
                    let _e354 = vec4<f32>(_e343.x, _e343.y, _e331.z, _e343.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e354 / _e353)), sign(_e354), (_e353 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e360 = local_5;
                    local_5 = clamp(_e360, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e363 = clamp(_e304, vec3<f32>(0f, 0f, 0f), _e302.www);
                    let _e369 = vec4<f32>(_e363.x, _e302.y, _e302.z, _e302.w);
                    let _e375 = vec4<f32>(_e369.x, _e363.y, _e369.z, _e369.w);
                    phi_5890_ = vec4<f32>(_e375.x, _e375.y, _e363.z, _e375.w);
                    if (_e302.w == 0f) {
                        phi_5890_ = vec4<f32>(_e363.x, _e363.y, _e363.z, 1f);
                    }
                    let _e385 = phi_5890_;
                    let _e389 = (vec3(_e385.w) - _e385.xyz);
                    let _e390 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e389 / (_e390 * _e385.w))), sign(_e389), (_e390 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e398 = local_5;
                    let _e399 = (_e398 * _e310);
                    local_4 = (select(_e399, (((_e398 + _e310) - _e399) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e398 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_5830_ = 0i;
                    loop {
                        let _e407 = phi_5830_;
                        if (_e407 < 3i) {
                            let _e410 = local_5[_e407];
                            if (_e410 <= 0.5f) {
                                let _e413 = local_3[_e407];
                                local_4[_e407] = (1f - _e413);
                            } else {
                                let _e417 = local_3[_e407];
                                if (_e417 <= 0.25f) {
                                    let _e419 = local_3[_e407];
                                    let _e422 = local_3[_e407];
                                    local_4[_e407] = ((((16f * _e419) - 12f) * _e422) + 3f);
                                } else {
                                    let _e426 = local_3[_e407];
                                    local_4[_e407] = (inverseSqrt(_e426) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_5830_ = (_e407 + 1i);
                        }
                    }
                    let _e431 = local_5;
                    let _e435 = local_4;
                    local_4 = (_e310 + ((_e310 * ((_e431 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e435));
                    break;
                }
                case 9: {
                    let _e438 = local_5;
                    local_4 = abs((_e310 - _e438));
                    break;
                }
                case 10: {
                    let _e441 = local_5;
                    local_4 = ((_e441 + _e310) - ((_e441 * 2f) * _e310));
                    break;
                }
                case 12: {
                    if cj {
                        let _e446 = local_5;
                        let _e447 = clamp(_e446, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e447;
                        let _e462 = (_e447 - vec3(min(min(_e447.x, _e447.y), _e447.z)));
                        let _e470 = (_e462 * ((max(max(_e310.x, _e310.y), _e310.z) - min(min(_e310.x, _e310.y), _e310.z)) / max(0.000062f, max(max(_e462.x, _e462.y), _e462.z))));
                        let _e471 = dot(_e310, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e474 = (_e470 - vec3(dot(_e470, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e487 = (vec2<f32>(_e471, (1f - _e471)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e474.x, _e474.y), _e474.z)), max(max(_e474.x, _e474.y), _e474.z))));
                        local_4 = ((_e474 * min(1f, min(_e487.x, _e487.y))) + vec3(_e471));
                    }
                    break;
                }
                case 13: {
                    if cj {
                        let _e495 = local_5;
                        let _e496 = clamp(_e495, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e496;
                        let _e511 = (_e310 - vec3(min(min(_e310.x, _e310.y), _e310.z)));
                        let _e519 = (_e511 * ((max(max(_e496.x, _e496.y), _e496.z) - min(min(_e496.x, _e496.y), _e496.z)) / max(0.000062f, max(max(_e511.x, _e511.y), _e511.z))));
                        let _e520 = dot(_e310, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e523 = (_e519 - vec3(dot(_e519, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e536 = (vec2<f32>(_e520, (1f - _e520)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e523.x, _e523.y), _e523.z)), max(max(_e523.x, _e523.y), _e523.z))));
                        local_4 = ((_e523 * min(1f, min(_e536.x, _e536.y))) + vec3(_e520));
                    }
                    break;
                }
                case 14: {
                    if cj {
                        let _e544 = local_5;
                        let _e545 = clamp(_e544, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e545;
                        let _e546 = dot(_e310, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e549 = (_e545 - vec3(dot(_e545, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e562 = (vec2<f32>(_e546, (1f - _e546)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e549.x, _e549.y), _e549.z)), max(max(_e549.x, _e549.y), _e549.z))));
                        local_4 = ((_e549 * min(1f, min(_e562.x, _e562.y))) + vec3(_e546));
                    }
                    break;
                }
                case 15: {
                    if cj {
                        let _e570 = local_5;
                        let _e571 = clamp(_e570, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e571;
                        let _e572 = dot(_e571, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e575 = (_e310 - vec3(dot(_e310, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e588 = (vec2<f32>(_e572, (1f - _e572)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e575.x, _e575.y), _e575.z)), max(max(_e575.x, _e575.y), _e575.z))));
                        local_4 = ((_e575 * min(1f, min(_e588.x, _e588.y))) + vec3(_e572));
                    }
                    break;
                }
                default: {
                }
            }
            let _e596 = local_4;
            let _e598 = mix(_e303, _e596, vec3(_e302.w));
            let _e604 = vec4<f32>(_e598.x, _e295.y, _e295.z, _e295.w);
            let _e610 = vec4<f32>(_e604.x, _e598.y, _e604.z, _e604.w);
            phi_6038_ = vec4<f32>(_e610.x, _e610.y, _e598.z, _e610.w);
        }
        let _e618 = phi_6038_;
        let _e621 = (_e618.xyz * _e618.w);
        let _e627 = vec4<f32>(_e621.x, _e618.y, _e618.z, _e618.w);
        let _e633 = vec4<f32>(_e627.x, _e621.y, _e627.z, _e627.w);
        phi_6051_ = vec4<f32>(_e633.x, _e633.y, _e621.z, _e633.w);
    }
    let _e641 = phi_6051_;
    let _e642 = (_e641 * _e217);
    phi_1380_ = Wi;
    if Wi {
        let _e643 = S3_1;
        phi_1380_ = (_e643 != 0u);
    }
    let _e646 = phi_1380_;
    phi_7484_ = _e133;
    if _e646 {
        if (_e293 != 0u) {
            phi_6077_ = _e293;
        } else {
            let _e650 = m0_.r2_[_e121];
            phi_6077_ = _e650;
        }
        let _e652 = phi_6077_;
        let _e653 = S3_1;
        if (_e653 == (_e652 >> bitcast<u32>(16i))) {
            phi_6105_ = min(_e133, unpack2x16float(_e652).x);
        } else {
            phi_6105_ = 0f;
        }
        let _e661 = phi_6105_;
        phi_7484_ = _e661;
    }
    let _e663 = phi_7484_;
    let _e664 = T1_1;
    let _e665 = (_e123 * _e664);
    phi_1412_ = Yi;
    if Yi {
        let _e666 = J1_1;
        phi_1412_ = (_e666 != 0u);
    }
    let _e669 = phi_1412_;
    phi_7496_ = _e665;
    if _e669 {
        let _e672 = n0_.r2_[_e121];
        let _e677 = ((unpack4x8unorm(_e672) * (1f - _e642.w)) + _e642);
        if (_e665.w != 0f) {
            phi_6141_ = (1f / _e665.w);
        } else {
            phi_6141_ = 0f;
        }
        let _e683 = phi_6141_;
        let _e684 = (_e665.xyz * _e683);
        let _e685 = J1_1;
        local_2 = _e684;
        let _e686 = _e677.xyz;
        if (_e677.w != 0f) {
            phi_6142_ = (1f / _e677.w);
        } else {
            phi_6142_ = 0f;
        }
        let _e691 = phi_6142_;
        let _e692 = (_e686 * _e691);
        local = _e692;
        switch bitcast<i32>(_e685) {
            case 11: {
                let _e694 = local_2;
                local_1 = (_e694 * _e692);
                break;
            }
            case 1: {
                let _e696 = local_2;
                local_1 = ((_e696 + _e692) - (_e696 * _e692));
                break;
            }
            case 2: {
                let _e700 = local_2;
                let _e701 = (_e700 * _e692);
                local_1 = (select(_e701, (((_e700 + _e692) - _e701) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e692 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e708 = local_2;
                local_1 = min(_e708, _e692);
                break;
            }
            case 4: {
                let _e710 = local_2;
                local_1 = max(_e710, _e692);
                break;
            }
            case 5: {
                let _e713 = clamp(_e686, vec3<f32>(0f, 0f, 0f), _e677.www);
                let _e719 = vec4<f32>(_e713.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e725 = vec4<f32>(_e719.x, _e713.y, _e719.z, _e719.w);
                let _e732 = local_2;
                let _e735 = (clamp((vec3<f32>(1f, 1f, 1f) - _e732), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e677.w);
                let _e736 = vec4<f32>(_e725.x, _e725.y, _e713.z, _e725.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e736 / _e735)), sign(_e736), (_e735 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e742 = local_2;
                local_2 = clamp(_e742, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e745 = clamp(_e686, vec3<f32>(0f, 0f, 0f), _e677.www);
                let _e751 = vec4<f32>(_e745.x, _e677.y, _e677.z, _e677.w);
                let _e757 = vec4<f32>(_e751.x, _e745.y, _e751.z, _e751.w);
                phi_7171_ = vec4<f32>(_e757.x, _e757.y, _e745.z, _e757.w);
                if (_e677.w == 0f) {
                    phi_7171_ = vec4<f32>(_e745.x, _e745.y, _e745.z, 1f);
                }
                let _e767 = phi_7171_;
                let _e771 = (vec3(_e767.w) - _e767.xyz);
                let _e772 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e771 / (_e772 * _e767.w))), sign(_e771), (_e772 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e780 = local_2;
                let _e781 = (_e780 * _e692);
                local_1 = (select(_e781, (((_e780 + _e692) - _e781) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e780 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7039_ = 0i;
                loop {
                    let _e789 = phi_7039_;
                    if (_e789 < 3i) {
                        let _e792 = local_2[_e789];
                        if (_e792 <= 0.5f) {
                            let _e795 = local[_e789];
                            local_1[_e789] = (1f - _e795);
                        } else {
                            let _e799 = local[_e789];
                            if (_e799 <= 0.25f) {
                                let _e801 = local[_e789];
                                let _e804 = local[_e789];
                                local_1[_e789] = ((((16f * _e801) - 12f) * _e804) + 3f);
                            } else {
                                let _e808 = local[_e789];
                                local_1[_e789] = (inverseSqrt(_e808) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7039_ = (_e789 + 1i);
                    }
                }
                let _e813 = local_2;
                let _e817 = local_1;
                local_1 = (_e692 + ((_e692 * ((_e813 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e817));
                break;
            }
            case 9: {
                let _e820 = local_2;
                local_1 = abs((_e692 - _e820));
                break;
            }
            case 10: {
                let _e823 = local_2;
                local_1 = ((_e823 + _e692) - ((_e823 * 2f) * _e692));
                break;
            }
            case 12: {
                if cj {
                    let _e828 = local_2;
                    let _e829 = clamp(_e828, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e829;
                    let _e844 = (_e829 - vec3(min(min(_e829.x, _e829.y), _e829.z)));
                    let _e852 = (_e844 * ((max(max(_e692.x, _e692.y), _e692.z) - min(min(_e692.x, _e692.y), _e692.z)) / max(0.000062f, max(max(_e844.x, _e844.y), _e844.z))));
                    let _e853 = dot(_e692, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e856 = (_e852 - vec3(dot(_e852, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e869 = (vec2<f32>(_e853, (1f - _e853)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e856.x, _e856.y), _e856.z)), max(max(_e856.x, _e856.y), _e856.z))));
                    local_1 = ((_e856 * min(1f, min(_e869.x, _e869.y))) + vec3(_e853));
                }
                break;
            }
            case 13: {
                if cj {
                    let _e877 = local_2;
                    let _e878 = clamp(_e877, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e878;
                    let _e893 = (_e692 - vec3(min(min(_e692.x, _e692.y), _e692.z)));
                    let _e901 = (_e893 * ((max(max(_e878.x, _e878.y), _e878.z) - min(min(_e878.x, _e878.y), _e878.z)) / max(0.000062f, max(max(_e893.x, _e893.y), _e893.z))));
                    let _e902 = dot(_e692, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e905 = (_e901 - vec3(dot(_e901, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e918 = (vec2<f32>(_e902, (1f - _e902)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e905.x, _e905.y), _e905.z)), max(max(_e905.x, _e905.y), _e905.z))));
                    local_1 = ((_e905 * min(1f, min(_e918.x, _e918.y))) + vec3(_e902));
                }
                break;
            }
            case 14: {
                if cj {
                    let _e926 = local_2;
                    let _e927 = clamp(_e926, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e927;
                    let _e928 = dot(_e692, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e931 = (_e927 - vec3(dot(_e927, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e944 = (vec2<f32>(_e928, (1f - _e928)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e931.x, _e931.y), _e931.z)), max(max(_e931.x, _e931.y), _e931.z))));
                    local_1 = ((_e931 * min(1f, min(_e944.x, _e944.y))) + vec3(_e928));
                }
                break;
            }
            case 15: {
                if cj {
                    let _e952 = local_2;
                    let _e953 = clamp(_e952, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e953;
                    let _e954 = dot(_e953, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e957 = (_e692 - vec3(dot(_e692, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e970 = (vec2<f32>(_e954, (1f - _e954)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e957.x, _e957.y), _e957.z)), max(max(_e957.x, _e957.y), _e957.z))));
                    local_1 = ((_e957 * min(1f, min(_e970.x, _e970.y))) + vec3(_e954));
                }
                break;
            }
            default: {
            }
        }
        let _e978 = local_1;
        let _e981 = (mix(_e684, _e978, vec3(_e677.w)) * _e665.w);
        let _e987 = vec4<f32>(_e981.x, _e665.y, _e665.z, _e665.w);
        let _e993 = vec4<f32>(_e987.x, _e981.y, _e987.z, _e987.w);
        phi_7496_ = vec4<f32>(_e993.x, _e993.y, _e981.z, _e993.w);
    }
    let _e1001 = phi_7496_;
    let _e1002 = (_e1001 * _e663);
    let _e1006 = ((_e642 * (1f - _e1002.w)) + _e1002);
    let _e1007 = _e1006.xyz;
    let _e1010 = j.F3_;
    let _e1012 = j.G3_;
    if (dj && (_e1006.w != 0f)) {
        phi_7509_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e86.x) + (0.00583715f * _e86.y))))) * _e1010) + _e1012)) + _e1007);
    } else {
        phi_7509_ = _e1007;
    }
    let _e1028 = phi_7509_;
    let _e1034 = vec4<f32>(_e1028.x, _e1006.y, _e1006.z, _e1006.w);
    let _e1040 = vec4<f32>(_e1034.x, _e1028.y, _e1034.z, _e1034.w);
    let _e1046 = vec4<f32>(_e1040.x, _e1040.y, _e1028.z, _e1040.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1028.x + _e1028.y) + _e1028.z) + _e1006.w) == 0f) {
                break;
            }
            let _e1052 = (1f - _e1006.w);
            phi_7511_ = _e1046;
            if (_e1052 != 0f) {
                let _e1056 = n0_.r2_[_e121];
                phi_7511_ = (_e1046 + (unpack4x8unorm(_e1056) * _e1052));
            }
            let _e1061 = phi_7511_;
            n0_.r2_[_e121] = pack4x8unorm(_e1061);
            break;
        }
    }
    if (_e293 != 0u) {
        m0_.r2_[_e121] = _e293;
    }
    R4_.r2_[_e121] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) l2_: vec2<f32>, @location(1) V0_: vec4<f32>, @location(4) @interpolate(flat, either) S3_: u32, @location(3) @interpolate(flat, either) T1_: vec4<f32>, @location(5) @interpolate(flat, either) J1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    l2_1 = l2_;
    V0_1 = V0_;
    S3_1 = S3_;
    T1_1 = T1_;
    J1_1 = J1_;
    main_1();
}
