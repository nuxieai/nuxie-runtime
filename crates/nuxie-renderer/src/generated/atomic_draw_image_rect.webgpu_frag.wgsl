struct lf {
    g2_: array<vec2<u32>>,
}

struct i0Td {
    g2_: array<u32>,
}

struct mf {
    g2_: array<vec4<f32>>,
}

struct l0Td {
    g2_: array<u32>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct z4Td {
    g2_: array<u32>,
}

@id(7) override Lh: bool = true;
@id(6) override Kh: bool = true;
@id(4) override Ih: bool = true;
@id(0) override Eh: bool = true;
@id(1) override Fh: bool = true;
@id(2) override Gh: bool = true;

@group(0) @binding(3)
var<storage> CD: lf;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Td;
@group(0) @binding(4)
var<storage> PB: mf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(2) @binding(0)
var<storage, read_write> l0_: l0Td;
@group(0) @binding(0)
var<uniform> j: AC;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;
var<private> c2_1: vec2<f32>;
var<private> Y4_1: f32;
var<private> N0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
var<private> B3_1: u32;
var<private> P5_1: vec4<f32>;
var<private> J1_1: vec4<f32>;
var<private> C1_1: u32;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6048_: f32;
    var phi_1632_: bool;
    var phi_5363_: f32;
    var phi_5362_: f32;
    var phi_5364_: f32;
    var phi_5367_: f32;
    var phi_5366_: f32;
    var phi_1669_: bool;
    var phi_5369_: f32;
    var phi_6015_: u32;
    var phi_5368_: f32;
    var phi_5392_: vec4<f32>;
    var phi_6014_: u32;
    var phi_5390_: vec4<f32>;
    var phi_5397_: f32;
    var phi_5851_: vec4<f32>;
    var phi_5791_: i32;
    var phi_5999_: vec4<f32>;
    var phi_6012_: vec4<f32>;
    var phi_1341_: bool;
    var phi_6038_: u32;
    var phi_6066_: f32;
    var phi_7577_: f32;
    var phi_6067_: f32;
    var phi_6068_: f32;
    var phi_6100_: vec4<f32>;
    var phi_1403_: bool;
    var phi_6110_: f32;
    var phi_6111_: f32;
    var phi_7236_: vec4<f32>;
    var phi_7092_: i32;
    var phi_7591_: vec4<f32>;
    var phi_7604_: vec3<f32>;
    var phi_7606_: vec4<f32>;

    let _e86 = gl_FragCoord_1;
    let _e87 = _e86.xy;
    let _e90 = bitcast<vec2<u32>>(vec2<i32>(floor(_e87)));
    let _e92 = j.q6_;
    let _e121 = bitcast<i32>((((((_e90.y >> bitcast<u32>(5u)) * (((_e92 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e90.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e90.x & 28u) << bitcast<u32>(5u)) + ((_e90.y & 28u) << bitcast<u32>(2i)))) + (((_e90.y & 3u) << bitcast<u32>(2i)) + (_e90.x & 3u))));
    let _e122 = c2_1;
    let _e123 = textureSample(GC, Y5_, _e122);
    let _e124 = Y4_1;
    let _e125 = min(_e124, 1f);
    phi_6048_ = _e125;
    if Fh {
        let _e126 = N0_1;
        let _e129 = min(_e126.xy, _e126.zw);
        phi_6048_ = clamp(min(_e129.x, _e129.y), 0f, _e125);
    }
    let _e135 = phi_6048_;
    let _e138 = z4_.g2_[_e121];
    let _e140 = (_e138 >> bitcast<u32>(17u));
    let _e144 = ((f32((_e138 & 131071u)) * 0.00048828125f) + -32f);
    let _e147 = CD.g2_[_e140];
    phi_5362_ = _e144;
    if ((_e147.x & 768u) != 0u) {
        let _e151 = abs(_e144);
        phi_1632_ = Ih;
        if Ih {
            phi_1632_ = ((_e147.x & 512u) != 0u);
        }
        let _e155 = phi_1632_;
        phi_5363_ = _e151;
        if _e155 {
            phi_5363_ = (1f - abs(((fract((_e151 * 0.5f)) * 2f) + -1f)));
        }
        let _e163 = phi_5363_;
        phi_5362_ = _e163;
    }
    let _e165 = phi_5362_;
    let _e166 = clamp(_e165, 0f, 1f);
    phi_5366_ = _e166;
    if Eh {
        let _e168 = (_e147.x >> bitcast<u32>(16u));
        phi_5367_ = _e166;
        if (_e168 != 0u) {
            let _e172 = i0_.g2_[_e121];
            if (_e168 == (_e172 >> bitcast<u32>(16i))) {
                phi_5364_ = min(_e166, unpack2x16float(_e172).x);
            } else {
                phi_5364_ = 0f;
            }
            let _e180 = phi_5364_;
            phi_5367_ = _e180;
        }
        let _e182 = phi_5367_;
        phi_5366_ = _e182;
    }
    let _e184 = phi_5366_;
    phi_1669_ = Fh;
    if Fh {
        phi_1669_ = ((_e147.x & 1024u) != 0u);
    }
    let _e188 = phi_1669_;
    phi_5369_ = _e184;
    if _e188 {
        let _e189 = (_e140 * 8u);
        let _e193 = PB.g2_[(_e189 + 2u)];
        let _e204 = PB.g2_[(_e189 + 3u)];
        let _e209 = _e204.zw;
        let _e211 = ((abs(((mat2x2<f32>(vec2<f32>(_e193.x, _e193.y), vec2<f32>(_e193.z, _e193.w)) * _e87) + _e204.xy)) * _e209) - _e209);
        phi_5369_ = min(_e184, clamp((min(_e211.x, _e211.y) + 0.5f), 0f, 1f));
    }
    let _e219 = phi_5369_;
    let _e220 = (_e147.x & 15u);
    let _e223 = ((_e147.x >> bitcast<u32>(4i)) & 15u);
    let _e225 = (Gh && (_e223 != 0u));
    if (_e220 <= 1u) {
        let _e230 = (Eh && (_e220 == 0u));
        phi_6015_ = 0u;
        if _e230 {
            phi_6015_ = (_e147.y | pack2x16float(vec2<f32>(_e219, 0f)));
        }
        let _e235 = phi_6015_;
        phi_6014_ = _e235;
        phi_5390_ = select(unpack4x8unorm(_e147.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e230));
    } else {
        let _e238 = (_e140 * 8u);
        let _e241 = PB.g2_[_e238];
        let _e252 = PB.g2_[(_e238 + 1u)];
        let _e255 = ((mat2x2<f32>(vec2<f32>(_e241.x, _e241.y), vec2<f32>(_e241.z, _e241.w)) * _e87) + _e252.xy);
        if (_e220 == 2u) {
            phi_5368_ = _e255.x;
        } else {
            phi_5368_ = length(_e255);
        }
        let _e260 = phi_5368_;
        let _e269 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e260, 0f, 1f) * _e252.z) + _e252.w), bitcast<f32>(_e147.y)), 0f);
        phi_5392_ = _e269;
        if !(_e225) {
            let _e273 = (_e269.xyz * _e269.w);
            let _e279 = vec4<f32>(_e273.x, _e269.y, _e269.z, _e269.w);
            let _e285 = vec4<f32>(_e279.x, _e273.y, _e279.z, _e279.w);
            phi_5392_ = vec4<f32>(_e285.x, _e285.y, _e273.z, _e285.w);
        }
        let _e293 = phi_5392_;
        phi_6014_ = 0u;
        phi_5390_ = _e293;
    }
    let _e295 = phi_6014_;
    let _e297 = phi_5390_;
    phi_6012_ = _e297;
    if _e225 {
        phi_5999_ = _e297;
        if ((_e297.w * _e219) != 0f) {
            let _e303 = l0_.g2_[_e121];
            let _e304 = unpack4x8unorm(_e303);
            let _e305 = _e297.xyz;
            local_5 = _e305;
            let _e306 = _e304.xyz;
            if (_e304.w != 0f) {
                phi_5397_ = (1f / _e304.w);
            } else {
                phi_5397_ = 0f;
            }
            let _e311 = phi_5397_;
            let _e312 = (_e306 * _e311);
            local_3 = _e312;
            switch bitcast<i32>(_e223) {
                case 11: {
                    let _e314 = local_5;
                    local_4 = (_e314 * _e312);
                    break;
                }
                case 1: {
                    let _e316 = local_5;
                    local_4 = ((_e316 + _e312) - (_e316 * _e312));
                    break;
                }
                case 2: {
                    let _e320 = local_5;
                    let _e321 = (_e320 * _e312);
                    local_4 = (select(_e321, (((_e320 + _e312) - _e321) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e312 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e328 = local_5;
                    local_4 = min(_e328, _e312);
                    break;
                }
                case 4: {
                    let _e330 = local_5;
                    local_4 = max(_e330, _e312);
                    break;
                }
                case 5: {
                    let _e333 = clamp(_e306, vec3<f32>(0f, 0f, 0f), _e304.www);
                    let _e339 = vec4<f32>(_e333.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e345 = vec4<f32>(_e339.x, _e333.y, _e339.z, _e339.w);
                    let _e352 = local_5;
                    let _e355 = (clamp((vec3<f32>(1f, 1f, 1f) - _e352), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e304.w);
                    let _e356 = vec4<f32>(_e345.x, _e345.y, _e333.z, _e345.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e356 / _e355)), sign(_e356), (_e355 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e362 = local_5;
                    local_5 = clamp(_e362, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e365 = clamp(_e306, vec3<f32>(0f, 0f, 0f), _e304.www);
                    let _e371 = vec4<f32>(_e365.x, _e304.y, _e304.z, _e304.w);
                    let _e377 = vec4<f32>(_e371.x, _e365.y, _e371.z, _e371.w);
                    phi_5851_ = vec4<f32>(_e377.x, _e377.y, _e365.z, _e377.w);
                    if (_e304.w == 0f) {
                        phi_5851_ = vec4<f32>(_e365.x, _e365.y, _e365.z, 1f);
                    }
                    let _e387 = phi_5851_;
                    let _e391 = (vec3(_e387.w) - _e387.xyz);
                    let _e392 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e391 / (_e392 * _e387.w))), sign(_e391), (_e392 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e400 = local_5;
                    let _e401 = (_e400 * _e312);
                    local_4 = (select(_e401, (((_e400 + _e312) - _e401) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e400 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_5791_ = 0i;
                    loop {
                        let _e409 = phi_5791_;
                        if (_e409 < 3i) {
                            let _e412 = local_5[_e409];
                            if (_e412 <= 0.5f) {
                                let _e415 = local_3[_e409];
                                local_4[_e409] = (1f - _e415);
                            } else {
                                let _e419 = local_3[_e409];
                                if (_e419 <= 0.25f) {
                                    let _e421 = local_3[_e409];
                                    let _e424 = local_3[_e409];
                                    local_4[_e409] = ((((16f * _e421) - 12f) * _e424) + 3f);
                                } else {
                                    let _e428 = local_3[_e409];
                                    local_4[_e409] = (inverseSqrt(_e428) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_5791_ = (_e409 + 1i);
                        }
                    }
                    let _e433 = local_5;
                    let _e437 = local_4;
                    local_4 = (_e312 + ((_e312 * ((_e433 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e437));
                    break;
                }
                case 9: {
                    let _e440 = local_5;
                    local_4 = abs((_e312 - _e440));
                    break;
                }
                case 10: {
                    let _e443 = local_5;
                    local_4 = ((_e443 + _e312) - ((_e443 * 2f) * _e312));
                    break;
                }
                case 12: {
                    if Kh {
                        let _e448 = local_5;
                        let _e449 = clamp(_e448, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e449;
                        let _e464 = (_e449 - vec3(min(min(_e449.x, _e449.y), _e449.z)));
                        let _e472 = (_e464 * ((max(max(_e312.x, _e312.y), _e312.z) - min(min(_e312.x, _e312.y), _e312.z)) / max(0.000062f, max(max(_e464.x, _e464.y), _e464.z))));
                        let _e473 = dot(_e312, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e476 = (_e472 - vec3(dot(_e472, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e489 = (vec2<f32>(_e473, (1f - _e473)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e476.x, _e476.y), _e476.z)), max(max(_e476.x, _e476.y), _e476.z))));
                        local_4 = ((_e476 * min(1f, min(_e489.x, _e489.y))) + vec3(_e473));
                    }
                    break;
                }
                case 13: {
                    if Kh {
                        let _e497 = local_5;
                        let _e498 = clamp(_e497, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e498;
                        let _e513 = (_e312 - vec3(min(min(_e312.x, _e312.y), _e312.z)));
                        let _e521 = (_e513 * ((max(max(_e498.x, _e498.y), _e498.z) - min(min(_e498.x, _e498.y), _e498.z)) / max(0.000062f, max(max(_e513.x, _e513.y), _e513.z))));
                        let _e522 = dot(_e312, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e525 = (_e521 - vec3(dot(_e521, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e538 = (vec2<f32>(_e522, (1f - _e522)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e525.x, _e525.y), _e525.z)), max(max(_e525.x, _e525.y), _e525.z))));
                        local_4 = ((_e525 * min(1f, min(_e538.x, _e538.y))) + vec3(_e522));
                    }
                    break;
                }
                case 14: {
                    if Kh {
                        let _e546 = local_5;
                        let _e547 = clamp(_e546, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e547;
                        let _e548 = dot(_e312, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e551 = (_e547 - vec3(dot(_e547, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e564 = (vec2<f32>(_e548, (1f - _e548)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e551.x, _e551.y), _e551.z)), max(max(_e551.x, _e551.y), _e551.z))));
                        local_4 = ((_e551 * min(1f, min(_e564.x, _e564.y))) + vec3(_e548));
                    }
                    break;
                }
                case 15: {
                    if Kh {
                        let _e572 = local_5;
                        let _e573 = clamp(_e572, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e573;
                        let _e574 = dot(_e573, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e577 = (_e312 - vec3(dot(_e312, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e590 = (vec2<f32>(_e574, (1f - _e574)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e577.x, _e577.y), _e577.z)), max(max(_e577.x, _e577.y), _e577.z))));
                        local_4 = ((_e577 * min(1f, min(_e590.x, _e590.y))) + vec3(_e574));
                    }
                    break;
                }
                default: {
                }
            }
            let _e598 = local_4;
            let _e600 = mix(_e305, _e598, vec3(_e304.w));
            let _e606 = vec4<f32>(_e600.x, _e297.y, _e297.z, _e297.w);
            let _e612 = vec4<f32>(_e606.x, _e600.y, _e606.z, _e606.w);
            phi_5999_ = vec4<f32>(_e612.x, _e612.y, _e600.z, _e612.w);
        }
        let _e620 = phi_5999_;
        let _e623 = (_e620.xyz * _e620.w);
        let _e629 = vec4<f32>(_e623.x, _e620.y, _e620.z, _e620.w);
        let _e635 = vec4<f32>(_e629.x, _e623.y, _e629.z, _e629.w);
        phi_6012_ = vec4<f32>(_e635.x, _e635.y, _e623.z, _e635.w);
    }
    let _e643 = phi_6012_;
    let _e644 = (_e643 * _e219);
    phi_1341_ = Eh;
    if Eh {
        let _e645 = B3_1;
        phi_1341_ = (_e645 != 0u);
    }
    let _e648 = phi_1341_;
    phi_7577_ = _e135;
    if _e648 {
        if (_e295 != 0u) {
            phi_6038_ = _e295;
        } else {
            let _e652 = i0_.g2_[_e121];
            phi_6038_ = _e652;
        }
        let _e654 = phi_6038_;
        let _e655 = B3_1;
        if (_e655 == (_e654 >> bitcast<u32>(16i))) {
            phi_6066_ = min(_e135, unpack2x16float(_e654).x);
        } else {
            phi_6066_ = 0f;
        }
        let _e663 = phi_6066_;
        phi_7577_ = _e663;
    }
    let _e665 = phi_7577_;
    let _e667 = P5_1[3u];
    phi_6100_ = _e123;
    if (_e667 != 0f) {
        let _e669 = P5_1;
        if (_e669.z > 0f) {
            phi_6067_ = _e669.x;
        } else {
            phi_6067_ = length(_e669.xy);
        }
        let _e676 = phi_6067_;
        let _e677 = clamp(_e676, 0f, 1f);
        let _e678 = abs(_e669.z);
        if (_e678 > 1f) {
            phi_6068_ = ((0.9980469f * _e677) + 0.0009765625f);
        } else {
            phi_6068_ = ((0.001953125f * _e677) + _e678);
        }
        let _e685 = phi_6068_;
        let _e688 = textureSampleLevel(DD, P9_, vec2<f32>(_e685, _e669.w), 0f);
        let _e691 = (_e688.xyz * _e688.w);
        let _e697 = vec4<f32>(_e691.x, _e688.y, _e688.z, _e688.w);
        let _e703 = vec4<f32>(_e697.x, _e691.y, _e697.z, _e697.w);
        phi_6100_ = (_e123 * vec4<f32>(_e703.x, _e703.y, _e691.z, _e703.w));
    }
    let _e712 = phi_6100_;
    let _e713 = J1_1;
    let _e714 = (_e712 * _e713);
    phi_1403_ = Gh;
    if Gh {
        let _e715 = C1_1;
        phi_1403_ = (_e715 != 0u);
    }
    let _e718 = phi_1403_;
    phi_7591_ = _e714;
    if _e718 {
        let _e721 = l0_.g2_[_e121];
        let _e726 = ((unpack4x8unorm(_e721) * (1f - _e644.w)) + _e644);
        if (_e714.w != 0f) {
            phi_6110_ = (1f / _e714.w);
        } else {
            phi_6110_ = 0f;
        }
        let _e732 = phi_6110_;
        let _e733 = (_e714.xyz * _e732);
        let _e734 = C1_1;
        local_2 = _e733;
        let _e735 = _e726.xyz;
        if (_e726.w != 0f) {
            phi_6111_ = (1f / _e726.w);
        } else {
            phi_6111_ = 0f;
        }
        let _e740 = phi_6111_;
        let _e741 = (_e735 * _e740);
        local = _e741;
        switch bitcast<i32>(_e734) {
            case 11: {
                let _e743 = local_2;
                local_1 = (_e743 * _e741);
                break;
            }
            case 1: {
                let _e745 = local_2;
                local_1 = ((_e745 + _e741) - (_e745 * _e741));
                break;
            }
            case 2: {
                let _e749 = local_2;
                let _e750 = (_e749 * _e741);
                local_1 = (select(_e750, (((_e749 + _e741) - _e750) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e741 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e757 = local_2;
                local_1 = min(_e757, _e741);
                break;
            }
            case 4: {
                let _e759 = local_2;
                local_1 = max(_e759, _e741);
                break;
            }
            case 5: {
                let _e762 = clamp(_e735, vec3<f32>(0f, 0f, 0f), _e726.www);
                let _e768 = vec4<f32>(_e762.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e774 = vec4<f32>(_e768.x, _e762.y, _e768.z, _e768.w);
                let _e781 = local_2;
                let _e784 = (clamp((vec3<f32>(1f, 1f, 1f) - _e781), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e726.w);
                let _e785 = vec4<f32>(_e774.x, _e774.y, _e762.z, _e774.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e785 / _e784)), sign(_e785), (_e784 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e791 = local_2;
                local_2 = clamp(_e791, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e794 = clamp(_e735, vec3<f32>(0f, 0f, 0f), _e726.www);
                let _e800 = vec4<f32>(_e794.x, _e726.y, _e726.z, _e726.w);
                let _e806 = vec4<f32>(_e800.x, _e794.y, _e800.z, _e800.w);
                phi_7236_ = vec4<f32>(_e806.x, _e806.y, _e794.z, _e806.w);
                if (_e726.w == 0f) {
                    phi_7236_ = vec4<f32>(_e794.x, _e794.y, _e794.z, 1f);
                }
                let _e816 = phi_7236_;
                let _e820 = (vec3(_e816.w) - _e816.xyz);
                let _e821 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e820 / (_e821 * _e816.w))), sign(_e820), (_e821 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e829 = local_2;
                let _e830 = (_e829 * _e741);
                local_1 = (select(_e830, (((_e829 + _e741) - _e830) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e829 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7092_ = 0i;
                loop {
                    let _e838 = phi_7092_;
                    if (_e838 < 3i) {
                        let _e841 = local_2[_e838];
                        if (_e841 <= 0.5f) {
                            let _e844 = local[_e838];
                            local_1[_e838] = (1f - _e844);
                        } else {
                            let _e848 = local[_e838];
                            if (_e848 <= 0.25f) {
                                let _e850 = local[_e838];
                                let _e853 = local[_e838];
                                local_1[_e838] = ((((16f * _e850) - 12f) * _e853) + 3f);
                            } else {
                                let _e857 = local[_e838];
                                local_1[_e838] = (inverseSqrt(_e857) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7092_ = (_e838 + 1i);
                    }
                }
                let _e862 = local_2;
                let _e866 = local_1;
                local_1 = (_e741 + ((_e741 * ((_e862 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e866));
                break;
            }
            case 9: {
                let _e869 = local_2;
                local_1 = abs((_e741 - _e869));
                break;
            }
            case 10: {
                let _e872 = local_2;
                local_1 = ((_e872 + _e741) - ((_e872 * 2f) * _e741));
                break;
            }
            case 12: {
                if Kh {
                    let _e877 = local_2;
                    let _e878 = clamp(_e877, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e878;
                    let _e893 = (_e878 - vec3(min(min(_e878.x, _e878.y), _e878.z)));
                    let _e901 = (_e893 * ((max(max(_e741.x, _e741.y), _e741.z) - min(min(_e741.x, _e741.y), _e741.z)) / max(0.000062f, max(max(_e893.x, _e893.y), _e893.z))));
                    let _e902 = dot(_e741, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e905 = (_e901 - vec3(dot(_e901, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e918 = (vec2<f32>(_e902, (1f - _e902)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e905.x, _e905.y), _e905.z)), max(max(_e905.x, _e905.y), _e905.z))));
                    local_1 = ((_e905 * min(1f, min(_e918.x, _e918.y))) + vec3(_e902));
                }
                break;
            }
            case 13: {
                if Kh {
                    let _e926 = local_2;
                    let _e927 = clamp(_e926, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e927;
                    let _e942 = (_e741 - vec3(min(min(_e741.x, _e741.y), _e741.z)));
                    let _e950 = (_e942 * ((max(max(_e927.x, _e927.y), _e927.z) - min(min(_e927.x, _e927.y), _e927.z)) / max(0.000062f, max(max(_e942.x, _e942.y), _e942.z))));
                    let _e951 = dot(_e741, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e954 = (_e950 - vec3(dot(_e950, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e967 = (vec2<f32>(_e951, (1f - _e951)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e954.x, _e954.y), _e954.z)), max(max(_e954.x, _e954.y), _e954.z))));
                    local_1 = ((_e954 * min(1f, min(_e967.x, _e967.y))) + vec3(_e951));
                }
                break;
            }
            case 14: {
                if Kh {
                    let _e975 = local_2;
                    let _e976 = clamp(_e975, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e976;
                    let _e977 = dot(_e741, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e980 = (_e976 - vec3(dot(_e976, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e993 = (vec2<f32>(_e977, (1f - _e977)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e980.x, _e980.y), _e980.z)), max(max(_e980.x, _e980.y), _e980.z))));
                    local_1 = ((_e980 * min(1f, min(_e993.x, _e993.y))) + vec3(_e977));
                }
                break;
            }
            case 15: {
                if Kh {
                    let _e1001 = local_2;
                    let _e1002 = clamp(_e1001, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e1002;
                    let _e1003 = dot(_e1002, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e1006 = (_e741 - vec3(dot(_e741, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e1019 = (vec2<f32>(_e1003, (1f - _e1003)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e1006.x, _e1006.y), _e1006.z)), max(max(_e1006.x, _e1006.y), _e1006.z))));
                    local_1 = ((_e1006 * min(1f, min(_e1019.x, _e1019.y))) + vec3(_e1003));
                }
                break;
            }
            default: {
            }
        }
        let _e1027 = local_1;
        let _e1030 = (mix(_e733, _e1027, vec3(_e726.w)) * _e714.w);
        let _e1036 = vec4<f32>(_e1030.x, _e714.y, _e714.z, _e714.w);
        let _e1042 = vec4<f32>(_e1036.x, _e1030.y, _e1036.z, _e1036.w);
        phi_7591_ = vec4<f32>(_e1042.x, _e1042.y, _e1030.z, _e1042.w);
    }
    let _e1050 = phi_7591_;
    let _e1051 = (_e1050 * _e665);
    let _e1055 = ((_e644 * (1f - _e1051.w)) + _e1051);
    let _e1056 = _e1055.xyz;
    let _e1059 = j.F3_;
    let _e1061 = j.G3_;
    if (Lh && (_e1055.w != 0f)) {
        phi_7604_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e86.x) + (0.00583715f * _e86.y))))) * _e1059) + _e1061)) + _e1056);
    } else {
        phi_7604_ = _e1056;
    }
    let _e1077 = phi_7604_;
    let _e1083 = vec4<f32>(_e1077.x, _e1055.y, _e1055.z, _e1055.w);
    let _e1089 = vec4<f32>(_e1083.x, _e1077.y, _e1083.z, _e1083.w);
    let _e1095 = vec4<f32>(_e1089.x, _e1089.y, _e1077.z, _e1089.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e1055.w == 0f) {
                break;
            }
            let _e1098 = (1f - _e1055.w);
            phi_7606_ = _e1095;
            if (_e1098 != 0f) {
                let _e1102 = l0_.g2_[_e121];
                phi_7606_ = (_e1095 + (unpack4x8unorm(_e1102) * _e1098));
            }
            let _e1107 = phi_7606_;
            l0_.g2_[_e121] = pack4x8unorm(_e1107);
            break;
        }
    }
    if (_e295 != 0u) {
        i0_.g2_[_e121] = _e295;
    }
    z4_.g2_[_e121] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) c2_: vec2<f32>, @location(1) Y4_: f32, @location(3) N0_: vec4<f32>, @location(5) @interpolate(flat, either) B3_: u32, @location(2) P5_: vec4<f32>, @location(4) @interpolate(flat, either) J1_: vec4<f32>, @location(6) @interpolate(flat, either) C1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    c2_1 = c2_;
    Y4_1 = Y4_;
    N0_1 = N0_;
    B3_1 = B3_;
    P5_1 = P5_;
    J1_1 = J1_;
    C1_1 = C1_;
    main_1();
}
