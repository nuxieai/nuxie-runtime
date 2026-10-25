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
    var phi_1516_: bool;
    var phi_3648_: f32;
    var phi_3647_: f32;
    var phi_3649_: f32;
    var phi_3652_: f32;
    var phi_3651_: f32;
    var phi_1553_: bool;
    var phi_3670_: f32;
    var phi_3653_: f32;
    var phi_3654_: f32;
    var phi_3655_: vec2<f32>;
    var phi_3667_: vec4<f32>;
    var phi_3665_: vec4<f32>;
    var phi_3675_: f32;
    var phi_4098_: i32;
    var phi_4162_: vec4<f32>;
    var phi_4319_: vec4<f32>;
    var phi_4332_: vec4<f32>;
    var phi_4333_: vec3<f32>;
    var phi_4335_: vec4<f32>;

    let _e80 = gl_FragCoord_1;
    let _e81 = _e80.xy;
    let _e84 = bitcast<vec2<u32>>(vec2<i32>(floor(_e81)));
    let _e86 = j.T6_;
    let _e115 = bitcast<i32>((((((_e84.y >> bitcast<u32>(5u)) * (((_e86 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e84.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e84.x & 28u) << bitcast<u32>(5u)) + ((_e84.y & 28u) << bitcast<u32>(2i)))) + (((_e84.y & 3u) << bitcast<u32>(2i)) + (_e84.x & 3u))));
    let _e118 = W4_.v2_[_e115];
    let _e122 = ((f32((_e118 & 131071u)) * 0.00048828125f) + -32f);
    let _e124 = (_e118 >> bitcast<u32>(17u));
    let _e127 = XC.v2_[_e124];
    phi_3647_ = _e122;
    if ((_e127.x & 768u) != 0u) {
        let _e131 = abs(_e122);
        phi_1516_ = oj;
        if oj {
            phi_1516_ = ((_e127.x & 512u) != 0u);
        }
        let _e135 = phi_1516_;
        phi_3648_ = _e131;
        if _e135 {
            phi_3648_ = (1f - abs(((fract((_e131 * 0.5f)) * 2f) + -1f)));
        }
        let _e143 = phi_3648_;
        phi_3647_ = _e143;
    }
    let _e145 = phi_3647_;
    let _e146 = clamp(_e145, 0f, 1f);
    phi_3651_ = _e146;
    if jj {
        let _e148 = (_e127.x >> bitcast<u32>(16u));
        phi_3652_ = _e146;
        if (_e148 != 0u) {
            let _e152 = m0_.v2_[_e115];
            if (_e148 == (_e152 >> bitcast<u32>(16i))) {
                phi_3649_ = min(_e146, unpack2x16float(_e152).x);
            } else {
                phi_3649_ = 0f;
            }
            let _e160 = phi_3649_;
            phi_3652_ = _e160;
        }
        let _e162 = phi_3652_;
        phi_3651_ = _e162;
    }
    let _e164 = phi_3651_;
    phi_1553_ = kj;
    if kj {
        phi_1553_ = ((_e127.x & 1024u) != 0u);
    }
    let _e168 = phi_1553_;
    phi_3670_ = _e164;
    if _e168 {
        let _e169 = (_e124 * 8u);
        let _e173 = JB.v2_[(_e169 + 2u)];
        let _e184 = JB.v2_[(_e169 + 3u)];
        let _e189 = _e184.zw;
        let _e191 = ((abs(((mat2x2<f32>(vec2<f32>(_e173.x, _e173.y), vec2<f32>(_e173.z, _e173.w)) * _e81) + _e184.xy)) * _e189) - _e189);
        phi_3670_ = min(_e164, clamp((min(_e191.x, _e191.y) + 0.5f), 0f, 1f));
    }
    let _e199 = phi_3670_;
    let _e200 = (_e127.x & 15u);
    let _e203 = ((_e127.x >> bitcast<u32>(4i)) & 15u);
    let _e205 = (lj && (_e203 != 0u));
    if (_e200 <= 1u) {
        phi_3665_ = select(unpack4x8unorm(_e127.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((jj && (_e200 == 0u))));
    } else {
        let _e207 = (_e124 * 8u);
        let _e210 = JB.v2_[_e207];
        let _e221 = JB.v2_[(_e207 + 1u)];
        let _e224 = ((mat2x2<f32>(vec2<f32>(_e210.x, _e210.y), vec2<f32>(_e210.z, _e210.w)) * _e81) + _e221.xy);
        let _e228 = f32(_e200);
        let _e230 = j.Q8_;
        let _e232 = j.R8_;
        let _e237 = ((_e221.w * _e230) + _e232);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e228 == 4f)) {
                    let _e251 = normalize(_e224);
                    let _e254 = acos(clamp(_e251.x, -1f, 1f));
                    if (_e251.y >= 0f) {
                        phi_3654_ = _e254;
                    } else {
                        phi_3654_ = -(_e254);
                    }
                    let _e259 = phi_3654_;
                    phi_3655_ = vec2<f32>((_e259 * 0.15915494f), _e237);
                    break;
                } else {
                    if (_e228 == 2f) {
                        phi_3653_ = _e224.x;
                    } else {
                        phi_3653_ = length(_e224);
                    }
                    let _e245 = phi_3653_;
                    phi_3655_ = vec2<f32>(((clamp(_e245, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e221.z < 0f))) + ((max(0f, _e221.z) * 0.001953125f) + 0.0009765625f)), _e237);
                    break;
                }
            }
        }
        let _e263 = phi_3655_;
        let _e264 = textureSampleLevel(ZC, S8_, _e263, 0f);
        phi_3667_ = _e264;
        if !(_e205) {
            let _e268 = (_e264.xyz * _e264.w);
            phi_3667_ = vec4<f32>(_e268.x, _e268.y, _e268.z, (_e264.w * abs(bitcast<f32>(_e127.y))));
        }
        let _e278 = phi_3667_;
        phi_3665_ = _e278;
    }
    let _e286 = phi_3665_;
    phi_4332_ = _e286;
    if _e205 {
        phi_4319_ = _e286;
        if ((_e286.w * _e199) != 0f) {
            let _e292 = n0_.v2_[_e115];
            let _e293 = unpack4x8unorm(_e292);
            let _e294 = _e286.xyz;
            local_2 = _e294;
            let _e295 = _e293.xyz;
            if (_e293.w != 0f) {
                phi_3675_ = (1f / _e293.w);
            } else {
                phi_3675_ = 0f;
            }
            let _e300 = phi_3675_;
            let _e301 = (_e295 * _e300);
            local = _e301;
            switch bitcast<i32>(_e203) {
                case 11: {
                    let _e585 = local_2;
                    local_1 = (_e585 * _e301);
                    break;
                }
                case 1: {
                    let _e581 = local_2;
                    local_1 = ((_e581 + _e301) - (_e581 * _e301));
                    break;
                }
                case 2: {
                    let _e573 = local_2;
                    let _e574 = (_e573 * _e301);
                    local_1 = (select(_e574, (((_e573 + _e301) - _e574) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e301 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e571 = local_2;
                    local_1 = min(_e571, _e301);
                    break;
                }
                case 4: {
                    let _e569 = local_2;
                    local_1 = max(_e569, _e301);
                    break;
                }
                case 5: {
                    let _e540 = clamp(_e295, vec3<f32>(0f, 0f, 0f), _e293.www);
                    let _e546 = vec4<f32>(_e540.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e552 = vec4<f32>(_e546.x, _e540.y, _e546.z, _e546.w);
                    let _e559 = local_2;
                    let _e562 = (clamp((vec3<f32>(1f, 1f, 1f) - _e559), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e293.w);
                    let _e563 = vec4<f32>(_e552.x, _e552.y, _e540.z, _e552.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e563 / _e562)), sign(_e563), (_e562 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e501 = local_2;
                    local_2 = clamp(_e501, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e504 = clamp(_e295, vec3<f32>(0f, 0f, 0f), _e293.www);
                    let _e510 = vec4<f32>(_e504.x, _e293.y, _e293.z, _e293.w);
                    let _e516 = vec4<f32>(_e510.x, _e504.y, _e510.z, _e510.w);
                    phi_4162_ = vec4<f32>(_e516.x, _e516.y, _e504.z, _e516.w);
                    if (_e293.w == 0f) {
                        phi_4162_ = vec4<f32>(_e504.x, _e504.y, _e504.z, 1f);
                    }
                    let _e526 = phi_4162_;
                    let _e530 = (vec3(_e526.w) - _e526.xyz);
                    let _e531 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e530 / (_e531 * _e526.w))), sign(_e530), (_e531 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e493 = local_2;
                    let _e494 = (_e493 * _e301);
                    local_1 = (select(_e494, (((_e493 + _e301) - _e494) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e493 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4098_ = 0i;
                    loop {
                        let _e462 = phi_4098_;
                        if (_e462 < 3i) {
                            let _e465 = local_2[_e462];
                            if (_e465 <= 0.5f) {
                                let _e482 = local[_e462];
                                local_1[_e462] = (1f - _e482);
                            } else {
                                let _e468 = local[_e462];
                                if (_e468 <= 0.25f) {
                                    let _e474 = local[_e462];
                                    let _e477 = local[_e462];
                                    local_1[_e462] = ((((16f * _e474) - 12f) * _e477) + 3f);
                                } else {
                                    let _e470 = local[_e462];
                                    local_1[_e462] = (inverseSqrt(_e470) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4098_ = (_e462 + 1i);
                        }
                    }
                    let _e486 = local_2;
                    let _e490 = local_1;
                    local_1 = (_e301 + ((_e301 * ((_e486 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e490));
                    break;
                }
                case 9: {
                    let _e458 = local_2;
                    local_1 = abs((_e301 - _e458));
                    break;
                }
                case 10: {
                    let _e453 = local_2;
                    local_1 = ((_e453 + _e301) - ((_e453 * 2f) * _e301));
                    break;
                }
                case 12: {
                    if qj {
                        let _e404 = local_2;
                        let _e405 = clamp(_e404, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e405;
                        let _e420 = (_e405 - vec3(min(min(_e405.x, _e405.y), _e405.z)));
                        let _e428 = (_e420 * ((max(max(_e301.x, _e301.y), _e301.z) - min(min(_e301.x, _e301.y), _e301.z)) / max(0.000062f, max(max(_e420.x, _e420.y), _e420.z))));
                        let _e429 = dot(_e301, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e432 = (_e428 - vec3(dot(_e428, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e445 = (vec2<f32>(_e429, (1f - _e429)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e432.x, _e432.y), _e432.z)), max(max(_e432.x, _e432.y), _e432.z))));
                        local_1 = ((_e432 * min(1f, min(_e445.x, _e445.y))) + vec3(_e429));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e355 = local_2;
                        let _e356 = clamp(_e355, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e356;
                        let _e371 = (_e301 - vec3(min(min(_e301.x, _e301.y), _e301.z)));
                        let _e379 = (_e371 * ((max(max(_e356.x, _e356.y), _e356.z) - min(min(_e356.x, _e356.y), _e356.z)) / max(0.000062f, max(max(_e371.x, _e371.y), _e371.z))));
                        let _e380 = dot(_e301, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e383 = (_e379 - vec3(dot(_e379, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e396 = (vec2<f32>(_e380, (1f - _e380)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e383.x, _e383.y), _e383.z)), max(max(_e383.x, _e383.y), _e383.z))));
                        local_1 = ((_e383 * min(1f, min(_e396.x, _e396.y))) + vec3(_e380));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e329 = local_2;
                        let _e330 = clamp(_e329, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e330;
                        let _e331 = dot(_e301, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e334 = (_e330 - vec3(dot(_e330, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e347 = (vec2<f32>(_e331, (1f - _e331)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e334.x, _e334.y), _e334.z)), max(max(_e334.x, _e334.y), _e334.z))));
                        local_1 = ((_e334 * min(1f, min(_e347.x, _e347.y))) + vec3(_e331));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e303 = local_2;
                        let _e304 = clamp(_e303, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e304;
                        let _e305 = dot(_e304, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e308 = (_e301 - vec3(dot(_e301, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e321 = (vec2<f32>(_e305, (1f - _e305)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e308.x, _e308.y), _e308.z)), max(max(_e308.x, _e308.y), _e308.z))));
                        local_1 = ((_e308 * min(1f, min(_e321.x, _e321.y))) + vec3(_e305));
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
            phi_4319_ = vec4<f32>(_e601.x, _e601.y, _e589.z, _e601.w);
        }
        let _e609 = phi_4319_;
        let _e612 = (_e609.xyz * _e609.w);
        let _e618 = vec4<f32>(_e612.x, _e609.y, _e609.z, _e609.w);
        let _e624 = vec4<f32>(_e618.x, _e612.y, _e618.z, _e618.w);
        phi_4332_ = vec4<f32>(_e624.x, _e624.y, _e612.z, _e624.w);
    }
    let _e632 = phi_4332_;
    let _e633 = (_e632 * _e199);
    let _e634 = _e633.xyz;
    let _e637 = j.H3_;
    let _e639 = j.I3_;
    if (rj && (_e633.w != 0f)) {
        phi_4333_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e80.x) + (0.00583715f * _e80.y))))) * _e637) + _e639)) + _e634);
    } else {
        phi_4333_ = _e634;
    }
    let _e655 = phi_4333_;
    let _e661 = vec4<f32>(_e655.x, _e633.y, _e633.z, _e633.w);
    let _e667 = vec4<f32>(_e661.x, _e655.y, _e661.z, _e661.w);
    let _e673 = vec4<f32>(_e667.x, _e667.y, _e655.z, _e667.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e655.x + _e655.y) + _e655.z) + _e633.w) == 0f) {
                break;
            }
            phi_4335_ = _e673;
            if (_e633.w != 1f) {
                let _e682 = n0_.v2_[_e115];
                phi_4335_ = (_e673 + (unpack4x8unorm(_e682) * (1f - _e633.w)));
            }
            let _e688 = phi_4335_;
            n0_.v2_[_e115] = pack4x8unorm(_e688);
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
