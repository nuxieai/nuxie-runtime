struct ug {
    v2_: array<vec2<u32>>,
}

struct m0cf {
    v2_: array<u32>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct n0cf {
    v2_: array<u32>,
}

struct W4cf {
    v2_: array<u32>,
}

@id(8) override rj: bool = true;
@id(7) override qj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

@group(0) @binding(3)
var<storage> XC: ug;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(4)
var<storage> JB: vg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0cf;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> I0_1: u32;
@group(0) @binding(10)
var JD: texture_2d<f32>;
@group(3) @binding(10)
var Ta: sampler;
var<private> U2_1: vec2<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1618_: bool;
    var phi_3786_: f32;
    var phi_3785_: f32;
    var phi_3787_: f32;
    var phi_3790_: f32;
    var phi_3789_: f32;
    var phi_1655_: bool;
    var phi_3794_: f32;
    var phi_3791_: f32;
    var phi_3792_: f32;
    var phi_3793_: vec2<f32>;
    var phi_3814_: vec4<f32>;
    var phi_4480_: u32;
    var phi_4479_: u32;
    var phi_3812_: vec4<f32>;
    var phi_3821_: f32;
    var phi_4243_: i32;
    var phi_4307_: vec4<f32>;
    var phi_4464_: vec4<f32>;
    var phi_4477_: vec4<f32>;
    var phi_4504_: vec3<f32>;
    var phi_4506_: vec4<f32>;

    let _e86 = gl_FragCoord_1;
    let _e87 = _e86.xy;
    let _e90 = bitcast<vec2<u32>>(vec2<i32>(floor(_e87)));
    let _e92 = j.T6_;
    let _e121 = bitcast<i32>((((((_e90.y >> bitcast<u32>(5u)) * (((_e92 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e90.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e90.x & 28u) << bitcast<u32>(5u)) + ((_e90.y & 28u) << bitcast<u32>(2i)))) + (((_e90.y & 3u) << bitcast<u32>(2i)) + (_e90.x & 3u))));
    let _e124 = W4_.v2_[_e121];
    let _e126 = (_e124 >> bitcast<u32>(17u));
    let _e127 = I0_1;
    let _e131 = U2_1;
    let _e132 = textureSampleLevel(JD, Ta, _e131, 0f);
    W4_.v2_[_e121] = (((_e127 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e132.x, 0f, 1f) * 2048f)))));
    let _e143 = ((f32((_e124 & 131071u)) * 0.00048828125f) + -32f);
    let _e146 = XC.v2_[_e126];
    phi_3785_ = _e143;
    if ((_e146.x & 768u) != 0u) {
        let _e150 = abs(_e143);
        phi_1618_ = oj;
        if oj {
            phi_1618_ = ((_e146.x & 512u) != 0u);
        }
        let _e154 = phi_1618_;
        phi_3786_ = _e150;
        if _e154 {
            phi_3786_ = (1f - abs(((fract((_e150 * 0.5f)) * 2f) + -1f)));
        }
        let _e162 = phi_3786_;
        phi_3785_ = _e162;
    }
    let _e164 = phi_3785_;
    let _e165 = clamp(_e164, 0f, 1f);
    phi_3789_ = _e165;
    if jj {
        let _e167 = (_e146.x >> bitcast<u32>(16u));
        phi_3790_ = _e165;
        if (_e167 != 0u) {
            let _e171 = m0_.v2_[_e121];
            if (_e167 == (_e171 >> bitcast<u32>(16i))) {
                phi_3787_ = min(_e165, unpack2x16float(_e171).x);
            } else {
                phi_3787_ = 0f;
            }
            let _e179 = phi_3787_;
            phi_3790_ = _e179;
        }
        let _e181 = phi_3790_;
        phi_3789_ = _e181;
    }
    let _e183 = phi_3789_;
    phi_1655_ = kj;
    if kj {
        phi_1655_ = ((_e146.x & 1024u) != 0u);
    }
    let _e187 = phi_1655_;
    phi_3794_ = _e183;
    if _e187 {
        let _e188 = (_e126 * 8u);
        let _e192 = JB.v2_[(_e188 + 2u)];
        let _e203 = JB.v2_[(_e188 + 3u)];
        let _e208 = _e203.zw;
        let _e210 = ((abs(((mat2x2<f32>(vec2<f32>(_e192.x, _e192.y), vec2<f32>(_e192.z, _e192.w)) * _e87) + _e203.xy)) * _e208) - _e208);
        phi_3794_ = min(_e183, clamp((min(_e210.x, _e210.y) + 0.5f), 0f, 1f));
    }
    let _e218 = phi_3794_;
    let _e219 = (_e146.x & 15u);
    let _e222 = ((_e146.x >> bitcast<u32>(4i)) & 15u);
    let _e224 = (lj && (_e222 != 0u));
    if (_e219 <= 1u) {
        let _e301 = (jj && (_e219 == 0u));
        phi_4480_ = 0u;
        if _e301 {
            phi_4480_ = (_e146.y | pack2x16float(vec2<f32>(_e218, 0f)));
        }
        let _e306 = phi_4480_;
        phi_4479_ = _e306;
        phi_3812_ = select(unpack4x8unorm(_e146.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e301));
    } else {
        let _e226 = (_e126 * 8u);
        let _e229 = JB.v2_[_e226];
        let _e240 = JB.v2_[(_e226 + 1u)];
        let _e243 = ((mat2x2<f32>(vec2<f32>(_e229.x, _e229.y), vec2<f32>(_e229.z, _e229.w)) * _e87) + _e240.xy);
        let _e247 = f32(_e219);
        let _e249 = j.Q8_;
        let _e251 = j.R8_;
        let _e256 = ((_e240.w * _e249) + _e251);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e247 == 4f)) {
                    let _e270 = normalize(_e243);
                    let _e273 = acos(clamp(_e270.x, -1f, 1f));
                    if (_e270.y >= 0f) {
                        phi_3792_ = _e273;
                    } else {
                        phi_3792_ = -(_e273);
                    }
                    let _e278 = phi_3792_;
                    phi_3793_ = vec2<f32>((_e278 * 0.15915494f), _e256);
                    break;
                } else {
                    if (_e247 == 2f) {
                        phi_3791_ = _e243.x;
                    } else {
                        phi_3791_ = length(_e243);
                    }
                    let _e264 = phi_3791_;
                    phi_3793_ = vec2<f32>(((clamp(_e264, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e240.z < 0f))) + ((max(0f, _e240.z) * 0.001953125f) + 0.0009765625f)), _e256);
                    break;
                }
            }
        }
        let _e282 = phi_3793_;
        let _e283 = textureSampleLevel(ZC, S8_, _e282, 0f);
        phi_3814_ = _e283;
        if !(_e224) {
            let _e287 = (_e283.xyz * _e283.w);
            phi_3814_ = vec4<f32>(_e287.x, _e287.y, _e287.z, (_e283.w * abs(bitcast<f32>(_e146.y))));
        }
        let _e297 = phi_3814_;
        phi_4479_ = 0u;
        phi_3812_ = _e297;
    }
    let _e310 = phi_4479_;
    let _e312 = phi_3812_;
    phi_4477_ = _e312;
    if _e224 {
        phi_4464_ = _e312;
        if ((_e312.w * _e218) != 0f) {
            let _e318 = n0_.v2_[_e121];
            let _e319 = unpack4x8unorm(_e318);
            let _e320 = _e312.xyz;
            local_2 = _e320;
            let _e321 = _e319.xyz;
            if (_e319.w != 0f) {
                phi_3821_ = (1f / _e319.w);
            } else {
                phi_3821_ = 0f;
            }
            let _e326 = phi_3821_;
            let _e327 = (_e321 * _e326);
            local = _e327;
            switch bitcast<i32>(_e222) {
                case 11: {
                    let _e611 = local_2;
                    local_1 = (_e611 * _e327);
                    break;
                }
                case 1: {
                    let _e607 = local_2;
                    local_1 = ((_e607 + _e327) - (_e607 * _e327));
                    break;
                }
                case 2: {
                    let _e599 = local_2;
                    let _e600 = (_e599 * _e327);
                    local_1 = (select(_e600, (((_e599 + _e327) - _e600) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e327 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e597 = local_2;
                    local_1 = min(_e597, _e327);
                    break;
                }
                case 4: {
                    let _e595 = local_2;
                    local_1 = max(_e595, _e327);
                    break;
                }
                case 5: {
                    let _e566 = clamp(_e321, vec3<f32>(0f, 0f, 0f), _e319.www);
                    let _e572 = vec4<f32>(_e566.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e578 = vec4<f32>(_e572.x, _e566.y, _e572.z, _e572.w);
                    let _e585 = local_2;
                    let _e588 = (clamp((vec3<f32>(1f, 1f, 1f) - _e585), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e319.w);
                    let _e589 = vec4<f32>(_e578.x, _e578.y, _e566.z, _e578.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e589 / _e588)), sign(_e589), (_e588 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e527 = local_2;
                    local_2 = clamp(_e527, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e530 = clamp(_e321, vec3<f32>(0f, 0f, 0f), _e319.www);
                    let _e536 = vec4<f32>(_e530.x, _e319.y, _e319.z, _e319.w);
                    let _e542 = vec4<f32>(_e536.x, _e530.y, _e536.z, _e536.w);
                    phi_4307_ = vec4<f32>(_e542.x, _e542.y, _e530.z, _e542.w);
                    if (_e319.w == 0f) {
                        phi_4307_ = vec4<f32>(_e530.x, _e530.y, _e530.z, 1f);
                    }
                    let _e552 = phi_4307_;
                    let _e556 = (vec3(_e552.w) - _e552.xyz);
                    let _e557 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e556 / (_e557 * _e552.w))), sign(_e556), (_e557 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e519 = local_2;
                    let _e520 = (_e519 * _e327);
                    local_1 = (select(_e520, (((_e519 + _e327) - _e520) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e519 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4243_ = 0i;
                    loop {
                        let _e488 = phi_4243_;
                        if (_e488 < 3i) {
                            let _e491 = local_2[_e488];
                            if (_e491 <= 0.5f) {
                                let _e508 = local[_e488];
                                local_1[_e488] = (1f - _e508);
                            } else {
                                let _e494 = local[_e488];
                                if (_e494 <= 0.25f) {
                                    let _e500 = local[_e488];
                                    let _e503 = local[_e488];
                                    local_1[_e488] = ((((16f * _e500) - 12f) * _e503) + 3f);
                                } else {
                                    let _e496 = local[_e488];
                                    local_1[_e488] = (inverseSqrt(_e496) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4243_ = (_e488 + 1i);
                        }
                    }
                    let _e512 = local_2;
                    let _e516 = local_1;
                    local_1 = (_e327 + ((_e327 * ((_e512 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e516));
                    break;
                }
                case 9: {
                    let _e484 = local_2;
                    local_1 = abs((_e327 - _e484));
                    break;
                }
                case 10: {
                    let _e479 = local_2;
                    local_1 = ((_e479 + _e327) - ((_e479 * 2f) * _e327));
                    break;
                }
                case 12: {
                    if qj {
                        let _e430 = local_2;
                        let _e431 = clamp(_e430, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e431;
                        let _e446 = (_e431 - vec3(min(min(_e431.x, _e431.y), _e431.z)));
                        let _e454 = (_e446 * ((max(max(_e327.x, _e327.y), _e327.z) - min(min(_e327.x, _e327.y), _e327.z)) / max(0.000062f, max(max(_e446.x, _e446.y), _e446.z))));
                        let _e455 = dot(_e327, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e458 = (_e454 - vec3(dot(_e454, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e471 = (vec2<f32>(_e455, (1f - _e455)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e458.x, _e458.y), _e458.z)), max(max(_e458.x, _e458.y), _e458.z))));
                        local_1 = ((_e458 * min(1f, min(_e471.x, _e471.y))) + vec3(_e455));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e381 = local_2;
                        let _e382 = clamp(_e381, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e382;
                        let _e397 = (_e327 - vec3(min(min(_e327.x, _e327.y), _e327.z)));
                        let _e405 = (_e397 * ((max(max(_e382.x, _e382.y), _e382.z) - min(min(_e382.x, _e382.y), _e382.z)) / max(0.000062f, max(max(_e397.x, _e397.y), _e397.z))));
                        let _e406 = dot(_e327, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e409 = (_e405 - vec3(dot(_e405, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e422 = (vec2<f32>(_e406, (1f - _e406)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e409.x, _e409.y), _e409.z)), max(max(_e409.x, _e409.y), _e409.z))));
                        local_1 = ((_e409 * min(1f, min(_e422.x, _e422.y))) + vec3(_e406));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e355 = local_2;
                        let _e356 = clamp(_e355, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e356;
                        let _e357 = dot(_e327, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e360 = (_e356 - vec3(dot(_e356, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e373 = (vec2<f32>(_e357, (1f - _e357)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e360.x, _e360.y), _e360.z)), max(max(_e360.x, _e360.y), _e360.z))));
                        local_1 = ((_e360 * min(1f, min(_e373.x, _e373.y))) + vec3(_e357));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e329 = local_2;
                        let _e330 = clamp(_e329, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e330;
                        let _e331 = dot(_e330, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e334 = (_e327 - vec3(dot(_e327, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e347 = (vec2<f32>(_e331, (1f - _e331)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e334.x, _e334.y), _e334.z)), max(max(_e334.x, _e334.y), _e334.z))));
                        local_1 = ((_e334 * min(1f, min(_e347.x, _e347.y))) + vec3(_e331));
                    }
                    break;
                }
                default: {
                }
            }
            let _e613 = local_1;
            let _e615 = mix(_e320, _e613, vec3(_e319.w));
            let _e621 = vec4<f32>(_e615.x, _e312.y, _e312.z, _e312.w);
            let _e627 = vec4<f32>(_e621.x, _e615.y, _e621.z, _e621.w);
            phi_4464_ = vec4<f32>(_e627.x, _e627.y, _e615.z, _e627.w);
        }
        let _e635 = phi_4464_;
        let _e638 = (_e635.xyz * _e635.w);
        let _e644 = vec4<f32>(_e638.x, _e635.y, _e635.z, _e635.w);
        let _e650 = vec4<f32>(_e644.x, _e638.y, _e644.z, _e644.w);
        phi_4477_ = vec4<f32>(_e650.x, _e650.y, _e638.z, _e650.w);
    }
    let _e658 = phi_4477_;
    let _e659 = (_e658 * _e218);
    let _e660 = _e659.xyz;
    let _e663 = j.H3_;
    let _e665 = j.I3_;
    if (rj && (_e659.w != 0f)) {
        phi_4504_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e86.x) + (0.00583715f * _e86.y))))) * _e663) + _e665)) + _e660);
    } else {
        phi_4504_ = _e660;
    }
    let _e681 = phi_4504_;
    let _e687 = vec4<f32>(_e681.x, _e659.y, _e659.z, _e659.w);
    let _e693 = vec4<f32>(_e687.x, _e681.y, _e687.z, _e687.w);
    let _e699 = vec4<f32>(_e693.x, _e693.y, _e681.z, _e693.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e681.x + _e681.y) + _e681.z) + _e659.w) == 0f) {
                break;
            }
            phi_4506_ = _e699;
            if (_e659.w != 1f) {
                let _e708 = n0_.v2_[_e121];
                phi_4506_ = (_e699 + (unpack4x8unorm(_e708) * (1f - _e659.w)));
            }
            let _e714 = phi_4506_;
            n0_.v2_[_e121] = pack4x8unorm(_e714);
            break;
        }
    }
    if (_e310 != 0u) {
        m0_.v2_[_e121] = _e310;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) I0_: u32, @location(0) U2_: vec2<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    I0_1 = I0_;
    U2_1 = U2_;
    main_1();
}
