struct pf {
    g2_: array<vec2<u32>>,
}

struct i0Xd {
    g2_: array<u32>,
}

struct qf {
    g2_: array<vec4<f32>>,
}

struct SB {
    xc: f32,
    Hd: f32,
    Mf: f32,
    Nf: f32,
    r6_: u32,
    Rb: u32,
    yf: u32,
    zf: u32,
    X7_: vec4<i32>,
    jh: vec2<f32>,
    Id: vec2<f32>,
    f2_: u32,
    nh: f32,
    g6_: u32,
    W2_: f32,
    Jd: f32,
    sf: u32,
    F3_: f32,
    G3_: f32,
    Kd: f32,
    gh: u32,
    Qb: u32,
    dc: f32,
    ec: f32,
}

struct m0Xd {
    g2_: array<u32>,
}

struct A4Xd {
    g2_: array<u32>,
}

@id(7) override Qh: bool = true;
@id(6) override Ph: bool = true;
@id(4) override Nh: bool = true;
@id(0) override Jh: bool = true;
@id(1) override Kh: bool = true;
@id(2) override Lh: bool = true;

@group(0) @binding(3)
var<storage> CD: pf;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Xd;
@group(0) @binding(4)
var<storage> PB: qf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: SB;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(2) @binding(0)
var<storage, read_write> m0_: m0Xd;
@group(2) @binding(3)
var<storage, read_write> A4_: A4Xd;
@group(3) @binding(9)
var ha: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Z5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1393_: bool;
    var phi_3434_: f32;
    var phi_3433_: f32;
    var phi_3435_: f32;
    var phi_3438_: f32;
    var phi_3437_: f32;
    var phi_1430_: bool;
    var phi_3454_: f32;
    var phi_3439_: f32;
    var phi_3451_: vec4<f32>;
    var phi_3449_: vec4<f32>;
    var phi_3457_: f32;
    var phi_3880_: vec4<f32>;
    var phi_3824_: i32;
    var phi_4019_: vec4<f32>;
    var phi_4032_: vec4<f32>;
    var phi_4033_: vec3<f32>;
    var phi_4035_: vec4<f32>;

    let _e75 = gl_FragCoord_1;
    let _e76 = _e75.xy;
    let _e79 = bitcast<vec2<u32>>(vec2<i32>(floor(_e76)));
    let _e81 = j.r6_;
    let _e110 = bitcast<i32>((((((_e79.y >> bitcast<u32>(5u)) * (((_e81 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e79.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e79.x & 28u) << bitcast<u32>(5u)) + ((_e79.y & 28u) << bitcast<u32>(2i)))) + (((_e79.y & 3u) << bitcast<u32>(2i)) + (_e79.x & 3u))));
    let _e113 = A4_.g2_[_e110];
    let _e117 = ((f32((_e113 & 131071u)) * 0.00048828125f) + -32f);
    let _e119 = (_e113 >> bitcast<u32>(17u));
    let _e122 = CD.g2_[_e119];
    phi_3433_ = _e117;
    if ((_e122.x & 768u) != 0u) {
        let _e126 = abs(_e117);
        phi_1393_ = Nh;
        if Nh {
            phi_1393_ = ((_e122.x & 512u) != 0u);
        }
        let _e130 = phi_1393_;
        phi_3434_ = _e126;
        if _e130 {
            phi_3434_ = (1f - abs(((fract((_e126 * 0.5f)) * 2f) + -1f)));
        }
        let _e138 = phi_3434_;
        phi_3433_ = _e138;
    }
    let _e140 = phi_3433_;
    let _e141 = clamp(_e140, 0f, 1f);
    phi_3437_ = _e141;
    if Jh {
        let _e143 = (_e122.x >> bitcast<u32>(16u));
        phi_3438_ = _e141;
        if (_e143 != 0u) {
            let _e147 = i0_.g2_[_e110];
            if (_e143 == (_e147 >> bitcast<u32>(16i))) {
                phi_3435_ = min(_e141, unpack2x16float(_e147).x);
            } else {
                phi_3435_ = 0f;
            }
            let _e155 = phi_3435_;
            phi_3438_ = _e155;
        }
        let _e157 = phi_3438_;
        phi_3437_ = _e157;
    }
    let _e159 = phi_3437_;
    phi_1430_ = Kh;
    if Kh {
        phi_1430_ = ((_e122.x & 1024u) != 0u);
    }
    let _e163 = phi_1430_;
    phi_3454_ = _e159;
    if _e163 {
        let _e164 = (_e119 * 8u);
        let _e168 = PB.g2_[(_e164 + 2u)];
        let _e179 = PB.g2_[(_e164 + 3u)];
        let _e184 = _e179.zw;
        let _e186 = ((abs(((mat2x2<f32>(vec2<f32>(_e168.x, _e168.y), vec2<f32>(_e168.z, _e168.w)) * _e76) + _e179.xy)) * _e184) - _e184);
        phi_3454_ = min(_e159, clamp((min(_e186.x, _e186.y) + 0.5f), 0f, 1f));
    }
    let _e194 = phi_3454_;
    let _e195 = (_e122.x & 15u);
    let _e198 = ((_e122.x >> bitcast<u32>(4i)) & 15u);
    let _e200 = (Lh && (_e198 != 0u));
    if (_e195 <= 1u) {
        phi_3449_ = select(unpack4x8unorm(_e122.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Jh && (_e195 == 0u))));
    } else {
        let _e208 = (_e119 * 8u);
        let _e211 = PB.g2_[_e208];
        let _e222 = PB.g2_[(_e208 + 1u)];
        let _e225 = ((mat2x2<f32>(vec2<f32>(_e211.x, _e211.y), vec2<f32>(_e211.z, _e211.w)) * _e76) + _e222.xy);
        if (_e195 == 2u) {
            phi_3439_ = _e225.x;
        } else {
            phi_3439_ = length(_e225);
        }
        let _e230 = phi_3439_;
        let _e237 = bitcast<f32>(_e122.y);
        let _e240 = j.dc;
        let _e243 = j.ec;
        let _e246 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e230, 0f, 1f) * _e222.z) + _e222.w), ((floor(_e237) * _e240) + _e243)), 0f);
        phi_3451_ = _e246;
        if !(_e200) {
            let _e250 = (_e246.xyz * _e246.w);
            phi_3451_ = vec4<f32>(_e250.x, _e250.y, _e250.z, (_e246.w * (fract(_e237) * 1.0039216f)));
        }
        let _e259 = phi_3451_;
        phi_3449_ = _e259;
    }
    let _e261 = phi_3449_;
    phi_4032_ = _e261;
    if _e200 {
        phi_4019_ = _e261;
        if ((_e261.w * _e194) != 0f) {
            let _e267 = m0_.g2_[_e110];
            let _e268 = unpack4x8unorm(_e267);
            let _e269 = _e261.xyz;
            local_2 = _e269;
            let _e270 = _e268.xyz;
            if (_e268.w != 0f) {
                phi_3457_ = (1f / _e268.w);
            } else {
                phi_3457_ = 0f;
            }
            let _e275 = phi_3457_;
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
                    phi_3880_ = vec4<f32>(_e341.x, _e341.y, _e329.z, _e341.w);
                    if (_e268.w == 0f) {
                        phi_3880_ = vec4<f32>(_e329.x, _e329.y, _e329.z, 1f);
                    }
                    let _e351 = phi_3880_;
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
                    phi_3824_ = 0i;
                    loop {
                        let _e373 = phi_3824_;
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
                            phi_3824_ = (_e373 + 1i);
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
                    if Ph {
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
                    if Ph {
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
                    if Ph {
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
                    if Ph {
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
            phi_4019_ = vec4<f32>(_e576.x, _e576.y, _e564.z, _e576.w);
        }
        let _e584 = phi_4019_;
        let _e587 = (_e584.xyz * _e584.w);
        let _e593 = vec4<f32>(_e587.x, _e584.y, _e584.z, _e584.w);
        let _e599 = vec4<f32>(_e593.x, _e587.y, _e593.z, _e593.w);
        phi_4032_ = vec4<f32>(_e599.x, _e599.y, _e587.z, _e599.w);
    }
    let _e607 = phi_4032_;
    let _e608 = (_e607 * _e194);
    let _e609 = _e608.xyz;
    let _e612 = j.F3_;
    let _e614 = j.G3_;
    if (Qh && (_e608.w != 0f)) {
        phi_4033_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e75.x) + (0.00583715f * _e75.y))))) * _e612) + _e614)) + _e609);
    } else {
        phi_4033_ = _e609;
    }
    let _e630 = phi_4033_;
    let _e636 = vec4<f32>(_e630.x, _e608.y, _e608.z, _e608.w);
    let _e642 = vec4<f32>(_e636.x, _e630.y, _e636.z, _e636.w);
    let _e648 = vec4<f32>(_e642.x, _e642.y, _e630.z, _e642.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e630.x + _e630.y) + _e630.z) + _e608.w) == 0f) {
                break;
            }
            let _e654 = (1f - _e608.w);
            phi_4035_ = _e648;
            if (_e654 != 0f) {
                let _e658 = m0_.g2_[_e110];
                phi_4035_ = (_e648 + (unpack4x8unorm(_e658) * _e654));
            }
            let _e663 = phi_4035_;
            m0_.g2_[_e110] = pack4x8unorm(_e663);
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
