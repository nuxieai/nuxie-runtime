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
    var phi_1521_: bool;
    var phi_3660_: f32;
    var phi_3659_: f32;
    var phi_3661_: f32;
    var phi_3664_: f32;
    var phi_3663_: f32;
    var phi_1558_: bool;
    var phi_3682_: f32;
    var phi_3665_: f32;
    var phi_3666_: f32;
    var phi_3667_: vec2<f32>;
    var phi_3679_: vec4<f32>;
    var phi_3677_: vec4<f32>;
    var phi_3687_: f32;
    var phi_4110_: i32;
    var phi_4174_: vec4<f32>;
    var phi_4331_: vec4<f32>;
    var phi_4344_: vec4<f32>;
    var phi_4345_: vec3<f32>;
    var phi_4347_: vec4<f32>;

    let _e80 = gl_FragCoord_1;
    let _e81 = _e80.xy;
    let _e84 = bitcast<vec2<u32>>(vec2<i32>(floor(_e81)));
    let _e86 = j.T6_;
    let _e115 = bitcast<i32>((((((_e84.y >> bitcast<u32>(5u)) * (((_e86 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e84.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e84.x & 28u) << bitcast<u32>(5u)) + ((_e84.y & 28u) << bitcast<u32>(2i)))) + (((_e84.y & 3u) << bitcast<u32>(2i)) + (_e84.x & 3u))));
    let _e118 = W4_.v2_[_e115];
    let _e122 = ((f32((_e118 & 131071u)) * 0.00048828125f) + -32f);
    let _e124 = (_e118 >> bitcast<u32>(17u));
    let _e127 = XC.v2_[_e124];
    phi_3659_ = _e122;
    if ((_e127.x & 768u) != 0u) {
        let _e131 = abs(_e122);
        phi_1521_ = oj;
        if oj {
            phi_1521_ = ((_e127.x & 512u) != 0u);
        }
        let _e135 = phi_1521_;
        phi_3660_ = _e131;
        if _e135 {
            phi_3660_ = (1f - abs(((fract((_e131 * 0.5f)) * 2f) + -1f)));
        }
        let _e143 = phi_3660_;
        phi_3659_ = _e143;
    }
    let _e145 = phi_3659_;
    let _e146 = clamp(_e145, 0f, 1f);
    phi_3663_ = _e146;
    if jj {
        let _e148 = (_e127.x >> bitcast<u32>(16u));
        phi_3664_ = _e146;
        if (_e148 != 0u) {
            let _e152 = m0_.v2_[_e115];
            if (_e148 == (_e152 >> bitcast<u32>(16i))) {
                phi_3661_ = min(_e146, unpack2x16float(_e152).x);
            } else {
                phi_3661_ = 0f;
            }
            let _e160 = phi_3661_;
            phi_3664_ = _e160;
        }
        let _e162 = phi_3664_;
        phi_3663_ = _e162;
    }
    let _e164 = phi_3663_;
    phi_1558_ = kj;
    if kj {
        phi_1558_ = ((_e127.x & 1024u) != 0u);
    }
    let _e168 = phi_1558_;
    phi_3682_ = _e164;
    if _e168 {
        let _e169 = (_e124 * 8u);
        let _e173 = JB.v2_[(_e169 + 2u)];
        let _e184 = JB.v2_[(_e169 + 3u)];
        let _e189 = _e184.zw;
        let _e191 = ((abs(((mat2x2<f32>(vec2<f32>(_e173.x, _e173.y), vec2<f32>(_e173.z, _e173.w)) * _e81) + _e184.xy)) * _e189) - _e189);
        phi_3682_ = min(_e164, clamp((min(_e191.x, _e191.y) + 0.5f), 0f, 1f));
    }
    let _e199 = phi_3682_;
    let _e200 = (_e127.x & 15u);
    let _e203 = ((_e127.x >> bitcast<u32>(4i)) & 15u);
    let _e205 = (lj && (_e203 != 0u));
    if (_e200 <= 1u) {
        phi_3677_ = select(unpack4x8unorm(_e127.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((jj && (_e200 == 0u))));
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
                    let _e255 = normalize(vec2<f32>(_e224.x, -(_e224.y)));
                    let _e258 = acos(clamp(_e255.x, -1f, 1f));
                    if (_e255.y >= 0f) {
                        phi_3666_ = _e258;
                    } else {
                        phi_3666_ = -(_e258);
                    }
                    let _e263 = phi_3666_;
                    phi_3667_ = vec2<f32>((_e263 * 0.15915494f), _e237);
                    break;
                } else {
                    if (_e228 == 2f) {
                        phi_3665_ = _e224.x;
                    } else {
                        phi_3665_ = length(_e224);
                    }
                    let _e245 = phi_3665_;
                    phi_3667_ = vec2<f32>(((clamp(_e245, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e221.z < 0f))) + ((max(0f, _e221.z) * 0.001953125f) + 0.0009765625f)), _e237);
                    break;
                }
            }
        }
        let _e267 = phi_3667_;
        let _e268 = textureSampleLevel(ZC, S8_, _e267, 0f);
        phi_3679_ = _e268;
        if !(_e205) {
            let _e272 = (_e268.xyz * _e268.w);
            phi_3679_ = vec4<f32>(_e272.x, _e272.y, _e272.z, (_e268.w * abs(bitcast<f32>(_e127.y))));
        }
        let _e282 = phi_3679_;
        phi_3677_ = _e282;
    }
    let _e290 = phi_3677_;
    phi_4344_ = _e290;
    if _e205 {
        phi_4331_ = _e290;
        if ((_e290.w * _e199) != 0f) {
            let _e296 = n0_.v2_[_e115];
            let _e297 = unpack4x8unorm(_e296);
            let _e298 = _e290.xyz;
            local_2 = _e298;
            let _e299 = _e297.xyz;
            if (_e297.w != 0f) {
                phi_3687_ = (1f / _e297.w);
            } else {
                phi_3687_ = 0f;
            }
            let _e304 = phi_3687_;
            let _e305 = (_e299 * _e304);
            local = _e305;
            switch bitcast<i32>(_e203) {
                case 11: {
                    let _e589 = local_2;
                    local_1 = (_e589 * _e305);
                    break;
                }
                case 1: {
                    let _e585 = local_2;
                    local_1 = ((_e585 + _e305) - (_e585 * _e305));
                    break;
                }
                case 2: {
                    let _e577 = local_2;
                    let _e578 = (_e577 * _e305);
                    local_1 = (select(_e578, (((_e577 + _e305) - _e578) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e305 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e575 = local_2;
                    local_1 = min(_e575, _e305);
                    break;
                }
                case 4: {
                    let _e573 = local_2;
                    local_1 = max(_e573, _e305);
                    break;
                }
                case 5: {
                    let _e544 = clamp(_e299, vec3<f32>(0f, 0f, 0f), _e297.www);
                    let _e550 = vec4<f32>(_e544.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e556 = vec4<f32>(_e550.x, _e544.y, _e550.z, _e550.w);
                    let _e563 = local_2;
                    let _e566 = (clamp((vec3<f32>(1f, 1f, 1f) - _e563), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e297.w);
                    let _e567 = vec4<f32>(_e556.x, _e556.y, _e544.z, _e556.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e567 / _e566)), sign(_e567), (_e566 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e505 = local_2;
                    local_2 = clamp(_e505, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e508 = clamp(_e299, vec3<f32>(0f, 0f, 0f), _e297.www);
                    let _e514 = vec4<f32>(_e508.x, _e297.y, _e297.z, _e297.w);
                    let _e520 = vec4<f32>(_e514.x, _e508.y, _e514.z, _e514.w);
                    phi_4174_ = vec4<f32>(_e520.x, _e520.y, _e508.z, _e520.w);
                    if (_e297.w == 0f) {
                        phi_4174_ = vec4<f32>(_e508.x, _e508.y, _e508.z, 1f);
                    }
                    let _e530 = phi_4174_;
                    let _e534 = (vec3(_e530.w) - _e530.xyz);
                    let _e535 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e534 / (_e535 * _e530.w))), sign(_e534), (_e535 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e497 = local_2;
                    let _e498 = (_e497 * _e305);
                    local_1 = (select(_e498, (((_e497 + _e305) - _e498) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e497 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_4110_ = 0i;
                    loop {
                        let _e466 = phi_4110_;
                        if (_e466 < 3i) {
                            let _e469 = local_2[_e466];
                            if (_e469 <= 0.5f) {
                                let _e486 = local[_e466];
                                local_1[_e466] = (1f - _e486);
                            } else {
                                let _e472 = local[_e466];
                                if (_e472 <= 0.25f) {
                                    let _e478 = local[_e466];
                                    let _e481 = local[_e466];
                                    local_1[_e466] = ((((16f * _e478) - 12f) * _e481) + 3f);
                                } else {
                                    let _e474 = local[_e466];
                                    local_1[_e466] = (inverseSqrt(_e474) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_4110_ = (_e466 + 1i);
                        }
                    }
                    let _e490 = local_2;
                    let _e494 = local_1;
                    local_1 = (_e305 + ((_e305 * ((_e490 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e494));
                    break;
                }
                case 9: {
                    let _e462 = local_2;
                    local_1 = abs((_e305 - _e462));
                    break;
                }
                case 10: {
                    let _e457 = local_2;
                    local_1 = ((_e457 + _e305) - ((_e457 * 2f) * _e305));
                    break;
                }
                case 12: {
                    if qj {
                        let _e408 = local_2;
                        let _e409 = clamp(_e408, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e409;
                        let _e424 = (_e409 - vec3(min(min(_e409.x, _e409.y), _e409.z)));
                        let _e432 = (_e424 * ((max(max(_e305.x, _e305.y), _e305.z) - min(min(_e305.x, _e305.y), _e305.z)) / max(0.000062f, max(max(_e424.x, _e424.y), _e424.z))));
                        let _e433 = dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e436 = (_e432 - vec3(dot(_e432, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e449 = (vec2<f32>(_e433, (1f - _e433)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e436.x, _e436.y), _e436.z)), max(max(_e436.x, _e436.y), _e436.z))));
                        local_1 = ((_e436 * min(1f, min(_e449.x, _e449.y))) + vec3(_e433));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e359 = local_2;
                        let _e360 = clamp(_e359, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e360;
                        let _e375 = (_e305 - vec3(min(min(_e305.x, _e305.y), _e305.z)));
                        let _e383 = (_e375 * ((max(max(_e360.x, _e360.y), _e360.z) - min(min(_e360.x, _e360.y), _e360.z)) / max(0.000062f, max(max(_e375.x, _e375.y), _e375.z))));
                        let _e384 = dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e387 = (_e383 - vec3(dot(_e383, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e400 = (vec2<f32>(_e384, (1f - _e384)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e387.x, _e387.y), _e387.z)), max(max(_e387.x, _e387.y), _e387.z))));
                        local_1 = ((_e387 * min(1f, min(_e400.x, _e400.y))) + vec3(_e384));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e333 = local_2;
                        let _e334 = clamp(_e333, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e334;
                        let _e335 = dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e338 = (_e334 - vec3(dot(_e334, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e351 = (vec2<f32>(_e335, (1f - _e335)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e338.x, _e338.y), _e338.z)), max(max(_e338.x, _e338.y), _e338.z))));
                        local_1 = ((_e338 * min(1f, min(_e351.x, _e351.y))) + vec3(_e335));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e307 = local_2;
                        let _e308 = clamp(_e307, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e308;
                        let _e309 = dot(_e308, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e312 = (_e305 - vec3(dot(_e305, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e325 = (vec2<f32>(_e309, (1f - _e309)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e312.x, _e312.y), _e312.z)), max(max(_e312.x, _e312.y), _e312.z))));
                        local_1 = ((_e312 * min(1f, min(_e325.x, _e325.y))) + vec3(_e309));
                    }
                    break;
                }
                default: {
                }
            }
            let _e591 = local_1;
            let _e593 = mix(_e298, _e591, vec3(_e297.w));
            let _e599 = vec4<f32>(_e593.x, _e290.y, _e290.z, _e290.w);
            let _e605 = vec4<f32>(_e599.x, _e593.y, _e599.z, _e599.w);
            phi_4331_ = vec4<f32>(_e605.x, _e605.y, _e593.z, _e605.w);
        }
        let _e613 = phi_4331_;
        let _e616 = (_e613.xyz * _e613.w);
        let _e622 = vec4<f32>(_e616.x, _e613.y, _e613.z, _e613.w);
        let _e628 = vec4<f32>(_e622.x, _e616.y, _e622.z, _e622.w);
        phi_4344_ = vec4<f32>(_e628.x, _e628.y, _e616.z, _e628.w);
    }
    let _e636 = phi_4344_;
    let _e637 = (_e636 * _e199);
    let _e638 = _e637.xyz;
    let _e641 = j.H3_;
    let _e643 = j.I3_;
    if (rj && (_e637.w != 0f)) {
        phi_4345_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e80.x) + (0.00583715f * _e80.y))))) * _e641) + _e643)) + _e638);
    } else {
        phi_4345_ = _e638;
    }
    let _e659 = phi_4345_;
    let _e665 = vec4<f32>(_e659.x, _e637.y, _e637.z, _e637.w);
    let _e671 = vec4<f32>(_e665.x, _e659.y, _e665.z, _e665.w);
    let _e677 = vec4<f32>(_e671.x, _e671.y, _e659.z, _e671.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e659.x + _e659.y) + _e659.z) + _e637.w) == 0f) {
                break;
            }
            phi_4347_ = _e677;
            if (_e637.w != 1f) {
                let _e686 = n0_.v2_[_e115];
                phi_4347_ = (_e677 + (unpack4x8unorm(_e686) * (1f - _e637.w)));
            }
            let _e692 = phi_4347_;
            n0_.v2_[_e115] = pack4x8unorm(_e692);
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
