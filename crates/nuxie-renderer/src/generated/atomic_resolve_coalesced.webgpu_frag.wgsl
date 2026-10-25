struct og {
    v2_: array<vec2<u32>>,
}

struct m0Ue {
    v2_: array<u32>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct n0Ue {
    v2_: array<u32>,
}

struct T4Ue {
    v2_: array<u32>,
}

@id(7) override mj: bool = true;
@id(9) override oj: bool = true;
@id(5) override kj: bool = true;
@id(0) override fj: bool = true;
@id(1) override gj: bool = true;
@id(2) override hj: bool = true;

@group(0) @binding(3)
var<storage> XC: og;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Ue;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Ue;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1399_: bool;
    var phi_3439_: f32;
    var phi_3438_: f32;
    var phi_3440_: f32;
    var phi_3443_: f32;
    var phi_3442_: f32;
    var phi_1436_: bool;
    var phi_3461_: f32;
    var phi_3444_: f32;
    var phi_3445_: f32;
    var phi_3446_: vec2<f32>;
    var phi_3458_: vec4<f32>;
    var phi_3456_: vec4<f32>;
    var phi_3466_: f32;
    var phi_3889_: i32;
    var phi_3953_: vec4<f32>;
    var phi_4110_: vec4<f32>;
    var phi_4123_: vec4<f32>;
    var phi_4124_: vec4<f32>;

    let _e75 = gl_FragCoord_1;
    let _e76 = _e75.xy;
    let _e79 = bitcast<vec2<u32>>(vec2<i32>(floor(_e76)));
    let _e81 = j.T6_;
    let _e110 = bitcast<i32>((((((_e79.y >> bitcast<u32>(5u)) * (((_e81 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e79.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e79.x & 28u) << bitcast<u32>(5u)) + ((_e79.y & 28u) << bitcast<u32>(2i)))) + (((_e79.y & 3u) << bitcast<u32>(2i)) + (_e79.x & 3u))));
    let _e113 = T4_.v2_[_e110];
    let _e117 = ((f32((_e113 & 131071u)) * 0.00048828125f) + -32f);
    let _e119 = (_e113 >> bitcast<u32>(17u));
    let _e122 = XC.v2_[_e119];
    phi_3438_ = _e117;
    if ((_e122.x & 768u) != 0u) {
        let _e126 = abs(_e117);
        phi_1399_ = kj;
        if kj {
            phi_1399_ = ((_e122.x & 512u) != 0u);
        }
        let _e130 = phi_1399_;
        phi_3439_ = _e126;
        if _e130 {
            phi_3439_ = (1f - abs(((fract((_e126 * 0.5f)) * 2f) + -1f)));
        }
        let _e138 = phi_3439_;
        phi_3438_ = _e138;
    }
    let _e140 = phi_3438_;
    let _e141 = clamp(_e140, 0f, 1f);
    phi_3442_ = _e141;
    if fj {
        let _e143 = (_e122.x >> bitcast<u32>(16u));
        phi_3443_ = _e141;
        if (_e143 != 0u) {
            let _e147 = m0_.v2_[_e110];
            if (_e143 == (_e147 >> bitcast<u32>(16i))) {
                phi_3440_ = min(_e141, unpack2x16float(_e147).x);
            } else {
                phi_3440_ = 0f;
            }
            let _e155 = phi_3440_;
            phi_3443_ = _e155;
        }
        let _e157 = phi_3443_;
        phi_3442_ = _e157;
    }
    let _e159 = phi_3442_;
    phi_1436_ = gj;
    if gj {
        phi_1436_ = ((_e122.x & 1024u) != 0u);
    }
    let _e163 = phi_1436_;
    phi_3461_ = _e159;
    if _e163 {
        let _e164 = (_e119 * 8u);
        let _e168 = JB.v2_[(_e164 + 2u)];
        let _e179 = JB.v2_[(_e164 + 3u)];
        let _e184 = _e179.zw;
        let _e186 = ((abs(((mat2x2<f32>(vec2<f32>(_e168.x, _e168.y), vec2<f32>(_e168.z, _e168.w)) * _e76) + _e179.xy)) * _e184) - _e184);
        phi_3461_ = min(_e159, clamp((min(_e186.x, _e186.y) + 0.5f), 0f, 1f));
    }
    let _e194 = phi_3461_;
    let _e195 = (_e122.x & 15u);
    let _e198 = ((_e122.x >> bitcast<u32>(4i)) & 15u);
    let _e200 = (hj && (_e198 != 0u));
    if (_e195 <= 1u) {
        phi_3456_ = select(unpack4x8unorm(_e122.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((fj && (_e195 == 0u))));
    } else {
        let _e202 = (_e119 * 8u);
        let _e205 = JB.v2_[_e202];
        let _e216 = JB.v2_[(_e202 + 1u)];
        let _e219 = ((mat2x2<f32>(vec2<f32>(_e205.x, _e205.y), vec2<f32>(_e205.z, _e205.w)) * _e76) + _e216.xy);
        let _e223 = f32(_e195);
        let _e225 = j.P8_;
        let _e227 = j.Q8_;
        let _e232 = ((_e216.w * _e225) + _e227);
        switch bitcast<i32>(0u) {
            default: {
                if (oj && (_e223 == 4f)) {
                    let _e250 = normalize(vec2<f32>(_e219.x, -(_e219.y)));
                    let _e253 = acos(clamp(_e250.x, -1f, 1f));
                    if (_e250.y >= 0f) {
                        phi_3445_ = _e253;
                    } else {
                        phi_3445_ = -(_e253);
                    }
                    let _e258 = phi_3445_;
                    phi_3446_ = vec2<f32>((_e258 * 0.15915494f), _e232);
                    break;
                } else {
                    if (_e223 == 2f) {
                        phi_3444_ = _e219.x;
                    } else {
                        phi_3444_ = length(_e219);
                    }
                    let _e240 = phi_3444_;
                    phi_3446_ = vec2<f32>(((clamp(_e240, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e216.z < 0f))) + ((max(0f, _e216.z) * 0.001953125f) + 0.0009765625f)), _e232);
                    break;
                }
            }
        }
        let _e262 = phi_3446_;
        let _e263 = textureSampleLevel(ZC, R8_, _e262, 0f);
        phi_3458_ = _e263;
        if !(_e200) {
            let _e267 = (_e263.xyz * _e263.w);
            phi_3458_ = vec4<f32>(_e267.x, _e267.y, _e267.z, (_e263.w * abs(bitcast<f32>(_e122.y))));
        }
        let _e277 = phi_3458_;
        phi_3456_ = _e277;
    }
    let _e285 = phi_3456_;
    phi_4123_ = _e285;
    if _e200 {
        phi_4110_ = _e285;
        if ((_e285.w * _e194) != 0f) {
            let _e291 = n0_.v2_[_e110];
            let _e292 = unpack4x8unorm(_e291);
            let _e293 = _e285.xyz;
            local_2 = _e293;
            let _e294 = _e292.xyz;
            if (_e292.w != 0f) {
                phi_3466_ = (1f / _e292.w);
            } else {
                phi_3466_ = 0f;
            }
            let _e299 = phi_3466_;
            let _e300 = (_e294 * _e299);
            local = _e300;
            switch bitcast<i32>(_e198) {
                case 11: {
                    let _e584 = local_2;
                    local_1 = (_e584 * _e300);
                    break;
                }
                case 1: {
                    let _e580 = local_2;
                    local_1 = ((_e580 + _e300) - (_e580 * _e300));
                    break;
                }
                case 2: {
                    let _e572 = local_2;
                    let _e573 = (_e572 * _e300);
                    local_1 = (select(_e573, (((_e572 + _e300) - _e573) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e300 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e570 = local_2;
                    local_1 = min(_e570, _e300);
                    break;
                }
                case 4: {
                    let _e568 = local_2;
                    local_1 = max(_e568, _e300);
                    break;
                }
                case 5: {
                    let _e539 = clamp(_e294, vec3<f32>(0f, 0f, 0f), _e292.www);
                    let _e545 = vec4<f32>(_e539.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e551 = vec4<f32>(_e545.x, _e539.y, _e545.z, _e545.w);
                    let _e558 = local_2;
                    let _e561 = (clamp((vec3<f32>(1f, 1f, 1f) - _e558), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e292.w);
                    let _e562 = vec4<f32>(_e551.x, _e551.y, _e539.z, _e551.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e562 / _e561)), sign(_e562), (_e561 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e500 = local_2;
                    local_2 = clamp(_e500, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e503 = clamp(_e294, vec3<f32>(0f, 0f, 0f), _e292.www);
                    let _e509 = vec4<f32>(_e503.x, _e292.y, _e292.z, _e292.w);
                    let _e515 = vec4<f32>(_e509.x, _e503.y, _e509.z, _e509.w);
                    phi_3953_ = vec4<f32>(_e515.x, _e515.y, _e503.z, _e515.w);
                    if (_e292.w == 0f) {
                        phi_3953_ = vec4<f32>(_e503.x, _e503.y, _e503.z, 1f);
                    }
                    let _e525 = phi_3953_;
                    let _e529 = (vec3(_e525.w) - _e525.xyz);
                    let _e530 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e529 / (_e530 * _e525.w))), sign(_e529), (_e530 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e492 = local_2;
                    let _e493 = (_e492 * _e300);
                    local_1 = (select(_e493, (((_e492 + _e300) - _e493) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e492 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3889_ = 0i;
                    loop {
                        let _e461 = phi_3889_;
                        if (_e461 < 3i) {
                            let _e464 = local_2[_e461];
                            if (_e464 <= 0.5f) {
                                let _e481 = local[_e461];
                                local_1[_e461] = (1f - _e481);
                            } else {
                                let _e467 = local[_e461];
                                if (_e467 <= 0.25f) {
                                    let _e473 = local[_e461];
                                    let _e476 = local[_e461];
                                    local_1[_e461] = ((((16f * _e473) - 12f) * _e476) + 3f);
                                } else {
                                    let _e469 = local[_e461];
                                    local_1[_e461] = (inverseSqrt(_e469) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3889_ = (_e461 + 1i);
                        }
                    }
                    let _e485 = local_2;
                    let _e489 = local_1;
                    local_1 = (_e300 + ((_e300 * ((_e485 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e489));
                    break;
                }
                case 9: {
                    let _e457 = local_2;
                    local_1 = abs((_e300 - _e457));
                    break;
                }
                case 10: {
                    let _e452 = local_2;
                    local_1 = ((_e452 + _e300) - ((_e452 * 2f) * _e300));
                    break;
                }
                case 12: {
                    if mj {
                        let _e403 = local_2;
                        let _e404 = clamp(_e403, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e404;
                        let _e419 = (_e404 - vec3(min(min(_e404.x, _e404.y), _e404.z)));
                        let _e427 = (_e419 * ((max(max(_e300.x, _e300.y), _e300.z) - min(min(_e300.x, _e300.y), _e300.z)) / max(0.000062f, max(max(_e419.x, _e419.y), _e419.z))));
                        let _e428 = dot(_e300, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e431 = (_e427 - vec3(dot(_e427, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e444 = (vec2<f32>(_e428, (1f - _e428)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e431.x, _e431.y), _e431.z)), max(max(_e431.x, _e431.y), _e431.z))));
                        local_1 = ((_e431 * min(1f, min(_e444.x, _e444.y))) + vec3(_e428));
                    }
                    break;
                }
                case 13: {
                    if mj {
                        let _e354 = local_2;
                        let _e355 = clamp(_e354, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e355;
                        let _e370 = (_e300 - vec3(min(min(_e300.x, _e300.y), _e300.z)));
                        let _e378 = (_e370 * ((max(max(_e355.x, _e355.y), _e355.z) - min(min(_e355.x, _e355.y), _e355.z)) / max(0.000062f, max(max(_e370.x, _e370.y), _e370.z))));
                        let _e379 = dot(_e300, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e382 = (_e378 - vec3(dot(_e378, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e395 = (vec2<f32>(_e379, (1f - _e379)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e382.x, _e382.y), _e382.z)), max(max(_e382.x, _e382.y), _e382.z))));
                        local_1 = ((_e382 * min(1f, min(_e395.x, _e395.y))) + vec3(_e379));
                    }
                    break;
                }
                case 14: {
                    if mj {
                        let _e328 = local_2;
                        let _e329 = clamp(_e328, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e329;
                        let _e330 = dot(_e300, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e333 = (_e329 - vec3(dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e346 = (vec2<f32>(_e330, (1f - _e330)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e333.x, _e333.y), _e333.z)), max(max(_e333.x, _e333.y), _e333.z))));
                        local_1 = ((_e333 * min(1f, min(_e346.x, _e346.y))) + vec3(_e330));
                    }
                    break;
                }
                case 15: {
                    if mj {
                        let _e302 = local_2;
                        let _e303 = clamp(_e302, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e303;
                        let _e304 = dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e307 = (_e300 - vec3(dot(_e300, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e320 = (vec2<f32>(_e304, (1f - _e304)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e307.x, _e307.y), _e307.z)), max(max(_e307.x, _e307.y), _e307.z))));
                        local_1 = ((_e307 * min(1f, min(_e320.x, _e320.y))) + vec3(_e304));
                    }
                    break;
                }
                default: {
                }
            }
            let _e586 = local_1;
            let _e588 = mix(_e293, _e586, vec3(_e292.w));
            let _e594 = vec4<f32>(_e588.x, _e285.y, _e285.z, _e285.w);
            let _e600 = vec4<f32>(_e594.x, _e588.y, _e594.z, _e594.w);
            phi_4110_ = vec4<f32>(_e600.x, _e600.y, _e588.z, _e600.w);
        }
        let _e608 = phi_4110_;
        let _e611 = (_e608.xyz * _e608.w);
        let _e617 = vec4<f32>(_e611.x, _e608.y, _e608.z, _e608.w);
        let _e623 = vec4<f32>(_e617.x, _e611.y, _e617.z, _e617.w);
        phi_4123_ = vec4<f32>(_e623.x, _e623.y, _e611.z, _e623.w);
    }
    let _e631 = phi_4123_;
    let _e632 = (_e631 * _e194);
    let _e634 = (1f - _e632.w);
    phi_4124_ = _e632;
    if (_e634 != 0f) {
        let _e638 = n0_.v2_[_e110];
        phi_4124_ = (_e632 + (unpack4x8unorm(_e638) * _e634));
    }
    let _e643 = phi_4124_;
    Q1_ = _e643;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = Q1_;
    return _e3;
}
