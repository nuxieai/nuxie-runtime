struct jd {
    x2_: array<vec4<u32>>,
}

struct m0cf {
    x2_: array<u32>,
}

struct ug {
    x2_: array<vec4<f32>>,
}

struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct o0cf {
    x2_: array<u32>,
}

struct W4cf {
    x2_: array<u32>,
}

@id(8) override rj: bool = true;
@id(7) override qj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

@group(0) @binding(2)
var<storage> JB: jd;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(3)
var<storage> KB: ug;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var Q8_: sampler;
@group(2) @binding(0)
var<storage, read_write> o0_: o0cf;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;
var<private> o2_1: vec2<f32>;
var<private> w5_1: f32;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> V3_1: u32;
var<private> x5_1: vec4<f32>;
var<private> Y1_1: vec4<f32>;
var<private> N1_1: u32;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_6525_: f32;
    var phi_1805_: bool;
    var phi_5752_: f32;
    var phi_5751_: f32;
    var phi_5753_: f32;
    var phi_5756_: f32;
    var phi_5755_: f32;
    var phi_1842_: bool;
    var phi_5760_: f32;
    var phi_5757_: f32;
    var phi_5758_: f32;
    var phi_5759_: vec2<f32>;
    var phi_5783_: vec4<f32>;
    var phi_6490_: u32;
    var phi_6489_: u32;
    var phi_5781_: vec4<f32>;
    var phi_5790_: f32;
    var phi_6240_: i32;
    var phi_6308_: vec4<f32>;
    var phi_6474_: vec4<f32>;
    var phi_6487_: vec4<f32>;
    var phi_1494_: bool;
    var phi_6515_: u32;
    var phi_6545_: f32;
    var phi_8215_: f32;
    var phi_6577_: f32;
    var phi_6578_: f32;
    var phi_6579_: vec2<f32>;
    var phi_6614_: vec4<f32>;
    var phi_1562_: bool;
    var phi_6625_: f32;
    var phi_6626_: f32;
    var phi_7691_: i32;
    var phi_7847_: vec4<f32>;
    var phi_8230_: vec4<f32>;
    var phi_8243_: vec3<f32>;
    var phi_8245_: vec4<f32>;

    let _e93 = gl_FragCoord_1;
    let _e94 = _e93.xy;
    let _e97 = bitcast<vec2<u32>>(vec2<i32>(floor(_e94)));
    let _e99 = j.R6_;
    let _e128 = bitcast<i32>((((((_e97.y >> bitcast<u32>(5u)) * (((_e99 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e97.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e97.x & 28u) << bitcast<u32>(5u)) + ((_e97.y & 28u) << bitcast<u32>(2i)))) + (((_e97.y & 3u) << bitcast<u32>(2i)) + (_e97.x & 3u))));
    let _e129 = o2_1;
    let _e130 = textureSample(RB, v4_, _e129);
    let _e131 = w5_1;
    let _e132 = min(_e131, 1f);
    phi_6525_ = _e132;
    if kj {
        let _e133 = V0_1;
        let _e136 = min(_e133.xy, _e133.zw);
        phi_6525_ = clamp(min(_e136.x, _e136.y), 0f, _e132);
    }
    let _e142 = phi_6525_;
    let _e145 = W4_.x2_[_e128];
    let _e147 = (_e145 >> bitcast<u32>(17u));
    let _e151 = ((f32((_e145 & 131071u)) * 0.00048828125f) + -32f);
    let _e156 = JB.x2_[((_e147 * 4u) + 1u)];
    phi_5751_ = _e151;
    if ((_e156.z & 768u) != 0u) {
        let _e160 = abs(_e151);
        phi_1805_ = oj;
        if oj {
            phi_1805_ = ((_e156.z & 512u) != 0u);
        }
        let _e164 = phi_1805_;
        phi_5752_ = _e160;
        if _e164 {
            phi_5752_ = (1f - abs(((fract((_e160 * 0.5f)) * 2f) + -1f)));
        }
        let _e172 = phi_5752_;
        phi_5751_ = _e172;
    }
    let _e174 = phi_5751_;
    let _e175 = clamp(_e174, 0f, 1f);
    phi_5755_ = _e175;
    if jj {
        let _e177 = (_e156.z >> bitcast<u32>(16u));
        phi_5756_ = _e175;
        if (_e177 != 0u) {
            let _e181 = m0_.x2_[_e128];
            if (_e177 == (_e181 >> bitcast<u32>(16i))) {
                phi_5753_ = min(_e175, unpack2x16float(_e181).x);
            } else {
                phi_5753_ = 0f;
            }
            let _e189 = phi_5753_;
            phi_5756_ = _e189;
        }
        let _e191 = phi_5756_;
        phi_5755_ = _e191;
    }
    let _e193 = phi_5755_;
    phi_1842_ = kj;
    if kj {
        phi_1842_ = ((_e156.z & 1024u) != 0u);
    }
    let _e197 = phi_1842_;
    phi_5760_ = _e193;
    if _e197 {
        let _e198 = (_e147 * 8u);
        let _e202 = KB.x2_[(_e198 + 2u)];
        let _e213 = KB.x2_[(_e198 + 3u)];
        let _e218 = _e213.zw;
        let _e220 = ((abs(((mat2x2<f32>(vec2<f32>(_e202.x, _e202.y), vec2<f32>(_e202.z, _e202.w)) * _e94) + _e213.xy)) * _e218) - _e218);
        phi_5760_ = min(_e193, clamp((min(_e220.x, _e220.y) + 0.5f), 0f, 1f));
    }
    let _e228 = phi_5760_;
    let _e229 = (_e156.z & 15u);
    let _e232 = ((_e156.z >> bitcast<u32>(4i)) & 15u);
    let _e234 = (lj && (_e232 != 0u));
    if (_e229 <= 1u) {
        let _e311 = (jj && (_e229 == 0u));
        phi_6490_ = 0u;
        if _e311 {
            phi_6490_ = (_e156.w | pack2x16float(vec2<f32>(_e228, 0f)));
        }
        let _e316 = phi_6490_;
        phi_6489_ = _e316;
        phi_5781_ = select(unpack4x8unorm(_e156.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e311));
    } else {
        let _e236 = (_e147 * 8u);
        let _e239 = KB.x2_[_e236];
        let _e250 = KB.x2_[(_e236 + 1u)];
        let _e253 = ((mat2x2<f32>(vec2<f32>(_e239.x, _e239.y), vec2<f32>(_e239.z, _e239.w)) * _e94) + _e250.xy);
        let _e257 = f32(_e229);
        let _e259 = j.O8_;
        let _e261 = j.P8_;
        let _e266 = ((_e250.w * _e259) + _e261);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e257 == 4f)) {
                    let _e280 = normalize(_e253);
                    let _e283 = acos(clamp(_e280.x, -1f, 1f));
                    if (_e280.y >= 0f) {
                        phi_5758_ = _e283;
                    } else {
                        phi_5758_ = -(_e283);
                    }
                    let _e288 = phi_5758_;
                    phi_5759_ = vec2<f32>((_e288 * 0.15915494f), _e266);
                    break;
                } else {
                    if (_e257 == 2f) {
                        phi_5757_ = _e253.x;
                    } else {
                        phi_5757_ = length(_e253);
                    }
                    let _e274 = phi_5757_;
                    phi_5759_ = vec2<f32>(((clamp(_e274, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e250.z < 0f))) + ((max(0f, _e250.z) * 0.001953125f) + 0.0009765625f)), _e266);
                    break;
                }
            }
        }
        let _e292 = phi_5759_;
        let _e293 = textureSampleLevel(YC, Q8_, _e292, 0f);
        phi_5783_ = _e293;
        if !(_e234) {
            let _e297 = (_e293.xyz * _e293.w);
            phi_5783_ = vec4<f32>(_e297.x, _e297.y, _e297.z, (_e293.w * abs(bitcast<f32>(_e156.w))));
        }
        let _e307 = phi_5783_;
        phi_6489_ = 0u;
        phi_5781_ = _e307;
    }
    let _e320 = phi_6489_;
    let _e322 = phi_5781_;
    phi_6487_ = _e322;
    if _e234 {
        phi_6474_ = _e322;
        if ((_e322.w * _e228) != 0f) {
            let _e328 = o0_.x2_[_e128];
            let _e329 = unpack4x8unorm(_e328);
            let _e330 = _e322.xyz;
            local_5 = _e330;
            let _e331 = _e329.xyz;
            if (_e329.w != 0f) {
                phi_5790_ = (1f / _e329.w);
            } else {
                phi_5790_ = 0f;
            }
            let _e336 = phi_5790_;
            let _e337 = (_e331 * _e336);
            local_3 = _e337;
            switch bitcast<i32>(_e232) {
                case 11: {
                    let _e621 = local_5;
                    local_4 = (_e621 * _e337);
                    break;
                }
                case 1: {
                    let _e617 = local_5;
                    local_4 = ((_e617 + _e337) - (_e617 * _e337));
                    break;
                }
                case 2: {
                    let _e609 = local_5;
                    let _e610 = (_e609 * _e337);
                    local_4 = (select(_e610, (((_e609 + _e337) - _e610) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e337 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e607 = local_5;
                    local_4 = min(_e607, _e337);
                    break;
                }
                case 4: {
                    let _e605 = local_5;
                    local_4 = max(_e605, _e337);
                    break;
                }
                case 5: {
                    let _e576 = clamp(_e331, vec3<f32>(0f, 0f, 0f), _e329.www);
                    let _e582 = vec4<f32>(_e576.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e588 = vec4<f32>(_e582.x, _e576.y, _e582.z, _e582.w);
                    let _e595 = local_5;
                    let _e598 = (clamp((vec3<f32>(1f, 1f, 1f) - _e595), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e329.w);
                    let _e599 = vec4<f32>(_e588.x, _e588.y, _e576.z, _e588.w).xyz;
                    local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e599 / _e598)), sign(_e599), (_e598 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e537 = local_5;
                    local_5 = clamp(_e537, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e540 = clamp(_e331, vec3<f32>(0f, 0f, 0f), _e329.www);
                    let _e546 = vec4<f32>(_e540.x, _e329.y, _e329.z, _e329.w);
                    let _e552 = vec4<f32>(_e546.x, _e540.y, _e546.z, _e546.w);
                    phi_6308_ = vec4<f32>(_e552.x, _e552.y, _e540.z, _e552.w);
                    if (_e329.w == 0f) {
                        phi_6308_ = vec4<f32>(_e540.x, _e540.y, _e540.z, 1f);
                    }
                    let _e562 = phi_6308_;
                    let _e566 = (vec3(_e562.w) - _e562.xyz);
                    let _e567 = local_5;
                    local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e566 / (_e567 * _e562.w))), sign(_e566), (_e567 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e529 = local_5;
                    let _e530 = (_e529 * _e337);
                    local_4 = (select(_e530, (((_e529 + _e337) - _e530) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e529 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_6240_ = 0i;
                    loop {
                        let _e498 = phi_6240_;
                        if (_e498 < 3i) {
                            let _e501 = local_5[_e498];
                            if (_e501 <= 0.5f) {
                                let _e518 = local_3[_e498];
                                local_4[_e498] = (1f - _e518);
                            } else {
                                let _e504 = local_3[_e498];
                                if (_e504 <= 0.25f) {
                                    let _e510 = local_3[_e498];
                                    let _e513 = local_3[_e498];
                                    local_4[_e498] = ((((16f * _e510) - 12f) * _e513) + 3f);
                                } else {
                                    let _e506 = local_3[_e498];
                                    local_4[_e498] = (inverseSqrt(_e506) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_6240_ = (_e498 + 1i);
                        }
                    }
                    let _e522 = local_5;
                    let _e526 = local_4;
                    local_4 = (_e337 + ((_e337 * ((_e522 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e526));
                    break;
                }
                case 9: {
                    let _e494 = local_5;
                    local_4 = abs((_e337 - _e494));
                    break;
                }
                case 10: {
                    let _e489 = local_5;
                    local_4 = ((_e489 + _e337) - ((_e489 * 2f) * _e337));
                    break;
                }
                case 12: {
                    if qj {
                        let _e440 = local_5;
                        let _e441 = clamp(_e440, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e441;
                        let _e456 = (_e441 - vec3(min(min(_e441.x, _e441.y), _e441.z)));
                        let _e464 = (_e456 * ((max(max(_e337.x, _e337.y), _e337.z) - min(min(_e337.x, _e337.y), _e337.z)) / max(0.000062f, max(max(_e456.x, _e456.y), _e456.z))));
                        let _e465 = dot(_e337, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e468 = (_e464 - vec3(dot(_e464, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e481 = (vec2<f32>(_e465, (1f - _e465)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e468.x, _e468.y), _e468.z)), max(max(_e468.x, _e468.y), _e468.z))));
                        local_4 = ((_e468 * min(1f, min(_e481.x, _e481.y))) + vec3(_e465));
                    }
                    break;
                }
                case 13: {
                    if qj {
                        let _e391 = local_5;
                        let _e392 = clamp(_e391, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e392;
                        let _e407 = (_e337 - vec3(min(min(_e337.x, _e337.y), _e337.z)));
                        let _e415 = (_e407 * ((max(max(_e392.x, _e392.y), _e392.z) - min(min(_e392.x, _e392.y), _e392.z)) / max(0.000062f, max(max(_e407.x, _e407.y), _e407.z))));
                        let _e416 = dot(_e337, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e419 = (_e415 - vec3(dot(_e415, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e432 = (vec2<f32>(_e416, (1f - _e416)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e419.x, _e419.y), _e419.z)), max(max(_e419.x, _e419.y), _e419.z))));
                        local_4 = ((_e419 * min(1f, min(_e432.x, _e432.y))) + vec3(_e416));
                    }
                    break;
                }
                case 14: {
                    if qj {
                        let _e365 = local_5;
                        let _e366 = clamp(_e365, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e366;
                        let _e367 = dot(_e337, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e370 = (_e366 - vec3(dot(_e366, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e383 = (vec2<f32>(_e367, (1f - _e367)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e370.x, _e370.y), _e370.z)), max(max(_e370.x, _e370.y), _e370.z))));
                        local_4 = ((_e370 * min(1f, min(_e383.x, _e383.y))) + vec3(_e367));
                    }
                    break;
                }
                case 15: {
                    if qj {
                        let _e339 = local_5;
                        let _e340 = clamp(_e339, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_5 = _e340;
                        let _e341 = dot(_e340, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e344 = (_e337 - vec3(dot(_e337, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e357 = (vec2<f32>(_e341, (1f - _e341)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e344.x, _e344.y), _e344.z)), max(max(_e344.x, _e344.y), _e344.z))));
                        local_4 = ((_e344 * min(1f, min(_e357.x, _e357.y))) + vec3(_e341));
                    }
                    break;
                }
                default: {
                }
            }
            let _e623 = local_4;
            let _e625 = mix(_e330, _e623, vec3(_e329.w));
            let _e631 = vec4<f32>(_e625.x, _e322.y, _e322.z, _e322.w);
            let _e637 = vec4<f32>(_e631.x, _e625.y, _e631.z, _e631.w);
            phi_6474_ = vec4<f32>(_e637.x, _e637.y, _e625.z, _e637.w);
        }
        let _e645 = phi_6474_;
        let _e648 = (_e645.xyz * _e645.w);
        let _e654 = vec4<f32>(_e648.x, _e645.y, _e645.z, _e645.w);
        let _e660 = vec4<f32>(_e654.x, _e648.y, _e654.z, _e654.w);
        phi_6487_ = vec4<f32>(_e660.x, _e660.y, _e648.z, _e660.w);
    }
    let _e668 = phi_6487_;
    let _e669 = (_e668 * _e228);
    phi_1494_ = jj;
    if jj {
        let _e670 = V3_1;
        phi_1494_ = (_e670 != 0u);
    }
    let _e673 = phi_1494_;
    phi_8215_ = _e142;
    if _e673 {
        if (_e320 != 0u) {
            phi_6515_ = _e320;
        } else {
            let _e677 = m0_.x2_[_e128];
            phi_6515_ = _e677;
        }
        let _e679 = phi_6515_;
        let _e680 = V3_1;
        if (_e680 == (_e679 >> bitcast<u32>(16i))) {
            phi_6545_ = min(_e142, unpack2x16float(_e679).x);
        } else {
            phi_6545_ = 0f;
        }
        let _e688 = phi_6545_;
        phi_8215_ = _e688;
    }
    let _e690 = phi_8215_;
    let _e692 = x5_1[3u];
    phi_6614_ = _e130;
    if (_e692 != 0f) {
        let _e694 = x5_1;
        let _e696 = j.O8_;
        let _e698 = j.P8_;
        let _e701 = abs(_e694.z);
        let _e702 = floor(_e701);
        let _e705 = (((_e701 - _e702) * 8f) + 0.0009765625f);
        let _e707 = ((_e702 * _e696) + _e698);
        let _e708 = floor(_e705);
        let _e709 = _e694.xy;
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e708 == 4f)) {
                    let _e723 = normalize(_e709);
                    let _e726 = acos(clamp(_e723.x, -1f, 1f));
                    if (_e723.y >= 0f) {
                        phi_6578_ = _e726;
                    } else {
                        phi_6578_ = -(_e726);
                    }
                    let _e731 = phi_6578_;
                    phi_6579_ = vec2<f32>((_e731 * 0.15915494f), _e707);
                    break;
                } else {
                    if (_e708 == 2f) {
                        phi_6577_ = _e694.x;
                    } else {
                        phi_6577_ = length(_e709);
                    }
                    let _e717 = phi_6577_;
                    phi_6579_ = vec2<f32>(((clamp(_e717, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e694.z < 0f))) + _e705), _e707);
                    break;
                }
            }
        }
        let _e735 = phi_6579_;
        let _e736 = textureSampleLevel(YC, Q8_, _e735, 0f);
        let _e739 = (_e736.xyz * _e736.w);
        let _e745 = vec4<f32>(_e739.x, _e736.y, _e736.z, _e736.w);
        let _e751 = vec4<f32>(_e745.x, _e739.y, _e745.z, _e745.w);
        phi_6614_ = (_e130 * vec4<f32>(_e751.x, _e751.y, _e739.z, _e751.w));
    }
    let _e760 = phi_6614_;
    let _e761 = Y1_1;
    let _e762 = (_e760 * _e761);
    phi_1562_ = lj;
    if lj {
        let _e763 = N1_1;
        phi_1562_ = (_e763 != 0u);
    }
    let _e766 = phi_1562_;
    phi_8230_ = _e762;
    if _e766 {
        let _e769 = o0_.x2_[_e128];
        let _e774 = ((unpack4x8unorm(_e769) * (1f - _e669.w)) + _e669);
        if (_e762.w != 0f) {
            phi_6625_ = (1f / _e762.w);
        } else {
            phi_6625_ = 0f;
        }
        let _e780 = phi_6625_;
        let _e781 = (_e762.xyz * _e780);
        let _e782 = N1_1;
        local_2 = _e781;
        let _e783 = _e774.xyz;
        if (_e774.w != 0f) {
            phi_6626_ = (1f / _e774.w);
        } else {
            phi_6626_ = 0f;
        }
        let _e788 = phi_6626_;
        let _e789 = (_e783 * _e788);
        local = _e789;
        switch bitcast<i32>(_e782) {
            case 11: {
                let _e1073 = local_2;
                local_1 = (_e1073 * _e789);
                break;
            }
            case 1: {
                let _e1069 = local_2;
                local_1 = ((_e1069 + _e789) - (_e1069 * _e789));
                break;
            }
            case 2: {
                let _e1061 = local_2;
                let _e1062 = (_e1061 * _e789);
                local_1 = (select(_e1062, (((_e1061 + _e789) - _e1062) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e789 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e1059 = local_2;
                local_1 = min(_e1059, _e789);
                break;
            }
            case 4: {
                let _e1057 = local_2;
                local_1 = max(_e1057, _e789);
                break;
            }
            case 5: {
                let _e1028 = clamp(_e783, vec3<f32>(0f, 0f, 0f), _e774.www);
                let _e1034 = vec4<f32>(_e1028.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e1040 = vec4<f32>(_e1034.x, _e1028.y, _e1034.z, _e1034.w);
                let _e1047 = local_2;
                let _e1050 = (clamp((vec3<f32>(1f, 1f, 1f) - _e1047), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e774.w);
                let _e1051 = vec4<f32>(_e1040.x, _e1040.y, _e1028.z, _e1040.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e1051 / _e1050)), sign(_e1051), (_e1050 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e989 = local_2;
                local_2 = clamp(_e989, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e992 = clamp(_e783, vec3<f32>(0f, 0f, 0f), _e774.www);
                let _e998 = vec4<f32>(_e992.x, _e774.y, _e774.z, _e774.w);
                let _e1004 = vec4<f32>(_e998.x, _e992.y, _e998.z, _e998.w);
                phi_7847_ = vec4<f32>(_e1004.x, _e1004.y, _e992.z, _e1004.w);
                if (_e774.w == 0f) {
                    phi_7847_ = vec4<f32>(_e992.x, _e992.y, _e992.z, 1f);
                }
                let _e1014 = phi_7847_;
                let _e1018 = (vec3(_e1014.w) - _e1014.xyz);
                let _e1019 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e1018 / (_e1019 * _e1014.w))), sign(_e1018), (_e1019 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e981 = local_2;
                let _e982 = (_e981 * _e789);
                local_1 = (select(_e982, (((_e981 + _e789) - _e982) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e981 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_7691_ = 0i;
                loop {
                    let _e950 = phi_7691_;
                    if (_e950 < 3i) {
                        let _e953 = local_2[_e950];
                        if (_e953 <= 0.5f) {
                            let _e970 = local[_e950];
                            local_1[_e950] = (1f - _e970);
                        } else {
                            let _e956 = local[_e950];
                            if (_e956 <= 0.25f) {
                                let _e962 = local[_e950];
                                let _e965 = local[_e950];
                                local_1[_e950] = ((((16f * _e962) - 12f) * _e965) + 3f);
                            } else {
                                let _e958 = local[_e950];
                                local_1[_e950] = (inverseSqrt(_e958) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_7691_ = (_e950 + 1i);
                    }
                }
                let _e974 = local_2;
                let _e978 = local_1;
                local_1 = (_e789 + ((_e789 * ((_e974 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e978));
                break;
            }
            case 9: {
                let _e946 = local_2;
                local_1 = abs((_e789 - _e946));
                break;
            }
            case 10: {
                let _e941 = local_2;
                local_1 = ((_e941 + _e789) - ((_e941 * 2f) * _e789));
                break;
            }
            case 12: {
                if qj {
                    let _e892 = local_2;
                    let _e893 = clamp(_e892, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e893;
                    let _e908 = (_e893 - vec3(min(min(_e893.x, _e893.y), _e893.z)));
                    let _e916 = (_e908 * ((max(max(_e789.x, _e789.y), _e789.z) - min(min(_e789.x, _e789.y), _e789.z)) / max(0.000062f, max(max(_e908.x, _e908.y), _e908.z))));
                    let _e917 = dot(_e789, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e920 = (_e916 - vec3(dot(_e916, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e933 = (vec2<f32>(_e917, (1f - _e917)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e920.x, _e920.y), _e920.z)), max(max(_e920.x, _e920.y), _e920.z))));
                    local_1 = ((_e920 * min(1f, min(_e933.x, _e933.y))) + vec3(_e917));
                }
                break;
            }
            case 13: {
                if qj {
                    let _e843 = local_2;
                    let _e844 = clamp(_e843, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e844;
                    let _e859 = (_e789 - vec3(min(min(_e789.x, _e789.y), _e789.z)));
                    let _e867 = (_e859 * ((max(max(_e844.x, _e844.y), _e844.z) - min(min(_e844.x, _e844.y), _e844.z)) / max(0.000062f, max(max(_e859.x, _e859.y), _e859.z))));
                    let _e868 = dot(_e789, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e871 = (_e867 - vec3(dot(_e867, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e884 = (vec2<f32>(_e868, (1f - _e868)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e871.x, _e871.y), _e871.z)), max(max(_e871.x, _e871.y), _e871.z))));
                    local_1 = ((_e871 * min(1f, min(_e884.x, _e884.y))) + vec3(_e868));
                }
                break;
            }
            case 14: {
                if qj {
                    let _e817 = local_2;
                    let _e818 = clamp(_e817, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e818;
                    let _e819 = dot(_e789, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e822 = (_e818 - vec3(dot(_e818, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e835 = (vec2<f32>(_e819, (1f - _e819)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e822.x, _e822.y), _e822.z)), max(max(_e822.x, _e822.y), _e822.z))));
                    local_1 = ((_e822 * min(1f, min(_e835.x, _e835.y))) + vec3(_e819));
                }
                break;
            }
            case 15: {
                if qj {
                    let _e791 = local_2;
                    let _e792 = clamp(_e791, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e792;
                    let _e793 = dot(_e792, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e796 = (_e789 - vec3(dot(_e789, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e809 = (vec2<f32>(_e793, (1f - _e793)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e796.x, _e796.y), _e796.z)), max(max(_e796.x, _e796.y), _e796.z))));
                    local_1 = ((_e796 * min(1f, min(_e809.x, _e809.y))) + vec3(_e793));
                }
                break;
            }
            default: {
            }
        }
        let _e1075 = local_1;
        let _e1078 = (mix(_e781, _e1075, vec3(_e774.w)) * _e762.w);
        let _e1084 = vec4<f32>(_e1078.x, _e762.y, _e762.z, _e762.w);
        let _e1090 = vec4<f32>(_e1084.x, _e1078.y, _e1084.z, _e1084.w);
        phi_8230_ = vec4<f32>(_e1090.x, _e1090.y, _e1078.z, _e1090.w);
    }
    let _e1098 = phi_8230_;
    let _e1099 = (_e1098 * _e690);
    let _e1103 = ((_e669 * (1f - _e1099.w)) + _e1099);
    let _e1104 = _e1103.xyz;
    let _e1107 = j.I3_;
    let _e1109 = j.J3_;
    if (rj && (_e1103.w != 0f)) {
        phi_8243_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e93.x) + (0.00583715f * _e93.y))))) * _e1107) + _e1109)) + _e1104);
    } else {
        phi_8243_ = _e1104;
    }
    let _e1125 = phi_8243_;
    let _e1131 = vec4<f32>(_e1125.x, _e1103.y, _e1103.z, _e1103.w);
    let _e1137 = vec4<f32>(_e1131.x, _e1125.y, _e1131.z, _e1131.w);
    let _e1143 = vec4<f32>(_e1137.x, _e1137.y, _e1125.z, _e1137.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e1125.x + _e1125.y) + _e1125.z) + _e1103.w) == 0f) {
                break;
            }
            phi_8245_ = _e1143;
            if (_e1103.w != 1f) {
                let _e1152 = o0_.x2_[_e128];
                phi_8245_ = (_e1143 + (unpack4x8unorm(_e1152) * (1f - _e1103.w)));
            }
            let _e1158 = phi_8245_;
            o0_.x2_[_e128] = pack4x8unorm(_e1158);
            break;
        }
    }
    if (_e320 != 0u) {
        m0_.x2_[_e128] = _e320;
    }
    W4_.x2_[_e128] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) o2_: vec2<f32>, @location(1) w5_: f32, @location(3) V0_: vec4<f32>, @location(5) @interpolate(flat, either) V3_: u32, @location(2) x5_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec4<f32>, @location(6) @interpolate(flat, either) N1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    o2_1 = o2_;
    w5_1 = w5_;
    V0_1 = V0_;
    V3_1 = V3_;
    x5_1 = x5_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
}
