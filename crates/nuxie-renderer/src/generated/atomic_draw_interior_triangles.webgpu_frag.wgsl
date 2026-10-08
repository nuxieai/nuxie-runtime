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
@group(2) @binding(3)
var<storage, read_write> P4_: P4Pe;
var<private> G0_1: u32;
var<private> o1_1: f32;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3564_: u32;
    var phi_1490_: bool;
    var phi_3569_: f32;
    var phi_3568_: f32;
    var phi_3570_: f32;
    var phi_3573_: f32;
    var phi_3572_: f32;
    var phi_1527_: bool;
    var phi_3575_: f32;
    var phi_4220_: u32;
    var phi_3574_: f32;
    var phi_3597_: vec4<f32>;
    var phi_4219_: u32;
    var phi_3595_: vec4<f32>;
    var phi_3602_: f32;
    var phi_4056_: vec4<f32>;
    var phi_3996_: i32;
    var phi_4204_: vec4<f32>;
    var phi_4217_: vec4<f32>;
    var phi_4248_: u32;
    var phi_4242_: vec4<f32>;
    var phi_4243_: vec3<f32>;
    var phi_4245_: vec4<f32>;

    let _e79 = gl_FragCoord_1;
    let _e80 = _e79.xy;
    let _e83 = bitcast<vec2<u32>>(vec2<i32>(floor(_e80)));
    let _e85 = j.L6_;
    let _e114 = bitcast<i32>((((((_e83.y >> bitcast<u32>(5u)) * (((_e85 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e83.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e83.x & 28u) << bitcast<u32>(5u)) + ((_e83.y & 28u) << bitcast<u32>(2i)))) + (((_e83.y & 3u) << bitcast<u32>(2i)) + (_e83.x & 3u))));
    let _e117 = P4_.v2_[_e114];
    let _e119 = (_e117 >> bitcast<u32>(17u));
    let _e120 = G0_1;
    if (_e119 == _e120) {
        phi_3564_ = _e117;
    } else {
        phi_3564_ = ((_e120 << bitcast<u32>(17u)) + 65536u);
    }
    let _e126 = phi_3564_;
    let _e127 = o1_1;
    P4_.v2_[_e114] = (_e126 + bitcast<u32>(i32(round((_e127 * 2048f)))));
    phi_4248_ = 0u;
    phi_4242_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e119 != _e120) {
        let _e137 = ((f32((_e117 & 131071u)) * 0.00048828125f) + -32f);
        let _e140 = WC.v2_[_e119];
        phi_3568_ = _e137;
        if ((_e140.x & 768u) != 0u) {
            let _e144 = abs(_e137);
            phi_1490_ = cj;
            if cj {
                phi_1490_ = ((_e140.x & 512u) != 0u);
            }
            let _e148 = phi_1490_;
            phi_3569_ = _e144;
            if _e148 {
                phi_3569_ = (1f - abs(((fract((_e144 * 0.5f)) * 2f) + -1f)));
            }
            let _e156 = phi_3569_;
            phi_3568_ = _e156;
        }
        let _e158 = phi_3568_;
        let _e159 = clamp(_e158, 0f, 1f);
        phi_3572_ = _e159;
        if Yi {
            let _e161 = (_e140.x >> bitcast<u32>(16u));
            phi_3573_ = _e159;
            if (_e161 != 0u) {
                let _e165 = m0_.v2_[_e114];
                if (_e161 == (_e165 >> bitcast<u32>(16i))) {
                    phi_3570_ = min(_e159, unpack2x16float(_e165).x);
                } else {
                    phi_3570_ = 0f;
                }
                let _e173 = phi_3570_;
                phi_3573_ = _e173;
            }
            let _e175 = phi_3573_;
            phi_3572_ = _e175;
        }
        let _e177 = phi_3572_;
        phi_1527_ = Zi;
        if Zi {
            phi_1527_ = ((_e140.x & 1024u) != 0u);
        }
        let _e181 = phi_1527_;
        phi_3575_ = _e177;
        if _e181 {
            let _e182 = (_e119 * 8u);
            let _e186 = JB.v2_[(_e182 + 2u)];
            let _e197 = JB.v2_[(_e182 + 3u)];
            let _e202 = _e197.zw;
            let _e204 = ((abs(((mat2x2<f32>(vec2<f32>(_e186.x, _e186.y), vec2<f32>(_e186.z, _e186.w)) * _e80) + _e197.xy)) * _e202) - _e202);
            phi_3575_ = min(_e177, clamp((min(_e204.x, _e204.y) + 0.5f), 0f, 1f));
        }
        let _e212 = phi_3575_;
        let _e213 = (_e140.x & 15u);
        let _e216 = ((_e140.x >> bitcast<u32>(4i)) & 15u);
        let _e218 = (aj && (_e216 != 0u));
        if (_e213 <= 1u) {
            let _e223 = (Yi && (_e213 == 0u));
            phi_4220_ = 0u;
            if _e223 {
                phi_4220_ = (_e140.y | pack2x16float(vec2<f32>(_e212, 0f)));
            }
            let _e228 = phi_4220_;
            phi_4219_ = _e228;
            phi_3595_ = select(unpack4x8unorm(_e140.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e223));
        } else {
            let _e231 = (_e119 * 8u);
            let _e234 = JB.v2_[_e231];
            let _e245 = JB.v2_[(_e231 + 1u)];
            let _e248 = ((mat2x2<f32>(vec2<f32>(_e234.x, _e234.y), vec2<f32>(_e234.z, _e234.w)) * _e80) + _e245.xy);
            if (_e213 == 2u) {
                phi_3574_ = _e248.x;
            } else {
                phi_3574_ = length(_e248);
            }
            let _e253 = phi_3574_;
            let _e260 = bitcast<f32>(_e140.y);
            let _e263 = j.cd;
            let _e266 = j.g7_;
            let _e269 = textureSampleLevel(YC, I8_, vec2<f32>(((clamp(_e253, 0f, 1f) * _e245.z) + _e245.w), ((floor(_e260) * _e263) + _e266)), 0f);
            phi_3597_ = _e269;
            if !(_e218) {
                let _e273 = (_e269.xyz * _e269.w);
                phi_3597_ = vec4<f32>(_e273.x, _e273.y, _e273.z, (_e269.w * (fract(_e260) * 1.0039216f)));
            }
            let _e282 = phi_3597_;
            phi_4219_ = 0u;
            phi_3595_ = _e282;
        }
        let _e284 = phi_4219_;
        let _e286 = phi_3595_;
        phi_4217_ = _e286;
        if _e218 {
            phi_4204_ = _e286;
            if ((_e286.w * _e212) != 0f) {
                let _e292 = n0_.v2_[_e114];
                let _e293 = unpack4x8unorm(_e292);
                let _e294 = _e286.xyz;
                local_2 = _e294;
                let _e295 = _e293.xyz;
                if (_e293.w != 0f) {
                    phi_3602_ = (1f / _e293.w);
                } else {
                    phi_3602_ = 0f;
                }
                let _e300 = phi_3602_;
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
                        phi_4056_ = vec4<f32>(_e366.x, _e366.y, _e354.z, _e366.w);
                        if (_e293.w == 0f) {
                            phi_4056_ = vec4<f32>(_e354.x, _e354.y, _e354.z, 1f);
                        }
                        let _e376 = phi_4056_;
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
                        phi_3996_ = 0i;
                        loop {
                            let _e398 = phi_3996_;
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
                                phi_3996_ = (_e398 + 1i);
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
                        if ej {
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
                        if ej {
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
                        if ej {
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
                        if ej {
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
                phi_4204_ = vec4<f32>(_e601.x, _e601.y, _e589.z, _e601.w);
            }
            let _e609 = phi_4204_;
            let _e612 = (_e609.xyz * _e609.w);
            let _e618 = vec4<f32>(_e612.x, _e609.y, _e609.z, _e609.w);
            let _e624 = vec4<f32>(_e618.x, _e612.y, _e618.z, _e618.w);
            phi_4217_ = vec4<f32>(_e624.x, _e624.y, _e612.z, _e624.w);
        }
        let _e632 = phi_4217_;
        phi_4248_ = _e284;
        phi_4242_ = (_e632 * _e212);
    }
    let _e635 = phi_4248_;
    let _e637 = phi_4242_;
    let _e638 = _e637.xyz;
    let _e641 = j.E3_;
    let _e643 = j.F3_;
    if (fj && (_e637.w != 0f)) {
        phi_4243_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e79.x) + (0.00583715f * _e79.y))))) * _e641) + _e643)) + _e638);
    } else {
        phi_4243_ = _e638;
    }
    let _e659 = phi_4243_;
    let _e665 = vec4<f32>(_e659.x, _e637.y, _e637.z, _e637.w);
    let _e671 = vec4<f32>(_e665.x, _e659.y, _e665.z, _e665.w);
    let _e677 = vec4<f32>(_e671.x, _e671.y, _e659.z, _e671.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e659.x + _e659.y) + _e659.z) + _e637.w) == 0f) {
                break;
            }
            let _e683 = (1f - _e637.w);
            phi_4245_ = _e677;
            if (_e683 != 0f) {
                let _e687 = n0_.v2_[_e114];
                phi_4245_ = (_e677 + (unpack4x8unorm(_e687) * _e683));
            }
            let _e692 = phi_4245_;
            n0_.v2_[_e114] = pack4x8unorm(_e692);
            break;
        }
    }
    if (_e635 != 0u) {
        m0_.v2_[_e114] = _e635;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) @interpolate(flat, either) o1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    o1_1 = o1_;
    main_1();
}
