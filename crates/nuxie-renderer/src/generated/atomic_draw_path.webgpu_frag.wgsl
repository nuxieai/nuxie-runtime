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

struct R4Pe_1 {
    r2_: array<atomic<u32>>,
}

@id(7) override ej: bool = true;
@id(6) override dj: bool = true;
@id(4) override bj: bool = true;
@id(0) override Xi: bool = true;
@id(1) override Yi: bool = true;
@id(2) override Zi: bool = true;
@id(3) override aj: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;
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
var<private> S_1: vec4<f32>;
var<private> G0_1: u32;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe_1;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1582_: bool;
    var phi_1595_: bool;
    var phi_4294_: f32;
    var phi_4302_: f32;
    var phi_4310_: f32;
    var phi_4309_: f32;
    var phi_2100_: bool;
    var phi_4313_: f32;
    var phi_4312_: f32;
    var phi_4314_: f32;
    var phi_4317_: f32;
    var phi_4316_: f32;
    var phi_2137_: bool;
    var phi_4321_: f32;
    var phi_4318_: f32;
    var phi_4319_: f32;
    var phi_4320_: vec2<f32>;
    var phi_4353_: vec4<f32>;
    var phi_5265_: u32;
    var phi_5264_: u32;
    var phi_4351_: vec4<f32>;
    var phi_4360_: f32;
    var phi_4950_: i32;
    var phi_5038_: vec4<f32>;
    var phi_5249_: vec4<f32>;
    var phi_5262_: vec4<f32>;
    var phi_5296_: u32;
    var phi_5289_: vec4<f32>;
    var phi_5291_: vec3<f32>;
    var phi_5293_: vec4<f32>;

    let _e99 = gl_FragCoord_1;
    let _e100 = _e99.xy;
    let _e103 = bitcast<vec2<u32>>(vec2<i32>(floor(_e100)));
    let _e105 = j.P6_;
    let _e134 = bitcast<i32>((((((_e103.y >> bitcast<u32>(5u)) * (((_e105 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e103.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e103.x & 28u) << bitcast<u32>(5u)) + ((_e103.y & 28u) << bitcast<u32>(2i)))) + (((_e103.y & 3u) << bitcast<u32>(2i)) + (_e103.x & 3u))));
    phi_1582_ = aj;
    if aj {
        let _e135 = S_1;
        phi_1582_ = (_e135.x < -1.5f);
    }
    let _e139 = phi_1582_;
    if _e139 {
        let _e212 = S_1;
        let _e216 = textureSampleLevel(YC, ab, vec2<f32>((3f + _e212.x), 0f), 0f);
        let _e222 = textureSampleLevel(YC, ab, vec2<f32>((1f - _e212.y), 0f), 0f);
        phi_4309_ = ((1f - _e216.x) - _e222.x);
    } else {
        phi_1595_ = aj;
        if aj {
            let _e140 = S_1;
            phi_1595_ = (_e140.y < -1.5f);
        }
        let _e144 = phi_1595_;
        if _e144 {
            let _e152 = S_1;
            let _e155 = max(_e152.w, 0f);
            if (_e152.z >= 0f) {
                let _e158 = textureSampleLevel(YC, ab, vec2<f32>(_e155, 0f), 0f);
                phi_4294_ = _e158.x;
            } else {
                phi_4294_ = 0f;
            }
            let _e161 = phi_4294_;
            phi_4302_ = _e161;
            if (abs(_e152.z) < 1000f) {
                let _e168 = (-2f - _e152.y);
                let _e170 = ((_e168 - _e155) * 0.5984134f);
                let _e173 = (vec4(_e155) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e170));
                let _e179 = ((_e173 * -(_e152.z)) + vec4(((_e168 * _e152.z) + (abs(_e152.x) - 0.25f))));
                let _e182 = textureSampleLevel(YC, ab, vec2<f32>(_e179.x, 0f), 0f);
                let _e185 = textureSampleLevel(YC, ab, vec2<f32>(_e179.y, 0f), 0f);
                let _e188 = textureSampleLevel(YC, ab, vec2<f32>(_e179.z, 0f), 0f);
                let _e191 = textureSampleLevel(YC, ab, vec2<f32>(_e179.w, 0f), 0f);
                let _e197 = (_e173 * 5.0959306f);
                phi_4302_ = (_e161 + (dot(vec4<f32>(_e182.x, _e185.x, _e188.x, _e191.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e197) * (_e197 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e170));
            }
            let _e206 = phi_4302_;
            phi_4310_ = (_e206 * sign(_e152.x));
        } else {
            let _e146 = S_1[0u];
            let _e148 = S_1[1u];
            phi_4310_ = min(min(_e146, abs(_e148)), 1f);
        }
        let _e211 = phi_4310_;
        phi_4309_ = _e211;
    }
    let _e226 = phi_4309_;
    let _e230 = u32(round(((_e226 * 2048f) + 65536f)));
    let _e231 = G0_1;
    let _e234 = ((_e231 << bitcast<u32>(17u)) | _e230);
    let _e237 = atomicMax((&R4_.r2_[_e134]), _e234);
    let _e239 = (_e237 >> bitcast<u32>(17u));
    if (_e239 == _e231) {
        let _e764 = S_1;
        if (_e764.y < 0f) {
            let _e771 = atomicAdd((&R4_.r2_[_e134]), ((_e230 + (_e237 - max(_e234, _e237))) - 65536u));
        }
        phi_5296_ = 0u;
        phi_5289_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e244 = ((f32((_e237 & 131071u)) * 0.00048828125f) + -32f);
        let _e247 = VC.r2_[_e239];
        phi_4312_ = _e244;
        if ((_e247.x & 768u) != 0u) {
            let _e251 = abs(_e244);
            phi_2100_ = bj;
            if bj {
                phi_2100_ = ((_e247.x & 512u) != 0u);
            }
            let _e255 = phi_2100_;
            phi_4313_ = _e251;
            if _e255 {
                phi_4313_ = (1f - abs(((fract((_e251 * 0.5f)) * 2f) + -1f)));
            }
            let _e263 = phi_4313_;
            phi_4312_ = _e263;
        }
        let _e265 = phi_4312_;
        let _e266 = clamp(_e265, 0f, 1f);
        phi_4316_ = _e266;
        if Xi {
            let _e268 = (_e247.x >> bitcast<u32>(16u));
            phi_4317_ = _e266;
            if (_e268 != 0u) {
                let _e272 = m0_.r2_[_e134];
                if (_e268 == (_e272 >> bitcast<u32>(16i))) {
                    phi_4314_ = min(_e266, unpack2x16float(_e272).x);
                } else {
                    phi_4314_ = 0f;
                }
                let _e280 = phi_4314_;
                phi_4317_ = _e280;
            }
            let _e282 = phi_4317_;
            phi_4316_ = _e282;
        }
        let _e284 = phi_4316_;
        phi_2137_ = Yi;
        if Yi {
            phi_2137_ = ((_e247.x & 1024u) != 0u);
        }
        let _e288 = phi_2137_;
        phi_4321_ = _e284;
        if _e288 {
            let _e289 = (_e239 * 8u);
            let _e293 = JB.r2_[(_e289 + 2u)];
            let _e304 = JB.r2_[(_e289 + 3u)];
            let _e309 = _e304.zw;
            let _e311 = ((abs(((mat2x2<f32>(vec2<f32>(_e293.x, _e293.y), vec2<f32>(_e293.z, _e293.w)) * _e100) + _e304.xy)) * _e309) - _e309);
            phi_4321_ = min(_e284, clamp((min(_e311.x, _e311.y) + 0.5f), 0f, 1f));
        }
        let _e319 = phi_4321_;
        let _e320 = (_e247.x & 15u);
        let _e323 = ((_e247.x >> bitcast<u32>(4i)) & 15u);
        let _e325 = (Zi && (_e323 != 0u));
        if (_e320 <= 1u) {
            let _e405 = (Xi && (_e320 == 0u));
            phi_5265_ = 0u;
            if _e405 {
                phi_5265_ = (_e247.y | pack2x16float(vec2<f32>(_e319, 0f)));
            }
            let _e410 = phi_5265_;
            phi_5264_ = _e410;
            phi_4351_ = select(unpack4x8unorm(_e247.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e405));
        } else {
            let _e327 = (_e239 * 8u);
            let _e330 = JB.r2_[_e327];
            let _e341 = JB.r2_[(_e327 + 1u)];
            let _e344 = ((mat2x2<f32>(vec2<f32>(_e330.x, _e330.y), vec2<f32>(_e330.z, _e330.w)) * _e100) + _e341.xy);
            let _e348 = f32(_e320);
            let _e350 = j.M8_;
            let _e352 = j.N8_;
            let _e357 = ((_e341.w * _e350) + _e352);
            switch bitcast<i32>(0u) {
                default: {
                    if (_e348 == 4f) {
                        let _e374 = normalize(vec2<f32>(_e344.x, -(_e344.y)));
                        let _e377 = acos(clamp(_e374.x, -1f, 1f));
                        if (_e374.y >= 0f) {
                            phi_4319_ = _e377;
                        } else {
                            phi_4319_ = -(_e377);
                        }
                        let _e382 = phi_4319_;
                        phi_4320_ = vec2<f32>((_e382 * 0.15915507f), _e357);
                        break;
                    } else {
                        if (_e348 == 2f) {
                            phi_4318_ = _e344.x;
                        } else {
                            phi_4318_ = length(_e344);
                        }
                        let _e364 = phi_4318_;
                        phi_4320_ = vec2<f32>(((clamp(_e364, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e341.z < 0f))) + ((max(0f, _e341.z) * 0.001953125f) + 0.0009765625f)), _e357);
                        break;
                    }
                }
            }
            let _e386 = phi_4320_;
            let _e387 = textureSampleLevel(XC, O8_, _e386, 0f);
            phi_4353_ = _e387;
            if !(_e325) {
                let _e391 = (_e387.xyz * _e387.w);
                phi_4353_ = vec4<f32>(_e391.x, _e391.y, _e391.z, (_e387.w * abs(bitcast<f32>(_e247.y))));
            }
            let _e401 = phi_4353_;
            phi_5264_ = 0u;
            phi_4351_ = _e401;
        }
        let _e414 = phi_5264_;
        let _e416 = phi_4351_;
        phi_5262_ = _e416;
        if _e325 {
            phi_5249_ = _e416;
            if ((_e416.w * _e319) != 0f) {
                let _e422 = n0_.r2_[_e134];
                let _e423 = unpack4x8unorm(_e422);
                let _e424 = _e416.xyz;
                local_2 = _e424;
                let _e425 = _e423.xyz;
                if (_e423.w != 0f) {
                    phi_4360_ = (1f / _e423.w);
                } else {
                    phi_4360_ = 0f;
                }
                let _e430 = phi_4360_;
                let _e431 = (_e425 * _e430);
                local = _e431;
                switch bitcast<i32>(_e323) {
                    case 11: {
                        let _e715 = local_2;
                        local_1 = (_e715 * _e431);
                        break;
                    }
                    case 1: {
                        let _e711 = local_2;
                        local_1 = ((_e711 + _e431) - (_e711 * _e431));
                        break;
                    }
                    case 2: {
                        let _e703 = local_2;
                        let _e704 = (_e703 * _e431);
                        local_1 = (select(_e704, (((_e703 + _e431) - _e704) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e431 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e701 = local_2;
                        local_1 = min(_e701, _e431);
                        break;
                    }
                    case 4: {
                        let _e699 = local_2;
                        local_1 = max(_e699, _e431);
                        break;
                    }
                    case 5: {
                        let _e670 = clamp(_e425, vec3<f32>(0f, 0f, 0f), _e423.www);
                        let _e676 = vec4<f32>(_e670.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e682 = vec4<f32>(_e676.x, _e670.y, _e676.z, _e676.w);
                        let _e689 = local_2;
                        let _e692 = (clamp((vec3<f32>(1f, 1f, 1f) - _e689), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e423.w);
                        let _e693 = vec4<f32>(_e682.x, _e682.y, _e670.z, _e682.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e693 / _e692)), sign(_e693), (_e692 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e631 = local_2;
                        local_2 = clamp(_e631, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e634 = clamp(_e425, vec3<f32>(0f, 0f, 0f), _e423.www);
                        let _e640 = vec4<f32>(_e634.x, _e423.y, _e423.z, _e423.w);
                        let _e646 = vec4<f32>(_e640.x, _e634.y, _e640.z, _e640.w);
                        phi_5038_ = vec4<f32>(_e646.x, _e646.y, _e634.z, _e646.w);
                        if (_e423.w == 0f) {
                            phi_5038_ = vec4<f32>(_e634.x, _e634.y, _e634.z, 1f);
                        }
                        let _e656 = phi_5038_;
                        let _e660 = (vec3(_e656.w) - _e656.xyz);
                        let _e661 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e660 / (_e661 * _e656.w))), sign(_e660), (_e661 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e623 = local_2;
                        let _e624 = (_e623 * _e431);
                        local_1 = (select(_e624, (((_e623 + _e431) - _e624) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e623 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4950_ = 0i;
                        loop {
                            let _e592 = phi_4950_;
                            if (_e592 < 3i) {
                                let _e595 = local_2[_e592];
                                if (_e595 <= 0.5f) {
                                    let _e612 = local[_e592];
                                    local_1[_e592] = (1f - _e612);
                                } else {
                                    let _e598 = local[_e592];
                                    if (_e598 <= 0.25f) {
                                        let _e604 = local[_e592];
                                        let _e607 = local[_e592];
                                        local_1[_e592] = ((((16f * _e604) - 12f) * _e607) + 3f);
                                    } else {
                                        let _e600 = local[_e592];
                                        local_1[_e592] = (inverseSqrt(_e600) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4950_ = (_e592 + 1i);
                            }
                        }
                        let _e616 = local_2;
                        let _e620 = local_1;
                        local_1 = (_e431 + ((_e431 * ((_e616 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e620));
                        break;
                    }
                    case 9: {
                        let _e588 = local_2;
                        local_1 = abs((_e431 - _e588));
                        break;
                    }
                    case 10: {
                        let _e583 = local_2;
                        local_1 = ((_e583 + _e431) - ((_e583 * 2f) * _e431));
                        break;
                    }
                    case 12: {
                        if dj {
                            let _e534 = local_2;
                            let _e535 = clamp(_e534, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e535;
                            let _e550 = (_e535 - vec3(min(min(_e535.x, _e535.y), _e535.z)));
                            let _e558 = (_e550 * ((max(max(_e431.x, _e431.y), _e431.z) - min(min(_e431.x, _e431.y), _e431.z)) / max(0.000062f, max(max(_e550.x, _e550.y), _e550.z))));
                            let _e559 = dot(_e431, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e562 = (_e558 - vec3(dot(_e558, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e575 = (vec2<f32>(_e559, (1f - _e559)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e562.x, _e562.y), _e562.z)), max(max(_e562.x, _e562.y), _e562.z))));
                            local_1 = ((_e562 * min(1f, min(_e575.x, _e575.y))) + vec3(_e559));
                        }
                        break;
                    }
                    case 13: {
                        if dj {
                            let _e485 = local_2;
                            let _e486 = clamp(_e485, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e486;
                            let _e501 = (_e431 - vec3(min(min(_e431.x, _e431.y), _e431.z)));
                            let _e509 = (_e501 * ((max(max(_e486.x, _e486.y), _e486.z) - min(min(_e486.x, _e486.y), _e486.z)) / max(0.000062f, max(max(_e501.x, _e501.y), _e501.z))));
                            let _e510 = dot(_e431, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e513 = (_e509 - vec3(dot(_e509, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e526 = (vec2<f32>(_e510, (1f - _e510)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e513.x, _e513.y), _e513.z)), max(max(_e513.x, _e513.y), _e513.z))));
                            local_1 = ((_e513 * min(1f, min(_e526.x, _e526.y))) + vec3(_e510));
                        }
                        break;
                    }
                    case 14: {
                        if dj {
                            let _e459 = local_2;
                            let _e460 = clamp(_e459, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e460;
                            let _e461 = dot(_e431, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e464 = (_e460 - vec3(dot(_e460, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e477 = (vec2<f32>(_e461, (1f - _e461)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e464.x, _e464.y), _e464.z)), max(max(_e464.x, _e464.y), _e464.z))));
                            local_1 = ((_e464 * min(1f, min(_e477.x, _e477.y))) + vec3(_e461));
                        }
                        break;
                    }
                    case 15: {
                        if dj {
                            let _e433 = local_2;
                            let _e434 = clamp(_e433, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e434;
                            let _e435 = dot(_e434, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e438 = (_e431 - vec3(dot(_e431, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e451 = (vec2<f32>(_e435, (1f - _e435)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e438.x, _e438.y), _e438.z)), max(max(_e438.x, _e438.y), _e438.z))));
                            local_1 = ((_e438 * min(1f, min(_e451.x, _e451.y))) + vec3(_e435));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e717 = local_1;
                let _e719 = mix(_e424, _e717, vec3(_e423.w));
                let _e725 = vec4<f32>(_e719.x, _e416.y, _e416.z, _e416.w);
                let _e731 = vec4<f32>(_e725.x, _e719.y, _e725.z, _e725.w);
                phi_5249_ = vec4<f32>(_e731.x, _e731.y, _e719.z, _e731.w);
            }
            let _e739 = phi_5249_;
            let _e742 = (_e739.xyz * _e739.w);
            let _e748 = vec4<f32>(_e742.x, _e739.y, _e739.z, _e739.w);
            let _e754 = vec4<f32>(_e748.x, _e742.y, _e748.z, _e748.w);
            phi_5262_ = vec4<f32>(_e754.x, _e754.y, _e742.z, _e754.w);
        }
        let _e762 = phi_5262_;
        phi_5296_ = _e414;
        phi_5289_ = (_e762 * _e319);
    }
    let _e773 = phi_5296_;
    let _e775 = phi_5289_;
    let _e776 = _e775.xyz;
    let _e779 = j.F3_;
    let _e781 = j.G3_;
    if (ej && (_e775.w != 0f)) {
        phi_5291_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e99.x) + (0.00583715f * _e99.y))))) * _e779) + _e781)) + _e776);
    } else {
        phi_5291_ = _e776;
    }
    let _e797 = phi_5291_;
    let _e803 = vec4<f32>(_e797.x, _e775.y, _e775.z, _e775.w);
    let _e809 = vec4<f32>(_e803.x, _e797.y, _e803.z, _e803.w);
    let _e815 = vec4<f32>(_e809.x, _e809.y, _e797.z, _e809.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e797.x + _e797.y) + _e797.z) + _e775.w) == 0f) {
                break;
            }
            let _e821 = (1f - _e775.w);
            phi_5293_ = _e815;
            if (_e821 != 0f) {
                let _e825 = n0_.r2_[_e134];
                phi_5293_ = (_e815 + (unpack4x8unorm(_e825) * _e821));
            }
            let _e830 = phi_5293_;
            n0_.r2_[_e134] = pack4x8unorm(_e830);
            break;
        }
    }
    if (_e773 != 0u) {
        m0_.r2_[_e134] = _e773;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) S: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    S_1 = S;
    G0_1 = G0_;
    main_1();
}
