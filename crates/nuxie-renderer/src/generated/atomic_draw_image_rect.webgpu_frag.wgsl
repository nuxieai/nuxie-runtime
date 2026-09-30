struct kf {
    g2_: array<vec2<u32>>,
}

struct i0Sd {
    g2_: array<u32>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct m0Sd {
    g2_: array<u32>,
}

struct A4Sd {
    g2_: array<u32>,
}

@id(7) override Oh: bool = true;
@id(6) override Nh: bool = true;
@id(4) override Lh: bool = true;
@id(0) override Hh: bool = true;
@id(1) override Ih: bool = true;
@id(2) override Jh: bool = true;

@group(0) @binding(3)
var<storage> CD: kf;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Sd;
@group(0) @binding(4)
var<storage> PB: lf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: TB;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var M9_: sampler;
@group(2) @binding(0)
var<storage, read_write> m0_: m0Sd;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
var<private> c2_1: vec2<f32>;
var<private> Z4_1: f32;
var<private> O0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> A4_: A4Sd;
var<private> B3_1: u32;
var<private> a5_1: vec4<f32>;
var<private> K1_1: vec4<f32>;
var<private> D1_1: u32;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6120_: f32;
    var phi_1667_: bool;
    var phi_5435_: f32;
    var phi_5434_: f32;
    var phi_5436_: f32;
    var phi_5439_: f32;
    var phi_5438_: f32;
    var phi_1704_: bool;
    var phi_5441_: f32;
    var phi_6087_: u32;
    var phi_5440_: f32;
    var phi_5464_: vec4<f32>;
    var phi_6086_: u32;
    var phi_5462_: vec4<f32>;
    var phi_5469_: f32;
    var phi_5923_: vec4<f32>;
    var phi_5863_: i32;
    var phi_6071_: vec4<f32>;
    var phi_6084_: vec4<f32>;
    var phi_1374_: bool;
    var phi_6110_: u32;
    var phi_6138_: f32;
    var phi_7649_: f32;
    var phi_6139_: f32;
    var phi_6140_: f32;
    var phi_6172_: vec4<f32>;
    var phi_1436_: bool;
    var phi_6182_: f32;
    var phi_6183_: f32;
    var phi_7308_: vec4<f32>;
    var phi_7164_: i32;
    var phi_7663_: vec4<f32>;
    var phi_7676_: vec3<f32>;
    var phi_7678_: vec4<f32>;

    let _e89 = gl_FragCoord_1;
    let _e90 = _e89.xy;
    let _e93 = bitcast<vec2<u32>>(vec2<i32>(floor(_e90)));
    let _e95 = j.n6_;
    let _e124 = bitcast<i32>((((((_e93.y >> bitcast<u32>(5u)) * (((_e95 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e93.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e93.x & 28u) << bitcast<u32>(5u)) + ((_e93.y & 28u) << bitcast<u32>(2i)))) + (((_e93.y & 3u) << bitcast<u32>(2i)) + (_e93.x & 3u))));
    let _e125 = c2_1;
    let _e126 = textureSample(GC, W5_, _e125);
    let _e127 = Z4_1;
    let _e128 = min(_e127, 1f);
    phi_6120_ = _e128;
    if Ih {
        let _e129 = O0_1;
        let _e132 = min(_e129.xy, _e129.zw);
        phi_6120_ = clamp(min(_e132.x, _e132.y), 0f, _e128);
    }
    let _e138 = phi_6120_;
    let _e141 = A4_.g2_[_e124];
    let _e143 = (_e141 >> bitcast<u32>(17u));
    let _e147 = ((f32((_e141 & 131071u)) * 0.00048828125f) + -32f);
    let _e150 = CD.g2_[_e143];
    phi_5434_ = _e147;
    if ((_e150.x & 768u) != 0u) {
        let _e154 = abs(_e147);
        phi_1667_ = Lh;
        if Lh {
            phi_1667_ = ((_e150.x & 512u) != 0u);
        }
        let _e158 = phi_1667_;
        phi_5435_ = _e154;
        if _e158 {
            phi_5435_ = (1f - abs(((fract((_e154 * 0.5f)) * 2f) + -1f)));
        }
        let _e166 = phi_5435_;
        phi_5434_ = _e166;
    }
    let _e168 = phi_5434_;
    let _e169 = clamp(_e168, 0f, 1f);
    phi_5438_ = _e169;
    if Hh {
        let _e171 = (_e150.x >> bitcast<u32>(16u));
        phi_5439_ = _e169;
        if (_e171 != 0u) {
            let _e175 = i0_.g2_[_e124];
            if (_e171 == (_e175 >> bitcast<u32>(16i))) {
                phi_5436_ = min(_e169, unpack2x16float(_e175).x);
            } else {
                phi_5436_ = 0f;
            }
            let _e183 = phi_5436_;
            phi_5439_ = _e183;
        }
        let _e185 = phi_5439_;
        phi_5438_ = _e185;
    }
    let _e187 = phi_5438_;
    phi_1704_ = Ih;
    if Ih {
        phi_1704_ = ((_e150.x & 1024u) != 0u);
    }
    let _e191 = phi_1704_;
    phi_5441_ = _e187;
    if _e191 {
        let _e192 = (_e143 * 8u);
        let _e196 = PB.g2_[(_e192 + 2u)];
        let _e207 = PB.g2_[(_e192 + 3u)];
        let _e212 = _e207.zw;
        let _e214 = ((abs(((mat2x2<f32>(vec2<f32>(_e196.x, _e196.y), vec2<f32>(_e196.z, _e196.w)) * _e90) + _e207.xy)) * _e212) - _e212);
        phi_5441_ = min(_e187, clamp((min(_e214.x, _e214.y) + 0.5f), 0f, 1f));
    }
    let _e222 = phi_5441_;
    let _e223 = (_e150.x & 15u);
    let _e226 = ((_e150.x >> bitcast<u32>(4i)) & 15u);
    let _e228 = (Jh && (_e226 != 0u));
    if (_e223 <= 1u) {
        let _e233 = (Hh && (_e223 == 0u));
        phi_6087_ = 0u;
        if _e233 {
            phi_6087_ = (_e150.y | pack2x16float(vec2<f32>(_e222, 0f)));
        }
        let _e238 = phi_6087_;
        phi_6086_ = _e238;
        phi_5462_ = select(unpack4x8unorm(_e150.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e233));
    } else {
        let _e241 = (_e143 * 8u);
        let _e244 = PB.g2_[_e241];
        let _e255 = PB.g2_[(_e241 + 1u)];
        let _e258 = ((mat2x2<f32>(vec2<f32>(_e244.x, _e244.y), vec2<f32>(_e244.z, _e244.w)) * _e90) + _e255.xy);
        if (_e223 == 2u) {
            phi_5440_ = _e258.x;
        } else {
            phi_5440_ = length(_e258);
        }
        let _e263 = phi_5440_;
        let _e270 = bitcast<f32>(_e150.y);
        let _e273 = j.Zb;
        let _e276 = j.ac;
        let _e279 = textureSampleLevel(DD, M9_, vec2<f32>(((clamp(_e263, 0f, 1f) * _e255.z) + _e255.w), ((floor(_e270) * _e273) + _e276)), 0f);
        phi_5464_ = _e279;
        if !(_e228) {
            let _e283 = (_e279.xyz * _e279.w);
            phi_5464_ = vec4<f32>(_e283.x, _e283.y, _e283.z, (_e279.w * (fract(_e270) * 1.0039216f)));
        }
        let _e292 = phi_5464_;
        phi_6086_ = 0u;
        phi_5462_ = _e292;
    }
    let _e294 = phi_6086_;
    let _e296 = phi_5462_;
    phi_6084_ = _e296;
    if _e228 {
        phi_6071_ = _e296;
        if ((_e296.w * _e222) != 0f) {
            let _e302 = m0_.g2_[_e124];
            let _e303 = unpack4x8unorm(_e302);
            let _e304 = _e296.xyz;
            local_5 = _e304;
            let _e305 = _e303.xyz;
            if (_e303.w != 0f) {
                phi_5469_ = (1f / _e303.w);
            } else {
                phi_5469_ = 0f;
            }
            let _e310 = phi_5469_;
            let _e311 = (_e305 * _e310);
            local_3 = _e311;
            switch bitcast<i32>(_e226) {
                case 11: {
                    let _e313 = local_5;
                    local_4 = (_e313 * _e311);
                    break;
                }
                case 1: {
                    let _e315 = local_5;
                    local_4 = ((_e315 + _e311) - (_e315 * _e311));
                    break;
                }
                case 2: {
                    let _e319 = local_5;
                    let _e320 = (_e319 * _e311);
                    local_4 = (select(_e320, (((_e319 + _e311) - _e320) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e311 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e327 = local_5;
                    local_4 = min(_e327, _e311);
                    break;
                }
                case 4: {
                    let _e329 = local_5;
                    local_4 = max(_e329, _e311);
                    break;
                }
                case 5: {
                    let _e332 = clamp(_e305, vec3<f32>(0f, 0f, 0f), _e303.www);
                    let _e338 = vec4<f32>(_e332.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e344 = vec4<f32>(_e338.x, _e332.y, _e338.z, _e338.w);
                    let _e351 = local_5;
                    let _e354 = (clamp((vec3<f32>(1f, 1f, 1f) - _e351), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e303.w);
                    let _e355 = vec4<f32>(_e344.x, _e344.y, _e332.z, _e344.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e355 / _e354)), sign(_e355), (_e354 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e361 = local_5;
                    local_5 = clamp(_e361, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e364 = clamp(_e305, vec3<f32>(0f, 0f, 0f), _e303.www);
                    let _e370 = vec4<f32>(_e364.x, _e303.y, _e303.z, _e303.w);
                    let _e376 = vec4<f32>(_e370.x, _e364.y, _e370.z, _e370.w);
                    phi_5923_ = vec4<f32>(_e376.x, _e376.y, _e364.z, _e376.w);
                    if (_e303.w == 0f) {
                        phi_5923_ = vec4<f32>(_e364.x, _e364.y, _e364.z, 1f);
                    }
                    let _e386 = phi_5923_;
                    let _e390 = (vec3(_e386.w) - _e386.xyz);
                    let _e391 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e390 / (_e391 * _e386.w))), sign(_e390), (_e391 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e399 = local_5;
                    let _e400 = (_e399 * _e311);
                    local_4 = (select(_e400, (((_e399 + _e311) - _e400) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e399 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_5863_ = 0i;
                    loop {
                        let _e408 = phi_5863_;
                        if (_e408 < 3i) {
                            let _e411 = local_5[_e408];
                            if (_e411 <= 0.5f) {
                                let _e414 = local_3[_e408];
                                local_4[_e408] = (1f - _e414);
                            } else {
                                let _e418 = local_3[_e408];
                                if (_e418 <= 0.25f) {
                                    let _e420 = local_3[_e408];
                                    let _e423 = local_3[_e408];
                                    local_4[_e408] = ((((16f * _e420) - 12f) * _e423) + 3f);
                                } else {
                                    let _e427 = local_3[_e408];
                                    local_4[_e408] = (inverseSqrt(_e427) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_5863_ = (_e408 + 1i);
                        }
                    }
                    let _e432 = local_5;
                    let _e436 = local_4;
                    local_4 = (_e311 + ((_e311 * ((_e432 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e436));
                    break;
                }
                case 9: {
                    let _e439 = local_5;
                    local_4 = abs((_e311 - _e439));
                    break;
                }
                case 10: {
                    let _e442 = local_5;
                    local_4 = ((_e442 + _e311) - ((_e442 * 2f) * _e311));
                    break;
                }
                case 12: {
                    if Nh {
                        let _e447 = local_5;
                        let _e448 = clamp(_e447, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e448;
                        let _e463 = (_e448 - vec3(min(min(_e448.x, _e448.y), _e448.z)));
                        let _e471 = (_e463 * ((max(max(_e311.x, _e311.y), _e311.z) - min(min(_e311.x, _e311.y), _e311.z)) / max(0.000062f, max(max(_e463.x, _e463.y), _e463.z))));
                        let _e472 = dot(_e311, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e475 = (_e471 - vec3(dot(_e471, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e488 = (vec2<f32>(_e472, (1f - _e472)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e475.x, _e475.y), _e475.z)), max(max(_e475.x, _e475.y), _e475.z))));
                        local_4 = ((_e475 * min(1f, min(_e488.x, _e488.y))) + vec3(_e472));
                    }
                    break;
                }
                case 13: {
                    if Nh {
                        let _e496 = local_5;
                        let _e497 = clamp(_e496, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e497;
                        let _e512 = (_e311 - vec3(min(min(_e311.x, _e311.y), _e311.z)));
                        let _e520 = (_e512 * ((max(max(_e497.x, _e497.y), _e497.z) - min(min(_e497.x, _e497.y), _e497.z)) / max(0.000062f, max(max(_e512.x, _e512.y), _e512.z))));
                        let _e521 = dot(_e311, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e524 = (_e520 - vec3(dot(_e520, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e537 = (vec2<f32>(_e521, (1f - _e521)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e524.x, _e524.y), _e524.z)), max(max(_e524.x, _e524.y), _e524.z))));
                        local_4 = ((_e524 * min(1f, min(_e537.x, _e537.y))) + vec3(_e521));
                    }
                    break;
                }
                case 14: {
                    if Nh {
                        let _e545 = local_5;
                        let _e546 = clamp(_e545, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e546;
                        let _e547 = dot(_e311, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e550 = (_e546 - vec3(dot(_e546, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e563 = (vec2<f32>(_e547, (1f - _e547)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e550.x, _e550.y), _e550.z)), max(max(_e550.x, _e550.y), _e550.z))));
                        local_4 = ((_e550 * min(1f, min(_e563.x, _e563.y))) + vec3(_e547));
                    }
                    break;
                }
                case 15: {
                    if Nh {
                        let _e571 = local_5;
                        let _e572 = clamp(_e571, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e572;
                        let _e573 = dot(_e572, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e576 = (_e311 - vec3(dot(_e311, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e589 = (vec2<f32>(_e573, (1f - _e573)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e576.x, _e576.y), _e576.z)), max(max(_e576.x, _e576.y), _e576.z))));
                        local_4 = ((_e576 * min(1f, min(_e589.x, _e589.y))) + vec3(_e573));
                    }
                    break;
                }
                default: {
                }
            }
            let _e597 = local_4;
            let _e599 = mix(_e304, _e597, vec3(_e303.w));
            let _e605 = vec4<f32>(_e599.x, _e296.y, _e296.z, _e296.w);
            let _e611 = vec4<f32>(_e605.x, _e599.y, _e605.z, _e605.w);
            phi_6071_ = vec4<f32>(_e611.x, _e611.y, _e599.z, _e611.w);
        }
        let _e619 = phi_6071_;
        let _e622 = (_e619.xyz * _e619.w);
        let _e628 = vec4<f32>(_e622.x, _e619.y, _e619.z, _e619.w);
        let _e634 = vec4<f32>(_e628.x, _e622.y, _e628.z, _e628.w);
        phi_6084_ = vec4<f32>(_e634.x, _e634.y, _e622.z, _e634.w);
    }
    let _e642 = phi_6084_;
    let _e643 = (_e642 * _e222);
    phi_1374_ = Hh;
    if Hh {
        let _e644 = B3_1;
        phi_1374_ = (_e644 != 0u);
    }
    let _e647 = phi_1374_;
    phi_7649_ = _e138;
    if _e647 {
        if (_e294 != 0u) {
            phi_6110_ = _e294;
        } else {
            let _e651 = i0_.g2_[_e124];
            phi_6110_ = _e651;
        }
        let _e653 = phi_6110_;
        let _e654 = B3_1;
        if (_e654 == (_e653 >> bitcast<u32>(16i))) {
            phi_6138_ = min(_e138, unpack2x16float(_e653).x);
        } else {
            phi_6138_ = 0f;
        }
        let _e662 = phi_6138_;
        phi_7649_ = _e662;
    }
    let _e664 = phi_7649_;
    let _e666 = a5_1[3u];
    phi_6172_ = _e126;
    if (_e666 != 0f) {
        let _e668 = a5_1;
        if (_e668.z > 0f) {
            phi_6139_ = _e668.x;
        } else {
            phi_6139_ = length(_e668.xy);
        }
        let _e675 = phi_6139_;
        let _e676 = clamp(_e675, 0f, 1f);
        let _e677 = abs(_e668.z);
        if (_e677 > 1f) {
            phi_6140_ = ((0.9980469f * _e676) + 0.0009765625f);
        } else {
            phi_6140_ = ((0.001953125f * _e676) + _e677);
        }
        let _e684 = phi_6140_;
        let _e687 = textureSampleLevel(DD, M9_, vec2<f32>(_e684, _e668.w), 0f);
        let _e690 = (_e687.xyz * _e687.w);
        let _e696 = vec4<f32>(_e690.x, _e687.y, _e687.z, _e687.w);
        let _e702 = vec4<f32>(_e696.x, _e690.y, _e696.z, _e696.w);
        phi_6172_ = (_e126 * vec4<f32>(_e702.x, _e702.y, _e690.z, _e702.w));
    }
    let _e711 = phi_6172_;
    let _e712 = K1_1;
    let _e713 = (_e711 * _e712);
    phi_1436_ = Jh;
    if Jh {
        let _e714 = D1_1;
        phi_1436_ = (_e714 != 0u);
    }
    let _e717 = phi_1436_;
    phi_7663_ = _e713;
    if _e717 {
        let _e720 = m0_.g2_[_e124];
        let _e725 = ((unpack4x8unorm(_e720) * (1f - _e643.w)) + _e643);
        if (_e713.w != 0f) {
            phi_6182_ = (1f / _e713.w);
        } else {
            phi_6182_ = 0f;
        }
        let _e731 = phi_6182_;
        let _e732 = (_e713.xyz * _e731);
        let _e733 = D1_1;
        local_2 = _e732;
        let _e734 = _e725.xyz;
        if (_e725.w != 0f) {
            phi_6183_ = (1f / _e725.w);
        } else {
            phi_6183_ = 0f;
        }
        let _e739 = phi_6183_;
        let _e740 = (_e734 * _e739);
        local = _e740;
        switch bitcast<i32>(_e733) {
            case 11: {
                let _e742 = local_2;
                local_1 = (_e742 * _e740);
                break;
            }
            case 1: {
                let _e744 = local_2;
                local_1 = ((_e744 + _e740) - (_e744 * _e740));
                break;
            }
            case 2: {
                let _e748 = local_2;
                let _e749 = (_e748 * _e740);
                local_1 = (select(_e749, (((_e748 + _e740) - _e749) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e740 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e756 = local_2;
                local_1 = min(_e756, _e740);
                break;
            }
            case 4: {
                let _e758 = local_2;
                local_1 = max(_e758, _e740);
                break;
            }
            case 5: {
                let _e761 = clamp(_e734, vec3<f32>(0f, 0f, 0f), _e725.www);
                let _e767 = vec4<f32>(_e761.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e773 = vec4<f32>(_e767.x, _e761.y, _e767.z, _e767.w);
                let _e780 = local_2;
                let _e783 = (clamp((vec3<f32>(1f, 1f, 1f) - _e780), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e725.w);
                let _e784 = vec4<f32>(_e773.x, _e773.y, _e761.z, _e773.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e784 / _e783)), sign(_e784), (_e783 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e790 = local_2;
                local_2 = clamp(_e790, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e793 = clamp(_e734, vec3<f32>(0f, 0f, 0f), _e725.www);
                let _e799 = vec4<f32>(_e793.x, _e725.y, _e725.z, _e725.w);
                let _e805 = vec4<f32>(_e799.x, _e793.y, _e799.z, _e799.w);
                phi_7308_ = vec4<f32>(_e805.x, _e805.y, _e793.z, _e805.w);
                if (_e725.w == 0f) {
                    phi_7308_ = vec4<f32>(_e793.x, _e793.y, _e793.z, 1f);
                }
                let _e815 = phi_7308_;
                let _e819 = (vec3(_e815.w) - _e815.xyz);
                let _e820 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e819 / (_e820 * _e815.w))), sign(_e819), (_e820 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e828 = local_2;
                let _e829 = (_e828 * _e740);
                local_1 = (select(_e829, (((_e828 + _e740) - _e829) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e828 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7164_ = 0i;
                loop {
                    let _e837 = phi_7164_;
                    if (_e837 < 3i) {
                        let _e840 = local_2[_e837];
                        if (_e840 <= 0.5f) {
                            let _e843 = local[_e837];
                            local_1[_e837] = (1f - _e843);
                        } else {
                            let _e847 = local[_e837];
                            if (_e847 <= 0.25f) {
                                let _e849 = local[_e837];
                                let _e852 = local[_e837];
                                local_1[_e837] = ((((16f * _e849) - 12f) * _e852) + 3f);
                            } else {
                                let _e856 = local[_e837];
                                local_1[_e837] = (inverseSqrt(_e856) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7164_ = (_e837 + 1i);
                    }
                }
                let _e861 = local_2;
                let _e865 = local_1;
                local_1 = (_e740 + ((_e740 * ((_e861 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e865));
                break;
            }
            case 9: {
                let _e868 = local_2;
                local_1 = abs((_e740 - _e868));
                break;
            }
            case 10: {
                let _e871 = local_2;
                local_1 = ((_e871 + _e740) - ((_e871 * 2f) * _e740));
                break;
            }
            case 12: {
                if Nh {
                    let _e876 = local_2;
                    let _e877 = clamp(_e876, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e877;
                    let _e892 = (_e877 - vec3(min(min(_e877.x, _e877.y), _e877.z)));
                    let _e900 = (_e892 * ((max(max(_e740.x, _e740.y), _e740.z) - min(min(_e740.x, _e740.y), _e740.z)) / max(0.000062f, max(max(_e892.x, _e892.y), _e892.z))));
                    let _e901 = dot(_e740, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e904 = (_e900 - vec3(dot(_e900, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e917 = (vec2<f32>(_e901, (1f - _e901)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e904.x, _e904.y), _e904.z)), max(max(_e904.x, _e904.y), _e904.z))));
                    local_1 = ((_e904 * min(1f, min(_e917.x, _e917.y))) + vec3(_e901));
                }
                break;
            }
            case 13: {
                if Nh {
                    let _e925 = local_2;
                    let _e926 = clamp(_e925, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e926;
                    let _e941 = (_e740 - vec3(min(min(_e740.x, _e740.y), _e740.z)));
                    let _e949 = (_e941 * ((max(max(_e926.x, _e926.y), _e926.z) - min(min(_e926.x, _e926.y), _e926.z)) / max(0.000062f, max(max(_e941.x, _e941.y), _e941.z))));
                    let _e950 = dot(_e740, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e953 = (_e949 - vec3(dot(_e949, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e966 = (vec2<f32>(_e950, (1f - _e950)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e953.x, _e953.y), _e953.z)), max(max(_e953.x, _e953.y), _e953.z))));
                    local_1 = ((_e953 * min(1f, min(_e966.x, _e966.y))) + vec3(_e950));
                }
                break;
            }
            case 14: {
                if Nh {
                    let _e974 = local_2;
                    let _e975 = clamp(_e974, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e975;
                    let _e976 = dot(_e740, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e979 = (_e975 - vec3(dot(_e975, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e992 = (vec2<f32>(_e976, (1f - _e976)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e979.x, _e979.y), _e979.z)), max(max(_e979.x, _e979.y), _e979.z))));
                    local_1 = ((_e979 * min(1f, min(_e992.x, _e992.y))) + vec3(_e976));
                }
                break;
            }
            case 15: {
                if Nh {
                    let _e1000 = local_2;
                    let _e1001 = clamp(_e1000, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e1001;
                    let _e1002 = dot(_e1001, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e1005 = (_e740 - vec3(dot(_e740, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e1018 = (vec2<f32>(_e1002, (1f - _e1002)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e1005.x, _e1005.y), _e1005.z)), max(max(_e1005.x, _e1005.y), _e1005.z))));
                    local_1 = ((_e1005 * min(1f, min(_e1018.x, _e1018.y))) + vec3(_e1002));
                }
                break;
            }
            default: {
            }
        }
        let _e1026 = local_1;
        let _e1029 = (mix(_e732, _e1026, vec3(_e725.w)) * _e713.w);
        let _e1035 = vec4<f32>(_e1029.x, _e713.y, _e713.z, _e713.w);
        let _e1041 = vec4<f32>(_e1035.x, _e1029.y, _e1035.z, _e1035.w);
        phi_7663_ = vec4<f32>(_e1041.x, _e1041.y, _e1029.z, _e1041.w);
    }
    let _e1049 = phi_7663_;
    let _e1050 = (_e1049 * _e664);
    let _e1054 = ((_e643 * (1f - _e1050.w)) + _e1050);
    let _e1055 = _e1054.xyz;
    let _e1058 = j.F3_;
    let _e1060 = j.G3_;
    if (Oh && (_e1054.w != 0f)) {
        phi_7676_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e89.x) + (0.00583715f * _e89.y))))) * _e1058) + _e1060)) + _e1055);
    } else {
        phi_7676_ = _e1055;
    }
    let _e1076 = phi_7676_;
    let _e1082 = vec4<f32>(_e1076.x, _e1054.y, _e1054.z, _e1054.w);
    let _e1088 = vec4<f32>(_e1082.x, _e1076.y, _e1082.z, _e1082.w);
    let _e1094 = vec4<f32>(_e1088.x, _e1088.y, _e1076.z, _e1088.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1076.x + _e1076.y) + _e1076.z) + _e1054.w) == 0f) {
                break;
            }
            let _e1100 = (1f - _e1054.w);
            phi_7678_ = _e1094;
            if (_e1100 != 0f) {
                let _e1104 = m0_.g2_[_e124];
                phi_7678_ = (_e1094 + (unpack4x8unorm(_e1104) * _e1100));
            }
            let _e1109 = phi_7678_;
            m0_.g2_[_e124] = pack4x8unorm(_e1109);
            break;
        }
    }
    if (_e294 != 0u) {
        i0_.g2_[_e124] = _e294;
    }
    A4_.g2_[_e124] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) c2_: vec2<f32>, @location(1) Z4_: f32, @location(3) O0_: vec4<f32>, @location(5) @interpolate(flat, either) B3_: u32, @location(2) a5_: vec4<f32>, @location(4) @interpolate(flat, either) K1_: vec4<f32>, @location(6) @interpolate(flat, either) D1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    c2_1 = c2_;
    Z4_1 = Z4_;
    O0_1 = O0_;
    B3_1 = B3_;
    a5_1 = a5_;
    K1_1 = K1_;
    D1_1 = D1_;
    main_1();
}
