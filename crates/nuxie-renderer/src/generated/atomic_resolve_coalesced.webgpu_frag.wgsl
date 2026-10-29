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
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> Q1_: vec4<f32>;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1402_: bool;
    var phi_3435_: f32;
    var phi_3434_: f32;
    var phi_3436_: f32;
    var phi_3439_: f32;
    var phi_3438_: f32;
    var phi_1439_: bool;
    var phi_3457_: f32;
    var phi_3440_: f32;
    var phi_3441_: f32;
    var phi_3442_: vec2<f32>;
    var phi_3454_: vec4<f32>;
    var phi_3452_: vec4<f32>;
    var phi_3462_: f32;
    var phi_3885_: i32;
    var phi_3949_: vec4<f32>;
    var phi_4106_: vec4<f32>;
    var phi_4119_: vec4<f32>;
    var phi_4120_: vec4<f32>;

    let _e76 = gl_FragCoord_1;
    let _e77 = _e76.xy;
    let _e80 = bitcast<vec2<u32>>(vec2<i32>(floor(_e77)));
    let _e82 = j.R6_;
    let _e111 = bitcast<i32>((((((_e80.y >> bitcast<u32>(5u)) * (((_e82 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e80.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e80.x & 28u) << bitcast<u32>(5u)) + ((_e80.y & 28u) << bitcast<u32>(2i)))) + (((_e80.y & 3u) << bitcast<u32>(2i)) + (_e80.x & 3u))));
    let _e114 = W4_.x2_[_e111];
    let _e118 = ((f32((_e114 & 131071u)) * 0.00048828125f) + -32f);
    let _e120 = (_e114 >> bitcast<u32>(17u));
    let _e125 = JB.x2_[((_e120 * 4u) + 1u)];
    phi_3434_ = _e118;
    if ((_e125.z & 768u) != 0u) {
        let _e129 = abs(_e118);
        phi_1402_ = oj;
        if oj {
            phi_1402_ = ((_e125.z & 512u) != 0u);
        }
        let _e133 = phi_1402_;
        phi_3435_ = _e129;
        if _e133 {
            phi_3435_ = (1f - abs(((fract((_e129 * 0.5f)) * 2f) + -1f)));
        }
        let _e141 = phi_3435_;
        phi_3434_ = _e141;
    }
    let _e143 = phi_3434_;
    let _e144 = clamp(_e143, 0f, 1f);
    phi_3438_ = _e144;
    if jj {
        let _e146 = (_e125.z >> bitcast<u32>(16u));
        phi_3439_ = _e144;
        if (_e146 != 0u) {
            let _e150 = m0_.x2_[_e111];
            if (_e146 == (_e150 >> bitcast<u32>(16i))) {
                phi_3436_ = min(_e144, unpack2x16float(_e150).x);
            } else {
                phi_3436_ = 0f;
            }
            let _e158 = phi_3436_;
            phi_3439_ = _e158;
        }
        let _e160 = phi_3439_;
        phi_3438_ = _e160;
    }
    let _e162 = phi_3438_;
    phi_1439_ = kj;
    if kj {
        phi_1439_ = ((_e125.z & 1024u) != 0u);
    }
    let _e166 = phi_1439_;
    phi_3457_ = _e162;
    if _e166 {
        let _e167 = (_e120 * 8u);
        let _e171 = KB.x2_[(_e167 + 2u)];
        let _e182 = KB.x2_[(_e167 + 3u)];
        let _e187 = _e182.zw;
        let _e189 = ((abs(((mat2x2<f32>(vec2<f32>(_e171.x, _e171.y), vec2<f32>(_e171.z, _e171.w)) * _e77) + _e182.xy)) * _e187) - _e187);
        phi_3457_ = min(_e162, clamp((min(_e189.x, _e189.y) + 0.5f), 0f, 1f));
    }
    let _e197 = phi_3457_;
    let _e198 = (_e125.z & 15u);
    let _e201 = ((_e125.z >> bitcast<u32>(4i)) & 15u);
    let _e203 = (lj && (_e201 != 0u));
    if (_e198 <= 1u) {
        phi_3452_ = select(unpack4x8unorm(_e125.w), vec4<f32>(0f, 0f, 0f, 0f), vec4((jj && (_e198 == 0u))));
    } else {
        let _e205 = (_e120 * 8u);
        let _e208 = KB.x2_[_e205];
        let _e219 = KB.x2_[(_e205 + 1u)];
        let _e222 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e77) + _e219.xy);
        let _e226 = f32(_e198);
        let _e228 = j.O8_;
        let _e230 = j.P8_;
        let _e235 = ((_e219.w * _e228) + _e230);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e226 == 4f)) {
                    let _e249 = normalize(_e222);
                    let _e252 = acos(clamp(_e249.x, -1f, 1f));
                    if (_e249.y >= 0f) {
                        phi_3441_ = _e252;
                    } else {
                        phi_3441_ = -(_e252);
                    }
                    let _e257 = phi_3441_;
                    phi_3442_ = vec2<f32>((_e257 * 0.15915494f), _e235);
                    break;
                } else {
                    if (_e226 == 2f) {
                        phi_3440_ = _e222.x;
                    } else {
                        phi_3440_ = length(_e222);
                    }
                    let _e243 = phi_3440_;
                    phi_3442_ = vec2<f32>(((clamp(_e243, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e219.z < 0f))) + ((max(0f, _e219.z) * 0.001953125f) + 0.0009765625f)), _e235);
                    break;
                }
            }
        }
        let _e261 = phi_3442_;
        let _e262 = textureSampleLevel(YC, Q8_, _e261, 0f);
        phi_3454_ = _e262;
        if !(_e203) {
            let _e266 = (_e262.xyz * _e262.w);
            phi_3454_ = vec4<f32>(_e266.x, _e266.y, _e266.z, (_e262.w * abs(bitcast<f32>(_e125.w))));
        }
        let _e276 = phi_3454_;
        phi_3452_ = _e276;
    }
    let _e284 = phi_3452_;
    phi_4119_ = _e284;
    if _e203 {
        phi_4106_ = _e284;
        if ((_e284.w * _e197) != 0f) {
            let _e290 = o0_.x2_[_e111];
            let _e291 = unpack4x8unorm(_e290);
            let _e292 = _e284.xyz;
            local_2 = _e292;
            let _e293 = _e291.xyz;
            if (_e291.w != 0f) {
                phi_3462_ = (1f / _e291.w);
            } else {
                phi_3462_ = 0f;
            }
            let _e298 = phi_3462_;
            let _e299 = (_e293 * _e298);
            local = _e299;
            switch bitcast<i32>(_e201) {
                case 11: {
                    let _e583 = local_2;
                    local_1 = (_e583 * _e299);
                    break;
                }
                case 1: {
                    let _e579 = local_2;
                    local_1 = ((_e579 + _e299) - (_e579 * _e299));
                    break;
                }
                case 2: {
                    let _e571 = local_2;
                    let _e572 = (_e571 * _e299);
                    local_1 = (select(_e572, (((_e571 + _e299) - _e572) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e299 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e569 = local_2;
                    local_1 = min(_e569, _e299);
                    break;
                }
                case 4: {
                    let _e567 = local_2;
                    local_1 = max(_e567, _e299);
                    break;
                }
                case 5: {
                    let _e538 = clamp(_e293, vec3<f32>(0f, 0f, 0f), _e291.www);
                    let _e544 = vec4<f32>(_e538.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e550 = vec4<f32>(_e544.x, _e538.y, _e544.z, _e544.w);
                    let _e557 = local_2;
                    let _e560 = (clamp((vec3<f32>(1f, 1f, 1f) - _e557), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e291.w);
                    let _e561 = vec4<f32>(_e550.x, _e550.y, _e538.z, _e550.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e561 / _e560)), sign(_e561), (_e560 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e499 = local_2;
                    local_2 = clamp(_e499, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e502 = clamp(_e293, vec3<f32>(0f, 0f, 0f), _e291.www);
                    let _e508 = vec4<f32>(_e502.x, _e291.y, _e291.z, _e291.w);
                    let _e514 = vec4<f32>(_e508.x, _e502.y, _e508.z, _e508.w);
                    phi_3949_ = vec4<f32>(_e514.x, _e514.y, _e502.z, _e514.w);
                    if (_e291.w == 0f) {
                        phi_3949_ = vec4<f32>(_e502.x, _e502.y, _e502.z, 1f);
                    }
                    let _e524 = phi_3949_;
                    let _e528 = (vec3(_e524.w) - _e524.xyz);
                    let _e529 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e528 / (_e529 * _e524.w))), sign(_e528), (_e529 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e491 = local_2;
                    let _e492 = (_e491 * _e299);
                    local_1 = (select(_e492, (((_e491 + _e299) - _e492) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e491 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3885_ = 0i;
                    loop {
                        let _e460 = phi_3885_;
                        if (_e460 < 3i) {
                            let _e463 = local_2[_e460];
                            if (_e463 <= 0.5f) {
                                let _e480 = local[_e460];
                                local_1[_e460] = (1f - _e480);
                            } else {
                                let _e466 = local[_e460];
                                if (_e466 <= 0.25f) {
                                    let _e472 = local[_e460];
                                    let _e475 = local[_e460];
                                    local_1[_e460] = ((((16f * _e472) - 12f) * _e475) + 3f);
                                } else {
                                    let _e468 = local[_e460];
                                    local_1[_e460] = (inverseSqrt(_e468) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3885_ = (_e460 + 1i);
                        }
                    }
                    let _e484 = local_2;
                    let _e488 = local_1;
                    local_1 = (_e299 + ((_e299 * ((_e484 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e488));
                    break;
                }
                case 9: {
                    let _e456 = local_2;
                    local_1 = abs((_e299 - _e456));
                    break;
                }
                case 10: {
                    let _e451 = local_2;
                    local_1 = ((_e451 + _e299) - ((_e451 * 2f) * _e299));
                    break;
                }
                case 12: {
                    if qj {
                        let _e402 = local_2;
                        let _e403 = clamp(_e402, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e403;
                        let _e418 = (_e403 - vec3(min(min(_e403.x, _e403.y), _e403.z)));
                        let _e426 = (_e418 * ((max(max(_e299.x, _e299.y), _e299.z) - min(min(_e299.x, _e299.y), _e299.z)) / max(0.000062f, max(max(_e418.x, _e418.y), _e418.z))));
                        let _e427 = dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e430 = (_e426 - vec3(dot(_e426, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e443 = (vec2<f32>(_e427, (1f - _e427)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e430.x, _e430.y), _e430.z)), max(max(_e430.x, _e430.y), _e430.z))));
                        local_1 = ((_e430 * min(1f, min(_e443.x, _e443.y))) + vec3(_e427));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e353 = local_2;
                        let _e354 = clamp(_e353, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e354;
                        let _e369 = (_e299 - vec3(min(min(_e299.x, _e299.y), _e299.z)));
                        let _e377 = (_e369 * ((max(max(_e354.x, _e354.y), _e354.z) - min(min(_e354.x, _e354.y), _e354.z)) / max(0.000062f, max(max(_e369.x, _e369.y), _e369.z))));
                        let _e378 = dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e381 = (_e377 - vec3(dot(_e377, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e394 = (vec2<f32>(_e378, (1f - _e378)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e381.x, _e381.y), _e381.z)), max(max(_e381.x, _e381.y), _e381.z))));
                        local_1 = ((_e381 * min(1f, min(_e394.x, _e394.y))) + vec3(_e378));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e327 = local_2;
                        let _e328 = clamp(_e327, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e328;
                        let _e329 = dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e332 = (_e328 - vec3(dot(_e328, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e345 = (vec2<f32>(_e329, (1f - _e329)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e332.x, _e332.y), _e332.z)), max(max(_e332.x, _e332.y), _e332.z))));
                        local_1 = ((_e332 * min(1f, min(_e345.x, _e345.y))) + vec3(_e329));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e301 = local_2;
                        let _e302 = clamp(_e301, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e302;
                        let _e303 = dot(_e302, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e306 = (_e299 - vec3(dot(_e299, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e319 = (vec2<f32>(_e303, (1f - _e303)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e306.x, _e306.y), _e306.z)), max(max(_e306.x, _e306.y), _e306.z))));
                        local_1 = ((_e306 * min(1f, min(_e319.x, _e319.y))) + vec3(_e303));
                    }
                    break;
                }
                default: {
                }
            }
            let _e585 = local_1;
            let _e587 = mix(_e292, _e585, vec3(_e291.w));
            let _e593 = vec4<f32>(_e587.x, _e284.y, _e284.z, _e284.w);
            let _e599 = vec4<f32>(_e593.x, _e587.y, _e593.z, _e593.w);
            phi_4106_ = vec4<f32>(_e599.x, _e599.y, _e587.z, _e599.w);
        }
        let _e607 = phi_4106_;
        let _e610 = (_e607.xyz * _e607.w);
        let _e616 = vec4<f32>(_e610.x, _e607.y, _e607.z, _e607.w);
        let _e622 = vec4<f32>(_e616.x, _e610.y, _e616.z, _e616.w);
        phi_4119_ = vec4<f32>(_e622.x, _e622.y, _e610.z, _e622.w);
    }
    let _e630 = phi_4119_;
    let _e631 = (_e630 * _e197);
    let _e633 = (1f - _e631.w);
    phi_4120_ = _e631;
    if (_e633 != 0f) {
        let _e637 = o0_.x2_[_e111];
        phi_4120_ = (_e631 + (unpack4x8unorm(_e637) * _e633));
    }
    let _e642 = phi_4120_;
    Q1_ = _e642;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = Q1_;
    return _e3;
}
