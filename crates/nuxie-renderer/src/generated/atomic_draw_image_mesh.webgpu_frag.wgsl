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
var<private> N0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
var<private> B3_1: u32;
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
    var phi_5909_: f32;
    var phi_1547_: bool;
    var phi_5224_: f32;
    var phi_5223_: f32;
    var phi_5225_: f32;
    var phi_5228_: f32;
    var phi_5227_: f32;
    var phi_1584_: bool;
    var phi_5230_: f32;
    var phi_5876_: u32;
    var phi_5229_: f32;
    var phi_5253_: vec4<f32>;
    var phi_5875_: u32;
    var phi_5251_: vec4<f32>;
    var phi_5258_: f32;
    var phi_5712_: vec4<f32>;
    var phi_5652_: i32;
    var phi_5860_: vec4<f32>;
    var phi_5873_: vec4<f32>;
    var phi_1286_: bool;
    var phi_5899_: u32;
    var phi_5927_: f32;
    var phi_7306_: f32;
    var phi_1318_: bool;
    var phi_5963_: f32;
    var phi_5964_: f32;
    var phi_6993_: vec4<f32>;
    var phi_6861_: i32;
    var phi_7318_: vec4<f32>;
    var phi_7331_: vec3<f32>;
    var phi_7333_: vec4<f32>;

    let _e81 = gl_FragCoord_1;
    let _e82 = _e81.xy;
    let _e85 = bitcast<vec2<u32>>(vec2<i32>(floor(_e82)));
    let _e87 = j.q6_;
    let _e116 = bitcast<i32>((((((_e85.y >> bitcast<u32>(5u)) * (((_e87 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e85.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e85.x & 28u) << bitcast<u32>(5u)) + ((_e85.y & 28u) << bitcast<u32>(2i)))) + (((_e85.y & 3u) << bitcast<u32>(2i)) + (_e85.x & 3u))));
    let _e117 = c2_1;
    let _e118 = textureSample(GC, Y5_, _e117);
    phi_5909_ = 1f;
    if Fh {
        let _e119 = N0_1;
        let _e122 = min(_e119.xy, _e119.zw);
        phi_5909_ = clamp(min(_e122.x, _e122.y), 0f, 1f);
    }
    let _e128 = phi_5909_;
    let _e131 = z4_.g2_[_e116];
    let _e133 = (_e131 >> bitcast<u32>(17u));
    let _e137 = ((f32((_e131 & 131071u)) * 0.00048828125f) + -32f);
    let _e140 = CD.g2_[_e133];
    phi_5223_ = _e137;
    if ((_e140.x & 768u) != 0u) {
        let _e144 = abs(_e137);
        phi_1547_ = Ih;
        if Ih {
            phi_1547_ = ((_e140.x & 512u) != 0u);
        }
        let _e148 = phi_1547_;
        phi_5224_ = _e144;
        if _e148 {
            phi_5224_ = (1f - abs(((fract((_e144 * 0.5f)) * 2f) + -1f)));
        }
        let _e156 = phi_5224_;
        phi_5223_ = _e156;
    }
    let _e158 = phi_5223_;
    let _e159 = clamp(_e158, 0f, 1f);
    phi_5227_ = _e159;
    if Eh {
        let _e161 = (_e140.x >> bitcast<u32>(16u));
        phi_5228_ = _e159;
        if (_e161 != 0u) {
            let _e165 = i0_.g2_[_e116];
            if (_e161 == (_e165 >> bitcast<u32>(16i))) {
                phi_5225_ = min(_e159, unpack2x16float(_e165).x);
            } else {
                phi_5225_ = 0f;
            }
            let _e173 = phi_5225_;
            phi_5228_ = _e173;
        }
        let _e175 = phi_5228_;
        phi_5227_ = _e175;
    }
    let _e177 = phi_5227_;
    phi_1584_ = Fh;
    if Fh {
        phi_1584_ = ((_e140.x & 1024u) != 0u);
    }
    let _e181 = phi_1584_;
    phi_5230_ = _e177;
    if _e181 {
        let _e182 = (_e133 * 8u);
        let _e186 = PB.g2_[(_e182 + 2u)];
        let _e197 = PB.g2_[(_e182 + 3u)];
        let _e202 = _e197.zw;
        let _e204 = ((abs(((mat2x2<f32>(vec2<f32>(_e186.x, _e186.y), vec2<f32>(_e186.z, _e186.w)) * _e82) + _e197.xy)) * _e202) - _e202);
        phi_5230_ = min(_e177, clamp((min(_e204.x, _e204.y) + 0.5f), 0f, 1f));
    }
    let _e212 = phi_5230_;
    let _e213 = (_e140.x & 15u);
    let _e216 = ((_e140.x >> bitcast<u32>(4i)) & 15u);
    let _e218 = (Gh && (_e216 != 0u));
    if (_e213 <= 1u) {
        let _e223 = (Eh && (_e213 == 0u));
        phi_5876_ = 0u;
        if _e223 {
            phi_5876_ = (_e140.y | pack2x16float(vec2<f32>(_e212, 0f)));
        }
        let _e228 = phi_5876_;
        phi_5875_ = _e228;
        phi_5251_ = select(unpack4x8unorm(_e140.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e223));
    } else {
        let _e231 = (_e133 * 8u);
        let _e234 = PB.g2_[_e231];
        let _e245 = PB.g2_[(_e231 + 1u)];
        let _e248 = ((mat2x2<f32>(vec2<f32>(_e234.x, _e234.y), vec2<f32>(_e234.z, _e234.w)) * _e82) + _e245.xy);
        if (_e213 == 2u) {
            phi_5229_ = _e248.x;
        } else {
            phi_5229_ = length(_e248);
        }
        let _e253 = phi_5229_;
        let _e262 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e253, 0f, 1f) * _e245.z) + _e245.w), bitcast<f32>(_e140.y)), 0f);
        phi_5253_ = _e262;
        if !(_e218) {
            let _e266 = (_e262.xyz * _e262.w);
            let _e272 = vec4<f32>(_e266.x, _e262.y, _e262.z, _e262.w);
            let _e278 = vec4<f32>(_e272.x, _e266.y, _e272.z, _e272.w);
            phi_5253_ = vec4<f32>(_e278.x, _e278.y, _e266.z, _e278.w);
        }
        let _e286 = phi_5253_;
        phi_5875_ = 0u;
        phi_5251_ = _e286;
    }
    let _e288 = phi_5875_;
    let _e290 = phi_5251_;
    phi_5873_ = _e290;
    if _e218 {
        phi_5860_ = _e290;
        if ((_e290.w * _e212) != 0f) {
            let _e296 = l0_.g2_[_e116];
            let _e297 = unpack4x8unorm(_e296);
            let _e298 = _e290.xyz;
            local_5 = _e298;
            let _e299 = _e297.xyz;
            if (_e297.w != 0f) {
                phi_5258_ = (1f / _e297.w);
            } else {
                phi_5258_ = 0f;
            }
            let _e304 = phi_5258_;
            let _e305 = (_e299 * _e304);
            local_3 = _e305;
            switch bitcast<i32>(_e216) {
                case 11: {
                    let _e307 = local_5;
                    local_4 = (_e307 * _e305);
                    break;
                }
                case 1: {
                    let _e309 = local_5;
                    local_4 = ((_e309 + _e305) - (_e309 * _e305));
                    break;
                }
                case 2: {
                    let _e313 = local_5;
                    let _e314 = (_e313 * _e305);
                    local_4 = (select(_e314, (((_e313 + _e305) - _e314) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e305 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e321 = local_5;
                    local_4 = min(_e321, _e305);
                    break;
                }
                case 4: {
                    let _e323 = local_5;
                    local_4 = max(_e323, _e305);
                    break;
                }
                case 5: {
                    let _e326 = clamp(_e299, vec3<f32>(0f, 0f, 0f), _e297.www);
                    let _e332 = vec4<f32>(_e326.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e338 = vec4<f32>(_e332.x, _e326.y, _e332.z, _e332.w);
                    let _e345 = local_5;
                    let _e348 = (clamp((vec3<f32>(1f, 1f, 1f) - _e345), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e297.w);
                    let _e349 = vec4<f32>(_e338.x, _e338.y, _e326.z, _e338.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e349 / _e348)), sign(_e349), (_e348 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e355 = local_5;
                    local_5 = clamp(_e355, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e358 = clamp(_e299, vec3<f32>(0f, 0f, 0f), _e297.www);
                    let _e364 = vec4<f32>(_e358.x, _e297.y, _e297.z, _e297.w);
                    let _e370 = vec4<f32>(_e364.x, _e358.y, _e364.z, _e364.w);
                    phi_5712_ = vec4<f32>(_e370.x, _e370.y, _e358.z, _e370.w);
                    if (_e297.w == 0f) {
                        phi_5712_ = vec4<f32>(_e358.x, _e358.y, _e358.z, 1f);
                    }
                    let _e380 = phi_5712_;
                    let _e384 = (vec3(_e380.w) - _e380.xyz);
                    let _e385 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e384 / (_e385 * _e380.w))), sign(_e384), (_e385 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e393 = local_5;
                    let _e394 = (_e393 * _e305);
                    local_4 = (select(_e394, (((_e393 + _e305) - _e394) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e393 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_5652_ = 0i;
                    loop {
                        let _e402 = phi_5652_;
                        if (_e402 < 3i) {
                            let _e405 = local_5[_e402];
                            if (_e405 <= 0.5f) {
                                let _e408 = local_3[_e402];
                                local_4[_e402] = (1f - _e408);
                            } else {
                                let _e412 = local_3[_e402];
                                if (_e412 <= 0.25f) {
                                    let _e414 = local_3[_e402];
                                    let _e417 = local_3[_e402];
                                    local_4[_e402] = ((((16f * _e414) - 12f) * _e417) + 3f);
                                } else {
                                    let _e421 = local_3[_e402];
                                    local_4[_e402] = (inverseSqrt(_e421) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_5652_ = (_e402 + 1i);
                        }
                    }
                    let _e426 = local_5;
                    let _e430 = local_4;
                    local_4 = (_e305 + ((_e305 * ((_e426 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e430));
                    break;
                }
                case 9: {
                    let _e433 = local_5;
                    local_4 = abs((_e305 - _e433));
                    break;
                }
                case 10: {
                    let _e436 = local_5;
                    local_4 = ((_e436 + _e305) - ((_e436 * 2f) * _e305));
                    break;
                }
                case 12: {
                    if Kh {
                        let _e441 = local_5;
                        let _e442 = clamp(_e441, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e442;
                        let _e457 = (_e442 - vec3(min(min(_e442.x, _e442.y), _e442.z)));
                        let _e465 = (_e457 * ((max(max(_e305.x, _e305.y), _e305.z) - min(min(_e305.x, _e305.y), _e305.z)) / max(0.000062f, max(max(_e457.x, _e457.y), _e457.z))));
                        let _e466 = dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e469 = (_e465 - vec3(dot(_e465, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e482 = (vec2<f32>(_e466, (1f - _e466)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e469.x, _e469.y), _e469.z)), max(max(_e469.x, _e469.y), _e469.z))));
                        local_4 = ((_e469 * min(1f, min(_e482.x, _e482.y))) + vec3(_e466));
                    }
                    break;
                }
                case 13: {
                    if Kh {
                        let _e490 = local_5;
                        let _e491 = clamp(_e490, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e491;
                        let _e506 = (_e305 - vec3(min(min(_e305.x, _e305.y), _e305.z)));
                        let _e514 = (_e506 * ((max(max(_e491.x, _e491.y), _e491.z) - min(min(_e491.x, _e491.y), _e491.z)) / max(0.000062f, max(max(_e506.x, _e506.y), _e506.z))));
                        let _e515 = dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e518 = (_e514 - vec3(dot(_e514, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e531 = (vec2<f32>(_e515, (1f - _e515)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e518.x, _e518.y), _e518.z)), max(max(_e518.x, _e518.y), _e518.z))));
                        local_4 = ((_e518 * min(1f, min(_e531.x, _e531.y))) + vec3(_e515));
                    }
                    break;
                }
                case 14: {
                    if Kh {
                        let _e539 = local_5;
                        let _e540 = clamp(_e539, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e540;
                        let _e541 = dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e544 = (_e540 - vec3(dot(_e540, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e557 = (vec2<f32>(_e541, (1f - _e541)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e544.x, _e544.y), _e544.z)), max(max(_e544.x, _e544.y), _e544.z))));
                        local_4 = ((_e544 * min(1f, min(_e557.x, _e557.y))) + vec3(_e541));
                    }
                    break;
                }
                case 15: {
                    if Kh {
                        let _e565 = local_5;
                        let _e566 = clamp(_e565, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e566;
                        let _e567 = dot(_e566, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e570 = (_e305 - vec3(dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e583 = (vec2<f32>(_e567, (1f - _e567)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e570.x, _e570.y), _e570.z)), max(max(_e570.x, _e570.y), _e570.z))));
                        local_4 = ((_e570 * min(1f, min(_e583.x, _e583.y))) + vec3(_e567));
                    }
                    break;
                }
                default: {
                }
            }
            let _e591 = local_4;
            let _e593 = mix(_e298, _e591, vec3(_e297.w));
            let _e599 = vec4<f32>(_e593.x, _e290.y, _e290.z, _e290.w);
            let _e605 = vec4<f32>(_e599.x, _e593.y, _e599.z, _e599.w);
            phi_5860_ = vec4<f32>(_e605.x, _e605.y, _e593.z, _e605.w);
        }
        let _e613 = phi_5860_;
        let _e616 = (_e613.xyz * _e613.w);
        let _e622 = vec4<f32>(_e616.x, _e613.y, _e613.z, _e613.w);
        let _e628 = vec4<f32>(_e622.x, _e616.y, _e622.z, _e622.w);
        phi_5873_ = vec4<f32>(_e628.x, _e628.y, _e616.z, _e628.w);
    }
    let _e636 = phi_5873_;
    let _e637 = (_e636 * _e212);
    phi_1286_ = Eh;
    if Eh {
        let _e638 = B3_1;
        phi_1286_ = (_e638 != 0u);
    }
    let _e641 = phi_1286_;
    phi_7306_ = _e128;
    if _e641 {
        if (_e288 != 0u) {
            phi_5899_ = _e288;
        } else {
            let _e645 = i0_.g2_[_e116];
            phi_5899_ = _e645;
        }
        let _e647 = phi_5899_;
        let _e648 = B3_1;
        if (_e648 == (_e647 >> bitcast<u32>(16i))) {
            phi_5927_ = min(_e128, unpack2x16float(_e647).x);
        } else {
            phi_5927_ = 0f;
        }
        let _e656 = phi_5927_;
        phi_7306_ = _e656;
    }
    let _e658 = phi_7306_;
    let _e659 = J1_1;
    let _e660 = (_e118 * _e659);
    phi_1318_ = Gh;
    if Gh {
        let _e661 = C1_1;
        phi_1318_ = (_e661 != 0u);
    }
    let _e664 = phi_1318_;
    phi_7318_ = _e660;
    if _e664 {
        let _e667 = l0_.g2_[_e116];
        let _e672 = ((unpack4x8unorm(_e667) * (1f - _e637.w)) + _e637);
        if (_e660.w != 0f) {
            phi_5963_ = (1f / _e660.w);
        } else {
            phi_5963_ = 0f;
        }
        let _e678 = phi_5963_;
        let _e679 = (_e660.xyz * _e678);
        let _e680 = C1_1;
        local_2 = _e679;
        let _e681 = _e672.xyz;
        if (_e672.w != 0f) {
            phi_5964_ = (1f / _e672.w);
        } else {
            phi_5964_ = 0f;
        }
        let _e686 = phi_5964_;
        let _e687 = (_e681 * _e686);
        local = _e687;
        switch bitcast<i32>(_e680) {
            case 11: {
                let _e689 = local_2;
                local_1 = (_e689 * _e687);
                break;
            }
            case 1: {
                let _e691 = local_2;
                local_1 = ((_e691 + _e687) - (_e691 * _e687));
                break;
            }
            case 2: {
                let _e695 = local_2;
                let _e696 = (_e695 * _e687);
                local_1 = (select(_e696, (((_e695 + _e687) - _e696) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e687 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e703 = local_2;
                local_1 = min(_e703, _e687);
                break;
            }
            case 4: {
                let _e705 = local_2;
                local_1 = max(_e705, _e687);
                break;
            }
            case 5: {
                let _e708 = clamp(_e681, vec3<f32>(0f, 0f, 0f), _e672.www);
                let _e714 = vec4<f32>(_e708.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e720 = vec4<f32>(_e714.x, _e708.y, _e714.z, _e714.w);
                let _e727 = local_2;
                let _e730 = (clamp((vec3<f32>(1f, 1f, 1f) - _e727), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e672.w);
                let _e731 = vec4<f32>(_e720.x, _e720.y, _e708.z, _e720.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e731 / _e730)), sign(_e731), (_e730 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e737 = local_2;
                local_2 = clamp(_e737, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e740 = clamp(_e681, vec3<f32>(0f, 0f, 0f), _e672.www);
                let _e746 = vec4<f32>(_e740.x, _e672.y, _e672.z, _e672.w);
                let _e752 = vec4<f32>(_e746.x, _e740.y, _e746.z, _e746.w);
                phi_6993_ = vec4<f32>(_e752.x, _e752.y, _e740.z, _e752.w);
                if (_e672.w == 0f) {
                    phi_6993_ = vec4<f32>(_e740.x, _e740.y, _e740.z, 1f);
                }
                let _e762 = phi_6993_;
                let _e766 = (vec3(_e762.w) - _e762.xyz);
                let _e767 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e766 / (_e767 * _e762.w))), sign(_e766), (_e767 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e775 = local_2;
                let _e776 = (_e775 * _e687);
                local_1 = (select(_e776, (((_e775 + _e687) - _e776) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e775 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_6861_ = 0i;
                loop {
                    let _e784 = phi_6861_;
                    if (_e784 < 3i) {
                        let _e787 = local_2[_e784];
                        if (_e787 <= 0.5f) {
                            let _e790 = local[_e784];
                            local_1[_e784] = (1f - _e790);
                        } else {
                            let _e794 = local[_e784];
                            if (_e794 <= 0.25f) {
                                let _e796 = local[_e784];
                                let _e799 = local[_e784];
                                local_1[_e784] = ((((16f * _e796) - 12f) * _e799) + 3f);
                            } else {
                                let _e803 = local[_e784];
                                local_1[_e784] = (inverseSqrt(_e803) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_6861_ = (_e784 + 1i);
                    }
                }
                let _e808 = local_2;
                let _e812 = local_1;
                local_1 = (_e687 + ((_e687 * ((_e808 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e812));
                break;
            }
            case 9: {
                let _e815 = local_2;
                local_1 = abs((_e687 - _e815));
                break;
            }
            case 10: {
                let _e818 = local_2;
                local_1 = ((_e818 + _e687) - ((_e818 * 2f) * _e687));
                break;
            }
            case 12: {
                if Kh {
                    let _e823 = local_2;
                    let _e824 = clamp(_e823, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e824;
                    let _e839 = (_e824 - vec3(min(min(_e824.x, _e824.y), _e824.z)));
                    let _e847 = (_e839 * ((max(max(_e687.x, _e687.y), _e687.z) - min(min(_e687.x, _e687.y), _e687.z)) / max(0.000062f, max(max(_e839.x, _e839.y), _e839.z))));
                    let _e848 = dot(_e687, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e851 = (_e847 - vec3(dot(_e847, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e864 = (vec2<f32>(_e848, (1f - _e848)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e851.x, _e851.y), _e851.z)), max(max(_e851.x, _e851.y), _e851.z))));
                    local_1 = ((_e851 * min(1f, min(_e864.x, _e864.y))) + vec3(_e848));
                }
                break;
            }
            case 13: {
                if Kh {
                    let _e872 = local_2;
                    let _e873 = clamp(_e872, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e873;
                    let _e888 = (_e687 - vec3(min(min(_e687.x, _e687.y), _e687.z)));
                    let _e896 = (_e888 * ((max(max(_e873.x, _e873.y), _e873.z) - min(min(_e873.x, _e873.y), _e873.z)) / max(0.000062f, max(max(_e888.x, _e888.y), _e888.z))));
                    let _e897 = dot(_e687, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e900 = (_e896 - vec3(dot(_e896, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e913 = (vec2<f32>(_e897, (1f - _e897)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e900.x, _e900.y), _e900.z)), max(max(_e900.x, _e900.y), _e900.z))));
                    local_1 = ((_e900 * min(1f, min(_e913.x, _e913.y))) + vec3(_e897));
                }
                break;
            }
            case 14: {
                if Kh {
                    let _e921 = local_2;
                    let _e922 = clamp(_e921, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e922;
                    let _e923 = dot(_e687, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e926 = (_e922 - vec3(dot(_e922, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e939 = (vec2<f32>(_e923, (1f - _e923)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e926.x, _e926.y), _e926.z)), max(max(_e926.x, _e926.y), _e926.z))));
                    local_1 = ((_e926 * min(1f, min(_e939.x, _e939.y))) + vec3(_e923));
                }
                break;
            }
            case 15: {
                if Kh {
                    let _e947 = local_2;
                    let _e948 = clamp(_e947, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e948;
                    let _e949 = dot(_e948, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e952 = (_e687 - vec3(dot(_e687, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e965 = (vec2<f32>(_e949, (1f - _e949)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e952.x, _e952.y), _e952.z)), max(max(_e952.x, _e952.y), _e952.z))));
                    local_1 = ((_e952 * min(1f, min(_e965.x, _e965.y))) + vec3(_e949));
                }
                break;
            }
            default: {
            }
        }
        let _e973 = local_1;
        let _e976 = (mix(_e679, _e973, vec3(_e672.w)) * _e660.w);
        let _e982 = vec4<f32>(_e976.x, _e660.y, _e660.z, _e660.w);
        let _e988 = vec4<f32>(_e982.x, _e976.y, _e982.z, _e982.w);
        phi_7318_ = vec4<f32>(_e988.x, _e988.y, _e976.z, _e988.w);
    }
    let _e996 = phi_7318_;
    let _e997 = (_e996 * _e658);
    let _e1001 = ((_e637 * (1f - _e997.w)) + _e997);
    let _e1002 = _e1001.xyz;
    let _e1005 = j.F3_;
    let _e1007 = j.G3_;
    if (Lh && (_e1001.w != 0f)) {
        phi_7331_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e81.x) + (0.00583715f * _e81.y))))) * _e1005) + _e1007)) + _e1002);
    } else {
        phi_7331_ = _e1002;
    }
    let _e1023 = phi_7331_;
    let _e1029 = vec4<f32>(_e1023.x, _e1001.y, _e1001.z, _e1001.w);
    let _e1035 = vec4<f32>(_e1029.x, _e1023.y, _e1029.z, _e1029.w);
    let _e1041 = vec4<f32>(_e1035.x, _e1035.y, _e1023.z, _e1035.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e1001.w == 0f) {
                break;
            }
            let _e1044 = (1f - _e1001.w);
            phi_7333_ = _e1041;
            if (_e1044 != 0f) {
                let _e1048 = l0_.g2_[_e116];
                phi_7333_ = (_e1041 + (unpack4x8unorm(_e1048) * _e1044));
            }
            let _e1053 = phi_7333_;
            l0_.g2_[_e116] = pack4x8unorm(_e1053);
            break;
        }
    }
    if (_e288 != 0u) {
        i0_.g2_[_e116] = _e288;
    }
    z4_.g2_[_e116] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) c2_: vec2<f32>, @location(1) N0_: vec4<f32>, @location(4) @interpolate(flat, either) B3_: u32, @location(3) @interpolate(flat, either) J1_: vec4<f32>, @location(5) @interpolate(flat, either) C1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    c2_1 = c2_;
    N0_1 = N0_;
    B3_1 = B3_;
    J1_1 = J1_;
    C1_1 = C1_;
    main_1();
}
