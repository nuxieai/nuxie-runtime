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
    var phi_6274_: f32;
    var phi_1697_: bool;
    var phi_5501_: f32;
    var phi_5500_: f32;
    var phi_5502_: f32;
    var phi_5505_: f32;
    var phi_5504_: f32;
    var phi_1734_: bool;
    var phi_5509_: f32;
    var phi_5506_: f32;
    var phi_5507_: f32;
    var phi_5508_: vec2<f32>;
    var phi_5532_: vec4<f32>;
    var phi_6239_: u32;
    var phi_6238_: u32;
    var phi_5530_: vec4<f32>;
    var phi_5539_: f32;
    var phi_5989_: i32;
    var phi_6057_: vec4<f32>;
    var phi_6223_: vec4<f32>;
    var phi_6236_: vec4<f32>;
    var phi_1425_: bool;
    var phi_6264_: u32;
    var phi_6294_: f32;
    var phi_7757_: f32;
    var phi_1457_: bool;
    var phi_6332_: f32;
    var phi_6333_: f32;
    var phi_7286_: i32;
    var phi_7426_: vec4<f32>;
    var phi_7769_: vec4<f32>;
    var phi_7782_: vec3<f32>;
    var phi_7784_: vec4<f32>;

    let _e88 = gl_FragCoord_1;
    let _e89 = _e88.xy;
    let _e92 = bitcast<vec2<u32>>(vec2<i32>(floor(_e89)));
    let _e94 = j.P6_;
    let _e123 = bitcast<i32>((((((_e92.y >> bitcast<u32>(5u)) * (((_e94 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e92.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e92.x & 28u) << bitcast<u32>(5u)) + ((_e92.y & 28u) << bitcast<u32>(2i)))) + (((_e92.y & 3u) << bitcast<u32>(2i)) + (_e92.x & 3u))));
    let _e124 = l2_1;
    let _e125 = textureSample(TB, U4_, _e124);
    phi_6274_ = 1f;
    if Yi {
        let _e126 = V0_1;
        let _e129 = min(_e126.xy, _e126.zw);
        phi_6274_ = clamp(min(_e129.x, _e129.y), 0f, 1f);
    }
    let _e135 = phi_6274_;
    let _e138 = R4_.r2_[_e123];
    let _e140 = (_e138 >> bitcast<u32>(17u));
    let _e144 = ((f32((_e138 & 131071u)) * 0.00048828125f) + -32f);
    let _e147 = VC.r2_[_e140];
    phi_5500_ = _e144;
    if ((_e147.x & 768u) != 0u) {
        let _e151 = abs(_e144);
        phi_1697_ = bj;
        if bj {
            phi_1697_ = ((_e147.x & 512u) != 0u);
        }
        let _e155 = phi_1697_;
        phi_5501_ = _e151;
        if _e155 {
            phi_5501_ = (1f - abs(((fract((_e151 * 0.5f)) * 2f) + -1f)));
        }
        let _e163 = phi_5501_;
        phi_5500_ = _e163;
    }
    let _e165 = phi_5500_;
    let _e166 = clamp(_e165, 0f, 1f);
    phi_5504_ = _e166;
    if Xi {
        let _e168 = (_e147.x >> bitcast<u32>(16u));
        phi_5505_ = _e166;
        if (_e168 != 0u) {
            let _e172 = m0_.r2_[_e123];
            if (_e168 == (_e172 >> bitcast<u32>(16i))) {
                phi_5502_ = min(_e166, unpack2x16float(_e172).x);
            } else {
                phi_5502_ = 0f;
            }
            let _e180 = phi_5502_;
            phi_5505_ = _e180;
        }
        let _e182 = phi_5505_;
        phi_5504_ = _e182;
    }
    let _e184 = phi_5504_;
    phi_1734_ = Yi;
    if Yi {
        phi_1734_ = ((_e147.x & 1024u) != 0u);
    }
    let _e188 = phi_1734_;
    phi_5509_ = _e184;
    if _e188 {
        let _e189 = (_e140 * 8u);
        let _e193 = JB.r2_[(_e189 + 2u)];
        let _e204 = JB.r2_[(_e189 + 3u)];
        let _e209 = _e204.zw;
        let _e211 = ((abs(((mat2x2<f32>(vec2<f32>(_e193.x, _e193.y), vec2<f32>(_e193.z, _e193.w)) * _e89) + _e204.xy)) * _e209) - _e209);
        phi_5509_ = min(_e184, clamp((min(_e211.x, _e211.y) + 0.5f), 0f, 1f));
    }
    let _e219 = phi_5509_;
    let _e220 = (_e147.x & 15u);
    let _e223 = ((_e147.x >> bitcast<u32>(4i)) & 15u);
    let _e225 = (Zi && (_e223 != 0u));
    if (_e220 <= 1u) {
        let _e305 = (Xi && (_e220 == 0u));
        phi_6239_ = 0u;
        if _e305 {
            phi_6239_ = (_e147.y | pack2x16float(vec2<f32>(_e219, 0f)));
        }
        let _e310 = phi_6239_;
        phi_6238_ = _e310;
        phi_5530_ = select(unpack4x8unorm(_e147.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e305));
    } else {
        let _e227 = (_e140 * 8u);
        let _e230 = JB.r2_[_e227];
        let _e241 = JB.r2_[(_e227 + 1u)];
        let _e244 = ((mat2x2<f32>(vec2<f32>(_e230.x, _e230.y), vec2<f32>(_e230.z, _e230.w)) * _e89) + _e241.xy);
        let _e248 = f32(_e220);
        let _e250 = j.M8_;
        let _e252 = j.N8_;
        let _e257 = ((_e241.w * _e250) + _e252);
        switch bitcast<i32>(0u) {
            default: {
                if (_e248 == 4f) {
                    let _e274 = normalize(vec2<f32>(_e244.x, -(_e244.y)));
                    let _e277 = acos(clamp(_e274.x, -1f, 1f));
                    if (_e274.y >= 0f) {
                        phi_5507_ = _e277;
                    } else {
                        phi_5507_ = -(_e277);
                    }
                    let _e282 = phi_5507_;
                    phi_5508_ = vec2<f32>((_e282 * 0.15915507f), _e257);
                    break;
                } else {
                    if (_e248 == 2f) {
                        phi_5506_ = _e244.x;
                    } else {
                        phi_5506_ = length(_e244);
                    }
                    let _e264 = phi_5506_;
                    phi_5508_ = vec2<f32>(((clamp(_e264, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e241.z < 0f))) + ((max(0f, _e241.z) * 0.001953125f) + 0.0009765625f)), _e257);
                    break;
                }
            }
        }
        let _e286 = phi_5508_;
        let _e287 = textureSampleLevel(XC, O8_, _e286, 0f);
        phi_5532_ = _e287;
        if !(_e225) {
            let _e291 = (_e287.xyz * _e287.w);
            phi_5532_ = vec4<f32>(_e291.x, _e291.y, _e291.z, (_e287.w * abs(bitcast<f32>(_e147.y))));
        }
        let _e301 = phi_5532_;
        phi_6238_ = 0u;
        phi_5530_ = _e301;
    }
    let _e314 = phi_6238_;
    let _e316 = phi_5530_;
    phi_6236_ = _e316;
    if _e225 {
        phi_6223_ = _e316;
        if ((_e316.w * _e219) != 0f) {
            let _e322 = n0_.r2_[_e123];
            let _e323 = unpack4x8unorm(_e322);
            let _e324 = _e316.xyz;
            local_5 = _e324;
            let _e325 = _e323.xyz;
            if (_e323.w != 0f) {
                phi_5539_ = (1f / _e323.w);
            } else {
                phi_5539_ = 0f;
            }
            let _e330 = phi_5539_;
            let _e331 = (_e325 * _e330);
            local_3 = _e331;
            switch bitcast<i32>(_e223) {
                case 11: {
                    let _e615 = local_5;
                    local_4 = (_e615 * _e331);
                    break;
                }
                case 1: {
                    let _e611 = local_5;
                    local_4 = ((_e611 + _e331) - (_e611 * _e331));
                    break;
                }
                case 2: {
                    let _e603 = local_5;
                    let _e604 = (_e603 * _e331);
                    local_4 = (select(_e604, (((_e603 + _e331) - _e604) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e331 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e601 = local_5;
                    local_4 = min(_e601, _e331);
                    break;
                }
                case 4: {
                    let _e599 = local_5;
                    local_4 = max(_e599, _e331);
                    break;
                }
                case 5: {
                    let _e570 = clamp(_e325, vec3<f32>(0f, 0f, 0f), _e323.www);
                    let _e576 = vec4<f32>(_e570.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e582 = vec4<f32>(_e576.x, _e570.y, _e576.z, _e576.w);
                    let _e589 = local_5;
                    let _e592 = (clamp((vec3<f32>(1f, 1f, 1f) - _e589), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e323.w);
                    let _e593 = vec4<f32>(_e582.x, _e582.y, _e570.z, _e582.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e593 / _e592)), sign(_e593), (_e592 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e531 = local_5;
                    local_5 = clamp(_e531, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e534 = clamp(_e325, vec3<f32>(0f, 0f, 0f), _e323.www);
                    let _e540 = vec4<f32>(_e534.x, _e323.y, _e323.z, _e323.w);
                    let _e546 = vec4<f32>(_e540.x, _e534.y, _e540.z, _e540.w);
                    phi_6057_ = vec4<f32>(_e546.x, _e546.y, _e534.z, _e546.w);
                    if (_e323.w == 0f) {
                        phi_6057_ = vec4<f32>(_e534.x, _e534.y, _e534.z, 1f);
                    }
                    let _e556 = phi_6057_;
                    let _e560 = (vec3(_e556.w) - _e556.xyz);
                    let _e561 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e560 / (_e561 * _e556.w))), sign(_e560), (_e561 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e523 = local_5;
                    let _e524 = (_e523 * _e331);
                    local_4 = (select(_e524, (((_e523 + _e331) - _e524) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e523 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_5989_ = 0i;
                    loop {
                        let _e492 = phi_5989_;
                        if (_e492 < 3i) {
                            let _e495 = local_5[_e492];
                            if (_e495 <= 0.5f) {
                                let _e512 = local_3[_e492];
                                local_4[_e492] = (1f - _e512);
                            } else {
                                let _e498 = local_3[_e492];
                                if (_e498 <= 0.25f) {
                                    let _e504 = local_3[_e492];
                                    let _e507 = local_3[_e492];
                                    local_4[_e492] = ((((16f * _e504) - 12f) * _e507) + 3f);
                                } else {
                                    let _e500 = local_3[_e492];
                                    local_4[_e492] = (inverseSqrt(_e500) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_5989_ = (_e492 + 1i);
                        }
                    }
                    let _e516 = local_5;
                    let _e520 = local_4;
                    local_4 = (_e331 + ((_e331 * ((_e516 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e520));
                    break;
                }
                case 9: {
                    let _e488 = local_5;
                    local_4 = abs((_e331 - _e488));
                    break;
                }
                case 10: {
                    let _e483 = local_5;
                    local_4 = ((_e483 + _e331) - ((_e483 * 2f) * _e331));
                    break;
                }
                case 12: {
                    if dj {
                        let _e434 = local_5;
                        let _e435 = clamp(_e434, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e435;
                        let _e450 = (_e435 - vec3(min(min(_e435.x, _e435.y), _e435.z)));
                        let _e458 = (_e450 * ((max(max(_e331.x, _e331.y), _e331.z) - min(min(_e331.x, _e331.y), _e331.z)) / max(0.000062f, max(max(_e450.x, _e450.y), _e450.z))));
                        let _e459 = dot(_e331, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e462 = (_e458 - vec3(dot(_e458, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e475 = (vec2<f32>(_e459, (1f - _e459)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e462.x, _e462.y), _e462.z)), max(max(_e462.x, _e462.y), _e462.z))));
                        local_4 = ((_e462 * min(1f, min(_e475.x, _e475.y))) + vec3(_e459));
                    }
                    break;
                }
                case 13: {
                    if dj {
                        let _e385 = local_5;
                        let _e386 = clamp(_e385, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e386;
                        let _e401 = (_e331 - vec3(min(min(_e331.x, _e331.y), _e331.z)));
                        let _e409 = (_e401 * ((max(max(_e386.x, _e386.y), _e386.z) - min(min(_e386.x, _e386.y), _e386.z)) / max(0.000062f, max(max(_e401.x, _e401.y), _e401.z))));
                        let _e410 = dot(_e331, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e413 = (_e409 - vec3(dot(_e409, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e426 = (vec2<f32>(_e410, (1f - _e410)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e413.x, _e413.y), _e413.z)), max(max(_e413.x, _e413.y), _e413.z))));
                        local_4 = ((_e413 * min(1f, min(_e426.x, _e426.y))) + vec3(_e410));
                    }
                    break;
                }
                case 14: {
                    if dj {
                        let _e359 = local_5;
                        let _e360 = clamp(_e359, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e360;
                        let _e361 = dot(_e331, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e364 = (_e360 - vec3(dot(_e360, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e377 = (vec2<f32>(_e361, (1f - _e361)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e364.x, _e364.y), _e364.z)), max(max(_e364.x, _e364.y), _e364.z))));
                        local_4 = ((_e364 * min(1f, min(_e377.x, _e377.y))) + vec3(_e361));
                    }
                    break;
                }
                case 15: {
                    if dj {
                        let _e333 = local_5;
                        let _e334 = clamp(_e333, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e334;
                        let _e335 = dot(_e334, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e338 = (_e331 - vec3(dot(_e331, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e351 = (vec2<f32>(_e335, (1f - _e335)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e338.x, _e338.y), _e338.z)), max(max(_e338.x, _e338.y), _e338.z))));
                        local_4 = ((_e338 * min(1f, min(_e351.x, _e351.y))) + vec3(_e335));
                    }
                    break;
                }
                default: {
                }
            }
            let _e617 = local_4;
            let _e619 = mix(_e324, _e617, vec3(_e323.w));
            let _e625 = vec4<f32>(_e619.x, _e316.y, _e316.z, _e316.w);
            let _e631 = vec4<f32>(_e625.x, _e619.y, _e625.z, _e625.w);
            phi_6223_ = vec4<f32>(_e631.x, _e631.y, _e619.z, _e631.w);
        }
        let _e639 = phi_6223_;
        let _e642 = (_e639.xyz * _e639.w);
        let _e648 = vec4<f32>(_e642.x, _e639.y, _e639.z, _e639.w);
        let _e654 = vec4<f32>(_e648.x, _e642.y, _e648.z, _e648.w);
        phi_6236_ = vec4<f32>(_e654.x, _e654.y, _e642.z, _e654.w);
    }
    let _e662 = phi_6236_;
    let _e663 = (_e662 * _e219);
    phi_1425_ = Xi;
    if Xi {
        let _e664 = S3_1;
        phi_1425_ = (_e664 != 0u);
    }
    let _e667 = phi_1425_;
    phi_7757_ = _e135;
    if _e667 {
        if (_e314 != 0u) {
            phi_6264_ = _e314;
        } else {
            let _e671 = m0_.r2_[_e123];
            phi_6264_ = _e671;
        }
        let _e673 = phi_6264_;
        let _e674 = S3_1;
        if (_e674 == (_e673 >> bitcast<u32>(16i))) {
            phi_6294_ = min(_e135, unpack2x16float(_e673).x);
        } else {
            phi_6294_ = 0f;
        }
        let _e682 = phi_6294_;
        phi_7757_ = _e682;
    }
    let _e684 = phi_7757_;
    let _e685 = T1_1;
    let _e686 = (_e125 * _e685);
    phi_1457_ = Zi;
    if Zi {
        let _e687 = J1_1;
        phi_1457_ = (_e687 != 0u);
    }
    let _e690 = phi_1457_;
    phi_7769_ = _e686;
    if _e690 {
        let _e693 = n0_.r2_[_e123];
        let _e698 = ((unpack4x8unorm(_e693) * (1f - _e663.w)) + _e663);
        if (_e686.w != 0f) {
            phi_6332_ = (1f / _e686.w);
        } else {
            phi_6332_ = 0f;
        }
        let _e704 = phi_6332_;
        let _e705 = (_e686.xyz * _e704);
        let _e706 = J1_1;
        local_2 = _e705;
        let _e707 = _e698.xyz;
        if (_e698.w != 0f) {
            phi_6333_ = (1f / _e698.w);
        } else {
            phi_6333_ = 0f;
        }
        let _e712 = phi_6333_;
        let _e713 = (_e707 * _e712);
        local = _e713;
        switch bitcast<i32>(_e706) {
            case 11: {
                let _e997 = local_2;
                local_1 = (_e997 * _e713);
                break;
            }
            case 1: {
                let _e993 = local_2;
                local_1 = ((_e993 + _e713) - (_e993 * _e713));
                break;
            }
            case 2: {
                let _e985 = local_2;
                let _e986 = (_e985 * _e713);
                local_1 = (select(_e986, (((_e985 + _e713) - _e986) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e713 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e983 = local_2;
                local_1 = min(_e983, _e713);
                break;
            }
            case 4: {
                let _e981 = local_2;
                local_1 = max(_e981, _e713);
                break;
            }
            case 5: {
                let _e952 = clamp(_e707, vec3<f32>(0f, 0f, 0f), _e698.www);
                let _e958 = vec4<f32>(_e952.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e964 = vec4<f32>(_e958.x, _e952.y, _e958.z, _e958.w);
                let _e971 = local_2;
                let _e974 = (clamp((vec3<f32>(1f, 1f, 1f) - _e971), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e698.w);
                let _e975 = vec4<f32>(_e964.x, _e964.y, _e952.z, _e964.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e975 / _e974)), sign(_e975), (_e974 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e913 = local_2;
                local_2 = clamp(_e913, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e916 = clamp(_e707, vec3<f32>(0f, 0f, 0f), _e698.www);
                let _e922 = vec4<f32>(_e916.x, _e698.y, _e698.z, _e698.w);
                let _e928 = vec4<f32>(_e922.x, _e916.y, _e922.z, _e922.w);
                phi_7426_ = vec4<f32>(_e928.x, _e928.y, _e916.z, _e928.w);
                if (_e698.w == 0f) {
                    phi_7426_ = vec4<f32>(_e916.x, _e916.y, _e916.z, 1f);
                }
                let _e938 = phi_7426_;
                let _e942 = (vec3(_e938.w) - _e938.xyz);
                let _e943 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e942 / (_e943 * _e938.w))), sign(_e942), (_e943 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e905 = local_2;
                let _e906 = (_e905 * _e713);
                local_1 = (select(_e906, (((_e905 + _e713) - _e906) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e905 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7286_ = 0i;
                loop {
                    let _e874 = phi_7286_;
                    if (_e874 < 3i) {
                        let _e877 = local_2[_e874];
                        if (_e877 <= 0.5f) {
                            let _e894 = local[_e874];
                            local_1[_e874] = (1f - _e894);
                        } else {
                            let _e880 = local[_e874];
                            if (_e880 <= 0.25f) {
                                let _e886 = local[_e874];
                                let _e889 = local[_e874];
                                local_1[_e874] = ((((16f * _e886) - 12f) * _e889) + 3f);
                            } else {
                                let _e882 = local[_e874];
                                local_1[_e874] = (inverseSqrt(_e882) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7286_ = (_e874 + 1i);
                    }
                }
                let _e898 = local_2;
                let _e902 = local_1;
                local_1 = (_e713 + ((_e713 * ((_e898 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e902));
                break;
            }
            case 9: {
                let _e870 = local_2;
                local_1 = abs((_e713 - _e870));
                break;
            }
            case 10: {
                let _e865 = local_2;
                local_1 = ((_e865 + _e713) - ((_e865 * 2f) * _e713));
                break;
            }
            case 12: {
                if dj {
                    let _e816 = local_2;
                    let _e817 = clamp(_e816, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e817;
                    let _e832 = (_e817 - vec3(min(min(_e817.x, _e817.y), _e817.z)));
                    let _e840 = (_e832 * ((max(max(_e713.x, _e713.y), _e713.z) - min(min(_e713.x, _e713.y), _e713.z)) / max(0.000062f, max(max(_e832.x, _e832.y), _e832.z))));
                    let _e841 = dot(_e713, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e844 = (_e840 - vec3(dot(_e840, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e857 = (vec2<f32>(_e841, (1f - _e841)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e844.x, _e844.y), _e844.z)), max(max(_e844.x, _e844.y), _e844.z))));
                    local_1 = ((_e844 * min(1f, min(_e857.x, _e857.y))) + vec3(_e841));
                }
                break;
            }
            case 13: {
                if dj {
                    let _e767 = local_2;
                    let _e768 = clamp(_e767, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e768;
                    let _e783 = (_e713 - vec3(min(min(_e713.x, _e713.y), _e713.z)));
                    let _e791 = (_e783 * ((max(max(_e768.x, _e768.y), _e768.z) - min(min(_e768.x, _e768.y), _e768.z)) / max(0.000062f, max(max(_e783.x, _e783.y), _e783.z))));
                    let _e792 = dot(_e713, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e795 = (_e791 - vec3(dot(_e791, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e808 = (vec2<f32>(_e792, (1f - _e792)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e795.x, _e795.y), _e795.z)), max(max(_e795.x, _e795.y), _e795.z))));
                    local_1 = ((_e795 * min(1f, min(_e808.x, _e808.y))) + vec3(_e792));
                }
                break;
            }
            case 14: {
                if dj {
                    let _e741 = local_2;
                    let _e742 = clamp(_e741, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e742;
                    let _e743 = dot(_e713, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e746 = (_e742 - vec3(dot(_e742, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e759 = (vec2<f32>(_e743, (1f - _e743)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e746.x, _e746.y), _e746.z)), max(max(_e746.x, _e746.y), _e746.z))));
                    local_1 = ((_e746 * min(1f, min(_e759.x, _e759.y))) + vec3(_e743));
                }
                break;
            }
            case 15: {
                if dj {
                    let _e715 = local_2;
                    let _e716 = clamp(_e715, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e716;
                    let _e717 = dot(_e716, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e720 = (_e713 - vec3(dot(_e713, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e733 = (vec2<f32>(_e717, (1f - _e717)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e720.x, _e720.y), _e720.z)), max(max(_e720.x, _e720.y), _e720.z))));
                    local_1 = ((_e720 * min(1f, min(_e733.x, _e733.y))) + vec3(_e717));
                }
                break;
            }
            default: {
            }
        }
        let _e999 = local_1;
        let _e1002 = (mix(_e705, _e999, vec3(_e698.w)) * _e686.w);
        let _e1008 = vec4<f32>(_e1002.x, _e686.y, _e686.z, _e686.w);
        let _e1014 = vec4<f32>(_e1008.x, _e1002.y, _e1008.z, _e1008.w);
        phi_7769_ = vec4<f32>(_e1014.x, _e1014.y, _e1002.z, _e1014.w);
    }
    let _e1022 = phi_7769_;
    let _e1023 = (_e1022 * _e684);
    let _e1027 = ((_e663 * (1f - _e1023.w)) + _e1023);
    let _e1028 = _e1027.xyz;
    let _e1031 = j.F3_;
    let _e1033 = j.G3_;
    if (ej && (_e1027.w != 0f)) {
        phi_7782_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e88.x) + (0.00583715f * _e88.y))))) * _e1031) + _e1033)) + _e1028);
    } else {
        phi_7782_ = _e1028;
    }
    let _e1049 = phi_7782_;
    let _e1055 = vec4<f32>(_e1049.x, _e1027.y, _e1027.z, _e1027.w);
    let _e1061 = vec4<f32>(_e1055.x, _e1049.y, _e1055.z, _e1055.w);
    let _e1067 = vec4<f32>(_e1061.x, _e1061.y, _e1049.z, _e1061.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1049.x + _e1049.y) + _e1049.z) + _e1027.w) == 0f) {
                break;
            }
            let _e1073 = (1f - _e1027.w);
            phi_7784_ = _e1067;
            if (_e1073 != 0f) {
                let _e1077 = n0_.r2_[_e123];
                phi_7784_ = (_e1067 + (unpack4x8unorm(_e1077) * _e1073));
            }
            let _e1082 = phi_7784_;
            n0_.r2_[_e123] = pack4x8unorm(_e1082);
            break;
        }
    }
    if (_e314 != 0u) {
        m0_.r2_[_e123] = _e314;
    }
    R4_.r2_[_e123] = 65536u;
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
