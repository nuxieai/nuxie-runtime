struct Gf {
    k2_: array<vec2<u32>>,
}

struct m0he {
    k2_: array<u32>,
}

struct Hf {
    k2_: array<vec4<f32>>,
}

struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct n0he {
    k2_: array<u32>,
}

struct L4he {
    k2_: array<u32>,
}

@id(7) override ti: bool = true;
@id(6) override si: bool = true;
@id(4) override qi: bool = true;
@id(0) override mi: bool = true;
@id(1) override ni: bool = true;
@id(2) override oi: bool = true;

@group(0) @binding(3)
var<storage> WC: Gf;
@group(2) @binding(1)
var<storage, read_write> m0_: m0he;
@group(0) @binding(4)
var<storage> JB: Hf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0he;
@group(2) @binding(3)
var<storage, read_write> L4_: L4he;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1392_: bool;
    var phi_3433_: f32;
    var phi_3432_: f32;
    var phi_3434_: f32;
    var phi_3437_: f32;
    var phi_3436_: f32;
    var phi_1429_: bool;
    var phi_3453_: f32;
    var phi_3438_: f32;
    var phi_3450_: vec4<f32>;
    var phi_3448_: vec4<f32>;
    var phi_3456_: f32;
    var phi_3879_: vec4<f32>;
    var phi_3823_: i32;
    var phi_4018_: vec4<f32>;
    var phi_4031_: vec4<f32>;
    var phi_4032_: vec3<f32>;
    var phi_4034_: vec4<f32>;

    let _e75 = gl_FragCoord_1;
    let _e76 = _e75.xy;
    let _e79 = bitcast<vec2<u32>>(vec2<i32>(floor(_e76)));
    let _e81 = j.A6_;
    let _e110 = bitcast<i32>((((((_e79.y >> bitcast<u32>(5u)) * (((_e81 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e79.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e79.x & 28u) << bitcast<u32>(5u)) + ((_e79.y & 28u) << bitcast<u32>(2i)))) + (((_e79.y & 3u) << bitcast<u32>(2i)) + (_e79.x & 3u))));
    let _e113 = L4_.k2_[_e110];
    let _e117 = ((f32((_e113 & 131071u)) * 0.00048828125f) + -32f);
    let _e119 = (_e113 >> bitcast<u32>(17u));
    let _e122 = WC.k2_[_e119];
    phi_3432_ = _e117;
    if ((_e122.x & 768u) != 0u) {
        let _e126 = abs(_e117);
        phi_1392_ = qi;
        if qi {
            phi_1392_ = ((_e122.x & 512u) != 0u);
        }
        let _e130 = phi_1392_;
        phi_3433_ = _e126;
        if _e130 {
            phi_3433_ = (1f - abs(((fract((_e126 * 0.5f)) * 2f) + -1f)));
        }
        let _e138 = phi_3433_;
        phi_3432_ = _e138;
    }
    let _e140 = phi_3432_;
    let _e141 = clamp(_e140, 0f, 1f);
    phi_3436_ = _e141;
    if mi {
        let _e143 = (_e122.x >> bitcast<u32>(16u));
        phi_3437_ = _e141;
        if (_e143 != 0u) {
            let _e147 = m0_.k2_[_e110];
            if (_e143 == (_e147 >> bitcast<u32>(16i))) {
                phi_3434_ = min(_e141, unpack2x16float(_e147).x);
            } else {
                phi_3434_ = 0f;
            }
            let _e155 = phi_3434_;
            phi_3437_ = _e155;
        }
        let _e157 = phi_3437_;
        phi_3436_ = _e157;
    }
    let _e159 = phi_3436_;
    phi_1429_ = ni;
    if ni {
        phi_1429_ = ((_e122.x & 1024u) != 0u);
    }
    let _e163 = phi_1429_;
    phi_3453_ = _e159;
    if _e163 {
        let _e164 = (_e119 * 8u);
        let _e168 = JB.k2_[(_e164 + 2u)];
        let _e179 = JB.k2_[(_e164 + 3u)];
        let _e184 = _e179.zw;
        let _e186 = ((abs(((mat2x2<f32>(vec2<f32>(_e168.x, _e168.y), vec2<f32>(_e168.z, _e168.w)) * _e76) + _e179.xy)) * _e184) - _e184);
        phi_3453_ = min(_e159, clamp((min(_e186.x, _e186.y) + 0.5f), 0f, 1f));
    }
    let _e194 = phi_3453_;
    let _e195 = (_e122.x & 15u);
    let _e198 = ((_e122.x >> bitcast<u32>(4i)) & 15u);
    let _e200 = (oi && (_e198 != 0u));
    if (_e195 <= 1u) {
        phi_3448_ = select(unpack4x8unorm(_e122.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((mi && (_e195 == 0u))));
    } else {
        let _e208 = (_e119 * 8u);
        let _e211 = JB.k2_[_e208];
        let _e222 = JB.k2_[(_e208 + 1u)];
        let _e225 = ((mat2x2<f32>(vec2<f32>(_e211.x, _e211.y), vec2<f32>(_e211.z, _e211.w)) * _e76) + _e222.xy);
        if (_e195 == 2u) {
            phi_3438_ = _e225.x;
        } else {
            phi_3438_ = length(_e225);
        }
        let _e230 = phi_3438_;
        let _e237 = bitcast<f32>(_e122.y);
        let _e240 = j.xc;
        let _e243 = j.yc;
        let _e246 = textureSampleLevel(ED, ia, vec2<f32>(((clamp(_e230, 0f, 1f) * _e222.z) + _e222.w), ((floor(_e237) * _e240) + _e243)), 0f);
        phi_3450_ = _e246;
        if !(_e200) {
            let _e250 = (_e246.xyz * _e246.w);
            phi_3450_ = vec4<f32>(_e250.x, _e250.y, _e250.z, (_e246.w * (fract(_e237) * 1.0039216f)));
        }
        let _e259 = phi_3450_;
        phi_3448_ = _e259;
    }
    let _e261 = phi_3448_;
    phi_4031_ = _e261;
    if _e200 {
        phi_4018_ = _e261;
        if ((_e261.w * _e194) != 0f) {
            let _e267 = n0_.k2_[_e110];
            let _e268 = unpack4x8unorm(_e267);
            let _e269 = _e261.xyz;
            local_2 = _e269;
            let _e270 = _e268.xyz;
            if (_e268.w != 0f) {
                phi_3456_ = (1f / _e268.w);
            } else {
                phi_3456_ = 0f;
            }
            let _e275 = phi_3456_;
            let _e276 = (_e270 * _e275);
            local = _e276;
            switch bitcast<i32>(_e198) {
                case 11: {
                    let _e278 = local_2;
                    local_1 = (_e278 * _e276);
                    break;
                }
                case 1: {
                    let _e280 = local_2;
                    local_1 = ((_e280 + _e276) - (_e280 * _e276));
                    break;
                }
                case 2: {
                    let _e284 = local_2;
                    let _e285 = (_e284 * _e276);
                    local_1 = (select(_e285, (((_e284 + _e276) - _e285) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e276 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e292 = local_2;
                    local_1 = min(_e292, _e276);
                    break;
                }
                case 4: {
                    let _e294 = local_2;
                    local_1 = max(_e294, _e276);
                    break;
                }
                case 5: {
                    let _e297 = clamp(_e270, vec3<f32>(0f, 0f, 0f), _e268.www);
                    let _e303 = vec4<f32>(_e297.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e309 = vec4<f32>(_e303.x, _e297.y, _e303.z, _e303.w);
                    let _e316 = local_2;
                    let _e319 = (clamp((vec3<f32>(1f, 1f, 1f) - _e316), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e268.w);
                    let _e320 = vec4<f32>(_e309.x, _e309.y, _e297.z, _e309.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e320 / _e319)), sign(_e320), (_e319 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e326 = local_2;
                    local_2 = clamp(_e326, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e329 = clamp(_e270, vec3<f32>(0f, 0f, 0f), _e268.www);
                    let _e335 = vec4<f32>(_e329.x, _e268.y, _e268.z, _e268.w);
                    let _e341 = vec4<f32>(_e335.x, _e329.y, _e335.z, _e335.w);
                    phi_3879_ = vec4<f32>(_e341.x, _e341.y, _e329.z, _e341.w);
                    if (_e268.w == 0f) {
                        phi_3879_ = vec4<f32>(_e329.x, _e329.y, _e329.z, 1f);
                    }
                    let _e351 = phi_3879_;
                    let _e355 = (vec3(_e351.w) - _e351.xyz);
                    let _e356 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e355 / (_e356 * _e351.w))), sign(_e355), (_e356 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e364 = local_2;
                    let _e365 = (_e364 * _e276);
                    local_1 = (select(_e365, (((_e364 + _e276) - _e365) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e364 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3823_ = 0i;
                    loop {
                        let _e373 = phi_3823_;
                        if (_e373 < 3i) {
                            let _e376 = local_2[_e373];
                            if (_e376 <= 0.5f) {
                                let _e379 = local[_e373];
                                local_1[_e373] = (1f - _e379);
                            } else {
                                let _e383 = local[_e373];
                                if (_e383 <= 0.25f) {
                                    let _e385 = local[_e373];
                                    let _e388 = local[_e373];
                                    local_1[_e373] = ((((16f * _e385) - 12f) * _e388) + 3f);
                                } else {
                                    let _e392 = local[_e373];
                                    local_1[_e373] = (inverseSqrt(_e392) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3823_ = (_e373 + 1i);
                        }
                    }
                    let _e397 = local_2;
                    let _e401 = local_1;
                    local_1 = (_e276 + ((_e276 * ((_e397 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e401));
                    break;
                }
                case 9: {
                    let _e404 = local_2;
                    local_1 = abs((_e276 - _e404));
                    break;
                }
                case 10: {
                    let _e407 = local_2;
                    local_1 = ((_e407 + _e276) - ((_e407 * 2f) * _e276));
                    break;
                }
                case 12: {
                    if si {
                        let _e412 = local_2;
                        let _e413 = clamp(_e412, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e413;
                        let _e428 = (_e413 - vec3(min(min(_e413.x, _e413.y), _e413.z)));
                        let _e436 = (_e428 * ((max(max(_e276.x, _e276.y), _e276.z) - min(min(_e276.x, _e276.y), _e276.z)) / max(0.000062f, max(max(_e428.x, _e428.y), _e428.z))));
                        let _e437 = dot(_e276, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e440 = (_e436 - vec3(dot(_e436, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e453 = (vec2<f32>(_e437, (1f - _e437)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e440.x, _e440.y), _e440.z)), max(max(_e440.x, _e440.y), _e440.z))));
                        local_1 = ((_e440 * min(1f, min(_e453.x, _e453.y))) + vec3(_e437));
                    }
                    break;
                }
                case 13: {
                    if si {
                        let _e461 = local_2;
                        let _e462 = clamp(_e461, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e462;
                        let _e477 = (_e276 - vec3(min(min(_e276.x, _e276.y), _e276.z)));
                        let _e485 = (_e477 * ((max(max(_e462.x, _e462.y), _e462.z) - min(min(_e462.x, _e462.y), _e462.z)) / max(0.000062f, max(max(_e477.x, _e477.y), _e477.z))));
                        let _e486 = dot(_e276, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e489 = (_e485 - vec3(dot(_e485, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e502 = (vec2<f32>(_e486, (1f - _e486)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e489.x, _e489.y), _e489.z)), max(max(_e489.x, _e489.y), _e489.z))));
                        local_1 = ((_e489 * min(1f, min(_e502.x, _e502.y))) + vec3(_e486));
                    }
                    break;
                }
                case 14: {
                    if si {
                        let _e510 = local_2;
                        let _e511 = clamp(_e510, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e511;
                        let _e512 = dot(_e276, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e515 = (_e511 - vec3(dot(_e511, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e528 = (vec2<f32>(_e512, (1f - _e512)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e515.x, _e515.y), _e515.z)), max(max(_e515.x, _e515.y), _e515.z))));
                        local_1 = ((_e515 * min(1f, min(_e528.x, _e528.y))) + vec3(_e512));
                    }
                    break;
                }
                case 15: {
                    if si {
                        let _e536 = local_2;
                        let _e537 = clamp(_e536, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e537;
                        let _e538 = dot(_e537, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e541 = (_e276 - vec3(dot(_e276, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e554 = (vec2<f32>(_e538, (1f - _e538)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e541.x, _e541.y), _e541.z)), max(max(_e541.x, _e541.y), _e541.z))));
                        local_1 = ((_e541 * min(1f, min(_e554.x, _e554.y))) + vec3(_e538));
                    }
                    break;
                }
                default: {
                }
            }
            let _e562 = local_1;
            let _e564 = mix(_e269, _e562, vec3(_e268.w));
            let _e570 = vec4<f32>(_e564.x, _e261.y, _e261.z, _e261.w);
            let _e576 = vec4<f32>(_e570.x, _e564.y, _e570.z, _e570.w);
            phi_4018_ = vec4<f32>(_e576.x, _e576.y, _e564.z, _e576.w);
        }
        let _e584 = phi_4018_;
        let _e587 = (_e584.xyz * _e584.w);
        let _e593 = vec4<f32>(_e587.x, _e584.y, _e584.z, _e584.w);
        let _e599 = vec4<f32>(_e593.x, _e587.y, _e593.z, _e593.w);
        phi_4031_ = vec4<f32>(_e599.x, _e599.y, _e587.z, _e599.w);
    }
    let _e607 = phi_4031_;
    let _e608 = (_e607 * _e194);
    let _e609 = _e608.xyz;
    let _e612 = j.M3_;
    let _e614 = j.N3_;
    if (ti && (_e608.w != 0f)) {
        phi_4032_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e75.x) + (0.00583715f * _e75.y))))) * _e612) + _e614)) + _e609);
    } else {
        phi_4032_ = _e609;
    }
    let _e630 = phi_4032_;
    let _e636 = vec4<f32>(_e630.x, _e608.y, _e608.z, _e608.w);
    let _e642 = vec4<f32>(_e636.x, _e630.y, _e636.z, _e636.w);
    let _e648 = vec4<f32>(_e642.x, _e642.y, _e630.z, _e642.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e630.x + _e630.y) + _e630.z) + _e608.w) == 0f) {
                break;
            }
            let _e654 = (1f - _e608.w);
            phi_4034_ = _e648;
            if (_e654 != 0f) {
                let _e658 = n0_.k2_[_e110];
                phi_4034_ = (_e648 + (unpack4x8unorm(_e658) * _e654));
            }
            let _e663 = phi_4034_;
            n0_.k2_[_e110] = pack4x8unorm(_e663);
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
