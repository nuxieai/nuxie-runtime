struct jg {
    v2_: array<vec2<u32>>,
}

struct m0Pe {
    v2_: array<u32>,
}

struct kg {
    v2_: array<vec4<f32>>,
}

struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct n0Pe {
    v2_: array<u32>,
}

struct P4Pe {
    v2_: array<u32>,
}

@id(7) override fj: bool = true;
@id(6) override ej: bool = true;
@id(4) override cj: bool = true;
@id(0) override Yi: bool = true;
@id(1) override Zi: bool = true;
@id(2) override aj: bool = true;

@group(0) @binding(3)
var<storage> WC: jg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var I8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Pe;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
var<private> m2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> P4_: P4Pe;
var<private> R3_1: u32;
var<private> U1_1: vec4<f32>;
var<private> K1_1: u32;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_5982_: f32;
    var phi_1583_: bool;
    var phi_5297_: f32;
    var phi_5296_: f32;
    var phi_5298_: f32;
    var phi_5301_: f32;
    var phi_5300_: f32;
    var phi_1620_: bool;
    var phi_5303_: f32;
    var phi_5949_: u32;
    var phi_5302_: f32;
    var phi_5326_: vec4<f32>;
    var phi_5948_: u32;
    var phi_5324_: vec4<f32>;
    var phi_5331_: f32;
    var phi_5785_: vec4<f32>;
    var phi_5725_: i32;
    var phi_5933_: vec4<f32>;
    var phi_5946_: vec4<f32>;
    var phi_1319_: bool;
    var phi_5972_: u32;
    var phi_6000_: f32;
    var phi_7379_: f32;
    var phi_1351_: bool;
    var phi_6036_: f32;
    var phi_6037_: f32;
    var phi_7066_: vec4<f32>;
    var phi_6934_: i32;
    var phi_7391_: vec4<f32>;
    var phi_7404_: vec3<f32>;
    var phi_7406_: vec4<f32>;

    let _e84 = gl_FragCoord_1;
    let _e85 = _e84.xy;
    let _e88 = bitcast<vec2<u32>>(vec2<i32>(floor(_e85)));
    let _e90 = j.L6_;
    let _e119 = bitcast<i32>((((((_e88.y >> bitcast<u32>(5u)) * (((_e90 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e88.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e88.x & 28u) << bitcast<u32>(5u)) + ((_e88.y & 28u) << bitcast<u32>(2i)))) + (((_e88.y & 3u) << bitcast<u32>(2i)) + (_e88.x & 3u))));
    let _e120 = m2_1;
    let _e121 = textureSample(TB, S4_, _e120);
    phi_5982_ = 1f;
    if Zi {
        let _e122 = W0_1;
        let _e125 = min(_e122.xy, _e122.zw);
        phi_5982_ = clamp(min(_e125.x, _e125.y), 0f, 1f);
    }
    let _e131 = phi_5982_;
    let _e134 = P4_.v2_[_e119];
    let _e136 = (_e134 >> bitcast<u32>(17u));
    let _e140 = ((f32((_e134 & 131071u)) * 0.00048828125f) + -32f);
    let _e143 = WC.v2_[_e136];
    phi_5296_ = _e140;
    if ((_e143.x & 768u) != 0u) {
        let _e147 = abs(_e140);
        phi_1583_ = cj;
        if cj {
            phi_1583_ = ((_e143.x & 512u) != 0u);
        }
        let _e151 = phi_1583_;
        phi_5297_ = _e147;
        if _e151 {
            phi_5297_ = (1f - abs(((fract((_e147 * 0.5f)) * 2f) + -1f)));
        }
        let _e159 = phi_5297_;
        phi_5296_ = _e159;
    }
    let _e161 = phi_5296_;
    let _e162 = clamp(_e161, 0f, 1f);
    phi_5300_ = _e162;
    if Yi {
        let _e164 = (_e143.x >> bitcast<u32>(16u));
        phi_5301_ = _e162;
        if (_e164 != 0u) {
            let _e168 = m0_.v2_[_e119];
            if (_e164 == (_e168 >> bitcast<u32>(16i))) {
                phi_5298_ = min(_e162, unpack2x16float(_e168).x);
            } else {
                phi_5298_ = 0f;
            }
            let _e176 = phi_5298_;
            phi_5301_ = _e176;
        }
        let _e178 = phi_5301_;
        phi_5300_ = _e178;
    }
    let _e180 = phi_5300_;
    phi_1620_ = Zi;
    if Zi {
        phi_1620_ = ((_e143.x & 1024u) != 0u);
    }
    let _e184 = phi_1620_;
    phi_5303_ = _e180;
    if _e184 {
        let _e185 = (_e136 * 8u);
        let _e189 = JB.v2_[(_e185 + 2u)];
        let _e200 = JB.v2_[(_e185 + 3u)];
        let _e205 = _e200.zw;
        let _e207 = ((abs(((mat2x2<f32>(vec2<f32>(_e189.x, _e189.y), vec2<f32>(_e189.z, _e189.w)) * _e85) + _e200.xy)) * _e205) - _e205);
        phi_5303_ = min(_e180, clamp((min(_e207.x, _e207.y) + 0.5f), 0f, 1f));
    }
    let _e215 = phi_5303_;
    let _e216 = (_e143.x & 15u);
    let _e219 = ((_e143.x >> bitcast<u32>(4i)) & 15u);
    let _e221 = (aj && (_e219 != 0u));
    if (_e216 <= 1u) {
        let _e226 = (Yi && (_e216 == 0u));
        phi_5949_ = 0u;
        if _e226 {
            phi_5949_ = (_e143.y | pack2x16float(vec2<f32>(_e215, 0f)));
        }
        let _e231 = phi_5949_;
        phi_5948_ = _e231;
        phi_5324_ = select(unpack4x8unorm(_e143.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e226));
    } else {
        let _e234 = (_e136 * 8u);
        let _e237 = JB.v2_[_e234];
        let _e248 = JB.v2_[(_e234 + 1u)];
        let _e251 = ((mat2x2<f32>(vec2<f32>(_e237.x, _e237.y), vec2<f32>(_e237.z, _e237.w)) * _e85) + _e248.xy);
        if (_e216 == 2u) {
            phi_5302_ = _e251.x;
        } else {
            phi_5302_ = length(_e251);
        }
        let _e256 = phi_5302_;
        let _e263 = bitcast<f32>(_e143.y);
        let _e266 = j.cd;
        let _e269 = j.g7_;
        let _e272 = textureSampleLevel(YC, I8_, vec2<f32>(((clamp(_e256, 0f, 1f) * _e248.z) + _e248.w), ((floor(_e263) * _e266) + _e269)), 0f);
        phi_5326_ = _e272;
        if !(_e221) {
            let _e276 = (_e272.xyz * _e272.w);
            phi_5326_ = vec4<f32>(_e276.x, _e276.y, _e276.z, (_e272.w * (fract(_e263) * 1.0039216f)));
        }
        let _e285 = phi_5326_;
        phi_5948_ = 0u;
        phi_5324_ = _e285;
    }
    let _e287 = phi_5948_;
    let _e289 = phi_5324_;
    phi_5946_ = _e289;
    if _e221 {
        phi_5933_ = _e289;
        if ((_e289.w * _e215) != 0f) {
            let _e295 = n0_.v2_[_e119];
            let _e296 = unpack4x8unorm(_e295);
            let _e297 = _e289.xyz;
            local_5 = _e297;
            let _e298 = _e296.xyz;
            if (_e296.w != 0f) {
                phi_5331_ = (1f / _e296.w);
            } else {
                phi_5331_ = 0f;
            }
            let _e303 = phi_5331_;
            let _e304 = (_e298 * _e303);
            local_3 = _e304;
            switch bitcast<i32>(_e219) {
                case 11: {
                    let _e306 = local_5;
                    local_4 = (_e306 * _e304);
                    break;
                }
                case 1: {
                    let _e308 = local_5;
                    local_4 = ((_e308 + _e304) - (_e308 * _e304));
                    break;
                }
                case 2: {
                    let _e312 = local_5;
                    let _e313 = (_e312 * _e304);
                    local_4 = (select(_e313, (((_e312 + _e304) - _e313) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e304 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e320 = local_5;
                    local_4 = min(_e320, _e304);
                    break;
                }
                case 4: {
                    let _e322 = local_5;
                    local_4 = max(_e322, _e304);
                    break;
                }
                case 5: {
                    let _e325 = clamp(_e298, vec3<f32>(0f, 0f, 0f), _e296.www);
                    let _e331 = vec4<f32>(_e325.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e337 = vec4<f32>(_e331.x, _e325.y, _e331.z, _e331.w);
                    let _e344 = local_5;
                    let _e347 = (clamp((vec3<f32>(1f, 1f, 1f) - _e344), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e296.w);
                    let _e348 = vec4<f32>(_e337.x, _e337.y, _e325.z, _e337.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e348 / _e347)), sign(_e348), (_e347 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e354 = local_5;
                    local_5 = clamp(_e354, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e357 = clamp(_e298, vec3<f32>(0f, 0f, 0f), _e296.www);
                    let _e363 = vec4<f32>(_e357.x, _e296.y, _e296.z, _e296.w);
                    let _e369 = vec4<f32>(_e363.x, _e357.y, _e363.z, _e363.w);
                    phi_5785_ = vec4<f32>(_e369.x, _e369.y, _e357.z, _e369.w);
                    if (_e296.w == 0f) {
                        phi_5785_ = vec4<f32>(_e357.x, _e357.y, _e357.z, 1f);
                    }
                    let _e379 = phi_5785_;
                    let _e383 = (vec3(_e379.w) - _e379.xyz);
                    let _e384 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e383 / (_e384 * _e379.w))), sign(_e383), (_e384 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e392 = local_5;
                    let _e393 = (_e392 * _e304);
                    local_4 = (select(_e393, (((_e392 + _e304) - _e393) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e392 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_5725_ = 0i;
                    loop {
                        let _e401 = phi_5725_;
                        if (_e401 < 3i) {
                            let _e404 = local_5[_e401];
                            if (_e404 <= 0.5f) {
                                let _e407 = local_3[_e401];
                                local_4[_e401] = (1f - _e407);
                            } else {
                                let _e411 = local_3[_e401];
                                if (_e411 <= 0.25f) {
                                    let _e413 = local_3[_e401];
                                    let _e416 = local_3[_e401];
                                    local_4[_e401] = ((((16f * _e413) - 12f) * _e416) + 3f);
                                } else {
                                    let _e420 = local_3[_e401];
                                    local_4[_e401] = (inverseSqrt(_e420) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_5725_ = (_e401 + 1i);
                        }
                    }
                    let _e425 = local_5;
                    let _e429 = local_4;
                    local_4 = (_e304 + ((_e304 * ((_e425 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e429));
                    break;
                }
                case 9: {
                    let _e432 = local_5;
                    local_4 = abs((_e304 - _e432));
                    break;
                }
                case 10: {
                    let _e435 = local_5;
                    local_4 = ((_e435 + _e304) - ((_e435 * 2f) * _e304));
                    break;
                }
                case 12: {
                    if ej {
                        let _e440 = local_5;
                        let _e441 = clamp(_e440, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e441;
                        let _e456 = (_e441 - vec3(min(min(_e441.x, _e441.y), _e441.z)));
                        let _e464 = (_e456 * ((max(max(_e304.x, _e304.y), _e304.z) - min(min(_e304.x, _e304.y), _e304.z)) / max(0.000062f, max(max(_e456.x, _e456.y), _e456.z))));
                        let _e465 = dot(_e304, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e468 = (_e464 - vec3(dot(_e464, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e481 = (vec2<f32>(_e465, (1f - _e465)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e468.x, _e468.y), _e468.z)), max(max(_e468.x, _e468.y), _e468.z))));
                        local_4 = ((_e468 * min(1f, min(_e481.x, _e481.y))) + vec3(_e465));
                    }
                    break;
                }
                case 13: {
                    if ej {
                        let _e489 = local_5;
                        let _e490 = clamp(_e489, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e490;
                        let _e505 = (_e304 - vec3(min(min(_e304.x, _e304.y), _e304.z)));
                        let _e513 = (_e505 * ((max(max(_e490.x, _e490.y), _e490.z) - min(min(_e490.x, _e490.y), _e490.z)) / max(0.000062f, max(max(_e505.x, _e505.y), _e505.z))));
                        let _e514 = dot(_e304, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e517 = (_e513 - vec3(dot(_e513, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e530 = (vec2<f32>(_e514, (1f - _e514)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e517.x, _e517.y), _e517.z)), max(max(_e517.x, _e517.y), _e517.z))));
                        local_4 = ((_e517 * min(1f, min(_e530.x, _e530.y))) + vec3(_e514));
                    }
                    break;
                }
                case 14: {
                    if ej {
                        let _e538 = local_5;
                        let _e539 = clamp(_e538, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e539;
                        let _e540 = dot(_e304, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e543 = (_e539 - vec3(dot(_e539, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e556 = (vec2<f32>(_e540, (1f - _e540)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e543.x, _e543.y), _e543.z)), max(max(_e543.x, _e543.y), _e543.z))));
                        local_4 = ((_e543 * min(1f, min(_e556.x, _e556.y))) + vec3(_e540));
                    }
                    break;
                }
                case 15: {
                    if ej {
                        let _e564 = local_5;
                        let _e565 = clamp(_e564, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e565;
                        let _e566 = dot(_e565, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e569 = (_e304 - vec3(dot(_e304, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e582 = (vec2<f32>(_e566, (1f - _e566)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e569.x, _e569.y), _e569.z)), max(max(_e569.x, _e569.y), _e569.z))));
                        local_4 = ((_e569 * min(1f, min(_e582.x, _e582.y))) + vec3(_e566));
                    }
                    break;
                }
                default: {
                }
            }
            let _e590 = local_4;
            let _e592 = mix(_e297, _e590, vec3(_e296.w));
            let _e598 = vec4<f32>(_e592.x, _e289.y, _e289.z, _e289.w);
            let _e604 = vec4<f32>(_e598.x, _e592.y, _e598.z, _e598.w);
            phi_5933_ = vec4<f32>(_e604.x, _e604.y, _e592.z, _e604.w);
        }
        let _e612 = phi_5933_;
        let _e615 = (_e612.xyz * _e612.w);
        let _e621 = vec4<f32>(_e615.x, _e612.y, _e612.z, _e612.w);
        let _e627 = vec4<f32>(_e621.x, _e615.y, _e621.z, _e621.w);
        phi_5946_ = vec4<f32>(_e627.x, _e627.y, _e615.z, _e627.w);
    }
    let _e635 = phi_5946_;
    let _e636 = (_e635 * _e215);
    phi_1319_ = Yi;
    if Yi {
        let _e637 = R3_1;
        phi_1319_ = (_e637 != 0u);
    }
    let _e640 = phi_1319_;
    phi_7379_ = _e131;
    if _e640 {
        if (_e287 != 0u) {
            phi_5972_ = _e287;
        } else {
            let _e644 = m0_.v2_[_e119];
            phi_5972_ = _e644;
        }
        let _e646 = phi_5972_;
        let _e647 = R3_1;
        if (_e647 == (_e646 >> bitcast<u32>(16i))) {
            phi_6000_ = min(_e131, unpack2x16float(_e646).x);
        } else {
            phi_6000_ = 0f;
        }
        let _e655 = phi_6000_;
        phi_7379_ = _e655;
    }
    let _e657 = phi_7379_;
    let _e658 = U1_1;
    let _e659 = (_e121 * _e658);
    phi_1351_ = aj;
    if aj {
        let _e660 = K1_1;
        phi_1351_ = (_e660 != 0u);
    }
    let _e663 = phi_1351_;
    phi_7391_ = _e659;
    if _e663 {
        let _e666 = n0_.v2_[_e119];
        let _e671 = ((unpack4x8unorm(_e666) * (1f - _e636.w)) + _e636);
        if (_e659.w != 0f) {
            phi_6036_ = (1f / _e659.w);
        } else {
            phi_6036_ = 0f;
        }
        let _e677 = phi_6036_;
        let _e678 = (_e659.xyz * _e677);
        let _e679 = K1_1;
        local_2 = _e678;
        let _e680 = _e671.xyz;
        if (_e671.w != 0f) {
            phi_6037_ = (1f / _e671.w);
        } else {
            phi_6037_ = 0f;
        }
        let _e685 = phi_6037_;
        let _e686 = (_e680 * _e685);
        local = _e686;
        switch bitcast<i32>(_e679) {
            case 11: {
                let _e688 = local_2;
                local_1 = (_e688 * _e686);
                break;
            }
            case 1: {
                let _e690 = local_2;
                local_1 = ((_e690 + _e686) - (_e690 * _e686));
                break;
            }
            case 2: {
                let _e694 = local_2;
                let _e695 = (_e694 * _e686);
                local_1 = (select(_e695, (((_e694 + _e686) - _e695) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e686 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e702 = local_2;
                local_1 = min(_e702, _e686);
                break;
            }
            case 4: {
                let _e704 = local_2;
                local_1 = max(_e704, _e686);
                break;
            }
            case 5: {
                let _e707 = clamp(_e680, vec3<f32>(0f, 0f, 0f), _e671.www);
                let _e713 = vec4<f32>(_e707.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e719 = vec4<f32>(_e713.x, _e707.y, _e713.z, _e713.w);
                let _e726 = local_2;
                let _e729 = (clamp((vec3<f32>(1f, 1f, 1f) - _e726), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e671.w);
                let _e730 = vec4<f32>(_e719.x, _e719.y, _e707.z, _e719.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e730 / _e729)), sign(_e730), (_e729 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e736 = local_2;
                local_2 = clamp(_e736, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e739 = clamp(_e680, vec3<f32>(0f, 0f, 0f), _e671.www);
                let _e745 = vec4<f32>(_e739.x, _e671.y, _e671.z, _e671.w);
                let _e751 = vec4<f32>(_e745.x, _e739.y, _e745.z, _e745.w);
                phi_7066_ = vec4<f32>(_e751.x, _e751.y, _e739.z, _e751.w);
                if (_e671.w == 0f) {
                    phi_7066_ = vec4<f32>(_e739.x, _e739.y, _e739.z, 1f);
                }
                let _e761 = phi_7066_;
                let _e765 = (vec3(_e761.w) - _e761.xyz);
                let _e766 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e765 / (_e766 * _e761.w))), sign(_e765), (_e766 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e774 = local_2;
                let _e775 = (_e774 * _e686);
                local_1 = (select(_e775, (((_e774 + _e686) - _e775) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e774 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_6934_ = 0i;
                loop {
                    let _e783 = phi_6934_;
                    if (_e783 < 3i) {
                        let _e786 = local_2[_e783];
                        if (_e786 <= 0.5f) {
                            let _e789 = local[_e783];
                            local_1[_e783] = (1f - _e789);
                        } else {
                            let _e793 = local[_e783];
                            if (_e793 <= 0.25f) {
                                let _e795 = local[_e783];
                                let _e798 = local[_e783];
                                local_1[_e783] = ((((16f * _e795) - 12f) * _e798) + 3f);
                            } else {
                                let _e802 = local[_e783];
                                local_1[_e783] = (inverseSqrt(_e802) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_6934_ = (_e783 + 1i);
                    }
                }
                let _e807 = local_2;
                let _e811 = local_1;
                local_1 = (_e686 + ((_e686 * ((_e807 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e811));
                break;
            }
            case 9: {
                let _e814 = local_2;
                local_1 = abs((_e686 - _e814));
                break;
            }
            case 10: {
                let _e817 = local_2;
                local_1 = ((_e817 + _e686) - ((_e817 * 2f) * _e686));
                break;
            }
            case 12: {
                if ej {
                    let _e822 = local_2;
                    let _e823 = clamp(_e822, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e823;
                    let _e838 = (_e823 - vec3(min(min(_e823.x, _e823.y), _e823.z)));
                    let _e846 = (_e838 * ((max(max(_e686.x, _e686.y), _e686.z) - min(min(_e686.x, _e686.y), _e686.z)) / max(0.000062f, max(max(_e838.x, _e838.y), _e838.z))));
                    let _e847 = dot(_e686, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e850 = (_e846 - vec3(dot(_e846, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e863 = (vec2<f32>(_e847, (1f - _e847)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e850.x, _e850.y), _e850.z)), max(max(_e850.x, _e850.y), _e850.z))));
                    local_1 = ((_e850 * min(1f, min(_e863.x, _e863.y))) + vec3(_e847));
                }
                break;
            }
            case 13: {
                if ej {
                    let _e871 = local_2;
                    let _e872 = clamp(_e871, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e872;
                    let _e887 = (_e686 - vec3(min(min(_e686.x, _e686.y), _e686.z)));
                    let _e895 = (_e887 * ((max(max(_e872.x, _e872.y), _e872.z) - min(min(_e872.x, _e872.y), _e872.z)) / max(0.000062f, max(max(_e887.x, _e887.y), _e887.z))));
                    let _e896 = dot(_e686, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e899 = (_e895 - vec3(dot(_e895, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e912 = (vec2<f32>(_e896, (1f - _e896)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e899.x, _e899.y), _e899.z)), max(max(_e899.x, _e899.y), _e899.z))));
                    local_1 = ((_e899 * min(1f, min(_e912.x, _e912.y))) + vec3(_e896));
                }
                break;
            }
            case 14: {
                if ej {
                    let _e920 = local_2;
                    let _e921 = clamp(_e920, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e921;
                    let _e922 = dot(_e686, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e925 = (_e921 - vec3(dot(_e921, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e938 = (vec2<f32>(_e922, (1f - _e922)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e925.x, _e925.y), _e925.z)), max(max(_e925.x, _e925.y), _e925.z))));
                    local_1 = ((_e925 * min(1f, min(_e938.x, _e938.y))) + vec3(_e922));
                }
                break;
            }
            case 15: {
                if ej {
                    let _e946 = local_2;
                    let _e947 = clamp(_e946, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e947;
                    let _e948 = dot(_e947, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e951 = (_e686 - vec3(dot(_e686, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e964 = (vec2<f32>(_e948, (1f - _e948)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e951.x, _e951.y), _e951.z)), max(max(_e951.x, _e951.y), _e951.z))));
                    local_1 = ((_e951 * min(1f, min(_e964.x, _e964.y))) + vec3(_e948));
                }
                break;
            }
            default: {
            }
        }
        let _e972 = local_1;
        let _e975 = (mix(_e678, _e972, vec3(_e671.w)) * _e659.w);
        let _e981 = vec4<f32>(_e975.x, _e659.y, _e659.z, _e659.w);
        let _e987 = vec4<f32>(_e981.x, _e975.y, _e981.z, _e981.w);
        phi_7391_ = vec4<f32>(_e987.x, _e987.y, _e975.z, _e987.w);
    }
    let _e995 = phi_7391_;
    let _e996 = (_e995 * _e657);
    let _e1000 = ((_e636 * (1f - _e996.w)) + _e996);
    let _e1001 = _e1000.xyz;
    let _e1004 = j.E3_;
    let _e1006 = j.F3_;
    if (fj && (_e1000.w != 0f)) {
        phi_7404_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e84.x) + (0.00583715f * _e84.y))))) * _e1004) + _e1006)) + _e1001);
    } else {
        phi_7404_ = _e1001;
    }
    let _e1022 = phi_7404_;
    let _e1028 = vec4<f32>(_e1022.x, _e1000.y, _e1000.z, _e1000.w);
    let _e1034 = vec4<f32>(_e1028.x, _e1022.y, _e1028.z, _e1028.w);
    let _e1040 = vec4<f32>(_e1034.x, _e1034.y, _e1022.z, _e1034.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1022.x + _e1022.y) + _e1022.z) + _e1000.w) == 0f) {
                break;
            }
            let _e1046 = (1f - _e1000.w);
            phi_7406_ = _e1040;
            if (_e1046 != 0f) {
                let _e1050 = n0_.v2_[_e119];
                phi_7406_ = (_e1040 + (unpack4x8unorm(_e1050) * _e1046));
            }
            let _e1055 = phi_7406_;
            n0_.v2_[_e119] = pack4x8unorm(_e1055);
            break;
        }
    }
    if (_e287 != 0u) {
        m0_.v2_[_e119] = _e287;
    }
    P4_.v2_[_e119] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) m2_: vec2<f32>, @location(1) W0_: vec4<f32>, @location(4) @interpolate(flat, either) R3_: u32, @location(3) @interpolate(flat, either) U1_: vec4<f32>, @location(5) @interpolate(flat, either) K1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    m2_1 = m2_;
    W0_1 = W0_;
    R3_1 = R3_;
    U1_1 = U1_;
    K1_1 = K1_;
    main_1();
}
