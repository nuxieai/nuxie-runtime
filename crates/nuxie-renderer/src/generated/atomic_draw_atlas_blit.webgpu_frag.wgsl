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
    var phi_1560_: bool;
    var phi_3678_: f32;
    var phi_3677_: f32;
    var phi_3679_: f32;
    var phi_3682_: f32;
    var phi_3681_: f32;
    var phi_1597_: bool;
    var phi_3684_: f32;
    var phi_4286_: u32;
    var phi_3683_: f32;
    var phi_3704_: vec4<f32>;
    var phi_4285_: u32;
    var phi_3702_: vec4<f32>;
    var phi_3709_: f32;
    var phi_4131_: vec4<f32>;
    var phi_4075_: i32;
    var phi_4270_: vec4<f32>;
    var phi_4283_: vec4<f32>;
    var phi_4308_: vec3<f32>;
    var phi_4310_: vec4<f32>;

    let _e83 = gl_FragCoord_1;
    let _e84 = _e83.xy;
    let _e87 = bitcast<vec2<u32>>(vec2<i32>(floor(_e84)));
    let _e89 = j.P6_;
    let _e118 = bitcast<i32>((((((_e87.y >> bitcast<u32>(5u)) * (((_e89 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e87.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e87.x & 28u) << bitcast<u32>(5u)) + ((_e87.y & 28u) << bitcast<u32>(2i)))) + (((_e87.y & 3u) << bitcast<u32>(2i)) + (_e87.x & 3u))));
    let _e121 = R4_.r2_[_e118];
    let _e123 = (_e121 >> bitcast<u32>(17u));
    let _e124 = G0_1;
    let _e128 = S2_1;
    let _e129 = textureSampleLevel(HD, Pa, _e128, 0f);
    R4_.r2_[_e118] = (((_e124 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e129.x, 0f, 1f) * 2048f)))));
    let _e140 = ((f32((_e121 & 131071u)) * 0.00048828125f) + -32f);
    let _e143 = VC.r2_[_e123];
    phi_3677_ = _e140;
    if ((_e143.x & 768u) != 0u) {
        let _e147 = abs(_e140);
        phi_1560_ = aj;
        if aj {
            phi_1560_ = ((_e143.x & 512u) != 0u);
        }
        let _e151 = phi_1560_;
        phi_3678_ = _e147;
        if _e151 {
            phi_3678_ = (1f - abs(((fract((_e147 * 0.5f)) * 2f) + -1f)));
        }
        let _e159 = phi_3678_;
        phi_3677_ = _e159;
    }
    let _e161 = phi_3677_;
    let _e162 = clamp(_e161, 0f, 1f);
    phi_3681_ = _e162;
    if Wi {
        let _e164 = (_e143.x >> bitcast<u32>(16u));
        phi_3682_ = _e162;
        if (_e164 != 0u) {
            let _e168 = m0_.r2_[_e118];
            if (_e164 == (_e168 >> bitcast<u32>(16i))) {
                phi_3679_ = min(_e162, unpack2x16float(_e168).x);
            } else {
                phi_3679_ = 0f;
            }
            let _e176 = phi_3679_;
            phi_3682_ = _e176;
        }
        let _e178 = phi_3682_;
        phi_3681_ = _e178;
    }
    let _e180 = phi_3681_;
    phi_1597_ = Xi;
    if Xi {
        phi_1597_ = ((_e143.x & 1024u) != 0u);
    }
    let _e184 = phi_1597_;
    phi_3684_ = _e180;
    if _e184 {
        let _e185 = (_e123 * 8u);
        let _e189 = JB.r2_[(_e185 + 2u)];
        let _e200 = JB.r2_[(_e185 + 3u)];
        let _e205 = _e200.zw;
        let _e207 = ((abs(((mat2x2<f32>(vec2<f32>(_e189.x, _e189.y), vec2<f32>(_e189.z, _e189.w)) * _e84) + _e200.xy)) * _e205) - _e205);
        phi_3684_ = min(_e180, clamp((min(_e207.x, _e207.y) + 0.5f), 0f, 1f));
    }
    let _e215 = phi_3684_;
    let _e216 = (_e143.x & 15u);
    let _e219 = ((_e143.x >> bitcast<u32>(4i)) & 15u);
    let _e221 = (Yi && (_e219 != 0u));
    if (_e216 <= 1u) {
        let _e226 = (Wi && (_e216 == 0u));
        phi_4286_ = 0u;
        if _e226 {
            phi_4286_ = (_e143.y | pack2x16float(vec2<f32>(_e215, 0f)));
        }
        let _e231 = phi_4286_;
        phi_4285_ = _e231;
        phi_3702_ = select(unpack4x8unorm(_e143.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e226));
    } else {
        let _e234 = (_e123 * 8u);
        let _e237 = JB.r2_[_e234];
        let _e248 = JB.r2_[(_e234 + 1u)];
        let _e251 = ((mat2x2<f32>(vec2<f32>(_e237.x, _e237.y), vec2<f32>(_e237.z, _e237.w)) * _e84) + _e248.xy);
        let _e257 = j.L8_;
        let _e259 = j.M8_;
        if (f32(_e216) == 2f) {
            phi_3683_ = _e251.x;
        } else {
            phi_3683_ = length(_e251);
        }
        let _e269 = phi_3683_;
        let _e275 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e269, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e248.z < 0f))) + ((max(0f, _e248.z) * 0.001953125f) + 0.0009765625f)), ((_e248.w * _e257) + _e259)), 0f);
        phi_3704_ = _e275;
        if !(_e221) {
            let _e279 = (_e275.xyz * _e275.w);
            phi_3704_ = vec4<f32>(_e279.x, _e279.y, _e279.z, (_e275.w * abs(bitcast<f32>(_e143.y))));
        }
        let _e289 = phi_3704_;
        phi_4285_ = 0u;
        phi_3702_ = _e289;
    }
    let _e291 = phi_4285_;
    let _e293 = phi_3702_;
    phi_4283_ = _e293;
    if _e221 {
        phi_4270_ = _e293;
        if ((_e293.w * _e215) != 0f) {
            let _e299 = n0_.r2_[_e118];
            let _e300 = unpack4x8unorm(_e299);
            let _e301 = _e293.xyz;
            local_2 = _e301;
            let _e302 = _e300.xyz;
            if (_e300.w != 0f) {
                phi_3709_ = (1f / _e300.w);
            } else {
                phi_3709_ = 0f;
            }
            let _e307 = phi_3709_;
            let _e308 = (_e302 * _e307);
            local = _e308;
            switch bitcast<i32>(_e219) {
                case 11: {
                    let _e310 = local_2;
                    local_1 = (_e310 * _e308);
                    break;
                }
                case 1: {
                    let _e312 = local_2;
                    local_1 = ((_e312 + _e308) - (_e312 * _e308));
                    break;
                }
                case 2: {
                    let _e316 = local_2;
                    let _e317 = (_e316 * _e308);
                    local_1 = (select(_e317, (((_e316 + _e308) - _e317) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e308 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e324 = local_2;
                    local_1 = min(_e324, _e308);
                    break;
                }
                case 4: {
                    let _e326 = local_2;
                    local_1 = max(_e326, _e308);
                    break;
                }
                case 5: {
                    let _e329 = clamp(_e302, vec3<f32>(0f, 0f, 0f), _e300.www);
                    let _e335 = vec4<f32>(_e329.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e341 = vec4<f32>(_e335.x, _e329.y, _e335.z, _e335.w);
                    let _e348 = local_2;
                    let _e351 = (clamp((vec3<f32>(1f, 1f, 1f) - _e348), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e300.w);
                    let _e352 = vec4<f32>(_e341.x, _e341.y, _e329.z, _e341.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e352 / _e351)), sign(_e352), (_e351 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e358 = local_2;
                    local_2 = clamp(_e358, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e361 = clamp(_e302, vec3<f32>(0f, 0f, 0f), _e300.www);
                    let _e367 = vec4<f32>(_e361.x, _e300.y, _e300.z, _e300.w);
                    let _e373 = vec4<f32>(_e367.x, _e361.y, _e367.z, _e367.w);
                    phi_4131_ = vec4<f32>(_e373.x, _e373.y, _e361.z, _e373.w);
                    if (_e300.w == 0f) {
                        phi_4131_ = vec4<f32>(_e361.x, _e361.y, _e361.z, 1f);
                    }
                    let _e383 = phi_4131_;
                    let _e387 = (vec3(_e383.w) - _e383.xyz);
                    let _e388 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e387 / (_e388 * _e383.w))), sign(_e387), (_e388 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e396 = local_2;
                    let _e397 = (_e396 * _e308);
                    local_1 = (select(_e397, (((_e396 + _e308) - _e397) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e396 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4075_ = 0i;
                    loop {
                        let _e405 = phi_4075_;
                        if (_e405 < 3i) {
                            let _e408 = local_2[_e405];
                            if (_e408 <= 0.5f) {
                                let _e411 = local[_e405];
                                local_1[_e405] = (1f - _e411);
                            } else {
                                let _e415 = local[_e405];
                                if (_e415 <= 0.25f) {
                                    let _e417 = local[_e405];
                                    let _e420 = local[_e405];
                                    local_1[_e405] = ((((16f * _e417) - 12f) * _e420) + 3f);
                                } else {
                                    let _e424 = local[_e405];
                                    local_1[_e405] = (inverseSqrt(_e424) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4075_ = (_e405 + 1i);
                        }
                    }
                    let _e429 = local_2;
                    let _e433 = local_1;
                    local_1 = (_e308 + ((_e308 * ((_e429 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e433));
                    break;
                }
                case 9: {
                    let _e436 = local_2;
                    local_1 = abs((_e308 - _e436));
                    break;
                }
                case 10: {
                    let _e439 = local_2;
                    local_1 = ((_e439 + _e308) - ((_e439 * 2f) * _e308));
                    break;
                }
                case 12: {
                    if cj {
                        let _e444 = local_2;
                        let _e445 = clamp(_e444, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e445;
                        let _e460 = (_e445 - vec3(min(min(_e445.x, _e445.y), _e445.z)));
                        let _e468 = (_e460 * ((max(max(_e308.x, _e308.y), _e308.z) - min(min(_e308.x, _e308.y), _e308.z)) / max(0.000062f, max(max(_e460.x, _e460.y), _e460.z))));
                        let _e469 = dot(_e308, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e472 = (_e468 - vec3(dot(_e468, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e485 = (vec2<f32>(_e469, (1f - _e469)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e472.x, _e472.y), _e472.z)), max(max(_e472.x, _e472.y), _e472.z))));
                        local_1 = ((_e472 * min(1f, min(_e485.x, _e485.y))) + vec3(_e469));
                    }
                    break;
                }
                case 13: {
                    if cj {
                        let _e493 = local_2;
                        let _e494 = clamp(_e493, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e494;
                        let _e509 = (_e308 - vec3(min(min(_e308.x, _e308.y), _e308.z)));
                        let _e517 = (_e509 * ((max(max(_e494.x, _e494.y), _e494.z) - min(min(_e494.x, _e494.y), _e494.z)) / max(0.000062f, max(max(_e509.x, _e509.y), _e509.z))));
                        let _e518 = dot(_e308, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e521 = (_e517 - vec3(dot(_e517, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e534 = (vec2<f32>(_e518, (1f - _e518)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e521.x, _e521.y), _e521.z)), max(max(_e521.x, _e521.y), _e521.z))));
                        local_1 = ((_e521 * min(1f, min(_e534.x, _e534.y))) + vec3(_e518));
                    }
                    break;
                }
                case 14: {
                    if cj {
                        let _e542 = local_2;
                        let _e543 = clamp(_e542, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e543;
                        let _e544 = dot(_e308, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e547 = (_e543 - vec3(dot(_e543, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e560 = (vec2<f32>(_e544, (1f - _e544)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e547.x, _e547.y), _e547.z)), max(max(_e547.x, _e547.y), _e547.z))));
                        local_1 = ((_e547 * min(1f, min(_e560.x, _e560.y))) + vec3(_e544));
                    }
                    break;
                }
                case 15: {
                    if cj {
                        let _e568 = local_2;
                        let _e569 = clamp(_e568, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e569;
                        let _e570 = dot(_e569, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e573 = (_e308 - vec3(dot(_e308, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e586 = (vec2<f32>(_e570, (1f - _e570)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e573.x, _e573.y), _e573.z)), max(max(_e573.x, _e573.y), _e573.z))));
                        local_1 = ((_e573 * min(1f, min(_e586.x, _e586.y))) + vec3(_e570));
                    }
                    break;
                }
                default: {
                }
            }
            let _e594 = local_1;
            let _e596 = mix(_e301, _e594, vec3(_e300.w));
            let _e602 = vec4<f32>(_e596.x, _e293.y, _e293.z, _e293.w);
            let _e608 = vec4<f32>(_e602.x, _e596.y, _e602.z, _e602.w);
            phi_4270_ = vec4<f32>(_e608.x, _e608.y, _e596.z, _e608.w);
        }
        let _e616 = phi_4270_;
        let _e619 = (_e616.xyz * _e616.w);
        let _e625 = vec4<f32>(_e619.x, _e616.y, _e616.z, _e616.w);
        let _e631 = vec4<f32>(_e625.x, _e619.y, _e625.z, _e625.w);
        phi_4283_ = vec4<f32>(_e631.x, _e631.y, _e619.z, _e631.w);
    }
    let _e639 = phi_4283_;
    let _e640 = (_e639 * _e215);
    let _e641 = _e640.xyz;
    let _e644 = j.F3_;
    let _e646 = j.G3_;
    if (dj && (_e640.w != 0f)) {
        phi_4308_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e83.x) + (0.00583715f * _e83.y))))) * _e644) + _e646)) + _e641);
    } else {
        phi_4308_ = _e641;
    }
    let _e662 = phi_4308_;
    let _e668 = vec4<f32>(_e662.x, _e640.y, _e640.z, _e640.w);
    let _e674 = vec4<f32>(_e668.x, _e662.y, _e668.z, _e668.w);
    let _e680 = vec4<f32>(_e674.x, _e674.y, _e662.z, _e674.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e662.x + _e662.y) + _e662.z) + _e640.w) == 0f) {
                break;
            }
            let _e686 = (1f - _e640.w);
            phi_4310_ = _e680;
            if (_e686 != 0f) {
                let _e690 = n0_.r2_[_e118];
                phi_4310_ = (_e680 + (unpack4x8unorm(_e690) * _e686));
            }
            let _e695 = phi_4310_;
            n0_.r2_[_e118] = pack4x8unorm(_e695);
            break;
        }
    }
    if (_e291 != 0u) {
        m0_.r2_[_e118] = _e291;
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
