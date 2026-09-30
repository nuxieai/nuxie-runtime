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
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
var<private> D0_1: u32;
@group(0) @binding(10)
var ED: texture_2d<f32>;
@group(3) @binding(10)
var T9_: sampler;
var<private> F2_1: vec2<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1460_: bool;
    var phi_3500_: f32;
    var phi_3499_: f32;
    var phi_3501_: f32;
    var phi_3504_: f32;
    var phi_3503_: f32;
    var phi_1497_: bool;
    var phi_3506_: f32;
    var phi_4108_: u32;
    var phi_3505_: f32;
    var phi_3526_: vec4<f32>;
    var phi_4107_: u32;
    var phi_3524_: vec4<f32>;
    var phi_3531_: f32;
    var phi_3953_: vec4<f32>;
    var phi_3897_: i32;
    var phi_4092_: vec4<f32>;
    var phi_4105_: vec4<f32>;
    var phi_4130_: vec3<f32>;
    var phi_4132_: vec4<f32>;

    let _e78 = gl_FragCoord_1;
    let _e79 = _e78.xy;
    let _e82 = bitcast<vec2<u32>>(vec2<i32>(floor(_e79)));
    let _e84 = j.q6_;
    let _e113 = bitcast<i32>((((((_e82.y >> bitcast<u32>(5u)) * (((_e84 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e82.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e82.x & 28u) << bitcast<u32>(5u)) + ((_e82.y & 28u) << bitcast<u32>(2i)))) + (((_e82.y & 3u) << bitcast<u32>(2i)) + (_e82.x & 3u))));
    let _e116 = z4_.g2_[_e113];
    let _e118 = (_e116 >> bitcast<u32>(17u));
    let _e119 = D0_1;
    let _e123 = F2_1;
    let _e124 = textureSampleLevel(ED, T9_, _e123, 0f);
    z4_.g2_[_e113] = (((_e119 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e124.x, 0f, 1f) * 2048f)))));
    let _e135 = ((f32((_e116 & 131071u)) * 0.00048828125f) + -32f);
    let _e138 = CD.g2_[_e118];
    phi_3499_ = _e135;
    if ((_e138.x & 768u) != 0u) {
        let _e142 = abs(_e135);
        phi_1460_ = Ih;
        if Ih {
            phi_1460_ = ((_e138.x & 512u) != 0u);
        }
        let _e146 = phi_1460_;
        phi_3500_ = _e142;
        if _e146 {
            phi_3500_ = (1f - abs(((fract((_e142 * 0.5f)) * 2f) + -1f)));
        }
        let _e154 = phi_3500_;
        phi_3499_ = _e154;
    }
    let _e156 = phi_3499_;
    let _e157 = clamp(_e156, 0f, 1f);
    phi_3503_ = _e157;
    if Eh {
        let _e159 = (_e138.x >> bitcast<u32>(16u));
        phi_3504_ = _e157;
        if (_e159 != 0u) {
            let _e163 = i0_.g2_[_e113];
            if (_e159 == (_e163 >> bitcast<u32>(16i))) {
                phi_3501_ = min(_e157, unpack2x16float(_e163).x);
            } else {
                phi_3501_ = 0f;
            }
            let _e171 = phi_3501_;
            phi_3504_ = _e171;
        }
        let _e173 = phi_3504_;
        phi_3503_ = _e173;
    }
    let _e175 = phi_3503_;
    phi_1497_ = Fh;
    if Fh {
        phi_1497_ = ((_e138.x & 1024u) != 0u);
    }
    let _e179 = phi_1497_;
    phi_3506_ = _e175;
    if _e179 {
        let _e180 = (_e118 * 8u);
        let _e184 = PB.g2_[(_e180 + 2u)];
        let _e195 = PB.g2_[(_e180 + 3u)];
        let _e200 = _e195.zw;
        let _e202 = ((abs(((mat2x2<f32>(vec2<f32>(_e184.x, _e184.y), vec2<f32>(_e184.z, _e184.w)) * _e79) + _e195.xy)) * _e200) - _e200);
        phi_3506_ = min(_e175, clamp((min(_e202.x, _e202.y) + 0.5f), 0f, 1f));
    }
    let _e210 = phi_3506_;
    let _e211 = (_e138.x & 15u);
    let _e214 = ((_e138.x >> bitcast<u32>(4i)) & 15u);
    let _e216 = (Gh && (_e214 != 0u));
    if (_e211 <= 1u) {
        let _e221 = (Eh && (_e211 == 0u));
        phi_4108_ = 0u;
        if _e221 {
            phi_4108_ = (_e138.y | pack2x16float(vec2<f32>(_e210, 0f)));
        }
        let _e226 = phi_4108_;
        phi_4107_ = _e226;
        phi_3524_ = select(unpack4x8unorm(_e138.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e221));
    } else {
        let _e229 = (_e118 * 8u);
        let _e232 = PB.g2_[_e229];
        let _e243 = PB.g2_[(_e229 + 1u)];
        let _e246 = ((mat2x2<f32>(vec2<f32>(_e232.x, _e232.y), vec2<f32>(_e232.z, _e232.w)) * _e79) + _e243.xy);
        if (_e211 == 2u) {
            phi_3505_ = _e246.x;
        } else {
            phi_3505_ = length(_e246);
        }
        let _e251 = phi_3505_;
        let _e260 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e251, 0f, 1f) * _e243.z) + _e243.w), bitcast<f32>(_e138.y)), 0f);
        phi_3526_ = _e260;
        if !(_e216) {
            let _e264 = (_e260.xyz * _e260.w);
            let _e270 = vec4<f32>(_e264.x, _e260.y, _e260.z, _e260.w);
            let _e276 = vec4<f32>(_e270.x, _e264.y, _e270.z, _e270.w);
            phi_3526_ = vec4<f32>(_e276.x, _e276.y, _e264.z, _e276.w);
        }
        let _e284 = phi_3526_;
        phi_4107_ = 0u;
        phi_3524_ = _e284;
    }
    let _e286 = phi_4107_;
    let _e288 = phi_3524_;
    phi_4105_ = _e288;
    if _e216 {
        phi_4092_ = _e288;
        if ((_e288.w * _e210) != 0f) {
            let _e294 = l0_.g2_[_e113];
            let _e295 = unpack4x8unorm(_e294);
            let _e296 = _e288.xyz;
            local_2 = _e296;
            let _e297 = _e295.xyz;
            if (_e295.w != 0f) {
                phi_3531_ = (1f / _e295.w);
            } else {
                phi_3531_ = 0f;
            }
            let _e302 = phi_3531_;
            let _e303 = (_e297 * _e302);
            local = _e303;
            switch bitcast<i32>(_e214) {
                case 11: {
                    let _e305 = local_2;
                    local_1 = (_e305 * _e303);
                    break;
                }
                case 1: {
                    let _e307 = local_2;
                    local_1 = ((_e307 + _e303) - (_e307 * _e303));
                    break;
                }
                case 2: {
                    let _e311 = local_2;
                    let _e312 = (_e311 * _e303);
                    local_1 = (select(_e312, (((_e311 + _e303) - _e312) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e303 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e319 = local_2;
                    local_1 = min(_e319, _e303);
                    break;
                }
                case 4: {
                    let _e321 = local_2;
                    local_1 = max(_e321, _e303);
                    break;
                }
                case 5: {
                    let _e324 = clamp(_e297, vec3<f32>(0f, 0f, 0f), _e295.www);
                    let _e330 = vec4<f32>(_e324.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e336 = vec4<f32>(_e330.x, _e324.y, _e330.z, _e330.w);
                    let _e343 = local_2;
                    let _e346 = (clamp((vec3<f32>(1f, 1f, 1f) - _e343), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e295.w);
                    let _e347 = vec4<f32>(_e336.x, _e336.y, _e324.z, _e336.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e347 / _e346)), sign(_e347), (_e346 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e353 = local_2;
                    local_2 = clamp(_e353, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e356 = clamp(_e297, vec3<f32>(0f, 0f, 0f), _e295.www);
                    let _e362 = vec4<f32>(_e356.x, _e295.y, _e295.z, _e295.w);
                    let _e368 = vec4<f32>(_e362.x, _e356.y, _e362.z, _e362.w);
                    phi_3953_ = vec4<f32>(_e368.x, _e368.y, _e356.z, _e368.w);
                    if (_e295.w == 0f) {
                        phi_3953_ = vec4<f32>(_e356.x, _e356.y, _e356.z, 1f);
                    }
                    let _e378 = phi_3953_;
                    let _e382 = (vec3(_e378.w) - _e378.xyz);
                    let _e383 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e382 / (_e383 * _e378.w))), sign(_e382), (_e383 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e391 = local_2;
                    let _e392 = (_e391 * _e303);
                    local_1 = (select(_e392, (((_e391 + _e303) - _e392) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e391 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3897_ = 0i;
                    loop {
                        let _e400 = phi_3897_;
                        if (_e400 < 3i) {
                            let _e403 = local_2[_e400];
                            if (_e403 <= 0.5f) {
                                let _e406 = local[_e400];
                                local_1[_e400] = (1f - _e406);
                            } else {
                                let _e410 = local[_e400];
                                if (_e410 <= 0.25f) {
                                    let _e412 = local[_e400];
                                    let _e415 = local[_e400];
                                    local_1[_e400] = ((((16f * _e412) - 12f) * _e415) + 3f);
                                } else {
                                    let _e419 = local[_e400];
                                    local_1[_e400] = (inverseSqrt(_e419) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3897_ = (_e400 + 1i);
                        }
                    }
                    let _e424 = local_2;
                    let _e428 = local_1;
                    local_1 = (_e303 + ((_e303 * ((_e424 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e428));
                    break;
                }
                case 9: {
                    let _e431 = local_2;
                    local_1 = abs((_e303 - _e431));
                    break;
                }
                case 10: {
                    let _e434 = local_2;
                    local_1 = ((_e434 + _e303) - ((_e434 * 2f) * _e303));
                    break;
                }
                case 12: {
                    if Kh {
                        let _e439 = local_2;
                        let _e440 = clamp(_e439, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e440;
                        let _e455 = (_e440 - vec3(min(min(_e440.x, _e440.y), _e440.z)));
                        let _e463 = (_e455 * ((max(max(_e303.x, _e303.y), _e303.z) - min(min(_e303.x, _e303.y), _e303.z)) / max(0.000062f, max(max(_e455.x, _e455.y), _e455.z))));
                        let _e464 = dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e467 = (_e463 - vec3(dot(_e463, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e480 = (vec2<f32>(_e464, (1f - _e464)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e467.x, _e467.y), _e467.z)), max(max(_e467.x, _e467.y), _e467.z))));
                        local_1 = ((_e467 * min(1f, min(_e480.x, _e480.y))) + vec3(_e464));
                    }
                    break;
                }
                case 13: {
                    if Kh {
                        let _e488 = local_2;
                        let _e489 = clamp(_e488, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e489;
                        let _e504 = (_e303 - vec3(min(min(_e303.x, _e303.y), _e303.z)));
                        let _e512 = (_e504 * ((max(max(_e489.x, _e489.y), _e489.z) - min(min(_e489.x, _e489.y), _e489.z)) / max(0.000062f, max(max(_e504.x, _e504.y), _e504.z))));
                        let _e513 = dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e516 = (_e512 - vec3(dot(_e512, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e529 = (vec2<f32>(_e513, (1f - _e513)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e516.x, _e516.y), _e516.z)), max(max(_e516.x, _e516.y), _e516.z))));
                        local_1 = ((_e516 * min(1f, min(_e529.x, _e529.y))) + vec3(_e513));
                    }
                    break;
                }
                case 14: {
                    if Kh {
                        let _e537 = local_2;
                        let _e538 = clamp(_e537, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e538;
                        let _e539 = dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e542 = (_e538 - vec3(dot(_e538, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e555 = (vec2<f32>(_e539, (1f - _e539)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e542.x, _e542.y), _e542.z)), max(max(_e542.x, _e542.y), _e542.z))));
                        local_1 = ((_e542 * min(1f, min(_e555.x, _e555.y))) + vec3(_e539));
                    }
                    break;
                }
                case 15: {
                    if Kh {
                        let _e563 = local_2;
                        let _e564 = clamp(_e563, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e564;
                        let _e565 = dot(_e564, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e568 = (_e303 - vec3(dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e581 = (vec2<f32>(_e565, (1f - _e565)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e568.x, _e568.y), _e568.z)), max(max(_e568.x, _e568.y), _e568.z))));
                        local_1 = ((_e568 * min(1f, min(_e581.x, _e581.y))) + vec3(_e565));
                    }
                    break;
                }
                default: {
                }
            }
            let _e589 = local_1;
            let _e591 = mix(_e296, _e589, vec3(_e295.w));
            let _e597 = vec4<f32>(_e591.x, _e288.y, _e288.z, _e288.w);
            let _e603 = vec4<f32>(_e597.x, _e591.y, _e597.z, _e597.w);
            phi_4092_ = vec4<f32>(_e603.x, _e603.y, _e591.z, _e603.w);
        }
        let _e611 = phi_4092_;
        let _e614 = (_e611.xyz * _e611.w);
        let _e620 = vec4<f32>(_e614.x, _e611.y, _e611.z, _e611.w);
        let _e626 = vec4<f32>(_e620.x, _e614.y, _e620.z, _e620.w);
        phi_4105_ = vec4<f32>(_e626.x, _e626.y, _e614.z, _e626.w);
    }
    let _e634 = phi_4105_;
    let _e635 = (_e634 * _e210);
    let _e636 = _e635.xyz;
    let _e639 = j.F3_;
    let _e641 = j.G3_;
    if (Lh && (_e635.w != 0f)) {
        phi_4130_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e78.x) + (0.00583715f * _e78.y))))) * _e639) + _e641)) + _e636);
    } else {
        phi_4130_ = _e636;
    }
    let _e657 = phi_4130_;
    let _e663 = vec4<f32>(_e657.x, _e635.y, _e635.z, _e635.w);
    let _e669 = vec4<f32>(_e663.x, _e657.y, _e663.z, _e663.w);
    let _e675 = vec4<f32>(_e669.x, _e669.y, _e657.z, _e669.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e635.w == 0f) {
                break;
            }
            let _e678 = (1f - _e635.w);
            phi_4132_ = _e675;
            if (_e678 != 0f) {
                let _e682 = l0_.g2_[_e113];
                phi_4132_ = (_e675 + (unpack4x8unorm(_e682) * _e678));
            }
            let _e687 = phi_4132_;
            l0_.g2_[_e113] = pack4x8unorm(_e687);
            break;
        }
    }
    if (_e286 != 0u) {
        i0_.g2_[_e113] = _e286;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) D0_: u32, @location(0) F2_: vec2<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    F2_1 = F2_;
    main_1();
}
