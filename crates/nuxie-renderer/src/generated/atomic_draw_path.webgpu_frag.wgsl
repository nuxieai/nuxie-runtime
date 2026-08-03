struct He {
    c2_: array<vec2<u32>>,
}

struct h0zd {
    c2_: array<u32>,
}

struct Ie {
    c2_: array<vec4<f32>>,
}

struct j0zd {
    c2_: array<u32>,
}

struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct q4zd {
    c2_: array<u32>,
}

struct q4zd_1 {
    c2_: array<atomic<u32>>,
}

@id(7) override dh: bool = true;
@id(6) override ch: bool = true;
@id(4) override ah: bool = true;
@id(0) override Wg: bool = true;
@id(1) override Xg: bool = true;
@id(2) override Yg: bool = true;
@id(3) override Zg: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(3)
var<storage> AD: He;
@group(2) @binding(1)
var<storage, read_write> h0_: h0zd;
@group(0) @binding(4)
var<storage> RB: Ie;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(2) @binding(0)
var<storage, read_write> j0_: j0zd;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> O_1: vec4<f32>;
var<private> B0_1: u32;
@group(2) @binding(3)
var<storage, read_write> q4_: q4zd_1;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1427_: bool;
    var phi_1440_: bool;
    var phi_3981_: f32;
    var phi_3989_: f32;
    var phi_3997_: f32;
    var phi_3996_: f32;
    var phi_1932_: bool;
    var phi_4000_: f32;
    var phi_3999_: f32;
    var phi_4001_: f32;
    var phi_4004_: f32;
    var phi_4003_: f32;
    var phi_1969_: bool;
    var phi_4006_: f32;
    var phi_4916_: u32;
    var phi_4005_: f32;
    var phi_4915_: u32;
    var phi_4039_: vec4<f32>;
    var phi_2088_: bool;
    var phi_4043_: u32;
    var phi_2097_: bool;
    var phi_4063_: f32;
    var phi_4709_: vec4<f32>;
    var phi_4625_: i32;
    var phi_4911_: vec4<f32>;
    var phi_4943_: u32;
    var phi_4936_: vec4<f32>;
    var phi_4938_: vec3<f32>;
    var phi_4940_: vec4<f32>;

    let _e93 = gl_FragCoord_1;
    let _e94 = _e93.xy;
    let _e97 = bitcast<vec2<u32>>(vec2<i32>(floor(_e94)));
    let _e99 = n.m6_;
    let _e128 = bitcast<i32>((((((_e97.y >> bitcast<u32>(5u)) * (((_e99 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e97.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e97.x & 28u) << bitcast<u32>(5u)) + ((_e97.y & 28u) << bitcast<u32>(2i)))) + (((_e97.y & 3u) << bitcast<u32>(2i)) + (_e97.x & 3u))));
    phi_1427_ = Zg;
    if Zg {
        let _e129 = O_1;
        phi_1427_ = (_e129.x < -1.5f);
    }
    let _e133 = phi_1427_;
    if _e133 {
        let _e134 = O_1;
        let _e138 = textureSampleLevel(XC, aa, vec2<f32>((3f + _e134.x), 0f), 0f);
        let _e144 = textureSampleLevel(XC, aa, vec2<f32>((1f - _e134.y), 0f), 0f);
        phi_3996_ = ((1f - _e138.x) - _e144.x);
    } else {
        phi_1440_ = Zg;
        if Zg {
            let _e147 = O_1;
            phi_1440_ = (_e147.y < -1.5f);
        }
        let _e151 = phi_1440_;
        if _e151 {
            let _e152 = O_1;
            let _e155 = max(_e152.w, 0f);
            if (_e152.z >= 0f) {
                let _e158 = textureSampleLevel(XC, aa, vec2<f32>(_e155, 0f), 0f);
                phi_3981_ = _e158.x;
            } else {
                phi_3981_ = 0f;
            }
            let _e161 = phi_3981_;
            phi_3989_ = _e161;
            if (abs(_e152.z) < 1000f) {
                let _e168 = (-2f - _e152.y);
                let _e170 = ((_e168 - _e155) * 0.5984134f);
                let _e173 = (vec4(_e155) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e170));
                let _e179 = ((_e173 * -(_e152.z)) + vec4(((_e168 * _e152.z) + (abs(_e152.x) - 0.25f))));
                let _e182 = textureSampleLevel(XC, aa, vec2<f32>(_e179.x, 0f), 0f);
                let _e185 = textureSampleLevel(XC, aa, vec2<f32>(_e179.y, 0f), 0f);
                let _e188 = textureSampleLevel(XC, aa, vec2<f32>(_e179.z, 0f), 0f);
                let _e191 = textureSampleLevel(XC, aa, vec2<f32>(_e179.w, 0f), 0f);
                let _e197 = (_e173 * 5.0959306f);
                phi_3989_ = (_e161 + (dot(vec4<f32>(_e182.x, _e185.x, _e188.x, _e191.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e197) * (_e197 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e170));
            }
            let _e206 = phi_3989_;
            phi_3997_ = (_e206 * sign(_e152.x));
        } else {
            let _e211 = O_1[0u];
            let _e213 = O_1[1u];
            phi_3997_ = min(min(_e211, abs(_e213)), 1f);
        }
        let _e218 = phi_3997_;
        phi_3996_ = _e218;
    }
    let _e220 = phi_3996_;
    let _e224 = u32(round(((_e220 * 2048f) + 65536f)));
    let _e225 = B0_1;
    let _e228 = ((_e225 << bitcast<u32>(17u)) | _e224);
    let _e231 = atomicMax((&q4_.c2_[_e128]), _e228);
    let _e233 = (_e231 >> bitcast<u32>(17u));
    if (_e233 == _e225) {
        let _e235 = O_1;
        if (_e235.y < 0f) {
            let _e242 = atomicAdd((&q4_.c2_[_e128]), ((_e224 + (_e231 - max(_e228, _e231))) - 65536u));
        }
        phi_4943_ = 0u;
        phi_4936_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e246 = ((f32((_e231 & 131071u)) * 0.00048828125f) + -32f);
        let _e249 = AD.c2_[_e233];
        phi_3999_ = _e246;
        if ((_e249.x & 768u) != 0u) {
            let _e253 = abs(_e246);
            phi_1932_ = ah;
            if ah {
                phi_1932_ = ((_e249.x & 512u) != 0u);
            }
            let _e257 = phi_1932_;
            phi_4000_ = _e253;
            if _e257 {
                phi_4000_ = (1f - abs(((fract((_e253 * 0.5f)) * 2f) + -1f)));
            }
            let _e265 = phi_4000_;
            phi_3999_ = _e265;
        }
        let _e267 = phi_3999_;
        let _e268 = clamp(_e267, 0f, 1f);
        phi_4003_ = _e268;
        if Wg {
            let _e270 = (_e249.x >> bitcast<u32>(16u));
            phi_4004_ = _e268;
            if (_e270 != 0u) {
                let _e274 = h0_.c2_[_e128];
                if (_e270 == (_e274 >> bitcast<u32>(16i))) {
                    phi_4001_ = min(_e268, unpack2x16float(_e274).x);
                } else {
                    phi_4001_ = 0f;
                }
                let _e282 = phi_4001_;
                phi_4004_ = _e282;
            }
            let _e284 = phi_4004_;
            phi_4003_ = _e284;
        }
        let _e286 = phi_4003_;
        phi_1969_ = Xg;
        if Xg {
            phi_1969_ = ((_e249.x & 1024u) != 0u);
        }
        let _e290 = phi_1969_;
        phi_4006_ = _e286;
        if _e290 {
            let _e291 = (_e233 * 4u);
            let _e295 = RB.c2_[(_e291 + 2u)];
            let _e306 = RB.c2_[(_e291 + 3u)];
            let _e311 = _e306.zw;
            let _e313 = ((abs(((mat2x2<f32>(vec2<f32>(_e295.x, _e295.y), vec2<f32>(_e295.z, _e295.w)) * _e94) + _e306.xy)) * _e311) - _e311);
            phi_4006_ = min(_e286, clamp((min(_e313.x, _e313.y) + 0.5f), 0f, 1f));
        }
        let _e321 = phi_4006_;
        let _e322 = (_e249.x & 15u);
        if (_e322 <= 1u) {
            let _e327 = (Wg && (_e322 == 0u));
            phi_4916_ = 0u;
            if _e327 {
                phi_4916_ = (_e249.y | pack2x16float(vec2<f32>(_e321, 0f)));
            }
            let _e332 = phi_4916_;
            phi_4915_ = _e332;
            phi_4039_ = select(unpack4x8unorm(_e249.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e327));
        } else {
            let _e335 = (_e233 * 4u);
            let _e338 = RB.c2_[_e335];
            let _e349 = RB.c2_[(_e335 + 1u)];
            let _e352 = ((mat2x2<f32>(vec2<f32>(_e338.x, _e338.y), vec2<f32>(_e338.z, _e338.w)) * _e94) + _e349.xy);
            if (_e322 == 2u) {
                phi_4005_ = _e352.x;
            } else {
                phi_4005_ = length(_e352);
            }
            let _e357 = phi_4005_;
            let _e366 = textureSampleLevel(KD, Kb, vec2<f32>(((clamp(_e357, 0f, 1f) * _e349.z) + _e349.w), bitcast<f32>(_e249.y)), 0f);
            phi_4915_ = 0u;
            phi_4039_ = _e366;
        }
        let _e368 = phi_4915_;
        let _e370 = phi_4039_;
        let _e372 = (_e370.w * _e321);
        let _e377 = vec4<f32>(_e370.x, _e370.y, _e370.z, _e372);
        phi_2088_ = Yg;
        if Yg {
            phi_2088_ = (_e372 != 0f);
        }
        let _e380 = phi_2088_;
        phi_4043_ = u32();
        phi_2097_ = _e380;
        if _e380 {
            let _e383 = ((_e249.x >> bitcast<u32>(4i)) & 15u);
            phi_4043_ = _e383;
            phi_2097_ = (_e383 != 0u);
        }
        let _e386 = phi_4043_;
        let _e388 = phi_2097_;
        phi_4911_ = _e377;
        if _e388 {
            let _e391 = j0_.c2_[_e128];
            let _e392 = unpack4x8unorm(_e391);
            let _e393 = _e377.xyz;
            local_2 = _e393;
            let _e394 = _e392.xyz;
            if (_e392.w != 0f) {
                phi_4063_ = (1f / _e392.w);
            } else {
                phi_4063_ = 0f;
            }
            let _e399 = phi_4063_;
            let _e400 = (_e394 * _e399);
            local = _e400;
            switch bitcast<i32>(_e386) {
                case 11: {
                    let _e402 = local_2;
                    local_1 = (_e402 * _e400);
                    break;
                }
                case 1: {
                    let _e404 = local_2;
                    local_1 = ((_e404 + _e400) - (_e404 * _e400));
                    break;
                }
                case 2: {
                    let _e408 = local_2;
                    let _e409 = (_e408 * _e400);
                    local_1 = (select(_e409, (((_e408 + _e400) - _e409) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e400 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e416 = local_2;
                    local_1 = min(_e416, _e400);
                    break;
                }
                case 4: {
                    let _e418 = local_2;
                    local_1 = max(_e418, _e400);
                    break;
                }
                case 5: {
                    let _e421 = clamp(_e394, vec3<f32>(0f, 0f, 0f), _e392.www);
                    let _e427 = vec4<f32>(_e421.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e433 = vec4<f32>(_e427.x, _e421.y, _e427.z, _e427.w);
                    let _e440 = local_2;
                    let _e443 = (clamp((vec3<f32>(1f, 1f, 1f) - _e440), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e392.w);
                    let _e444 = vec4<f32>(_e433.x, _e433.y, _e421.z, _e433.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e444 / _e443)), sign(_e444), (_e443 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e450 = local_2;
                    local_2 = clamp(_e450, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e453 = clamp(_e394, vec3<f32>(0f, 0f, 0f), _e392.www);
                    let _e459 = vec4<f32>(_e453.x, _e392.y, _e392.z, _e392.w);
                    let _e465 = vec4<f32>(_e459.x, _e453.y, _e459.z, _e459.w);
                    phi_4709_ = vec4<f32>(_e465.x, _e465.y, _e453.z, _e465.w);
                    if (_e392.w == 0f) {
                        phi_4709_ = vec4<f32>(_e453.x, _e453.y, _e453.z, 1f);
                    }
                    let _e475 = phi_4709_;
                    let _e479 = (vec3(_e475.w) - _e475.xyz);
                    let _e480 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e479 / (_e480 * _e475.w))), sign(_e479), (_e480 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e488 = local_2;
                    let _e489 = (_e488 * _e400);
                    local_1 = (select(_e489, (((_e488 + _e400) - _e489) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e488 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4625_ = 0i;
                    loop {
                        let _e497 = phi_4625_;
                        if (_e497 < 3i) {
                            let _e500 = local_2[_e497];
                            if (_e500 <= 0.5f) {
                                let _e503 = local[_e497];
                                local_1[_e497] = (1f - _e503);
                            } else {
                                let _e507 = local[_e497];
                                if (_e507 <= 0.25f) {
                                    let _e509 = local[_e497];
                                    let _e512 = local[_e497];
                                    local_1[_e497] = ((((16f * _e509) - 12f) * _e512) + 3f);
                                } else {
                                    let _e516 = local[_e497];
                                    local_1[_e497] = (inverseSqrt(_e516) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4625_ = (_e497 + 1i);
                        }
                    }
                    let _e521 = local_2;
                    let _e525 = local_1;
                    local_1 = (_e400 + ((_e400 * ((_e521 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e525));
                    break;
                }
                case 9: {
                    let _e528 = local_2;
                    local_1 = abs((_e400 - _e528));
                    break;
                }
                case 10: {
                    let _e531 = local_2;
                    local_1 = ((_e531 + _e400) - ((_e531 * 2f) * _e400));
                    break;
                }
                case 12: {
                    if ch {
                        let _e536 = local_2;
                        let _e537 = clamp(_e536, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e537;
                        let _e552 = (_e537 - vec3(min(min(_e537.x, _e537.y), _e537.z)));
                        let _e560 = (_e552 * ((max(max(_e400.x, _e400.y), _e400.z) - min(min(_e400.x, _e400.y), _e400.z)) / max(0.000062f, max(max(_e552.x, _e552.y), _e552.z))));
                        let _e561 = dot(_e400, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e564 = (_e560 - vec3(dot(_e560, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e577 = (vec2<f32>(_e561, (1f - _e561)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e564.x, _e564.y), _e564.z)), max(max(_e564.x, _e564.y), _e564.z))));
                        local_1 = ((_e564 * min(1f, min(_e577.x, _e577.y))) + vec3(_e561));
                    }
                    break;
                }
                case 13: {
                    if ch {
                        let _e585 = local_2;
                        let _e586 = clamp(_e585, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e586;
                        let _e601 = (_e400 - vec3(min(min(_e400.x, _e400.y), _e400.z)));
                        let _e609 = (_e601 * ((max(max(_e586.x, _e586.y), _e586.z) - min(min(_e586.x, _e586.y), _e586.z)) / max(0.000062f, max(max(_e601.x, _e601.y), _e601.z))));
                        let _e610 = dot(_e400, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e613 = (_e609 - vec3(dot(_e609, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e626 = (vec2<f32>(_e610, (1f - _e610)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e613.x, _e613.y), _e613.z)), max(max(_e613.x, _e613.y), _e613.z))));
                        local_1 = ((_e613 * min(1f, min(_e626.x, _e626.y))) + vec3(_e610));
                    }
                    break;
                }
                case 14: {
                    if ch {
                        let _e634 = local_2;
                        let _e635 = clamp(_e634, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e635;
                        let _e636 = dot(_e400, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e639 = (_e635 - vec3(dot(_e635, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e652 = (vec2<f32>(_e636, (1f - _e636)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e639.x, _e639.y), _e639.z)), max(max(_e639.x, _e639.y), _e639.z))));
                        local_1 = ((_e639 * min(1f, min(_e652.x, _e652.y))) + vec3(_e636));
                    }
                    break;
                }
                case 15: {
                    if ch {
                        let _e660 = local_2;
                        let _e661 = clamp(_e660, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e661;
                        let _e662 = dot(_e661, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e665 = (_e400 - vec3(dot(_e400, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e678 = (vec2<f32>(_e662, (1f - _e662)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e665.x, _e665.y), _e665.z)), max(max(_e665.x, _e665.y), _e665.z))));
                        local_1 = ((_e665 * min(1f, min(_e678.x, _e678.y))) + vec3(_e662));
                    }
                    break;
                }
                default: {
                }
            }
            let _e686 = local_1;
            let _e688 = mix(_e393, _e686, vec3(_e392.w));
            phi_4911_ = vec4<f32>(_e688.x, _e688.y, _e688.z, _e372);
        }
        let _e694 = phi_4911_;
        let _e697 = (_e694.xyz * _e694.w);
        let _e703 = vec4<f32>(_e697.x, _e694.y, _e694.z, _e694.w);
        let _e709 = vec4<f32>(_e703.x, _e697.y, _e703.z, _e703.w);
        phi_4943_ = _e368;
        phi_4936_ = vec4<f32>(_e709.x, _e709.y, _e697.z, _e709.w);
    }
    let _e717 = phi_4943_;
    let _e719 = phi_4936_;
    let _e720 = _e719.xyz;
    let _e723 = n.z3_;
    let _e725 = n.A3_;
    if (dh && (_e719.w != 0f)) {
        phi_4938_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e93.x) + (0.00583715f * _e93.y))))) * _e723) + _e725)) + _e720);
    } else {
        phi_4938_ = _e720;
    }
    let _e741 = phi_4938_;
    let _e747 = vec4<f32>(_e741.x, _e719.y, _e719.z, _e719.w);
    let _e753 = vec4<f32>(_e747.x, _e741.y, _e747.z, _e747.w);
    let _e759 = vec4<f32>(_e753.x, _e753.y, _e741.z, _e753.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e719.w == 0f) {
                break;
            }
            let _e762 = (1f - _e719.w);
            phi_4940_ = _e759;
            if (_e762 != 0f) {
                let _e766 = j0_.c2_[_e128];
                phi_4940_ = (_e759 + (unpack4x8unorm(_e766) * _e762));
            }
            let _e771 = phi_4940_;
            j0_.c2_[_e128] = pack4x8unorm(_e771);
            break;
        }
    }
    if (_e717 != 0u) {
        h0_.c2_[_e128] = _e717;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) O: vec4<f32>, @location(1) @interpolate(flat, either) B0_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    O_1 = O;
    B0_1 = B0_;
    main_1();
}
