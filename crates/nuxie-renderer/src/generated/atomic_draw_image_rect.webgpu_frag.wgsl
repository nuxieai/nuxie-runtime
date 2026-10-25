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
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;
var<private> n2_1: vec2<f32>;
var<private> x5_1: f32;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> V3_1: u32;
var<private> y5_1: vec4<f32>;
var<private> Y1_1: vec4<f32>;
var<private> N1_1: u32;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6517_: f32;
    var phi_1797_: bool;
    var phi_5744_: f32;
    var phi_5743_: f32;
    var phi_5745_: f32;
    var phi_5748_: f32;
    var phi_5747_: f32;
    var phi_1834_: bool;
    var phi_5752_: f32;
    var phi_5749_: f32;
    var phi_5750_: f32;
    var phi_5751_: vec2<f32>;
    var phi_5775_: vec4<f32>;
    var phi_6482_: u32;
    var phi_6481_: u32;
    var phi_5773_: vec4<f32>;
    var phi_5782_: f32;
    var phi_6232_: i32;
    var phi_6300_: vec4<f32>;
    var phi_6466_: vec4<f32>;
    var phi_6479_: vec4<f32>;
    var phi_1489_: bool;
    var phi_6507_: u32;
    var phi_6537_: f32;
    var phi_8207_: f32;
    var phi_6569_: f32;
    var phi_6570_: f32;
    var phi_6571_: vec2<f32>;
    var phi_6606_: vec4<f32>;
    var phi_1557_: bool;
    var phi_6617_: f32;
    var phi_6618_: f32;
    var phi_7683_: i32;
    var phi_7839_: vec4<f32>;
    var phi_8222_: vec4<f32>;
    var phi_8235_: vec3<f32>;
    var phi_8237_: vec4<f32>;

    let _e92 = gl_FragCoord_1;
    let _e93 = _e92.xy;
    let _e96 = bitcast<vec2<u32>>(vec2<i32>(floor(_e93)));
    let _e98 = j.T6_;
    let _e127 = bitcast<i32>((((((_e96.y >> bitcast<u32>(5u)) * (((_e98 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e96.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e96.x & 28u) << bitcast<u32>(5u)) + ((_e96.y & 28u) << bitcast<u32>(2i)))) + (((_e96.y & 3u) << bitcast<u32>(2i)) + (_e96.x & 3u))));
    let _e128 = n2_1;
    let _e129 = textureSample(RB, v4_, _e128);
    let _e130 = x5_1;
    let _e131 = min(_e130, 1f);
    phi_6517_ = _e131;
    if kj {
        let _e132 = W0_1;
        let _e135 = min(_e132.xy, _e132.zw);
        phi_6517_ = clamp(min(_e135.x, _e135.y), 0f, _e131);
    }
    let _e141 = phi_6517_;
    let _e144 = W4_.v2_[_e127];
    let _e146 = (_e144 >> bitcast<u32>(17u));
    let _e150 = ((f32((_e144 & 131071u)) * 0.00048828125f) + -32f);
    let _e153 = XC.v2_[_e146];
    phi_5743_ = _e150;
    if ((_e153.x & 768u) != 0u) {
        let _e157 = abs(_e150);
        phi_1797_ = oj;
        if oj {
            phi_1797_ = ((_e153.x & 512u) != 0u);
        }
        let _e161 = phi_1797_;
        phi_5744_ = _e157;
        if _e161 {
            phi_5744_ = (1f - abs(((fract((_e157 * 0.5f)) * 2f) + -1f)));
        }
        let _e169 = phi_5744_;
        phi_5743_ = _e169;
    }
    let _e171 = phi_5743_;
    let _e172 = clamp(_e171, 0f, 1f);
    phi_5747_ = _e172;
    if jj {
        let _e174 = (_e153.x >> bitcast<u32>(16u));
        phi_5748_ = _e172;
        if (_e174 != 0u) {
            let _e178 = m0_.v2_[_e127];
            if (_e174 == (_e178 >> bitcast<u32>(16i))) {
                phi_5745_ = min(_e172, unpack2x16float(_e178).x);
            } else {
                phi_5745_ = 0f;
            }
            let _e186 = phi_5745_;
            phi_5748_ = _e186;
        }
        let _e188 = phi_5748_;
        phi_5747_ = _e188;
    }
    let _e190 = phi_5747_;
    phi_1834_ = kj;
    if kj {
        phi_1834_ = ((_e153.x & 1024u) != 0u);
    }
    let _e194 = phi_1834_;
    phi_5752_ = _e190;
    if _e194 {
        let _e195 = (_e146 * 8u);
        let _e199 = JB.v2_[(_e195 + 2u)];
        let _e210 = JB.v2_[(_e195 + 3u)];
        let _e215 = _e210.zw;
        let _e217 = ((abs(((mat2x2<f32>(vec2<f32>(_e199.x, _e199.y), vec2<f32>(_e199.z, _e199.w)) * _e93) + _e210.xy)) * _e215) - _e215);
        phi_5752_ = min(_e190, clamp((min(_e217.x, _e217.y) + 0.5f), 0f, 1f));
    }
    let _e225 = phi_5752_;
    let _e226 = (_e153.x & 15u);
    let _e229 = ((_e153.x >> bitcast<u32>(4i)) & 15u);
    let _e231 = (lj && (_e229 != 0u));
    if (_e226 <= 1u) {
        let _e308 = (jj && (_e226 == 0u));
        phi_6482_ = 0u;
        if _e308 {
            phi_6482_ = (_e153.y | pack2x16float(vec2<f32>(_e225, 0f)));
        }
        let _e313 = phi_6482_;
        phi_6481_ = _e313;
        phi_5773_ = select(unpack4x8unorm(_e153.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e308));
    } else {
        let _e233 = (_e146 * 8u);
        let _e236 = JB.v2_[_e233];
        let _e247 = JB.v2_[(_e233 + 1u)];
        let _e250 = ((mat2x2<f32>(vec2<f32>(_e236.x, _e236.y), vec2<f32>(_e236.z, _e236.w)) * _e93) + _e247.xy);
        let _e254 = f32(_e226);
        let _e256 = j.Q8_;
        let _e258 = j.R8_;
        let _e263 = ((_e247.w * _e256) + _e258);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e254 == 4f)) {
                    let _e277 = normalize(_e250);
                    let _e280 = acos(clamp(_e277.x, -1f, 1f));
                    if (_e277.y >= 0f) {
                        phi_5750_ = _e280;
                    } else {
                        phi_5750_ = -(_e280);
                    }
                    let _e285 = phi_5750_;
                    phi_5751_ = vec2<f32>((_e285 * 0.15915494f), _e263);
                    break;
                } else {
                    if (_e254 == 2f) {
                        phi_5749_ = _e250.x;
                    } else {
                        phi_5749_ = length(_e250);
                    }
                    let _e271 = phi_5749_;
                    phi_5751_ = vec2<f32>(((clamp(_e271, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e247.z < 0f))) + ((max(0f, _e247.z) * 0.001953125f) + 0.0009765625f)), _e263);
                    break;
                }
            }
        }
        let _e289 = phi_5751_;
        let _e290 = textureSampleLevel(ZC, S8_, _e289, 0f);
        phi_5775_ = _e290;
        if !(_e231) {
            let _e294 = (_e290.xyz * _e290.w);
            phi_5775_ = vec4<f32>(_e294.x, _e294.y, _e294.z, (_e290.w * abs(bitcast<f32>(_e153.y))));
        }
        let _e304 = phi_5775_;
        phi_6481_ = 0u;
        phi_5773_ = _e304;
    }
    let _e317 = phi_6481_;
    let _e319 = phi_5773_;
    phi_6479_ = _e319;
    if _e231 {
        phi_6466_ = _e319;
        if ((_e319.w * _e225) != 0f) {
            let _e325 = n0_.v2_[_e127];
            let _e326 = unpack4x8unorm(_e325);
            let _e327 = _e319.xyz;
            local_5 = _e327;
            let _e328 = _e326.xyz;
            if (_e326.w != 0f) {
                phi_5782_ = (1f / _e326.w);
            } else {
                phi_5782_ = 0f;
            }
            let _e333 = phi_5782_;
            let _e334 = (_e328 * _e333);
            local_3 = _e334;
            switch bitcast<i32>(_e229) {
                case 11: {
                    let _e618 = local_5;
                    local_4 = (_e618 * _e334);
                    break;
                }
                case 1: {
                    let _e614 = local_5;
                    local_4 = ((_e614 + _e334) - (_e614 * _e334));
                    break;
                }
                case 2: {
                    let _e606 = local_5;
                    let _e607 = (_e606 * _e334);
                    local_4 = (select(_e607, (((_e606 + _e334) - _e607) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e334 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e604 = local_5;
                    local_4 = min(_e604, _e334);
                    break;
                }
                case 4: {
                    let _e602 = local_5;
                    local_4 = max(_e602, _e334);
                    break;
                }
                case 5: {
                    let _e573 = clamp(_e328, vec3<f32>(0f, 0f, 0f), _e326.www);
                    let _e579 = vec4<f32>(_e573.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e585 = vec4<f32>(_e579.x, _e573.y, _e579.z, _e579.w);
                    let _e592 = local_5;
                    let _e595 = (clamp((vec3<f32>(1f, 1f, 1f) - _e592), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e326.w);
                    let _e596 = vec4<f32>(_e585.x, _e585.y, _e573.z, _e585.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e596 / _e595)), sign(_e596), (_e595 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e534 = local_5;
                    local_5 = clamp(_e534, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e537 = clamp(_e328, vec3<f32>(0f, 0f, 0f), _e326.www);
                    let _e543 = vec4<f32>(_e537.x, _e326.y, _e326.z, _e326.w);
                    let _e549 = vec4<f32>(_e543.x, _e537.y, _e543.z, _e543.w);
                    phi_6300_ = vec4<f32>(_e549.x, _e549.y, _e537.z, _e549.w);
                    if (_e326.w == 0f) {
                        phi_6300_ = vec4<f32>(_e537.x, _e537.y, _e537.z, 1f);
                    }
                    let _e559 = phi_6300_;
                    let _e563 = (vec3(_e559.w) - _e559.xyz);
                    let _e564 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e563 / (_e564 * _e559.w))), sign(_e563), (_e564 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e526 = local_5;
                    let _e527 = (_e526 * _e334);
                    local_4 = (select(_e527, (((_e526 + _e334) - _e527) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e526 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_6232_ = 0i;
                    loop {
                        let _e495 = phi_6232_;
                        if (_e495 < 3i) {
                            let _e498 = local_5[_e495];
                            if (_e498 <= 0.5f) {
                                let _e515 = local_3[_e495];
                                local_4[_e495] = (1f - _e515);
                            } else {
                                let _e501 = local_3[_e495];
                                if (_e501 <= 0.25f) {
                                    let _e507 = local_3[_e495];
                                    let _e510 = local_3[_e495];
                                    local_4[_e495] = ((((16f * _e507) - 12f) * _e510) + 3f);
                                } else {
                                    let _e503 = local_3[_e495];
                                    local_4[_e495] = (inverseSqrt(_e503) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_6232_ = (_e495 + 1i);
                        }
                    }
                    let _e519 = local_5;
                    let _e523 = local_4;
                    local_4 = (_e334 + ((_e334 * ((_e519 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e523));
                    break;
                }
                case 9: {
                    let _e491 = local_5;
                    local_4 = abs((_e334 - _e491));
                    break;
                }
                case 10: {
                    let _e486 = local_5;
                    local_4 = ((_e486 + _e334) - ((_e486 * 2f) * _e334));
                    break;
                }
                case 12: {
                    if qj {
                        let _e437 = local_5;
                        let _e438 = clamp(_e437, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e438;
                        let _e453 = (_e438 - vec3(min(min(_e438.x, _e438.y), _e438.z)));
                        let _e461 = (_e453 * ((max(max(_e334.x, _e334.y), _e334.z) - min(min(_e334.x, _e334.y), _e334.z)) / max(0.000062f, max(max(_e453.x, _e453.y), _e453.z))));
                        let _e462 = dot(_e334, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e465 = (_e461 - vec3(dot(_e461, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e478 = (vec2<f32>(_e462, (1f - _e462)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e465.x, _e465.y), _e465.z)), max(max(_e465.x, _e465.y), _e465.z))));
                        local_4 = ((_e465 * min(1f, min(_e478.x, _e478.y))) + vec3(_e462));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e388 = local_5;
                        let _e389 = clamp(_e388, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e389;
                        let _e404 = (_e334 - vec3(min(min(_e334.x, _e334.y), _e334.z)));
                        let _e412 = (_e404 * ((max(max(_e389.x, _e389.y), _e389.z) - min(min(_e389.x, _e389.y), _e389.z)) / max(0.000062f, max(max(_e404.x, _e404.y), _e404.z))));
                        let _e413 = dot(_e334, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e416 = (_e412 - vec3(dot(_e412, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e429 = (vec2<f32>(_e413, (1f - _e413)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e416.x, _e416.y), _e416.z)), max(max(_e416.x, _e416.y), _e416.z))));
                        local_4 = ((_e416 * min(1f, min(_e429.x, _e429.y))) + vec3(_e413));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e362 = local_5;
                        let _e363 = clamp(_e362, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e363;
                        let _e364 = dot(_e334, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e367 = (_e363 - vec3(dot(_e363, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e380 = (vec2<f32>(_e364, (1f - _e364)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e367.x, _e367.y), _e367.z)), max(max(_e367.x, _e367.y), _e367.z))));
                        local_4 = ((_e367 * min(1f, min(_e380.x, _e380.y))) + vec3(_e364));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e336 = local_5;
                        let _e337 = clamp(_e336, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e337;
                        let _e338 = dot(_e337, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e341 = (_e334 - vec3(dot(_e334, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e354 = (vec2<f32>(_e338, (1f - _e338)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e341.x, _e341.y), _e341.z)), max(max(_e341.x, _e341.y), _e341.z))));
                        local_4 = ((_e341 * min(1f, min(_e354.x, _e354.y))) + vec3(_e338));
                    }
                    break;
                }
                default: {
                }
            }
            let _e620 = local_4;
            let _e622 = mix(_e327, _e620, vec3(_e326.w));
            let _e628 = vec4<f32>(_e622.x, _e319.y, _e319.z, _e319.w);
            let _e634 = vec4<f32>(_e628.x, _e622.y, _e628.z, _e628.w);
            phi_6466_ = vec4<f32>(_e634.x, _e634.y, _e622.z, _e634.w);
        }
        let _e642 = phi_6466_;
        let _e645 = (_e642.xyz * _e642.w);
        let _e651 = vec4<f32>(_e645.x, _e642.y, _e642.z, _e642.w);
        let _e657 = vec4<f32>(_e651.x, _e645.y, _e651.z, _e651.w);
        phi_6479_ = vec4<f32>(_e657.x, _e657.y, _e645.z, _e657.w);
    }
    let _e665 = phi_6479_;
    let _e666 = (_e665 * _e225);
    phi_1489_ = jj;
    if jj {
        let _e667 = V3_1;
        phi_1489_ = (_e667 != 0u);
    }
    let _e670 = phi_1489_;
    phi_8207_ = _e141;
    if _e670 {
        if (_e317 != 0u) {
            phi_6507_ = _e317;
        } else {
            let _e674 = m0_.v2_[_e127];
            phi_6507_ = _e674;
        }
        let _e676 = phi_6507_;
        let _e677 = V3_1;
        if (_e677 == (_e676 >> bitcast<u32>(16i))) {
            phi_6537_ = min(_e141, unpack2x16float(_e676).x);
        } else {
            phi_6537_ = 0f;
        }
        let _e685 = phi_6537_;
        phi_8207_ = _e685;
    }
    let _e687 = phi_8207_;
    let _e689 = y5_1[3u];
    phi_6606_ = _e129;
    if (_e689 != 0f) {
        let _e691 = y5_1;
        let _e693 = j.Q8_;
        let _e695 = j.R8_;
        let _e698 = abs(_e691.z);
        let _e699 = floor(_e698);
        let _e702 = (((_e698 - _e699) * 8f) + 0.0009765625f);
        let _e704 = ((_e699 * _e693) + _e695);
        let _e705 = floor(_e702);
        let _e706 = _e691.xy;
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e705 == 4f)) {
                    let _e720 = normalize(_e706);
                    let _e723 = acos(clamp(_e720.x, -1f, 1f));
                    if (_e720.y >= 0f) {
                        phi_6570_ = _e723;
                    } else {
                        phi_6570_ = -(_e723);
                    }
                    let _e728 = phi_6570_;
                    phi_6571_ = vec2<f32>((_e728 * 0.15915494f), _e704);
                    break;
                } else {
                    if (_e705 == 2f) {
                        phi_6569_ = _e691.x;
                    } else {
                        phi_6569_ = length(_e706);
                    }
                    let _e714 = phi_6569_;
                    phi_6571_ = vec2<f32>(((clamp(_e714, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e691.z < 0f))) + _e702), _e704);
                    break;
                }
            }
        }
        let _e732 = phi_6571_;
        let _e733 = textureSampleLevel(ZC, S8_, _e732, 0f);
        let _e736 = (_e733.xyz * _e733.w);
        let _e742 = vec4<f32>(_e736.x, _e733.y, _e733.z, _e733.w);
        let _e748 = vec4<f32>(_e742.x, _e736.y, _e742.z, _e742.w);
        phi_6606_ = (_e129 * vec4<f32>(_e748.x, _e748.y, _e736.z, _e748.w));
    }
    let _e757 = phi_6606_;
    let _e758 = Y1_1;
    let _e759 = (_e757 * _e758);
    phi_1557_ = lj;
    if lj {
        let _e760 = N1_1;
        phi_1557_ = (_e760 != 0u);
    }
    let _e763 = phi_1557_;
    phi_8222_ = _e759;
    if _e763 {
        let _e766 = n0_.v2_[_e127];
        let _e771 = ((unpack4x8unorm(_e766) * (1f - _e666.w)) + _e666);
        if (_e759.w != 0f) {
            phi_6617_ = (1f / _e759.w);
        } else {
            phi_6617_ = 0f;
        }
        let _e777 = phi_6617_;
        let _e778 = (_e759.xyz * _e777);
        let _e779 = N1_1;
        local_2 = _e778;
        let _e780 = _e771.xyz;
        if (_e771.w != 0f) {
            phi_6618_ = (1f / _e771.w);
        } else {
            phi_6618_ = 0f;
        }
        let _e785 = phi_6618_;
        let _e786 = (_e780 * _e785);
        local = _e786;
        switch bitcast<i32>(_e779) {
            case 11: {
                let _e1070 = local_2;
                local_1 = (_e1070 * _e786);
                break;
            }
            case 1: {
                let _e1066 = local_2;
                local_1 = ((_e1066 + _e786) - (_e1066 * _e786));
                break;
            }
            case 2: {
                let _e1058 = local_2;
                let _e1059 = (_e1058 * _e786);
                local_1 = (select(_e1059, (((_e1058 + _e786) - _e1059) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e786 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e1056 = local_2;
                local_1 = min(_e1056, _e786);
                break;
            }
            case 4: {
                let _e1054 = local_2;
                local_1 = max(_e1054, _e786);
                break;
            }
            case 5: {
                let _e1025 = clamp(_e780, vec3<f32>(0f, 0f, 0f), _e771.www);
                let _e1031 = vec4<f32>(_e1025.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e1037 = vec4<f32>(_e1031.x, _e1025.y, _e1031.z, _e1031.w);
                let _e1044 = local_2;
                let _e1047 = (clamp((vec3<f32>(1f, 1f, 1f) - _e1044), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e771.w);
                let _e1048 = vec4<f32>(_e1037.x, _e1037.y, _e1025.z, _e1037.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e1048 / _e1047)), sign(_e1048), (_e1047 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e986 = local_2;
                local_2 = clamp(_e986, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e989 = clamp(_e780, vec3<f32>(0f, 0f, 0f), _e771.www);
                let _e995 = vec4<f32>(_e989.x, _e771.y, _e771.z, _e771.w);
                let _e1001 = vec4<f32>(_e995.x, _e989.y, _e995.z, _e995.w);
                phi_7839_ = vec4<f32>(_e1001.x, _e1001.y, _e989.z, _e1001.w);
                if (_e771.w == 0f) {
                    phi_7839_ = vec4<f32>(_e989.x, _e989.y, _e989.z, 1f);
                }
                let _e1011 = phi_7839_;
                let _e1015 = (vec3(_e1011.w) - _e1011.xyz);
                let _e1016 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e1015 / (_e1016 * _e1011.w))), sign(_e1015), (_e1016 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e978 = local_2;
                let _e979 = (_e978 * _e786);
                local_1 = (select(_e979, (((_e978 + _e786) - _e979) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e978 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7683_ = 0i;
                loop {
                    let _e947 = phi_7683_;
                    if (_e947 < 3i) {
                        let _e950 = local_2[_e947];
                        if (_e950 <= 0.5f) {
                            let _e967 = local[_e947];
                            local_1[_e947] = (1f - _e967);
                        } else {
                            let _e953 = local[_e947];
                            if (_e953 <= 0.25f) {
                                let _e959 = local[_e947];
                                let _e962 = local[_e947];
                                local_1[_e947] = ((((16f * _e959) - 12f) * _e962) + 3f);
                            } else {
                                let _e955 = local[_e947];
                                local_1[_e947] = (inverseSqrt(_e955) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7683_ = (_e947 + 1i);
                    }
                }
                let _e971 = local_2;
                let _e975 = local_1;
                local_1 = (_e786 + ((_e786 * ((_e971 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e975));
                break;
            }
            case 9: {
                let _e943 = local_2;
                local_1 = abs((_e786 - _e943));
                break;
            }
            case 10: {
                let _e938 = local_2;
                local_1 = ((_e938 + _e786) - ((_e938 * 2f) * _e786));
                break;
            }
            case 12: {
                if qj {
                    let _e889 = local_2;
                    let _e890 = clamp(_e889, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e890;
                    let _e905 = (_e890 - vec3(min(min(_e890.x, _e890.y), _e890.z)));
                    let _e913 = (_e905 * ((max(max(_e786.x, _e786.y), _e786.z) - min(min(_e786.x, _e786.y), _e786.z)) / max(0.000062f, max(max(_e905.x, _e905.y), _e905.z))));
                    let _e914 = dot(_e786, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e917 = (_e913 - vec3(dot(_e913, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e930 = (vec2<f32>(_e914, (1f - _e914)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e917.x, _e917.y), _e917.z)), max(max(_e917.x, _e917.y), _e917.z))));
                    local_1 = ((_e917 * min(1f, min(_e930.x, _e930.y))) + vec3(_e914));
                }
                break;
            }
            case 13: {
                if qj {
                    let _e840 = local_2;
                    let _e841 = clamp(_e840, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e841;
                    let _e856 = (_e786 - vec3(min(min(_e786.x, _e786.y), _e786.z)));
                    let _e864 = (_e856 * ((max(max(_e841.x, _e841.y), _e841.z) - min(min(_e841.x, _e841.y), _e841.z)) / max(0.000062f, max(max(_e856.x, _e856.y), _e856.z))));
                    let _e865 = dot(_e786, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e868 = (_e864 - vec3(dot(_e864, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e881 = (vec2<f32>(_e865, (1f - _e865)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e868.x, _e868.y), _e868.z)), max(max(_e868.x, _e868.y), _e868.z))));
                    local_1 = ((_e868 * min(1f, min(_e881.x, _e881.y))) + vec3(_e865));
                }
                break;
            }
            case 14: {
                if qj {
                    let _e814 = local_2;
                    let _e815 = clamp(_e814, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e815;
                    let _e816 = dot(_e786, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e819 = (_e815 - vec3(dot(_e815, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e832 = (vec2<f32>(_e816, (1f - _e816)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e819.x, _e819.y), _e819.z)), max(max(_e819.x, _e819.y), _e819.z))));
                    local_1 = ((_e819 * min(1f, min(_e832.x, _e832.y))) + vec3(_e816));
                }
                break;
            }
            case 15: {
                if qj {
                    let _e788 = local_2;
                    let _e789 = clamp(_e788, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e789;
                    let _e790 = dot(_e789, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e793 = (_e786 - vec3(dot(_e786, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e806 = (vec2<f32>(_e790, (1f - _e790)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e793.x, _e793.y), _e793.z)), max(max(_e793.x, _e793.y), _e793.z))));
                    local_1 = ((_e793 * min(1f, min(_e806.x, _e806.y))) + vec3(_e790));
                }
                break;
            }
            default: {
            }
        }
        let _e1072 = local_1;
        let _e1075 = (mix(_e778, _e1072, vec3(_e771.w)) * _e759.w);
        let _e1081 = vec4<f32>(_e1075.x, _e759.y, _e759.z, _e759.w);
        let _e1087 = vec4<f32>(_e1081.x, _e1075.y, _e1081.z, _e1081.w);
        phi_8222_ = vec4<f32>(_e1087.x, _e1087.y, _e1075.z, _e1087.w);
    }
    let _e1095 = phi_8222_;
    let _e1096 = (_e1095 * _e687);
    let _e1100 = ((_e666 * (1f - _e1096.w)) + _e1096);
    let _e1101 = _e1100.xyz;
    let _e1104 = j.H3_;
    let _e1106 = j.I3_;
    if (rj && (_e1100.w != 0f)) {
        phi_8235_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e92.x) + (0.00583715f * _e92.y))))) * _e1104) + _e1106)) + _e1101);
    } else {
        phi_8235_ = _e1101;
    }
    let _e1122 = phi_8235_;
    let _e1128 = vec4<f32>(_e1122.x, _e1100.y, _e1100.z, _e1100.w);
    let _e1134 = vec4<f32>(_e1128.x, _e1122.y, _e1128.z, _e1128.w);
    let _e1140 = vec4<f32>(_e1134.x, _e1134.y, _e1122.z, _e1134.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1122.x + _e1122.y) + _e1122.z) + _e1100.w) == 0f) {
                break;
            }
            phi_8237_ = _e1140;
            if (_e1100.w != 1f) {
                let _e1149 = n0_.v2_[_e127];
                phi_8237_ = (_e1140 + (unpack4x8unorm(_e1149) * (1f - _e1100.w)));
            }
            let _e1155 = phi_8237_;
            n0_.v2_[_e127] = pack4x8unorm(_e1155);
            break;
        }
    }
    if (_e317 != 0u) {
        m0_.v2_[_e127] = _e317;
    }
    W4_.v2_[_e127] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) n2_: vec2<f32>, @location(1) x5_: f32, @location(3) W0_: vec4<f32>, @location(5) @interpolate(flat, either) V3_: u32, @location(2) y5_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec4<f32>, @location(6) @interpolate(flat, either) N1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    n2_1 = n2_;
    x5_1 = x5_;
    W0_1 = W0_;
    V3_1 = V3_;
    y5_1 = y5_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
}
