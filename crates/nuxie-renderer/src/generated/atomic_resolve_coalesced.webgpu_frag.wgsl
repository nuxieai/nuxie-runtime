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
var<private> L1_: vec4<f32>;
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
    var phi_1345_: bool;
    var phi_3335_: f32;
    var phi_3334_: f32;
    var phi_3336_: f32;
    var phi_3339_: f32;
    var phi_3338_: f32;
    var phi_1382_: bool;
    var phi_3355_: f32;
    var phi_3340_: f32;
    var phi_3352_: vec4<f32>;
    var phi_3350_: vec4<f32>;
    var phi_3358_: f32;
    var phi_3781_: vec4<f32>;
    var phi_3725_: i32;
    var phi_3920_: vec4<f32>;
    var phi_3933_: vec4<f32>;
    var phi_3934_: vec4<f32>;

    let _e72 = gl_FragCoord_1;
    let _e73 = _e72.xy;
    let _e76 = bitcast<vec2<u32>>(vec2<i32>(floor(_e73)));
    let _e78 = j.P6_;
    let _e107 = bitcast<i32>((((((_e76.y >> bitcast<u32>(5u)) * (((_e78 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e76.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e76.x & 28u) << bitcast<u32>(5u)) + ((_e76.y & 28u) << bitcast<u32>(2i)))) + (((_e76.y & 3u) << bitcast<u32>(2i)) + (_e76.x & 3u))));
    let _e110 = R4_.r2_[_e107];
    let _e114 = ((f32((_e110 & 131071u)) * 0.00048828125f) + -32f);
    let _e116 = (_e110 >> bitcast<u32>(17u));
    let _e119 = VC.r2_[_e116];
    phi_3334_ = _e114;
    if ((_e119.x & 768u) != 0u) {
        let _e123 = abs(_e114);
        phi_1345_ = aj;
        if aj {
            phi_1345_ = ((_e119.x & 512u) != 0u);
        }
        let _e127 = phi_1345_;
        phi_3335_ = _e123;
        if _e127 {
            phi_3335_ = (1f - abs(((fract((_e123 * 0.5f)) * 2f) + -1f)));
        }
        let _e135 = phi_3335_;
        phi_3334_ = _e135;
    }
    let _e137 = phi_3334_;
    let _e138 = clamp(_e137, 0f, 1f);
    phi_3338_ = _e138;
    if Wi {
        let _e140 = (_e119.x >> bitcast<u32>(16u));
        phi_3339_ = _e138;
        if (_e140 != 0u) {
            let _e144 = m0_.r2_[_e107];
            if (_e140 == (_e144 >> bitcast<u32>(16i))) {
                phi_3336_ = min(_e138, unpack2x16float(_e144).x);
            } else {
                phi_3336_ = 0f;
            }
            let _e152 = phi_3336_;
            phi_3339_ = _e152;
        }
        let _e154 = phi_3339_;
        phi_3338_ = _e154;
    }
    let _e156 = phi_3338_;
    phi_1382_ = Xi;
    if Xi {
        phi_1382_ = ((_e119.x & 1024u) != 0u);
    }
    let _e160 = phi_1382_;
    phi_3355_ = _e156;
    if _e160 {
        let _e161 = (_e116 * 8u);
        let _e165 = JB.r2_[(_e161 + 2u)];
        let _e176 = JB.r2_[(_e161 + 3u)];
        let _e181 = _e176.zw;
        let _e183 = ((abs(((mat2x2<f32>(vec2<f32>(_e165.x, _e165.y), vec2<f32>(_e165.z, _e165.w)) * _e73) + _e176.xy)) * _e181) - _e181);
        phi_3355_ = min(_e156, clamp((min(_e183.x, _e183.y) + 0.5f), 0f, 1f));
    }
    let _e191 = phi_3355_;
    let _e192 = (_e119.x & 15u);
    let _e195 = ((_e119.x >> bitcast<u32>(4i)) & 15u);
    let _e197 = (Yi && (_e195 != 0u));
    if (_e192 <= 1u) {
        phi_3350_ = select(unpack4x8unorm(_e119.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Wi && (_e192 == 0u))));
    } else {
        let _e205 = (_e116 * 8u);
        let _e208 = JB.r2_[_e205];
        let _e219 = JB.r2_[(_e205 + 1u)];
        let _e222 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e73) + _e219.xy);
        let _e228 = j.L8_;
        let _e230 = j.M8_;
        if (f32(_e192) == 2f) {
            phi_3340_ = _e222.x;
        } else {
            phi_3340_ = length(_e222);
        }
        let _e240 = phi_3340_;
        let _e246 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e240, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e219.z < 0f))) + ((max(0f, _e219.z) * 0.001953125f) + 0.0009765625f)), ((_e219.w * _e228) + _e230)), 0f);
        phi_3352_ = _e246;
        if !(_e197) {
            let _e250 = (_e246.xyz * _e246.w);
            phi_3352_ = vec4<f32>(_e250.x, _e250.y, _e250.z, (_e246.w * abs(bitcast<f32>(_e119.y))));
        }
        let _e260 = phi_3352_;
        phi_3350_ = _e260;
    }
    let _e262 = phi_3350_;
    phi_3933_ = _e262;
    if _e197 {
        phi_3920_ = _e262;
        if ((_e262.w * _e191) != 0f) {
            let _e268 = n0_.r2_[_e107];
            let _e269 = unpack4x8unorm(_e268);
            let _e270 = _e262.xyz;
            local_2 = _e270;
            let _e271 = _e269.xyz;
            if (_e269.w != 0f) {
                phi_3358_ = (1f / _e269.w);
            } else {
                phi_3358_ = 0f;
            }
            let _e276 = phi_3358_;
            let _e277 = (_e271 * _e276);
            local = _e277;
            switch bitcast<i32>(_e195) {
                case 11: {
                    let _e279 = local_2;
                    local_1 = (_e279 * _e277);
                    break;
                }
                case 1: {
                    let _e281 = local_2;
                    local_1 = ((_e281 + _e277) - (_e281 * _e277));
                    break;
                }
                case 2: {
                    let _e285 = local_2;
                    let _e286 = (_e285 * _e277);
                    local_1 = (select(_e286, (((_e285 + _e277) - _e286) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e277 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e293 = local_2;
                    local_1 = min(_e293, _e277);
                    break;
                }
                case 4: {
                    let _e295 = local_2;
                    local_1 = max(_e295, _e277);
                    break;
                }
                case 5: {
                    let _e298 = clamp(_e271, vec3<f32>(0f, 0f, 0f), _e269.www);
                    let _e304 = vec4<f32>(_e298.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e310 = vec4<f32>(_e304.x, _e298.y, _e304.z, _e304.w);
                    let _e317 = local_2;
                    let _e320 = (clamp((vec3<f32>(1f, 1f, 1f) - _e317), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e269.w);
                    let _e321 = vec4<f32>(_e310.x, _e310.y, _e298.z, _e310.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e321 / _e320)), sign(_e321), (_e320 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e327 = local_2;
                    local_2 = clamp(_e327, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e330 = clamp(_e271, vec3<f32>(0f, 0f, 0f), _e269.www);
                    let _e336 = vec4<f32>(_e330.x, _e269.y, _e269.z, _e269.w);
                    let _e342 = vec4<f32>(_e336.x, _e330.y, _e336.z, _e336.w);
                    phi_3781_ = vec4<f32>(_e342.x, _e342.y, _e330.z, _e342.w);
                    if (_e269.w == 0f) {
                        phi_3781_ = vec4<f32>(_e330.x, _e330.y, _e330.z, 1f);
                    }
                    let _e352 = phi_3781_;
                    let _e356 = (vec3(_e352.w) - _e352.xyz);
                    let _e357 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e356 / (_e357 * _e352.w))), sign(_e356), (_e357 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e365 = local_2;
                    let _e366 = (_e365 * _e277);
                    local_1 = (select(_e366, (((_e365 + _e277) - _e366) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e365 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3725_ = 0i;
                    loop {
                        let _e374 = phi_3725_;
                        if (_e374 < 3i) {
                            let _e377 = local_2[_e374];
                            if (_e377 <= 0.5f) {
                                let _e380 = local[_e374];
                                local_1[_e374] = (1f - _e380);
                            } else {
                                let _e384 = local[_e374];
                                if (_e384 <= 0.25f) {
                                    let _e386 = local[_e374];
                                    let _e389 = local[_e374];
                                    local_1[_e374] = ((((16f * _e386) - 12f) * _e389) + 3f);
                                } else {
                                    let _e393 = local[_e374];
                                    local_1[_e374] = (inverseSqrt(_e393) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3725_ = (_e374 + 1i);
                        }
                    }
                    let _e398 = local_2;
                    let _e402 = local_1;
                    local_1 = (_e277 + ((_e277 * ((_e398 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e402));
                    break;
                }
                case 9: {
                    let _e405 = local_2;
                    local_1 = abs((_e277 - _e405));
                    break;
                }
                case 10: {
                    let _e408 = local_2;
                    local_1 = ((_e408 + _e277) - ((_e408 * 2f) * _e277));
                    break;
                }
                case 12: {
                    if cj {
                        let _e413 = local_2;
                        let _e414 = clamp(_e413, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e414;
                        let _e429 = (_e414 - vec3(min(min(_e414.x, _e414.y), _e414.z)));
                        let _e437 = (_e429 * ((max(max(_e277.x, _e277.y), _e277.z) - min(min(_e277.x, _e277.y), _e277.z)) / max(0.000062f, max(max(_e429.x, _e429.y), _e429.z))));
                        let _e438 = dot(_e277, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e441 = (_e437 - vec3(dot(_e437, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e454 = (vec2<f32>(_e438, (1f - _e438)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e441.x, _e441.y), _e441.z)), max(max(_e441.x, _e441.y), _e441.z))));
                        local_1 = ((_e441 * min(1f, min(_e454.x, _e454.y))) + vec3(_e438));
                    }
                    break;
                }
                case 13: {
                    if cj {
                        let _e462 = local_2;
                        let _e463 = clamp(_e462, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e463;
                        let _e478 = (_e277 - vec3(min(min(_e277.x, _e277.y), _e277.z)));
                        let _e486 = (_e478 * ((max(max(_e463.x, _e463.y), _e463.z) - min(min(_e463.x, _e463.y), _e463.z)) / max(0.000062f, max(max(_e478.x, _e478.y), _e478.z))));
                        let _e487 = dot(_e277, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e490 = (_e486 - vec3(dot(_e486, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e503 = (vec2<f32>(_e487, (1f - _e487)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e490.x, _e490.y), _e490.z)), max(max(_e490.x, _e490.y), _e490.z))));
                        local_1 = ((_e490 * min(1f, min(_e503.x, _e503.y))) + vec3(_e487));
                    }
                    break;
                }
                case 14: {
                    if cj {
                        let _e511 = local_2;
                        let _e512 = clamp(_e511, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e512;
                        let _e513 = dot(_e277, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e516 = (_e512 - vec3(dot(_e512, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e529 = (vec2<f32>(_e513, (1f - _e513)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e516.x, _e516.y), _e516.z)), max(max(_e516.x, _e516.y), _e516.z))));
                        local_1 = ((_e516 * min(1f, min(_e529.x, _e529.y))) + vec3(_e513));
                    }
                    break;
                }
                case 15: {
                    if cj {
                        let _e537 = local_2;
                        let _e538 = clamp(_e537, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e538;
                        let _e539 = dot(_e538, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e542 = (_e277 - vec3(dot(_e277, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e555 = (vec2<f32>(_e539, (1f - _e539)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e542.x, _e542.y), _e542.z)), max(max(_e542.x, _e542.y), _e542.z))));
                        local_1 = ((_e542 * min(1f, min(_e555.x, _e555.y))) + vec3(_e539));
                    }
                    break;
                }
                default: {
                }
            }
            let _e563 = local_1;
            let _e565 = mix(_e270, _e563, vec3(_e269.w));
            let _e571 = vec4<f32>(_e565.x, _e262.y, _e262.z, _e262.w);
            let _e577 = vec4<f32>(_e571.x, _e565.y, _e571.z, _e571.w);
            phi_3920_ = vec4<f32>(_e577.x, _e577.y, _e565.z, _e577.w);
        }
        let _e585 = phi_3920_;
        let _e588 = (_e585.xyz * _e585.w);
        let _e594 = vec4<f32>(_e588.x, _e585.y, _e585.z, _e585.w);
        let _e600 = vec4<f32>(_e594.x, _e588.y, _e594.z, _e594.w);
        phi_3933_ = vec4<f32>(_e600.x, _e600.y, _e588.z, _e600.w);
    }
    let _e608 = phi_3933_;
    let _e609 = (_e608 * _e191);
    let _e611 = (1f - _e609.w);
    phi_3934_ = _e609;
    if (_e611 != 0f) {
        let _e615 = n0_.r2_[_e107];
        phi_3934_ = (_e609 + (unpack4x8unorm(_e615) * _e611));
    }
    let _e620 = phi_3934_;
    L1_ = _e620;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = L1_;
    return _e3;
}
