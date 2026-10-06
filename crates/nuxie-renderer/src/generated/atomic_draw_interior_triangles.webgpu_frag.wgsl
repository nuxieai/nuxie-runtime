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
var<private> F0_1: u32;
var<private> m1_1: f32;
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
    var phi_3562_: u32;
    var phi_1488_: bool;
    var phi_3567_: f32;
    var phi_3566_: f32;
    var phi_3568_: f32;
    var phi_3571_: f32;
    var phi_3570_: f32;
    var phi_1525_: bool;
    var phi_3573_: f32;
    var phi_4218_: u32;
    var phi_3572_: f32;
    var phi_3595_: vec4<f32>;
    var phi_4217_: u32;
    var phi_3593_: vec4<f32>;
    var phi_3600_: f32;
    var phi_4054_: vec4<f32>;
    var phi_3994_: i32;
    var phi_4202_: vec4<f32>;
    var phi_4215_: vec4<f32>;
    var phi_4246_: u32;
    var phi_4240_: vec4<f32>;
    var phi_4241_: vec3<f32>;
    var phi_4243_: vec4<f32>;

    let _e79 = gl_FragCoord_1;
    let _e80 = _e79.xy;
    let _e83 = bitcast<vec2<u32>>(vec2<i32>(floor(_e80)));
    let _e85 = j.A6_;
    let _e114 = bitcast<i32>((((((_e83.y >> bitcast<u32>(5u)) * (((_e85 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e83.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e83.x & 28u) << bitcast<u32>(5u)) + ((_e83.y & 28u) << bitcast<u32>(2i)))) + (((_e83.y & 3u) << bitcast<u32>(2i)) + (_e83.x & 3u))));
    let _e117 = L4_.k2_[_e114];
    let _e119 = (_e117 >> bitcast<u32>(17u));
    let _e120 = F0_1;
    if (_e119 == _e120) {
        phi_3562_ = _e117;
    } else {
        phi_3562_ = ((_e120 << bitcast<u32>(17u)) + 65536u);
    }
    let _e126 = phi_3562_;
    let _e127 = m1_1;
    L4_.k2_[_e114] = (_e126 + bitcast<u32>(i32(round((_e127 * 2048f)))));
    phi_4246_ = 0u;
    phi_4240_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e119 != _e120) {
        let _e137 = ((f32((_e117 & 131071u)) * 0.00048828125f) + -32f);
        let _e140 = WC.k2_[_e119];
        phi_3566_ = _e137;
        if ((_e140.x & 768u) != 0u) {
            let _e144 = abs(_e137);
            phi_1488_ = qi;
            if qi {
                phi_1488_ = ((_e140.x & 512u) != 0u);
            }
            let _e148 = phi_1488_;
            phi_3567_ = _e144;
            if _e148 {
                phi_3567_ = (1f - abs(((fract((_e144 * 0.5f)) * 2f) + -1f)));
            }
            let _e156 = phi_3567_;
            phi_3566_ = _e156;
        }
        let _e158 = phi_3566_;
        let _e159 = clamp(_e158, 0f, 1f);
        phi_3570_ = _e159;
        if mi {
            let _e161 = (_e140.x >> bitcast<u32>(16u));
            phi_3571_ = _e159;
            if (_e161 != 0u) {
                let _e165 = m0_.k2_[_e114];
                if (_e161 == (_e165 >> bitcast<u32>(16i))) {
                    phi_3568_ = min(_e159, unpack2x16float(_e165).x);
                } else {
                    phi_3568_ = 0f;
                }
                let _e173 = phi_3568_;
                phi_3571_ = _e173;
            }
            let _e175 = phi_3571_;
            phi_3570_ = _e175;
        }
        let _e177 = phi_3570_;
        phi_1525_ = ni;
        if ni {
            phi_1525_ = ((_e140.x & 1024u) != 0u);
        }
        let _e181 = phi_1525_;
        phi_3573_ = _e177;
        if _e181 {
            let _e182 = (_e119 * 8u);
            let _e186 = JB.k2_[(_e182 + 2u)];
            let _e197 = JB.k2_[(_e182 + 3u)];
            let _e202 = _e197.zw;
            let _e204 = ((abs(((mat2x2<f32>(vec2<f32>(_e186.x, _e186.y), vec2<f32>(_e186.z, _e186.w)) * _e80) + _e197.xy)) * _e202) - _e202);
            phi_3573_ = min(_e177, clamp((min(_e204.x, _e204.y) + 0.5f), 0f, 1f));
        }
        let _e212 = phi_3573_;
        let _e213 = (_e140.x & 15u);
        let _e216 = ((_e140.x >> bitcast<u32>(4i)) & 15u);
        let _e218 = (oi && (_e216 != 0u));
        if (_e213 <= 1u) {
            let _e223 = (mi && (_e213 == 0u));
            phi_4218_ = 0u;
            if _e223 {
                phi_4218_ = (_e140.y | pack2x16float(vec2<f32>(_e212, 0f)));
            }
            let _e228 = phi_4218_;
            phi_4217_ = _e228;
            phi_3593_ = select(unpack4x8unorm(_e140.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e223));
        } else {
            let _e231 = (_e119 * 8u);
            let _e234 = JB.k2_[_e231];
            let _e245 = JB.k2_[(_e231 + 1u)];
            let _e248 = ((mat2x2<f32>(vec2<f32>(_e234.x, _e234.y), vec2<f32>(_e234.z, _e234.w)) * _e80) + _e245.xy);
            if (_e213 == 2u) {
                phi_3572_ = _e248.x;
            } else {
                phi_3572_ = length(_e248);
            }
            let _e253 = phi_3572_;
            let _e260 = bitcast<f32>(_e140.y);
            let _e263 = j.xc;
            let _e266 = j.yc;
            let _e269 = textureSampleLevel(ED, ia, vec2<f32>(((clamp(_e253, 0f, 1f) * _e245.z) + _e245.w), ((floor(_e260) * _e263) + _e266)), 0f);
            phi_3595_ = _e269;
            if !(_e218) {
                let _e273 = (_e269.xyz * _e269.w);
                phi_3595_ = vec4<f32>(_e273.x, _e273.y, _e273.z, (_e269.w * (fract(_e260) * 1.0039216f)));
            }
            let _e282 = phi_3595_;
            phi_4217_ = 0u;
            phi_3593_ = _e282;
        }
        let _e284 = phi_4217_;
        let _e286 = phi_3593_;
        phi_4215_ = _e286;
        if _e218 {
            phi_4202_ = _e286;
            if ((_e286.w * _e212) != 0f) {
                let _e292 = n0_.k2_[_e114];
                let _e293 = unpack4x8unorm(_e292);
                let _e294 = _e286.xyz;
                local_2 = _e294;
                let _e295 = _e293.xyz;
                if (_e293.w != 0f) {
                    phi_3600_ = (1f / _e293.w);
                } else {
                    phi_3600_ = 0f;
                }
                let _e300 = phi_3600_;
                let _e301 = (_e295 * _e300);
                local = _e301;
                switch bitcast<i32>(_e216) {
                    case 11: {
                        let _e303 = local_2;
                        local_1 = (_e303 * _e301);
                        break;
                    }
                    case 1: {
                        let _e305 = local_2;
                        local_1 = ((_e305 + _e301) - (_e305 * _e301));
                        break;
                    }
                    case 2: {
                        let _e309 = local_2;
                        let _e310 = (_e309 * _e301);
                        local_1 = (select(_e310, (((_e309 + _e301) - _e310) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e301 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e317 = local_2;
                        local_1 = min(_e317, _e301);
                        break;
                    }
                    case 4: {
                        let _e319 = local_2;
                        local_1 = max(_e319, _e301);
                        break;
                    }
                    case 5: {
                        let _e322 = clamp(_e295, vec3<f32>(0f, 0f, 0f), _e293.www);
                        let _e328 = vec4<f32>(_e322.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e334 = vec4<f32>(_e328.x, _e322.y, _e328.z, _e328.w);
                        let _e341 = local_2;
                        let _e344 = (clamp((vec3<f32>(1f, 1f, 1f) - _e341), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e293.w);
                        let _e345 = vec4<f32>(_e334.x, _e334.y, _e322.z, _e334.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e345 / _e344)), sign(_e345), (_e344 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e351 = local_2;
                        local_2 = clamp(_e351, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e354 = clamp(_e295, vec3<f32>(0f, 0f, 0f), _e293.www);
                        let _e360 = vec4<f32>(_e354.x, _e293.y, _e293.z, _e293.w);
                        let _e366 = vec4<f32>(_e360.x, _e354.y, _e360.z, _e360.w);
                        phi_4054_ = vec4<f32>(_e366.x, _e366.y, _e354.z, _e366.w);
                        if (_e293.w == 0f) {
                            phi_4054_ = vec4<f32>(_e354.x, _e354.y, _e354.z, 1f);
                        }
                        let _e376 = phi_4054_;
                        let _e380 = (vec3(_e376.w) - _e376.xyz);
                        let _e381 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e380 / (_e381 * _e376.w))), sign(_e380), (_e381 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e389 = local_2;
                        let _e390 = (_e389 * _e301);
                        local_1 = (select(_e390, (((_e389 + _e301) - _e390) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e389 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_3994_ = 0i;
                        loop {
                            let _e398 = phi_3994_;
                            if (_e398 < 3i) {
                                let _e401 = local_2[_e398];
                                if (_e401 <= 0.5f) {
                                    let _e404 = local[_e398];
                                    local_1[_e398] = (1f - _e404);
                                } else {
                                    let _e408 = local[_e398];
                                    if (_e408 <= 0.25f) {
                                        let _e410 = local[_e398];
                                        let _e413 = local[_e398];
                                        local_1[_e398] = ((((16f * _e410) - 12f) * _e413) + 3f);
                                    } else {
                                        let _e417 = local[_e398];
                                        local_1[_e398] = (inverseSqrt(_e417) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_3994_ = (_e398 + 1i);
                            }
                        }
                        let _e422 = local_2;
                        let _e426 = local_1;
                        local_1 = (_e301 + ((_e301 * ((_e422 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e426));
                        break;
                    }
                    case 9: {
                        let _e429 = local_2;
                        local_1 = abs((_e301 - _e429));
                        break;
                    }
                    case 10: {
                        let _e432 = local_2;
                        local_1 = ((_e432 + _e301) - ((_e432 * 2f) * _e301));
                        break;
                    }
                    case 12: {
                        if si {
                            let _e437 = local_2;
                            let _e438 = clamp(_e437, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e438;
                            let _e453 = (_e438 - vec3(min(min(_e438.x, _e438.y), _e438.z)));
                            let _e461 = (_e453 * ((max(max(_e301.x, _e301.y), _e301.z) - min(min(_e301.x, _e301.y), _e301.z)) / max(0.000062f, max(max(_e453.x, _e453.y), _e453.z))));
                            let _e462 = dot(_e301, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e465 = (_e461 - vec3(dot(_e461, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e478 = (vec2<f32>(_e462, (1f - _e462)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e465.x, _e465.y), _e465.z)), max(max(_e465.x, _e465.y), _e465.z))));
                            local_1 = ((_e465 * min(1f, min(_e478.x, _e478.y))) + vec3(_e462));
                        }
                        break;
                    }
                    case 13: {
                        if si {
                            let _e486 = local_2;
                            let _e487 = clamp(_e486, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e487;
                            let _e502 = (_e301 - vec3(min(min(_e301.x, _e301.y), _e301.z)));
                            let _e510 = (_e502 * ((max(max(_e487.x, _e487.y), _e487.z) - min(min(_e487.x, _e487.y), _e487.z)) / max(0.000062f, max(max(_e502.x, _e502.y), _e502.z))));
                            let _e511 = dot(_e301, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e514 = (_e510 - vec3(dot(_e510, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e527 = (vec2<f32>(_e511, (1f - _e511)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e514.x, _e514.y), _e514.z)), max(max(_e514.x, _e514.y), _e514.z))));
                            local_1 = ((_e514 * min(1f, min(_e527.x, _e527.y))) + vec3(_e511));
                        }
                        break;
                    }
                    case 14: {
                        if si {
                            let _e535 = local_2;
                            let _e536 = clamp(_e535, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e536;
                            let _e537 = dot(_e301, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e540 = (_e536 - vec3(dot(_e536, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e553 = (vec2<f32>(_e537, (1f - _e537)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e540.x, _e540.y), _e540.z)), max(max(_e540.x, _e540.y), _e540.z))));
                            local_1 = ((_e540 * min(1f, min(_e553.x, _e553.y))) + vec3(_e537));
                        }
                        break;
                    }
                    case 15: {
                        if si {
                            let _e561 = local_2;
                            let _e562 = clamp(_e561, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e562;
                            let _e563 = dot(_e562, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e566 = (_e301 - vec3(dot(_e301, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e579 = (vec2<f32>(_e563, (1f - _e563)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e566.x, _e566.y), _e566.z)), max(max(_e566.x, _e566.y), _e566.z))));
                            local_1 = ((_e566 * min(1f, min(_e579.x, _e579.y))) + vec3(_e563));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e587 = local_1;
                let _e589 = mix(_e294, _e587, vec3(_e293.w));
                let _e595 = vec4<f32>(_e589.x, _e286.y, _e286.z, _e286.w);
                let _e601 = vec4<f32>(_e595.x, _e589.y, _e595.z, _e595.w);
                phi_4202_ = vec4<f32>(_e601.x, _e601.y, _e589.z, _e601.w);
            }
            let _e609 = phi_4202_;
            let _e612 = (_e609.xyz * _e609.w);
            let _e618 = vec4<f32>(_e612.x, _e609.y, _e609.z, _e609.w);
            let _e624 = vec4<f32>(_e618.x, _e612.y, _e618.z, _e618.w);
            phi_4215_ = vec4<f32>(_e624.x, _e624.y, _e612.z, _e624.w);
        }
        let _e632 = phi_4215_;
        phi_4246_ = _e284;
        phi_4240_ = (_e632 * _e212);
    }
    let _e635 = phi_4246_;
    let _e637 = phi_4240_;
    let _e638 = _e637.xyz;
    let _e641 = j.M3_;
    let _e643 = j.N3_;
    if (ti && (_e637.w != 0f)) {
        phi_4241_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e79.x) + (0.00583715f * _e79.y))))) * _e641) + _e643)) + _e638);
    } else {
        phi_4241_ = _e638;
    }
    let _e659 = phi_4241_;
    let _e665 = vec4<f32>(_e659.x, _e637.y, _e637.z, _e637.w);
    let _e671 = vec4<f32>(_e665.x, _e659.y, _e665.z, _e665.w);
    let _e677 = vec4<f32>(_e671.x, _e671.y, _e659.z, _e671.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e659.x + _e659.y) + _e659.z) + _e637.w) == 0f) {
                break;
            }
            let _e683 = (1f - _e637.w);
            phi_4243_ = _e677;
            if (_e683 != 0f) {
                let _e687 = n0_.k2_[_e114];
                phi_4243_ = (_e677 + (unpack4x8unorm(_e687) * _e683));
            }
            let _e692 = phi_4243_;
            n0_.k2_[_e114] = pack4x8unorm(_e692);
            break;
        }
    }
    if (_e635 != 0u) {
        m0_.k2_[_e114] = _e635;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) F0_: u32, @location(0) @interpolate(flat, either) m1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    F0_1 = F0_;
    m1_1 = m1_;
    main_1();
}
