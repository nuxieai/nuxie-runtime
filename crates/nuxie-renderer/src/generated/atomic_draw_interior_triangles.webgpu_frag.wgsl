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
var<private> n1_1: f32;
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
    var phi_3669_: u32;
    var phi_1554_: bool;
    var phi_3674_: f32;
    var phi_3673_: f32;
    var phi_3675_: f32;
    var phi_3678_: f32;
    var phi_3677_: f32;
    var phi_1591_: bool;
    var phi_3680_: f32;
    var phi_4325_: u32;
    var phi_3679_: f32;
    var phi_3702_: vec4<f32>;
    var phi_4324_: u32;
    var phi_3700_: vec4<f32>;
    var phi_3707_: f32;
    var phi_4161_: vec4<f32>;
    var phi_4101_: i32;
    var phi_4309_: vec4<f32>;
    var phi_4322_: vec4<f32>;
    var phi_4353_: u32;
    var phi_4347_: vec4<f32>;
    var phi_4348_: vec3<f32>;
    var phi_4350_: vec4<f32>;

    let _e81 = gl_FragCoord_1;
    let _e82 = _e81.xy;
    let _e85 = bitcast<vec2<u32>>(vec2<i32>(floor(_e82)));
    let _e87 = j.P6_;
    let _e116 = bitcast<i32>((((((_e85.y >> bitcast<u32>(5u)) * (((_e87 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e85.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e85.x & 28u) << bitcast<u32>(5u)) + ((_e85.y & 28u) << bitcast<u32>(2i)))) + (((_e85.y & 3u) << bitcast<u32>(2i)) + (_e85.x & 3u))));
    let _e119 = R4_.r2_[_e116];
    let _e121 = (_e119 >> bitcast<u32>(17u));
    let _e122 = G0_1;
    if (_e121 == _e122) {
        phi_3669_ = _e119;
    } else {
        phi_3669_ = ((_e122 << bitcast<u32>(17u)) + 65536u);
    }
    let _e128 = phi_3669_;
    let _e129 = n1_1;
    R4_.r2_[_e116] = (_e128 + bitcast<u32>(i32(round((_e129 * 2048f)))));
    phi_4353_ = 0u;
    phi_4347_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e121 != _e122) {
        let _e139 = ((f32((_e119 & 131071u)) * 0.00048828125f) + -32f);
        let _e142 = VC.r2_[_e121];
        phi_3673_ = _e139;
        if ((_e142.x & 768u) != 0u) {
            let _e146 = abs(_e139);
            phi_1554_ = aj;
            if aj {
                phi_1554_ = ((_e142.x & 512u) != 0u);
            }
            let _e150 = phi_1554_;
            phi_3674_ = _e146;
            if _e150 {
                phi_3674_ = (1f - abs(((fract((_e146 * 0.5f)) * 2f) + -1f)));
            }
            let _e158 = phi_3674_;
            phi_3673_ = _e158;
        }
        let _e160 = phi_3673_;
        let _e161 = clamp(_e160, 0f, 1f);
        phi_3677_ = _e161;
        if Wi {
            let _e163 = (_e142.x >> bitcast<u32>(16u));
            phi_3678_ = _e161;
            if (_e163 != 0u) {
                let _e167 = m0_.r2_[_e116];
                if (_e163 == (_e167 >> bitcast<u32>(16i))) {
                    phi_3675_ = min(_e161, unpack2x16float(_e167).x);
                } else {
                    phi_3675_ = 0f;
                }
                let _e175 = phi_3675_;
                phi_3678_ = _e175;
            }
            let _e177 = phi_3678_;
            phi_3677_ = _e177;
        }
        let _e179 = phi_3677_;
        phi_1591_ = Xi;
        if Xi {
            phi_1591_ = ((_e142.x & 1024u) != 0u);
        }
        let _e183 = phi_1591_;
        phi_3680_ = _e179;
        if _e183 {
            let _e184 = (_e121 * 8u);
            let _e188 = JB.r2_[(_e184 + 2u)];
            let _e199 = JB.r2_[(_e184 + 3u)];
            let _e204 = _e199.zw;
            let _e206 = ((abs(((mat2x2<f32>(vec2<f32>(_e188.x, _e188.y), vec2<f32>(_e188.z, _e188.w)) * _e82) + _e199.xy)) * _e204) - _e204);
            phi_3680_ = min(_e179, clamp((min(_e206.x, _e206.y) + 0.5f), 0f, 1f));
        }
        let _e214 = phi_3680_;
        let _e215 = (_e142.x & 15u);
        let _e218 = ((_e142.x >> bitcast<u32>(4i)) & 15u);
        let _e220 = (Yi && (_e218 != 0u));
        if (_e215 <= 1u) {
            let _e225 = (Wi && (_e215 == 0u));
            phi_4325_ = 0u;
            if _e225 {
                phi_4325_ = (_e142.y | pack2x16float(vec2<f32>(_e214, 0f)));
            }
            let _e230 = phi_4325_;
            phi_4324_ = _e230;
            phi_3700_ = select(unpack4x8unorm(_e142.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e225));
        } else {
            let _e233 = (_e121 * 8u);
            let _e236 = JB.r2_[_e233];
            let _e247 = JB.r2_[(_e233 + 1u)];
            let _e250 = ((mat2x2<f32>(vec2<f32>(_e236.x, _e236.y), vec2<f32>(_e236.z, _e236.w)) * _e82) + _e247.xy);
            let _e256 = j.L8_;
            let _e258 = j.M8_;
            if (f32(_e215) == 2f) {
                phi_3679_ = _e250.x;
            } else {
                phi_3679_ = length(_e250);
            }
            let _e268 = phi_3679_;
            let _e274 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e268, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e247.z < 0f))) + ((max(0f, _e247.z) * 0.001953125f) + 0.0009765625f)), ((_e247.w * _e256) + _e258)), 0f);
            phi_3702_ = _e274;
            if !(_e220) {
                let _e278 = (_e274.xyz * _e274.w);
                phi_3702_ = vec4<f32>(_e278.x, _e278.y, _e278.z, (_e274.w * abs(bitcast<f32>(_e142.y))));
            }
            let _e288 = phi_3702_;
            phi_4324_ = 0u;
            phi_3700_ = _e288;
        }
        let _e290 = phi_4324_;
        let _e292 = phi_3700_;
        phi_4322_ = _e292;
        if _e220 {
            phi_4309_ = _e292;
            if ((_e292.w * _e214) != 0f) {
                let _e298 = n0_.r2_[_e116];
                let _e299 = unpack4x8unorm(_e298);
                let _e300 = _e292.xyz;
                local_2 = _e300;
                let _e301 = _e299.xyz;
                if (_e299.w != 0f) {
                    phi_3707_ = (1f / _e299.w);
                } else {
                    phi_3707_ = 0f;
                }
                let _e306 = phi_3707_;
                let _e307 = (_e301 * _e306);
                local = _e307;
                switch bitcast<i32>(_e218) {
                    case 11: {
                        let _e309 = local_2;
                        local_1 = (_e309 * _e307);
                        break;
                    }
                    case 1: {
                        let _e311 = local_2;
                        local_1 = ((_e311 + _e307) - (_e311 * _e307));
                        break;
                    }
                    case 2: {
                        let _e315 = local_2;
                        let _e316 = (_e315 * _e307);
                        local_1 = (select(_e316, (((_e315 + _e307) - _e316) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e307 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e323 = local_2;
                        local_1 = min(_e323, _e307);
                        break;
                    }
                    case 4: {
                        let _e325 = local_2;
                        local_1 = max(_e325, _e307);
                        break;
                    }
                    case 5: {
                        let _e328 = clamp(_e301, vec3<f32>(0f, 0f, 0f), _e299.www);
                        let _e334 = vec4<f32>(_e328.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e340 = vec4<f32>(_e334.x, _e328.y, _e334.z, _e334.w);
                        let _e347 = local_2;
                        let _e350 = (clamp((vec3<f32>(1f, 1f, 1f) - _e347), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e299.w);
                        let _e351 = vec4<f32>(_e340.x, _e340.y, _e328.z, _e340.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e351 / _e350)), sign(_e351), (_e350 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e357 = local_2;
                        local_2 = clamp(_e357, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e360 = clamp(_e301, vec3<f32>(0f, 0f, 0f), _e299.www);
                        let _e366 = vec4<f32>(_e360.x, _e299.y, _e299.z, _e299.w);
                        let _e372 = vec4<f32>(_e366.x, _e360.y, _e366.z, _e366.w);
                        phi_4161_ = vec4<f32>(_e372.x, _e372.y, _e360.z, _e372.w);
                        if (_e299.w == 0f) {
                            phi_4161_ = vec4<f32>(_e360.x, _e360.y, _e360.z, 1f);
                        }
                        let _e382 = phi_4161_;
                        let _e386 = (vec3(_e382.w) - _e382.xyz);
                        let _e387 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e386 / (_e387 * _e382.w))), sign(_e386), (_e387 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e395 = local_2;
                        let _e396 = (_e395 * _e307);
                        local_1 = (select(_e396, (((_e395 + _e307) - _e396) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e395 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4101_ = 0i;
                        loop {
                            let _e404 = phi_4101_;
                            if (_e404 < 3i) {
                                let _e407 = local_2[_e404];
                                if (_e407 <= 0.5f) {
                                    let _e410 = local[_e404];
                                    local_1[_e404] = (1f - _e410);
                                } else {
                                    let _e414 = local[_e404];
                                    if (_e414 <= 0.25f) {
                                        let _e416 = local[_e404];
                                        let _e419 = local[_e404];
                                        local_1[_e404] = ((((16f * _e416) - 12f) * _e419) + 3f);
                                    } else {
                                        let _e423 = local[_e404];
                                        local_1[_e404] = (inverseSqrt(_e423) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4101_ = (_e404 + 1i);
                            }
                        }
                        let _e428 = local_2;
                        let _e432 = local_1;
                        local_1 = (_e307 + ((_e307 * ((_e428 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e432));
                        break;
                    }
                    case 9: {
                        let _e435 = local_2;
                        local_1 = abs((_e307 - _e435));
                        break;
                    }
                    case 10: {
                        let _e438 = local_2;
                        local_1 = ((_e438 + _e307) - ((_e438 * 2f) * _e307));
                        break;
                    }
                    case 12: {
                        if cj {
                            let _e443 = local_2;
                            let _e444 = clamp(_e443, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e444;
                            let _e459 = (_e444 - vec3(min(min(_e444.x, _e444.y), _e444.z)));
                            let _e467 = (_e459 * ((max(max(_e307.x, _e307.y), _e307.z) - min(min(_e307.x, _e307.y), _e307.z)) / max(0.000062f, max(max(_e459.x, _e459.y), _e459.z))));
                            let _e468 = dot(_e307, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e471 = (_e467 - vec3(dot(_e467, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e484 = (vec2<f32>(_e468, (1f - _e468)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e471.x, _e471.y), _e471.z)), max(max(_e471.x, _e471.y), _e471.z))));
                            local_1 = ((_e471 * min(1f, min(_e484.x, _e484.y))) + vec3(_e468));
                        }
                        break;
                    }
                    case 13: {
                        if cj {
                            let _e492 = local_2;
                            let _e493 = clamp(_e492, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e493;
                            let _e508 = (_e307 - vec3(min(min(_e307.x, _e307.y), _e307.z)));
                            let _e516 = (_e508 * ((max(max(_e493.x, _e493.y), _e493.z) - min(min(_e493.x, _e493.y), _e493.z)) / max(0.000062f, max(max(_e508.x, _e508.y), _e508.z))));
                            let _e517 = dot(_e307, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e520 = (_e516 - vec3(dot(_e516, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e533 = (vec2<f32>(_e517, (1f - _e517)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e520.x, _e520.y), _e520.z)), max(max(_e520.x, _e520.y), _e520.z))));
                            local_1 = ((_e520 * min(1f, min(_e533.x, _e533.y))) + vec3(_e517));
                        }
                        break;
                    }
                    case 14: {
                        if cj {
                            let _e541 = local_2;
                            let _e542 = clamp(_e541, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e542;
                            let _e543 = dot(_e307, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e546 = (_e542 - vec3(dot(_e542, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e559 = (vec2<f32>(_e543, (1f - _e543)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e546.x, _e546.y), _e546.z)), max(max(_e546.x, _e546.y), _e546.z))));
                            local_1 = ((_e546 * min(1f, min(_e559.x, _e559.y))) + vec3(_e543));
                        }
                        break;
                    }
                    case 15: {
                        if cj {
                            let _e567 = local_2;
                            let _e568 = clamp(_e567, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e568;
                            let _e569 = dot(_e568, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e572 = (_e307 - vec3(dot(_e307, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e585 = (vec2<f32>(_e569, (1f - _e569)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e572.x, _e572.y), _e572.z)), max(max(_e572.x, _e572.y), _e572.z))));
                            local_1 = ((_e572 * min(1f, min(_e585.x, _e585.y))) + vec3(_e569));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e593 = local_1;
                let _e595 = mix(_e300, _e593, vec3(_e299.w));
                let _e601 = vec4<f32>(_e595.x, _e292.y, _e292.z, _e292.w);
                let _e607 = vec4<f32>(_e601.x, _e595.y, _e601.z, _e601.w);
                phi_4309_ = vec4<f32>(_e607.x, _e607.y, _e595.z, _e607.w);
            }
            let _e615 = phi_4309_;
            let _e618 = (_e615.xyz * _e615.w);
            let _e624 = vec4<f32>(_e618.x, _e615.y, _e615.z, _e615.w);
            let _e630 = vec4<f32>(_e624.x, _e618.y, _e624.z, _e624.w);
            phi_4322_ = vec4<f32>(_e630.x, _e630.y, _e618.z, _e630.w);
        }
        let _e638 = phi_4322_;
        phi_4353_ = _e290;
        phi_4347_ = (_e638 * _e214);
    }
    let _e641 = phi_4353_;
    let _e643 = phi_4347_;
    let _e644 = _e643.xyz;
    let _e647 = j.F3_;
    let _e649 = j.G3_;
    if (dj && (_e643.w != 0f)) {
        phi_4348_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e81.x) + (0.00583715f * _e81.y))))) * _e647) + _e649)) + _e644);
    } else {
        phi_4348_ = _e644;
    }
    let _e665 = phi_4348_;
    let _e671 = vec4<f32>(_e665.x, _e643.y, _e643.z, _e643.w);
    let _e677 = vec4<f32>(_e671.x, _e665.y, _e671.z, _e671.w);
    let _e683 = vec4<f32>(_e677.x, _e677.y, _e665.z, _e677.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e665.x + _e665.y) + _e665.z) + _e643.w) == 0f) {
                break;
            }
            let _e689 = (1f - _e643.w);
            phi_4350_ = _e683;
            if (_e689 != 0f) {
                let _e693 = n0_.r2_[_e116];
                phi_4350_ = (_e683 + (unpack4x8unorm(_e693) * _e689));
            }
            let _e698 = phi_4350_;
            n0_.r2_[_e116] = pack4x8unorm(_e698);
            break;
        }
    }
    if (_e641 != 0u) {
        m0_.r2_[_e116] = _e641;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) @interpolate(flat, either) n1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    n1_1 = n1_;
    main_1();
}
