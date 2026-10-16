struct lg {
    r2_: array<vec2<u32>>,
}

struct m0Re {
    r2_: array<u32>,
}

struct mg {
    r2_: array<vec4<f32>>,
}

struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Ci: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Gi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    G3_: f32,
    H3_: f32,
    He: f32,
    zi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct n0Re {
    r2_: array<u32>,
}

struct R4Re {
    r2_: array<u32>,
}

@id(7) override hj: bool = true;
@id(6) override gj: bool = true;
@id(4) override ej: bool = true;
@id(0) override aj: bool = true;
@id(1) override bj: bool = true;
@id(2) override cj: bool = true;

@group(0) @binding(3)
var<storage> XC: lg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Re;
@group(0) @binding(4)
var<storage> JB: mg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Re;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Re;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1519_: bool;
    var phi_3657_: f32;
    var phi_3656_: f32;
    var phi_3658_: f32;
    var phi_3661_: f32;
    var phi_3660_: f32;
    var phi_1556_: bool;
    var phi_3679_: f32;
    var phi_3662_: f32;
    var phi_3663_: f32;
    var phi_3664_: vec2<f32>;
    var phi_3676_: vec4<f32>;
    var phi_3674_: vec4<f32>;
    var phi_3684_: f32;
    var phi_4107_: i32;
    var phi_4171_: vec4<f32>;
    var phi_4328_: vec4<f32>;
    var phi_4341_: vec4<f32>;
    var phi_4342_: vec3<f32>;
    var phi_4344_: vec4<f32>;

    let _e79 = gl_FragCoord_1;
    let _e80 = _e79.xy;
    let _e83 = bitcast<vec2<u32>>(vec2<i32>(floor(_e80)));
    let _e85 = j.S6_;
    let _e114 = bitcast<i32>((((((_e83.y >> bitcast<u32>(5u)) * (((_e85 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e83.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e83.x & 28u) << bitcast<u32>(5u)) + ((_e83.y & 28u) << bitcast<u32>(2i)))) + (((_e83.y & 3u) << bitcast<u32>(2i)) + (_e83.x & 3u))));
    let _e117 = R4_.r2_[_e114];
    let _e121 = ((f32((_e117 & 131071u)) * 0.00048828125f) + -32f);
    let _e123 = (_e117 >> bitcast<u32>(17u));
    let _e126 = XC.r2_[_e123];
    phi_3656_ = _e121;
    if ((_e126.x & 768u) != 0u) {
        let _e130 = abs(_e121);
        phi_1519_ = ej;
        if ej {
            phi_1519_ = ((_e126.x & 512u) != 0u);
        }
        let _e134 = phi_1519_;
        phi_3657_ = _e130;
        if _e134 {
            phi_3657_ = (1f - abs(((fract((_e130 * 0.5f)) * 2f) + -1f)));
        }
        let _e142 = phi_3657_;
        phi_3656_ = _e142;
    }
    let _e144 = phi_3656_;
    let _e145 = clamp(_e144, 0f, 1f);
    phi_3660_ = _e145;
    if aj {
        let _e147 = (_e126.x >> bitcast<u32>(16u));
        phi_3661_ = _e145;
        if (_e147 != 0u) {
            let _e151 = m0_.r2_[_e114];
            if (_e147 == (_e151 >> bitcast<u32>(16i))) {
                phi_3658_ = min(_e145, unpack2x16float(_e151).x);
            } else {
                phi_3658_ = 0f;
            }
            let _e159 = phi_3658_;
            phi_3661_ = _e159;
        }
        let _e161 = phi_3661_;
        phi_3660_ = _e161;
    }
    let _e163 = phi_3660_;
    phi_1556_ = bj;
    if bj {
        phi_1556_ = ((_e126.x & 1024u) != 0u);
    }
    let _e167 = phi_1556_;
    phi_3679_ = _e163;
    if _e167 {
        let _e168 = (_e123 * 8u);
        let _e172 = JB.r2_[(_e168 + 2u)];
        let _e183 = JB.r2_[(_e168 + 3u)];
        let _e188 = _e183.zw;
        let _e190 = ((abs(((mat2x2<f32>(vec2<f32>(_e172.x, _e172.y), vec2<f32>(_e172.z, _e172.w)) * _e80) + _e183.xy)) * _e188) - _e188);
        phi_3679_ = min(_e163, clamp((min(_e190.x, _e190.y) + 0.5f), 0f, 1f));
    }
    let _e198 = phi_3679_;
    let _e199 = (_e126.x & 15u);
    let _e202 = ((_e126.x >> bitcast<u32>(4i)) & 15u);
    let _e204 = (cj && (_e202 != 0u));
    if (_e199 <= 1u) {
        phi_3674_ = select(unpack4x8unorm(_e126.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((aj && (_e199 == 0u))));
    } else {
        let _e206 = (_e123 * 8u);
        let _e209 = JB.r2_[_e206];
        let _e220 = JB.r2_[(_e206 + 1u)];
        let _e223 = ((mat2x2<f32>(vec2<f32>(_e209.x, _e209.y), vec2<f32>(_e209.z, _e209.w)) * _e80) + _e220.xy);
        let _e227 = f32(_e199);
        let _e229 = j.O8_;
        let _e231 = j.P8_;
        let _e236 = ((_e220.w * _e229) + _e231);
        switch bitcast<i32>(0u) {
            default: {
                if (_e227 == 4f) {
                    let _e253 = normalize(vec2<f32>(_e223.x, -(_e223.y)));
                    let _e256 = acos(clamp(_e253.x, -1f, 1f));
                    if (_e253.y >= 0f) {
                        phi_3663_ = _e256;
                    } else {
                        phi_3663_ = -(_e256);
                    }
                    let _e261 = phi_3663_;
                    phi_3664_ = vec2<f32>((_e261 * 0.15915507f), _e236);
                    break;
                } else {
                    if (_e227 == 2f) {
                        phi_3662_ = _e223.x;
                    } else {
                        phi_3662_ = length(_e223);
                    }
                    let _e243 = phi_3662_;
                    phi_3664_ = vec2<f32>(((clamp(_e243, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e220.z < 0f))) + ((max(0f, _e220.z) * 0.001953125f) + 0.0009765625f)), _e236);
                    break;
                }
            }
        }
        let _e265 = phi_3664_;
        let _e266 = textureSampleLevel(ZC, Q8_, _e265, 0f);
        phi_3676_ = _e266;
        if !(_e204) {
            let _e270 = (_e266.xyz * _e266.w);
            phi_3676_ = vec4<f32>(_e270.x, _e270.y, _e270.z, (_e266.w * abs(bitcast<f32>(_e126.y))));
        }
        let _e280 = phi_3676_;
        phi_3674_ = _e280;
    }
    let _e288 = phi_3674_;
    phi_4341_ = _e288;
    if _e204 {
        phi_4328_ = _e288;
        if ((_e288.w * _e198) != 0f) {
            let _e294 = n0_.r2_[_e114];
            let _e295 = unpack4x8unorm(_e294);
            let _e296 = _e288.xyz;
            local_2 = _e296;
            let _e297 = _e295.xyz;
            if (_e295.w != 0f) {
                phi_3684_ = (1f / _e295.w);
            } else {
                phi_3684_ = 0f;
            }
            let _e302 = phi_3684_;
            let _e303 = (_e297 * _e302);
            local = _e303;
            switch bitcast<i32>(_e202) {
                case 11: {
                    let _e587 = local_2;
                    local_1 = (_e587 * _e303);
                    break;
                }
                case 1: {
                    let _e583 = local_2;
                    local_1 = ((_e583 + _e303) - (_e583 * _e303));
                    break;
                }
                case 2: {
                    let _e575 = local_2;
                    let _e576 = (_e575 * _e303);
                    local_1 = (select(_e576, (((_e575 + _e303) - _e576) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e303 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e573 = local_2;
                    local_1 = min(_e573, _e303);
                    break;
                }
                case 4: {
                    let _e571 = local_2;
                    local_1 = max(_e571, _e303);
                    break;
                }
                case 5: {
                    let _e542 = clamp(_e297, vec3<f32>(0f, 0f, 0f), _e295.www);
                    let _e548 = vec4<f32>(_e542.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e554 = vec4<f32>(_e548.x, _e542.y, _e548.z, _e548.w);
                    let _e561 = local_2;
                    let _e564 = (clamp((vec3<f32>(1f, 1f, 1f) - _e561), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e295.w);
                    let _e565 = vec4<f32>(_e554.x, _e554.y, _e542.z, _e554.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e565 / _e564)), sign(_e565), (_e564 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e503 = local_2;
                    local_2 = clamp(_e503, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e506 = clamp(_e297, vec3<f32>(0f, 0f, 0f), _e295.www);
                    let _e512 = vec4<f32>(_e506.x, _e295.y, _e295.z, _e295.w);
                    let _e518 = vec4<f32>(_e512.x, _e506.y, _e512.z, _e512.w);
                    phi_4171_ = vec4<f32>(_e518.x, _e518.y, _e506.z, _e518.w);
                    if (_e295.w == 0f) {
                        phi_4171_ = vec4<f32>(_e506.x, _e506.y, _e506.z, 1f);
                    }
                    let _e528 = phi_4171_;
                    let _e532 = (vec3(_e528.w) - _e528.xyz);
                    let _e533 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e532 / (_e533 * _e528.w))), sign(_e532), (_e533 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e495 = local_2;
                    let _e496 = (_e495 * _e303);
                    local_1 = (select(_e496, (((_e495 + _e303) - _e496) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e495 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4107_ = 0i;
                    loop {
                        let _e464 = phi_4107_;
                        if (_e464 < 3i) {
                            let _e467 = local_2[_e464];
                            if (_e467 <= 0.5f) {
                                let _e484 = local[_e464];
                                local_1[_e464] = (1f - _e484);
                            } else {
                                let _e470 = local[_e464];
                                if (_e470 <= 0.25f) {
                                    let _e476 = local[_e464];
                                    let _e479 = local[_e464];
                                    local_1[_e464] = ((((16f * _e476) - 12f) * _e479) + 3f);
                                } else {
                                    let _e472 = local[_e464];
                                    local_1[_e464] = (inverseSqrt(_e472) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4107_ = (_e464 + 1i);
                        }
                    }
                    let _e488 = local_2;
                    let _e492 = local_1;
                    local_1 = (_e303 + ((_e303 * ((_e488 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e492));
                    break;
                }
                case 9: {
                    let _e460 = local_2;
                    local_1 = abs((_e303 - _e460));
                    break;
                }
                case 10: {
                    let _e455 = local_2;
                    local_1 = ((_e455 + _e303) - ((_e455 * 2f) * _e303));
                    break;
                }
                case 12: {
                    if gj {
                        let _e406 = local_2;
                        let _e407 = clamp(_e406, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e407;
                        let _e422 = (_e407 - vec3(min(min(_e407.x, _e407.y), _e407.z)));
                        let _e430 = (_e422 * ((max(max(_e303.x, _e303.y), _e303.z) - min(min(_e303.x, _e303.y), _e303.z)) / max(0.000062f, max(max(_e422.x, _e422.y), _e422.z))));
                        let _e431 = dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e434 = (_e430 - vec3(dot(_e430, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e447 = (vec2<f32>(_e431, (1f - _e431)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e434.x, _e434.y), _e434.z)), max(max(_e434.x, _e434.y), _e434.z))));
                        local_1 = ((_e434 * min(1f, min(_e447.x, _e447.y))) + vec3(_e431));
                    }
                    break;
                }
                case 13: {
                    if gj {
                        let _e357 = local_2;
                        let _e358 = clamp(_e357, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e358;
                        let _e373 = (_e303 - vec3(min(min(_e303.x, _e303.y), _e303.z)));
                        let _e381 = (_e373 * ((max(max(_e358.x, _e358.y), _e358.z) - min(min(_e358.x, _e358.y), _e358.z)) / max(0.000062f, max(max(_e373.x, _e373.y), _e373.z))));
                        let _e382 = dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e385 = (_e381 - vec3(dot(_e381, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e398 = (vec2<f32>(_e382, (1f - _e382)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e385.x, _e385.y), _e385.z)), max(max(_e385.x, _e385.y), _e385.z))));
                        local_1 = ((_e385 * min(1f, min(_e398.x, _e398.y))) + vec3(_e382));
                    }
                    break;
                }
                case 14: {
                    if gj {
                        let _e331 = local_2;
                        let _e332 = clamp(_e331, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e332;
                        let _e333 = dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e336 = (_e332 - vec3(dot(_e332, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e349 = (vec2<f32>(_e333, (1f - _e333)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e336.x, _e336.y), _e336.z)), max(max(_e336.x, _e336.y), _e336.z))));
                        local_1 = ((_e336 * min(1f, min(_e349.x, _e349.y))) + vec3(_e333));
                    }
                    break;
                }
                case 15: {
                    if gj {
                        let _e305 = local_2;
                        let _e306 = clamp(_e305, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e306;
                        let _e307 = dot(_e306, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e310 = (_e303 - vec3(dot(_e303, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e323 = (vec2<f32>(_e307, (1f - _e307)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e310.x, _e310.y), _e310.z)), max(max(_e310.x, _e310.y), _e310.z))));
                        local_1 = ((_e310 * min(1f, min(_e323.x, _e323.y))) + vec3(_e307));
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
            phi_4328_ = vec4<f32>(_e603.x, _e603.y, _e591.z, _e603.w);
        }
        let _e611 = phi_4328_;
        let _e614 = (_e611.xyz * _e611.w);
        let _e620 = vec4<f32>(_e614.x, _e611.y, _e611.z, _e611.w);
        let _e626 = vec4<f32>(_e620.x, _e614.y, _e620.z, _e620.w);
        phi_4341_ = vec4<f32>(_e626.x, _e626.y, _e614.z, _e626.w);
    }
    let _e634 = phi_4341_;
    let _e635 = (_e634 * _e198);
    let _e636 = _e635.xyz;
    let _e639 = j.G3_;
    let _e641 = j.H3_;
    if (hj && (_e635.w != 0f)) {
        phi_4342_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e79.x) + (0.00583715f * _e79.y))))) * _e639) + _e641)) + _e636);
    } else {
        phi_4342_ = _e636;
    }
    let _e657 = phi_4342_;
    let _e663 = vec4<f32>(_e657.x, _e635.y, _e635.z, _e635.w);
    let _e669 = vec4<f32>(_e663.x, _e657.y, _e663.z, _e663.w);
    let _e675 = vec4<f32>(_e669.x, _e669.y, _e657.z, _e669.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e657.x + _e657.y) + _e657.z) + _e635.w) == 0f) {
                break;
            }
            phi_4344_ = _e675;
            if (_e635.w != 1f) {
                let _e684 = n0_.r2_[_e114];
                phi_4344_ = (_e675 + (unpack4x8unorm(_e684) * (1f - _e635.w)));
            }
            let _e690 = phi_4344_;
            n0_.r2_[_e114] = pack4x8unorm(_e690);
            break;
        }
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
