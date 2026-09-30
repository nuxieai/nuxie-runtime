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

struct z4Td_1 {
    g2_: array<atomic<u32>>,
}

@id(7) override Lh: bool = true;
@id(6) override Kh: bool = true;
@id(4) override Ih: bool = true;
@id(0) override Eh: bool = true;
@id(1) override Fh: bool = true;
@id(2) override Gh: bool = true;
@id(3) override Hh: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ga: sampler;
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
var<private> M_1: vec4<f32>;
var<private> D0_1: u32;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td_1;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1443_: bool;
    var phi_1456_: bool;
    var phi_4017_: f32;
    var phi_4025_: f32;
    var phi_4033_: f32;
    var phi_4032_: f32;
    var phi_1950_: bool;
    var phi_4036_: f32;
    var phi_4035_: f32;
    var phi_4037_: f32;
    var phi_4040_: f32;
    var phi_4039_: f32;
    var phi_1987_: bool;
    var phi_4042_: f32;
    var phi_4902_: u32;
    var phi_4041_: f32;
    var phi_4074_: vec4<f32>;
    var phi_4901_: u32;
    var phi_4072_: vec4<f32>;
    var phi_4079_: f32;
    var phi_4693_: vec4<f32>;
    var phi_4613_: i32;
    var phi_4886_: vec4<f32>;
    var phi_4899_: vec4<f32>;
    var phi_4931_: u32;
    var phi_4924_: vec4<f32>;
    var phi_4926_: vec3<f32>;
    var phi_4928_: vec4<f32>;

    let _e92 = gl_FragCoord_1;
    let _e93 = _e92.xy;
    let _e96 = bitcast<vec2<u32>>(vec2<i32>(floor(_e93)));
    let _e98 = j.q6_;
    let _e127 = bitcast<i32>((((((_e96.y >> bitcast<u32>(5u)) * (((_e98 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e96.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e96.x & 28u) << bitcast<u32>(5u)) + ((_e96.y & 28u) << bitcast<u32>(2i)))) + (((_e96.y & 3u) << bitcast<u32>(2i)) + (_e96.x & 3u))));
    phi_1443_ = Hh;
    if Hh {
        let _e128 = M_1;
        phi_1443_ = (_e128.x < -1.5f);
    }
    let _e132 = phi_1443_;
    if _e132 {
        let _e133 = M_1;
        let _e137 = textureSampleLevel(XC, ga, vec2<f32>((3f + _e133.x), 0f), 0f);
        let _e143 = textureSampleLevel(XC, ga, vec2<f32>((1f - _e133.y), 0f), 0f);
        phi_4032_ = ((1f - _e137.x) - _e143.x);
    } else {
        phi_1456_ = Hh;
        if Hh {
            let _e146 = M_1;
            phi_1456_ = (_e146.y < -1.5f);
        }
        let _e150 = phi_1456_;
        if _e150 {
            let _e151 = M_1;
            let _e154 = max(_e151.w, 0f);
            if (_e151.z >= 0f) {
                let _e157 = textureSampleLevel(XC, ga, vec2<f32>(_e154, 0f), 0f);
                phi_4017_ = _e157.x;
            } else {
                phi_4017_ = 0f;
            }
            let _e160 = phi_4017_;
            phi_4025_ = _e160;
            if (abs(_e151.z) < 1000f) {
                let _e167 = (-2f - _e151.y);
                let _e169 = ((_e167 - _e154) * 0.5984134f);
                let _e172 = (vec4(_e154) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e169));
                let _e178 = ((_e172 * -(_e151.z)) + vec4(((_e167 * _e151.z) + (abs(_e151.x) - 0.25f))));
                let _e181 = textureSampleLevel(XC, ga, vec2<f32>(_e178.x, 0f), 0f);
                let _e184 = textureSampleLevel(XC, ga, vec2<f32>(_e178.y, 0f), 0f);
                let _e187 = textureSampleLevel(XC, ga, vec2<f32>(_e178.z, 0f), 0f);
                let _e190 = textureSampleLevel(XC, ga, vec2<f32>(_e178.w, 0f), 0f);
                let _e196 = (_e172 * 5.0959306f);
                phi_4025_ = (_e160 + (dot(vec4<f32>(_e181.x, _e184.x, _e187.x, _e190.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e196) * (_e196 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e169));
            }
            let _e205 = phi_4025_;
            phi_4033_ = (_e205 * sign(_e151.x));
        } else {
            let _e210 = M_1[0u];
            let _e212 = M_1[1u];
            phi_4033_ = min(min(_e210, abs(_e212)), 1f);
        }
        let _e217 = phi_4033_;
        phi_4032_ = _e217;
    }
    let _e219 = phi_4032_;
    let _e223 = u32(round(((_e219 * 2048f) + 65536f)));
    let _e224 = D0_1;
    let _e227 = ((_e224 << bitcast<u32>(17u)) | _e223);
    let _e230 = atomicMax((&z4_.g2_[_e127]), _e227);
    let _e232 = (_e230 >> bitcast<u32>(17u));
    if (_e232 == _e224) {
        let _e234 = M_1;
        if (_e234.y < 0f) {
            let _e241 = atomicAdd((&z4_.g2_[_e127]), ((_e223 + (_e230 - max(_e227, _e230))) - 65536u));
        }
        phi_4931_ = 0u;
        phi_4924_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e245 = ((f32((_e230 & 131071u)) * 0.00048828125f) + -32f);
        let _e248 = CD.g2_[_e232];
        phi_4035_ = _e245;
        if ((_e248.x & 768u) != 0u) {
            let _e252 = abs(_e245);
            phi_1950_ = Ih;
            if Ih {
                phi_1950_ = ((_e248.x & 512u) != 0u);
            }
            let _e256 = phi_1950_;
            phi_4036_ = _e252;
            if _e256 {
                phi_4036_ = (1f - abs(((fract((_e252 * 0.5f)) * 2f) + -1f)));
            }
            let _e264 = phi_4036_;
            phi_4035_ = _e264;
        }
        let _e266 = phi_4035_;
        let _e267 = clamp(_e266, 0f, 1f);
        phi_4039_ = _e267;
        if Eh {
            let _e269 = (_e248.x >> bitcast<u32>(16u));
            phi_4040_ = _e267;
            if (_e269 != 0u) {
                let _e273 = i0_.g2_[_e127];
                if (_e269 == (_e273 >> bitcast<u32>(16i))) {
                    phi_4037_ = min(_e267, unpack2x16float(_e273).x);
                } else {
                    phi_4037_ = 0f;
                }
                let _e281 = phi_4037_;
                phi_4040_ = _e281;
            }
            let _e283 = phi_4040_;
            phi_4039_ = _e283;
        }
        let _e285 = phi_4039_;
        phi_1987_ = Fh;
        if Fh {
            phi_1987_ = ((_e248.x & 1024u) != 0u);
        }
        let _e289 = phi_1987_;
        phi_4042_ = _e285;
        if _e289 {
            let _e290 = (_e232 * 8u);
            let _e294 = PB.g2_[(_e290 + 2u)];
            let _e305 = PB.g2_[(_e290 + 3u)];
            let _e310 = _e305.zw;
            let _e312 = ((abs(((mat2x2<f32>(vec2<f32>(_e294.x, _e294.y), vec2<f32>(_e294.z, _e294.w)) * _e93) + _e305.xy)) * _e310) - _e310);
            phi_4042_ = min(_e285, clamp((min(_e312.x, _e312.y) + 0.5f), 0f, 1f));
        }
        let _e320 = phi_4042_;
        let _e321 = (_e248.x & 15u);
        let _e324 = ((_e248.x >> bitcast<u32>(4i)) & 15u);
        let _e326 = (Gh && (_e324 != 0u));
        if (_e321 <= 1u) {
            let _e331 = (Eh && (_e321 == 0u));
            phi_4902_ = 0u;
            if _e331 {
                phi_4902_ = (_e248.y | pack2x16float(vec2<f32>(_e320, 0f)));
            }
            let _e336 = phi_4902_;
            phi_4901_ = _e336;
            phi_4072_ = select(unpack4x8unorm(_e248.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e331));
        } else {
            let _e339 = (_e232 * 8u);
            let _e342 = PB.g2_[_e339];
            let _e353 = PB.g2_[(_e339 + 1u)];
            let _e356 = ((mat2x2<f32>(vec2<f32>(_e342.x, _e342.y), vec2<f32>(_e342.z, _e342.w)) * _e93) + _e353.xy);
            if (_e321 == 2u) {
                phi_4041_ = _e356.x;
            } else {
                phi_4041_ = length(_e356);
            }
            let _e361 = phi_4041_;
            let _e370 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e361, 0f, 1f) * _e353.z) + _e353.w), bitcast<f32>(_e248.y)), 0f);
            phi_4074_ = _e370;
            if !(_e326) {
                let _e374 = (_e370.xyz * _e370.w);
                let _e380 = vec4<f32>(_e374.x, _e370.y, _e370.z, _e370.w);
                let _e386 = vec4<f32>(_e380.x, _e374.y, _e380.z, _e380.w);
                phi_4074_ = vec4<f32>(_e386.x, _e386.y, _e374.z, _e386.w);
            }
            let _e394 = phi_4074_;
            phi_4901_ = 0u;
            phi_4072_ = _e394;
        }
        let _e396 = phi_4901_;
        let _e398 = phi_4072_;
        phi_4899_ = _e398;
        if _e326 {
            phi_4886_ = _e398;
            if ((_e398.w * _e320) != 0f) {
                let _e404 = l0_.g2_[_e127];
                let _e405 = unpack4x8unorm(_e404);
                let _e406 = _e398.xyz;
                local_2 = _e406;
                let _e407 = _e405.xyz;
                if (_e405.w != 0f) {
                    phi_4079_ = (1f / _e405.w);
                } else {
                    phi_4079_ = 0f;
                }
                let _e412 = phi_4079_;
                let _e413 = (_e407 * _e412);
                local = _e413;
                switch bitcast<i32>(_e324) {
                    case 11: {
                        let _e415 = local_2;
                        local_1 = (_e415 * _e413);
                        break;
                    }
                    case 1: {
                        let _e417 = local_2;
                        local_1 = ((_e417 + _e413) - (_e417 * _e413));
                        break;
                    }
                    case 2: {
                        let _e421 = local_2;
                        let _e422 = (_e421 * _e413);
                        local_1 = (select(_e422, (((_e421 + _e413) - _e422) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e413 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e429 = local_2;
                        local_1 = min(_e429, _e413);
                        break;
                    }
                    case 4: {
                        let _e431 = local_2;
                        local_1 = max(_e431, _e413);
                        break;
                    }
                    case 5: {
                        let _e434 = clamp(_e407, vec3<f32>(0f, 0f, 0f), _e405.www);
                        let _e440 = vec4<f32>(_e434.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e446 = vec4<f32>(_e440.x, _e434.y, _e440.z, _e440.w);
                        let _e453 = local_2;
                        let _e456 = (clamp((vec3<f32>(1f, 1f, 1f) - _e453), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e405.w);
                        let _e457 = vec4<f32>(_e446.x, _e446.y, _e434.z, _e446.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e457 / _e456)), sign(_e457), (_e456 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e463 = local_2;
                        local_2 = clamp(_e463, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e466 = clamp(_e407, vec3<f32>(0f, 0f, 0f), _e405.www);
                        let _e472 = vec4<f32>(_e466.x, _e405.y, _e405.z, _e405.w);
                        let _e478 = vec4<f32>(_e472.x, _e466.y, _e472.z, _e472.w);
                        phi_4693_ = vec4<f32>(_e478.x, _e478.y, _e466.z, _e478.w);
                        if (_e405.w == 0f) {
                            phi_4693_ = vec4<f32>(_e466.x, _e466.y, _e466.z, 1f);
                        }
                        let _e488 = phi_4693_;
                        let _e492 = (vec3(_e488.w) - _e488.xyz);
                        let _e493 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e492 / (_e493 * _e488.w))), sign(_e492), (_e493 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e501 = local_2;
                        let _e502 = (_e501 * _e413);
                        local_1 = (select(_e502, (((_e501 + _e413) - _e502) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e501 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4613_ = 0i;
                        loop {
                            let _e510 = phi_4613_;
                            if (_e510 < 3i) {
                                let _e513 = local_2[_e510];
                                if (_e513 <= 0.5f) {
                                    let _e516 = local[_e510];
                                    local_1[_e510] = (1f - _e516);
                                } else {
                                    let _e520 = local[_e510];
                                    if (_e520 <= 0.25f) {
                                        let _e522 = local[_e510];
                                        let _e525 = local[_e510];
                                        local_1[_e510] = ((((16f * _e522) - 12f) * _e525) + 3f);
                                    } else {
                                        let _e529 = local[_e510];
                                        local_1[_e510] = (inverseSqrt(_e529) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4613_ = (_e510 + 1i);
                            }
                        }
                        let _e534 = local_2;
                        let _e538 = local_1;
                        local_1 = (_e413 + ((_e413 * ((_e534 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e538));
                        break;
                    }
                    case 9: {
                        let _e541 = local_2;
                        local_1 = abs((_e413 - _e541));
                        break;
                    }
                    case 10: {
                        let _e544 = local_2;
                        local_1 = ((_e544 + _e413) - ((_e544 * 2f) * _e413));
                        break;
                    }
                    case 12: {
                        if Kh {
                            let _e549 = local_2;
                            let _e550 = clamp(_e549, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e550;
                            let _e565 = (_e550 - vec3(min(min(_e550.x, _e550.y), _e550.z)));
                            let _e573 = (_e565 * ((max(max(_e413.x, _e413.y), _e413.z) - min(min(_e413.x, _e413.y), _e413.z)) / max(0.000062f, max(max(_e565.x, _e565.y), _e565.z))));
                            let _e574 = dot(_e413, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e577 = (_e573 - vec3(dot(_e573, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e590 = (vec2<f32>(_e574, (1f - _e574)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e577.x, _e577.y), _e577.z)), max(max(_e577.x, _e577.y), _e577.z))));
                            local_1 = ((_e577 * min(1f, min(_e590.x, _e590.y))) + vec3(_e574));
                        }
                        break;
                    }
                    case 13: {
                        if Kh {
                            let _e598 = local_2;
                            let _e599 = clamp(_e598, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e599;
                            let _e614 = (_e413 - vec3(min(min(_e413.x, _e413.y), _e413.z)));
                            let _e622 = (_e614 * ((max(max(_e599.x, _e599.y), _e599.z) - min(min(_e599.x, _e599.y), _e599.z)) / max(0.000062f, max(max(_e614.x, _e614.y), _e614.z))));
                            let _e623 = dot(_e413, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e626 = (_e622 - vec3(dot(_e622, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e639 = (vec2<f32>(_e623, (1f - _e623)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e626.x, _e626.y), _e626.z)), max(max(_e626.x, _e626.y), _e626.z))));
                            local_1 = ((_e626 * min(1f, min(_e639.x, _e639.y))) + vec3(_e623));
                        }
                        break;
                    }
                    case 14: {
                        if Kh {
                            let _e647 = local_2;
                            let _e648 = clamp(_e647, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e648;
                            let _e649 = dot(_e413, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e652 = (_e648 - vec3(dot(_e648, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e665 = (vec2<f32>(_e649, (1f - _e649)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e652.x, _e652.y), _e652.z)), max(max(_e652.x, _e652.y), _e652.z))));
                            local_1 = ((_e652 * min(1f, min(_e665.x, _e665.y))) + vec3(_e649));
                        }
                        break;
                    }
                    case 15: {
                        if Kh {
                            let _e673 = local_2;
                            let _e674 = clamp(_e673, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e674;
                            let _e675 = dot(_e674, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e678 = (_e413 - vec3(dot(_e413, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e691 = (vec2<f32>(_e675, (1f - _e675)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e678.x, _e678.y), _e678.z)), max(max(_e678.x, _e678.y), _e678.z))));
                            local_1 = ((_e678 * min(1f, min(_e691.x, _e691.y))) + vec3(_e675));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e699 = local_1;
                let _e701 = mix(_e406, _e699, vec3(_e405.w));
                let _e707 = vec4<f32>(_e701.x, _e398.y, _e398.z, _e398.w);
                let _e713 = vec4<f32>(_e707.x, _e701.y, _e707.z, _e707.w);
                phi_4886_ = vec4<f32>(_e713.x, _e713.y, _e701.z, _e713.w);
            }
            let _e721 = phi_4886_;
            let _e724 = (_e721.xyz * _e721.w);
            let _e730 = vec4<f32>(_e724.x, _e721.y, _e721.z, _e721.w);
            let _e736 = vec4<f32>(_e730.x, _e724.y, _e730.z, _e730.w);
            phi_4899_ = vec4<f32>(_e736.x, _e736.y, _e724.z, _e736.w);
        }
        let _e744 = phi_4899_;
        phi_4931_ = _e396;
        phi_4924_ = (_e744 * _e320);
    }
    let _e747 = phi_4931_;
    let _e749 = phi_4924_;
    let _e750 = _e749.xyz;
    let _e753 = j.F3_;
    let _e755 = j.G3_;
    if (Lh && (_e749.w != 0f)) {
        phi_4926_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e92.x) + (0.00583715f * _e92.y))))) * _e753) + _e755)) + _e750);
    } else {
        phi_4926_ = _e750;
    }
    let _e771 = phi_4926_;
    let _e777 = vec4<f32>(_e771.x, _e749.y, _e749.z, _e749.w);
    let _e783 = vec4<f32>(_e777.x, _e771.y, _e777.z, _e777.w);
    let _e789 = vec4<f32>(_e783.x, _e783.y, _e771.z, _e783.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e749.w == 0f) {
                break;
            }
            let _e792 = (1f - _e749.w);
            phi_4928_ = _e789;
            if (_e792 != 0f) {
                let _e796 = l0_.g2_[_e127];
                phi_4928_ = (_e789 + (unpack4x8unorm(_e796) * _e792));
            }
            let _e801 = phi_4928_;
            l0_.g2_[_e127] = pack4x8unorm(_e801);
            break;
        }
    }
    if (_e747 != 0u) {
        i0_.g2_[_e127] = _e747;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) M: vec4<f32>, @location(1) @interpolate(flat, either) D0_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    M_1 = M;
    D0_1 = D0_;
    main_1();
}
