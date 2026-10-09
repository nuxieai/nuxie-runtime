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
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe;
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
    var phi_1458_: bool;
    var phi_3540_: f32;
    var phi_3539_: f32;
    var phi_3541_: f32;
    var phi_3544_: f32;
    var phi_3543_: f32;
    var phi_1495_: bool;
    var phi_3560_: f32;
    var phi_3545_: f32;
    var phi_3557_: vec4<f32>;
    var phi_3555_: vec4<f32>;
    var phi_3563_: f32;
    var phi_3986_: vec4<f32>;
    var phi_3930_: i32;
    var phi_4125_: vec4<f32>;
    var phi_4138_: vec4<f32>;
    var phi_4139_: vec3<f32>;
    var phi_4141_: vec4<f32>;

    let _e77 = gl_FragCoord_1;
    let _e78 = _e77.xy;
    let _e81 = bitcast<vec2<u32>>(vec2<i32>(floor(_e78)));
    let _e83 = j.P6_;
    let _e112 = bitcast<i32>((((((_e81.y >> bitcast<u32>(5u)) * (((_e83 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e81.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e81.x & 28u) << bitcast<u32>(5u)) + ((_e81.y & 28u) << bitcast<u32>(2i)))) + (((_e81.y & 3u) << bitcast<u32>(2i)) + (_e81.x & 3u))));
    let _e115 = R4_.r2_[_e112];
    let _e119 = ((f32((_e115 & 131071u)) * 0.00048828125f) + -32f);
    let _e121 = (_e115 >> bitcast<u32>(17u));
    let _e124 = VC.r2_[_e121];
    phi_3539_ = _e119;
    if ((_e124.x & 768u) != 0u) {
        let _e128 = abs(_e119);
        phi_1458_ = aj;
        if aj {
            phi_1458_ = ((_e124.x & 512u) != 0u);
        }
        let _e132 = phi_1458_;
        phi_3540_ = _e128;
        if _e132 {
            phi_3540_ = (1f - abs(((fract((_e128 * 0.5f)) * 2f) + -1f)));
        }
        let _e140 = phi_3540_;
        phi_3539_ = _e140;
    }
    let _e142 = phi_3539_;
    let _e143 = clamp(_e142, 0f, 1f);
    phi_3543_ = _e143;
    if Wi {
        let _e145 = (_e124.x >> bitcast<u32>(16u));
        phi_3544_ = _e143;
        if (_e145 != 0u) {
            let _e149 = m0_.r2_[_e112];
            if (_e145 == (_e149 >> bitcast<u32>(16i))) {
                phi_3541_ = min(_e143, unpack2x16float(_e149).x);
            } else {
                phi_3541_ = 0f;
            }
            let _e157 = phi_3541_;
            phi_3544_ = _e157;
        }
        let _e159 = phi_3544_;
        phi_3543_ = _e159;
    }
    let _e161 = phi_3543_;
    phi_1495_ = Xi;
    if Xi {
        phi_1495_ = ((_e124.x & 1024u) != 0u);
    }
    let _e165 = phi_1495_;
    phi_3560_ = _e161;
    if _e165 {
        let _e166 = (_e121 * 8u);
        let _e170 = JB.r2_[(_e166 + 2u)];
        let _e181 = JB.r2_[(_e166 + 3u)];
        let _e186 = _e181.zw;
        let _e188 = ((abs(((mat2x2<f32>(vec2<f32>(_e170.x, _e170.y), vec2<f32>(_e170.z, _e170.w)) * _e78) + _e181.xy)) * _e186) - _e186);
        phi_3560_ = min(_e161, clamp((min(_e188.x, _e188.y) + 0.5f), 0f, 1f));
    }
    let _e196 = phi_3560_;
    let _e197 = (_e124.x & 15u);
    let _e200 = ((_e124.x >> bitcast<u32>(4i)) & 15u);
    let _e202 = (Yi && (_e200 != 0u));
    if (_e197 <= 1u) {
        phi_3555_ = select(unpack4x8unorm(_e124.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Wi && (_e197 == 0u))));
    } else {
        let _e210 = (_e121 * 8u);
        let _e213 = JB.r2_[_e210];
        let _e224 = JB.r2_[(_e210 + 1u)];
        let _e227 = ((mat2x2<f32>(vec2<f32>(_e213.x, _e213.y), vec2<f32>(_e213.z, _e213.w)) * _e78) + _e224.xy);
        let _e233 = j.L8_;
        let _e235 = j.M8_;
        if (f32(_e197) == 2f) {
            phi_3545_ = _e227.x;
        } else {
            phi_3545_ = length(_e227);
        }
        let _e245 = phi_3545_;
        let _e251 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e245, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e224.z < 0f))) + ((max(0f, _e224.z) * 0.001953125f) + 0.0009765625f)), ((_e224.w * _e233) + _e235)), 0f);
        phi_3557_ = _e251;
        if !(_e202) {
            let _e255 = (_e251.xyz * _e251.w);
            phi_3557_ = vec4<f32>(_e255.x, _e255.y, _e255.z, (_e251.w * abs(bitcast<f32>(_e124.y))));
        }
        let _e265 = phi_3557_;
        phi_3555_ = _e265;
    }
    let _e267 = phi_3555_;
    phi_4138_ = _e267;
    if _e202 {
        phi_4125_ = _e267;
        if ((_e267.w * _e196) != 0f) {
            let _e273 = n0_.r2_[_e112];
            let _e274 = unpack4x8unorm(_e273);
            let _e275 = _e267.xyz;
            local_2 = _e275;
            let _e276 = _e274.xyz;
            if (_e274.w != 0f) {
                phi_3563_ = (1f / _e274.w);
            } else {
                phi_3563_ = 0f;
            }
            let _e281 = phi_3563_;
            let _e282 = (_e276 * _e281);
            local = _e282;
            switch bitcast<i32>(_e200) {
                case 11: {
                    let _e284 = local_2;
                    local_1 = (_e284 * _e282);
                    break;
                }
                case 1: {
                    let _e286 = local_2;
                    local_1 = ((_e286 + _e282) - (_e286 * _e282));
                    break;
                }
                case 2: {
                    let _e290 = local_2;
                    let _e291 = (_e290 * _e282);
                    local_1 = (select(_e291, (((_e290 + _e282) - _e291) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e282 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e298 = local_2;
                    local_1 = min(_e298, _e282);
                    break;
                }
                case 4: {
                    let _e300 = local_2;
                    local_1 = max(_e300, _e282);
                    break;
                }
                case 5: {
                    let _e303 = clamp(_e276, vec3<f32>(0f, 0f, 0f), _e274.www);
                    let _e309 = vec4<f32>(_e303.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e315 = vec4<f32>(_e309.x, _e303.y, _e309.z, _e309.w);
                    let _e322 = local_2;
                    let _e325 = (clamp((vec3<f32>(1f, 1f, 1f) - _e322), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e274.w);
                    let _e326 = vec4<f32>(_e315.x, _e315.y, _e303.z, _e315.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e326 / _e325)), sign(_e326), (_e325 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e332 = local_2;
                    local_2 = clamp(_e332, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e335 = clamp(_e276, vec3<f32>(0f, 0f, 0f), _e274.www);
                    let _e341 = vec4<f32>(_e335.x, _e274.y, _e274.z, _e274.w);
                    let _e347 = vec4<f32>(_e341.x, _e335.y, _e341.z, _e341.w);
                    phi_3986_ = vec4<f32>(_e347.x, _e347.y, _e335.z, _e347.w);
                    if (_e274.w == 0f) {
                        phi_3986_ = vec4<f32>(_e335.x, _e335.y, _e335.z, 1f);
                    }
                    let _e357 = phi_3986_;
                    let _e361 = (vec3(_e357.w) - _e357.xyz);
                    let _e362 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e361 / (_e362 * _e357.w))), sign(_e361), (_e362 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e370 = local_2;
                    let _e371 = (_e370 * _e282);
                    local_1 = (select(_e371, (((_e370 + _e282) - _e371) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e370 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3930_ = 0i;
                    loop {
                        let _e379 = phi_3930_;
                        if (_e379 < 3i) {
                            let _e382 = local_2[_e379];
                            if (_e382 <= 0.5f) {
                                let _e385 = local[_e379];
                                local_1[_e379] = (1f - _e385);
                            } else {
                                let _e389 = local[_e379];
                                if (_e389 <= 0.25f) {
                                    let _e391 = local[_e379];
                                    let _e394 = local[_e379];
                                    local_1[_e379] = ((((16f * _e391) - 12f) * _e394) + 3f);
                                } else {
                                    let _e398 = local[_e379];
                                    local_1[_e379] = (inverseSqrt(_e398) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3930_ = (_e379 + 1i);
                        }
                    }
                    let _e403 = local_2;
                    let _e407 = local_1;
                    local_1 = (_e282 + ((_e282 * ((_e403 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e407));
                    break;
                }
                case 9: {
                    let _e410 = local_2;
                    local_1 = abs((_e282 - _e410));
                    break;
                }
                case 10: {
                    let _e413 = local_2;
                    local_1 = ((_e413 + _e282) - ((_e413 * 2f) * _e282));
                    break;
                }
                case 12: {
                    if cj {
                        let _e418 = local_2;
                        let _e419 = clamp(_e418, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e419;
                        let _e434 = (_e419 - vec3(min(min(_e419.x, _e419.y), _e419.z)));
                        let _e442 = (_e434 * ((max(max(_e282.x, _e282.y), _e282.z) - min(min(_e282.x, _e282.y), _e282.z)) / max(0.000062f, max(max(_e434.x, _e434.y), _e434.z))));
                        let _e443 = dot(_e282, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e446 = (_e442 - vec3(dot(_e442, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e459 = (vec2<f32>(_e443, (1f - _e443)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e446.x, _e446.y), _e446.z)), max(max(_e446.x, _e446.y), _e446.z))));
                        local_1 = ((_e446 * min(1f, min(_e459.x, _e459.y))) + vec3(_e443));
                    }
                    break;
                }
                case 13: {
                    if cj {
                        let _e467 = local_2;
                        let _e468 = clamp(_e467, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e468;
                        let _e483 = (_e282 - vec3(min(min(_e282.x, _e282.y), _e282.z)));
                        let _e491 = (_e483 * ((max(max(_e468.x, _e468.y), _e468.z) - min(min(_e468.x, _e468.y), _e468.z)) / max(0.000062f, max(max(_e483.x, _e483.y), _e483.z))));
                        let _e492 = dot(_e282, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e495 = (_e491 - vec3(dot(_e491, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e508 = (vec2<f32>(_e492, (1f - _e492)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e495.x, _e495.y), _e495.z)), max(max(_e495.x, _e495.y), _e495.z))));
                        local_1 = ((_e495 * min(1f, min(_e508.x, _e508.y))) + vec3(_e492));
                    }
                    break;
                }
                case 14: {
                    if cj {
                        let _e516 = local_2;
                        let _e517 = clamp(_e516, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e517;
                        let _e518 = dot(_e282, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e521 = (_e517 - vec3(dot(_e517, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e534 = (vec2<f32>(_e518, (1f - _e518)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e521.x, _e521.y), _e521.z)), max(max(_e521.x, _e521.y), _e521.z))));
                        local_1 = ((_e521 * min(1f, min(_e534.x, _e534.y))) + vec3(_e518));
                    }
                    break;
                }
                case 15: {
                    if cj {
                        let _e542 = local_2;
                        let _e543 = clamp(_e542, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e543;
                        let _e544 = dot(_e543, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e547 = (_e282 - vec3(dot(_e282, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e560 = (vec2<f32>(_e544, (1f - _e544)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e547.x, _e547.y), _e547.z)), max(max(_e547.x, _e547.y), _e547.z))));
                        local_1 = ((_e547 * min(1f, min(_e560.x, _e560.y))) + vec3(_e544));
                    }
                    break;
                }
                default: {
                }
            }
            let _e568 = local_1;
            let _e570 = mix(_e275, _e568, vec3(_e274.w));
            let _e576 = vec4<f32>(_e570.x, _e267.y, _e267.z, _e267.w);
            let _e582 = vec4<f32>(_e576.x, _e570.y, _e576.z, _e576.w);
            phi_4125_ = vec4<f32>(_e582.x, _e582.y, _e570.z, _e582.w);
        }
        let _e590 = phi_4125_;
        let _e593 = (_e590.xyz * _e590.w);
        let _e599 = vec4<f32>(_e593.x, _e590.y, _e590.z, _e590.w);
        let _e605 = vec4<f32>(_e599.x, _e593.y, _e599.z, _e599.w);
        phi_4138_ = vec4<f32>(_e605.x, _e605.y, _e593.z, _e605.w);
    }
    let _e613 = phi_4138_;
    let _e614 = (_e613 * _e196);
    let _e615 = _e614.xyz;
    let _e618 = j.F3_;
    let _e620 = j.G3_;
    if (dj && (_e614.w != 0f)) {
        phi_4139_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e77.x) + (0.00583715f * _e77.y))))) * _e618) + _e620)) + _e615);
    } else {
        phi_4139_ = _e615;
    }
    let _e636 = phi_4139_;
    let _e642 = vec4<f32>(_e636.x, _e614.y, _e614.z, _e614.w);
    let _e648 = vec4<f32>(_e642.x, _e636.y, _e642.z, _e642.w);
    let _e654 = vec4<f32>(_e648.x, _e648.y, _e636.z, _e648.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e636.x + _e636.y) + _e636.z) + _e614.w) == 0f) {
                break;
            }
            let _e660 = (1f - _e614.w);
            phi_4141_ = _e654;
            if (_e660 != 0f) {
                let _e664 = n0_.r2_[_e112];
                phi_4141_ = (_e654 + (unpack4x8unorm(_e664) * _e660));
            }
            let _e669 = phi_4141_;
            n0_.r2_[_e112] = pack4x8unorm(_e669);
            break;
        }
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
