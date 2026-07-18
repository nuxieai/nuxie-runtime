struct Fe {
    c2_: array<vec2<u32>>,
}

struct h0xd {
    c2_: array<u32>,
}

struct Ge {
    c2_: array<vec4<f32>>,
}

struct j0xd {
    c2_: array<u32>,
}

struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct q4xd {
    c2_: array<u32>,
}

@id(7) override bh: bool = true;
@id(6) override ah: bool = true;
@id(4) override Yg: bool = true;
@id(0) override Ug: bool = true;
@id(1) override Vg: bool = true;
@id(2) override Wg: bool = true;

@group(0) @binding(3)
var<storage> AD: Fe;
@group(2) @binding(1)
var<storage, read_write> h0_: h0xd;
@group(0) @binding(4)
var<storage> RB: Ge;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(2) @binding(0)
var<storage, read_write> j0_: j0xd;
@group(0) @binding(0)
var<uniform> m: CC;
@group(2) @binding(3)
var<storage, read_write> q4_: q4xd;
var<private> A0_1: u32;
@group(0) @binding(10)
var BD: texture_2d<f32>;
@group(3) @binding(10)
var P9_: sampler;
var<private> B2_1: vec2<f32>;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1435_: bool;
    var phi_3453_: f32;
    var phi_3452_: f32;
    var phi_3454_: f32;
    var phi_3457_: f32;
    var phi_3456_: f32;
    var phi_1472_: bool;
    var phi_3459_: f32;
    var phi_4105_: u32;
    var phi_3458_: f32;
    var phi_4104_: u32;
    var phi_3480_: vec4<f32>;
    var phi_1591_: bool;
    var phi_3484_: u32;
    var phi_1600_: bool;
    var phi_3498_: f32;
    var phi_3952_: vec4<f32>;
    var phi_3892_: i32;
    var phi_4100_: vec4<f32>;
    var phi_4125_: vec3<f32>;
    var phi_4127_: vec4<f32>;

    let _e79 = gl_FragCoord_1;
    let _e80 = _e79.xy;
    let _e83 = bitcast<vec2<u32>>(vec2<i32>(floor(_e80)));
    let _e85 = m.m6_;
    let _e114 = bitcast<i32>((((((_e83.y >> bitcast<u32>(5u)) * (((_e85 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e83.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e83.x & 28u) << bitcast<u32>(5u)) + ((_e83.y & 28u) << bitcast<u32>(2i)))) + (((_e83.y & 3u) << bitcast<u32>(2i)) + (_e83.x & 3u))));
    let _e117 = q4_.c2_[_e114];
    let _e119 = (_e117 >> bitcast<u32>(17u));
    let _e120 = A0_1;
    let _e124 = B2_1;
    let _e125 = textureSampleLevel(BD, P9_, _e124, 0f);
    q4_.c2_[_e114] = (((_e120 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e125.x, 0f, 1f) * 2048f)))));
    let _e136 = ((f32((_e117 & 131071u)) * 0.00048828125f) + -32f);
    let _e139 = AD.c2_[_e119];
    phi_3452_ = _e136;
    if ((_e139.x & 768u) != 0u) {
        let _e143 = abs(_e136);
        phi_1435_ = Yg;
        if Yg {
            phi_1435_ = ((_e139.x & 512u) != 0u);
        }
        let _e147 = phi_1435_;
        phi_3453_ = _e143;
        if _e147 {
            phi_3453_ = (1f - abs(((fract((_e143 * 0.5f)) * 2f) + -1f)));
        }
        let _e155 = phi_3453_;
        phi_3452_ = _e155;
    }
    let _e157 = phi_3452_;
    let _e158 = clamp(_e157, 0f, 1f);
    phi_3456_ = _e158;
    if Ug {
        let _e160 = (_e139.x >> bitcast<u32>(16u));
        phi_3457_ = _e158;
        if (_e160 != 0u) {
            let _e164 = h0_.c2_[_e114];
            if (_e160 == (_e164 >> bitcast<u32>(16i))) {
                phi_3454_ = min(_e158, unpack2x16float(_e164).x);
            } else {
                phi_3454_ = 0f;
            }
            let _e172 = phi_3454_;
            phi_3457_ = _e172;
        }
        let _e174 = phi_3457_;
        phi_3456_ = _e174;
    }
    let _e176 = phi_3456_;
    phi_1472_ = Vg;
    if Vg {
        phi_1472_ = ((_e139.x & 1024u) != 0u);
    }
    let _e180 = phi_1472_;
    phi_3459_ = _e176;
    if _e180 {
        let _e181 = (_e119 * 4u);
        let _e185 = RB.c2_[(_e181 + 2u)];
        let _e196 = RB.c2_[(_e181 + 3u)];
        let _e201 = _e196.zw;
        let _e203 = ((abs(((mat2x2<f32>(vec2<f32>(_e185.x, _e185.y), vec2<f32>(_e185.z, _e185.w)) * _e80) + _e196.xy)) * _e201) - _e201);
        phi_3459_ = min(_e176, clamp((min(_e203.x, _e203.y) + 0.5f), 0f, 1f));
    }
    let _e211 = phi_3459_;
    let _e212 = (_e139.x & 15u);
    if (_e212 <= 1u) {
        let _e217 = (Ug && (_e212 == 0u));
        phi_4105_ = 0u;
        if _e217 {
            phi_4105_ = (_e139.y | pack2x16float(vec2<f32>(_e211, 0f)));
        }
        let _e222 = phi_4105_;
        phi_4104_ = _e222;
        phi_3480_ = select(unpack4x8unorm(_e139.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e217));
    } else {
        let _e225 = (_e119 * 4u);
        let _e228 = RB.c2_[_e225];
        let _e239 = RB.c2_[(_e225 + 1u)];
        let _e242 = ((mat2x2<f32>(vec2<f32>(_e228.x, _e228.y), vec2<f32>(_e228.z, _e228.w)) * _e80) + _e239.xy);
        if (_e212 == 2u) {
            phi_3458_ = _e242.x;
        } else {
            phi_3458_ = length(_e242);
        }
        let _e247 = phi_3458_;
        let _e256 = textureSampleLevel(KD, Jb, vec2<f32>(((clamp(_e247, 0f, 1f) * _e239.z) + _e239.w), bitcast<f32>(_e139.y)), 0f);
        phi_4104_ = 0u;
        phi_3480_ = _e256;
    }
    let _e258 = phi_4104_;
    let _e260 = phi_3480_;
    let _e262 = (_e260.w * _e211);
    let _e267 = vec4<f32>(_e260.x, _e260.y, _e260.z, _e262);
    phi_1591_ = Wg;
    if Wg {
        phi_1591_ = (_e262 != 0f);
    }
    let _e270 = phi_1591_;
    phi_3484_ = u32();
    phi_1600_ = _e270;
    if _e270 {
        let _e273 = ((_e139.x >> bitcast<u32>(4i)) & 15u);
        phi_3484_ = _e273;
        phi_1600_ = (_e273 != 0u);
    }
    let _e276 = phi_3484_;
    let _e278 = phi_1600_;
    phi_4100_ = _e267;
    if _e278 {
        let _e281 = j0_.c2_[_e114];
        let _e282 = unpack4x8unorm(_e281);
        let _e283 = _e267.xyz;
        local_2 = _e283;
        let _e284 = _e282.xyz;
        if (_e282.w != 0f) {
            phi_3498_ = (1f / _e282.w);
        } else {
            phi_3498_ = 0f;
        }
        let _e289 = phi_3498_;
        let _e290 = (_e284 * _e289);
        local = _e290;
        switch bitcast<i32>(_e276) {
            case 11: {
                let _e292 = local_2;
                local_1 = (_e292 * _e290);
                break;
            }
            case 1: {
                let _e294 = local_2;
                local_1 = ((_e294 + _e290) - (_e294 * _e290));
                break;
            }
            case 2: {
                let _e298 = local_2;
                let _e299 = (_e298 * _e290);
                local_1 = (select(_e299, (((_e298 + _e290) - _e299) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e290 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e306 = local_2;
                local_1 = min(_e306, _e290);
                break;
            }
            case 4: {
                let _e308 = local_2;
                local_1 = max(_e308, _e290);
                break;
            }
            case 5: {
                let _e311 = clamp(_e284, vec3<f32>(0f, 0f, 0f), _e282.www);
                let _e317 = vec4<f32>(_e311.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e323 = vec4<f32>(_e317.x, _e311.y, _e317.z, _e317.w);
                let _e330 = local_2;
                let _e333 = (clamp((vec3<f32>(1f, 1f, 1f) - _e330), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e282.w);
                let _e334 = vec4<f32>(_e323.x, _e323.y, _e311.z, _e323.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e334 / _e333)), sign(_e334), (_e333 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e340 = local_2;
                local_2 = clamp(_e340, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e343 = clamp(_e284, vec3<f32>(0f, 0f, 0f), _e282.www);
                let _e349 = vec4<f32>(_e343.x, _e282.y, _e282.z, _e282.w);
                let _e355 = vec4<f32>(_e349.x, _e343.y, _e349.z, _e349.w);
                phi_3952_ = vec4<f32>(_e355.x, _e355.y, _e343.z, _e355.w);
                if (_e282.w == 0f) {
                    phi_3952_ = vec4<f32>(_e343.x, _e343.y, _e343.z, 1f);
                }
                let _e365 = phi_3952_;
                let _e369 = (vec3(_e365.w) - _e365.xyz);
                let _e370 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e369 / (_e370 * _e365.w))), sign(_e369), (_e370 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e378 = local_2;
                let _e379 = (_e378 * _e290);
                local_1 = (select(_e379, (((_e378 + _e290) - _e379) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e378 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_3892_ = 0i;
                loop {
                    let _e387 = phi_3892_;
                    if (_e387 < 3i) {
                        let _e390 = local_2[_e387];
                        if (_e390 <= 0.5f) {
                            let _e393 = local[_e387];
                            local_1[_e387] = (1f - _e393);
                        } else {
                            let _e397 = local[_e387];
                            if (_e397 <= 0.25f) {
                                let _e399 = local[_e387];
                                let _e402 = local[_e387];
                                local_1[_e387] = ((((16f * _e399) - 12f) * _e402) + 3f);
                            } else {
                                let _e406 = local[_e387];
                                local_1[_e387] = (inverseSqrt(_e406) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_3892_ = (_e387 + 1i);
                    }
                }
                let _e411 = local_2;
                let _e415 = local_1;
                local_1 = (_e290 + ((_e290 * ((_e411 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e415));
                break;
            }
            case 9: {
                let _e418 = local_2;
                local_1 = abs((_e290 - _e418));
                break;
            }
            case 10: {
                let _e421 = local_2;
                local_1 = ((_e421 + _e290) - ((_e421 * 2f) * _e290));
                break;
            }
            case 12: {
                if ah {
                    let _e426 = local_2;
                    let _e427 = clamp(_e426, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e427;
                    let _e442 = (_e427 - vec3(min(min(_e427.x, _e427.y), _e427.z)));
                    let _e450 = (_e442 * ((max(max(_e290.x, _e290.y), _e290.z) - min(min(_e290.x, _e290.y), _e290.z)) / max(0.000062f, max(max(_e442.x, _e442.y), _e442.z))));
                    let _e451 = dot(_e290, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e454 = (_e450 - vec3(dot(_e450, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e467 = (vec2<f32>(_e451, (1f - _e451)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e454.x, _e454.y), _e454.z)), max(max(_e454.x, _e454.y), _e454.z))));
                    local_1 = ((_e454 * min(1f, min(_e467.x, _e467.y))) + vec3(_e451));
                }
                break;
            }
            case 13: {
                if ah {
                    let _e475 = local_2;
                    let _e476 = clamp(_e475, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e476;
                    let _e491 = (_e290 - vec3(min(min(_e290.x, _e290.y), _e290.z)));
                    let _e499 = (_e491 * ((max(max(_e476.x, _e476.y), _e476.z) - min(min(_e476.x, _e476.y), _e476.z)) / max(0.000062f, max(max(_e491.x, _e491.y), _e491.z))));
                    let _e500 = dot(_e290, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e503 = (_e499 - vec3(dot(_e499, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e516 = (vec2<f32>(_e500, (1f - _e500)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e503.x, _e503.y), _e503.z)), max(max(_e503.x, _e503.y), _e503.z))));
                    local_1 = ((_e503 * min(1f, min(_e516.x, _e516.y))) + vec3(_e500));
                }
                break;
            }
            case 14: {
                if ah {
                    let _e524 = local_2;
                    let _e525 = clamp(_e524, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e525;
                    let _e526 = dot(_e290, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e529 = (_e525 - vec3(dot(_e525, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e542 = (vec2<f32>(_e526, (1f - _e526)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e529.x, _e529.y), _e529.z)), max(max(_e529.x, _e529.y), _e529.z))));
                    local_1 = ((_e529 * min(1f, min(_e542.x, _e542.y))) + vec3(_e526));
                }
                break;
            }
            case 15: {
                if ah {
                    let _e550 = local_2;
                    let _e551 = clamp(_e550, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e551;
                    let _e552 = dot(_e551, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e555 = (_e290 - vec3(dot(_e290, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e568 = (vec2<f32>(_e552, (1f - _e552)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e555.x, _e555.y), _e555.z)), max(max(_e555.x, _e555.y), _e555.z))));
                    local_1 = ((_e555 * min(1f, min(_e568.x, _e568.y))) + vec3(_e552));
                }
                break;
            }
            default: {
            }
        }
        let _e576 = local_1;
        let _e578 = mix(_e283, _e576, vec3(_e282.w));
        phi_4100_ = vec4<f32>(_e578.x, _e578.y, _e578.z, _e262);
    }
    let _e584 = phi_4100_;
    let _e587 = (_e584.xyz * _e584.w);
    let _e593 = vec4<f32>(_e587.x, _e584.y, _e584.z, _e584.w);
    let _e599 = vec4<f32>(_e593.x, _e587.y, _e593.z, _e593.w);
    let _e605 = vec4<f32>(_e599.x, _e599.y, _e587.z, _e599.w);
    let _e606 = _e605.xyz;
    let _e608 = m.y3_;
    let _e610 = m.z3_;
    if bh {
        phi_4125_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e79.x) + (0.00583715f * _e79.y))))) * _e608) + _e610)) + _e606);
    } else {
        phi_4125_ = _e606;
    }
    let _e624 = phi_4125_;
    let _e630 = vec4<f32>(_e624.x, _e605.y, _e605.z, _e605.w);
    let _e636 = vec4<f32>(_e630.x, _e624.y, _e630.z, _e630.w);
    let _e642 = vec4<f32>(_e636.x, _e636.y, _e624.z, _e636.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e584.w == 0f) {
                break;
            }
            let _e645 = (1f - _e584.w);
            phi_4127_ = _e642;
            if (_e645 != 0f) {
                let _e649 = j0_.c2_[_e114];
                phi_4127_ = (_e642 + (unpack4x8unorm(_e649) * _e645));
            }
            let _e654 = phi_4127_;
            j0_.c2_[_e114] = pack4x8unorm(_e654);
            break;
        }
    }
    if (_e258 != 0u) {
        h0_.c2_[_e114] = _e258;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat) A0_: u32, @location(0) B2_: vec2<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    A0_1 = A0_;
    B2_1 = B2_;
    main_1();
}
