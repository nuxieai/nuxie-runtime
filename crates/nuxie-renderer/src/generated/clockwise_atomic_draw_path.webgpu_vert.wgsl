struct mg {
    d2_: array<vec4<u32>>,
}

struct lg {
    d2_: array<vec4<u32>>,
}

struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
}

struct Re {
    d2_: array<vec2<u32>>,
}

struct Se {
    d2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(2) member: vec4<f32>,
    @location(3) @interpolate(flat, either) member_1: f32,
    @location(4) @interpolate(flat, either) member_2: vec2<f32>,
    @location(6) @interpolate(flat, either) member_3: f32,
    @location(5) member_4: vec4<f32>,
    @location(0) member_5: vec4<f32>,
    @location(9) member_6: vec3<f32>,
    @location(7) @interpolate(flat, either) member_7: vec2<u32>,
    @location(8) member_8: vec2<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override jh: bool = true;
@id(2) override lh: bool = true;
@id(1) override kh: bool = true;
@id(8) override rh: bool = true;

@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ED: mg;
@group(0) @binding(2)
var<storage> PB: lg;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
var<private> L: vec4<f32>;
@group(0) @binding(3)
var<storage> AD: Re;
var<private> B0_: f32;
var<private> V1_: vec2<f32>;
var<private> f2_: f32;
@group(0) @binding(4)
var<storage> QB: Se;
var<private> M0_: vec4<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
var<private> f3_: vec2<u32>;
var<private> o4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_2791_: f32;
    var phi_2735_: f32;
    var phi_2707_: i32;
    var phi_1689_: bool;
    var phi_2720_: i32;
    var phi_2712_: vec4<u32>;
    var phi_2719_: i32;
    var phi_2711_: vec4<u32>;
    var phi_2718_: i32;
    var phi_2716_: vec4<u32>;
    var phi_2715_: u32;
    var phi_2722_: vec2<i32>;
    var phi_2723_: vec4<u32>;
    var phi_2727_: f32;
    var phi_2740_: f32;
    var phi_2808_: f32;
    var phi_2806_: f32;
    var phi_2749_: f32;
    var phi_2742_: f32;
    var phi_2739_: f32;
    var phi_2753_: f32;
    var phi_2828_: f32;
    var phi_2819_: f32;
    var phi_2804_: f32;
    var phi_2752_: f32;
    var phi_2802_: f32;
    var phi_2838_: f32;
    var phi_2837_: f32;
    var phi_2839_: f32;
    var phi_2843_: f32;
    var phi_2865_: f32;
    var phi_2863_: f32;
    var phi_2881_: vec4<f32>;
    var phi_3031_: vec2<f32>;
    var phi_2880_: vec4<f32>;
    var phi_3034_: vec4<f32>;
    var phi_2885_: f32;
    var phi_2896_: f32;
    var phi_2888_: f32;
    var phi_2977_: f32;
    var phi_2934_: i32;
    var phi_2943_: f32;
    var phi_2121_: bool;
    var phi_2950_: f32;
    var phi_2966_: vec2<f32>;
    var phi_2965_: vec2<f32>;
    var phi_2987_: vec4<f32>;
    var phi_3002_: vec2<f32>;
    var phi_2986_: vec4<f32>;
    var phi_3035_: vec4<f32>;
    var phi_3032_: vec4<f32>;
    var phi_3028_: vec2<f32>;
    var phi_3004_: vec2<f32>;
    var phi_3075_: vec4<f32>;
    var phi_3037_: vec2<f32>;
    var phi_3036_: bool;
    var local: u32;
    var local_1: u32;
    var local_2: u32;
    var phi_3076_: f32;
    var phi_3077_: u32;
    var phi_3078_: f32;
    var phi_3079_: f32;
    var local_3: u32;
    var phi_2521_: bool;
    var phi_3080_: vec4<f32>;
    var local_4: u32;
    var phi_3081_: f32;
    var phi_1376_: bool;
    var local_5: u32;
    var local_6: u32;
    var phi_3099_: vec4<f32>;

    let _e95 = gl_InstanceIndex_1;
    let _e96 = UB_1;
    let _e97 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e100 = i32(_e96.x);
            let _e104 = bitcast<i32>(_e96.w);
            let _e106 = (_e104 >> bitcast<u32>(2i));
            let _e107 = (_e104 & 3i);
            let _e109 = min(_e100, (_e106 - 1i));
            let _e111 = ((_e95 * _e106) + _e109);
            let _e116 = textureLoad(KC, vec2<i32>((_e111 & 2047i), (_e111 >> bitcast<u32>(11i))), 0i);
            let _e123 = ED.d2_[(max((_e116.w & 65535u), 1u) - 1u)];
            let _e125 = bitcast<vec2<f32>>(_e123.xy);
            let _e127 = (_e123.z & 65535u);
            let _e129 = (_e127 * 4u);
            let _e132 = PB.d2_[_e129];
            let _e133 = bitcast<vec4<f32>>(_e132);
            let _e140 = mat2x2<f32>(vec2<f32>(_e133.x, _e133.y), vec2<f32>(_e133.z, _e133.w));
            let _e144 = PB.d2_[(_e129 + 1u)];
            let _e148 = bitcast<f32>(_e144.z);
            let _e150 = bitcast<f32>(_e144.w);
            let _e151 = (_e116.w & 8388608u);
            phi_2791_ = _e96.z;
            phi_2735_ = _e96.y;
            phi_2707_ = _e100;
            local = _e127;
            local_1 = _e127;
            local_2 = _e127;
            local_3 = _e127;
            local_4 = _e127;
            local_5 = _e127;
            local_6 = _e129;
            if (_e151 != 0u) {
                phi_2791_ = _e97.z;
                phi_2735_ = _e97.y;
                phi_2707_ = i32(_e97.x);
            }
            let _e158 = phi_2791_;
            let _e160 = phi_2735_;
            let _e162 = phi_2707_;
            phi_2718_ = _e111;
            phi_2716_ = _e116;
            phi_2715_ = _e116.w;
            if (_e162 != _e109) {
                let _e165 = ((_e111 + _e162) - _e109);
                let _e170 = textureLoad(KC, vec2<i32>((_e165 & 2047i), (_e165 >> bitcast<u32>(11i))), 0i);
                if ((_e170.w & 8454143u) != (_e116.w & 8454143u)) {
                    let _e175 = (_e148 == 0f);
                    phi_1689_ = _e175;
                    if !(_e175) {
                        phi_1689_ = (_e125.x != 0f);
                    }
                    let _e180 = phi_1689_;
                    phi_2720_ = _e111;
                    phi_2712_ = _e116;
                    if _e180 {
                        let _e181 = bitcast<i32>(_e123.w);
                        let _e186 = textureLoad(KC, vec2<i32>((_e181 & 2047i), (_e181 >> bitcast<u32>(11i))), 0i);
                        phi_2720_ = _e181;
                        phi_2712_ = _e186;
                    }
                    let _e188 = phi_2720_;
                    let _e190 = phi_2712_;
                    phi_2719_ = _e188;
                    phi_2711_ = _e190;
                } else {
                    phi_2719_ = _e165;
                    phi_2711_ = _e170;
                }
                let _e192 = phi_2719_;
                let _e194 = phi_2711_;
                phi_2718_ = _e192;
                phi_2716_ = _e194;
                phi_2715_ = ((_e194.w & 4286578687u) | _e151);
            }
            let _e199 = phi_2718_;
            let _e201 = phi_2716_;
            let _e203 = phi_2715_;
            let _e204 = (_e203 & 469762048u);
            let _e207 = ((_e204 == 67108864u) && (_e107 == 0i));
            if _e207 {
                let _e212 = f32((_e201.z & 65535u));
                let _e215 = f32((_e201.z >> bitcast<u32>(16i)));
                let _e221 = vec2<i32>(i32((-1f - _e212)), i32(((_e215 - _e212) + 1f)));
                phi_2722_ = _e221;
                if ((_e203 & 8388608u) != 0u) {
                    phi_2722_ = -(_e221);
                }
                let _e226 = phi_2722_;
                let _e228 = (_e199 + _e226.x);
                let _e233 = textureLoad(KC, vec2<i32>((_e228 & 2047i), (_e228 >> bitcast<u32>(11i))), 0i);
                let _e235 = (_e199 + _e226.y);
                let _e240 = textureLoad(KC, vec2<i32>((_e235 & 2047i), (_e235 >> bitcast<u32>(11i))), 0i);
                phi_2723_ = _e240;
                if ((_e240.w & 8454143u) != (_e233.w & 8454143u)) {
                    let _e246 = bitcast<i32>(_e123.w);
                    let _e251 = textureLoad(KC, vec2<i32>((_e246 & 2047i), (_e246 >> bitcast<u32>(11i))), 0i);
                    phi_2723_ = _e251;
                }
                let _e253 = phi_2723_;
                let _e255 = bitcast<f32>(_e233.z);
                let _e257 = bitcast<f32>(_e253.z);
                let _e258 = (_e257 - _e255);
                phi_2727_ = _e258;
                if (abs(_e258) > 3.1415927f) {
                    phi_2727_ = (_e258 - (6.2831855f * sign(_e258)));
                }
                let _e265 = phi_2727_;
                let _e266 = (_e215 + -2f);
                let _e272 = clamp(round(((abs(_e265) * 0.31830987f) * _e266)), 1f, (_e215 + -3f));
                let _e273 = (_e266 - _e272);
                if (_e212 <= _e273) {
                    phi_2808_ = _e160;
                    if (_e212 == _e273) {
                        phi_2808_ = -(_e160);
                    }
                    let _e290 = phi_2808_;
                    phi_2806_ = _e290;
                    phi_2749_ = -(((3.1415927f * sign(_e265)) - _e265));
                    phi_2742_ = _e273;
                    phi_2739_ = _e212;
                } else {
                    let _e276 = (_e212 == (_e273 + 1f));
                    if _e276 {
                        phi_2740_ = 0f;
                    } else {
                        phi_2740_ = (_e212 - (_e273 + 2f));
                    }
                    let _e280 = phi_2740_;
                    phi_2806_ = select(_e160, 0f, _e276);
                    phi_2749_ = _e265;
                    phi_2742_ = select(_e272, 0f, _e276);
                    phi_2739_ = _e280;
                }
                let _e292 = phi_2806_;
                let _e294 = phi_2749_;
                let _e296 = phi_2742_;
                let _e298 = phi_2739_;
                if (_e298 == _e296) {
                    phi_2753_ = _e257;
                } else {
                    phi_2753_ = (_e255 + (_e294 * (_e298 / _e296)));
                }
                let _e304 = phi_2753_;
                phi_2828_ = _e255;
                phi_2819_ = _e294;
                phi_2804_ = _e292;
                phi_2752_ = _e304;
            } else {
                phi_2828_ = f32();
                phi_2819_ = f32();
                phi_2804_ = _e160;
                phi_2752_ = bitcast<f32>(_e201.z);
            }
            let _e306 = phi_2828_;
            let _e308 = phi_2819_;
            let _e310 = phi_2804_;
            let _e312 = phi_2752_;
            let _e316 = vec2<f32>(sin(_e312), -(cos(_e312)));
            let _e318 = bitcast<vec2<f32>>(_e201.xy);
            phi_2802_ = _e150;
            if (_e150 != 0f) {
                phi_2802_ = max(_e150, (1f / length((_e140 * _e316))));
            }
            let _e325 = phi_2802_;
            if (_e148 != 0f) {
                let _e433 = (_e310 * sign(determinant(_e140)));
                let _e435 = ((_e203 & 1048576u) != 0u);
                phi_2885_ = _e433;
                if _e435 {
                    phi_2885_ = min(_e433, 0f);
                }
                let _e438 = phi_2885_;
                phi_2896_ = _e438;
                if ((_e203 & 524288u) != 0u) {
                    phi_2896_ = max(_e438, 0f);
                }
                let _e443 = phi_2896_;
                let _e444 = (_e325 != 0f);
                if _e444 {
                    phi_2888_ = _e325;
                } else {
                    let _e445 = (_e140 * _e316);
                    phi_2888_ = (((abs(_e445.x) + abs(_e445.y)) * (1f / dot(_e445, _e445))) * 0.5f);
                }
                let _e456 = phi_2888_;
                let _e459 = ((_e456 > _e148) && (_e325 == 0f));
                phi_2977_ = 1f;
                if _e459 {
                    phi_2977_ = (_e148 / _e456);
                }
                let _e462 = phi_2977_;
                let _e463 = select(_e148, _e456, _e459);
                let _e464 = (_e463 + _e456);
                let _e465 = (_e316 * _e464);
                let _e466 = (_e443 * _e464);
                let _e473 = (((vec2<f32>(_e466, -(_e466)) + vec2(_e463)) * (0.5f / _e456)) + vec2<f32>(0.5f, 0.5f));
                let _e476 = vec4<f32>(_e473.x, _e473.y, 0f, 0f);
                phi_3002_ = _e465;
                phi_2986_ = _e476;
                if (_e204 > 134217728u) {
                    let _e478 = (_e203 & 4194304u);
                    let _e480 = select(2i, -2i, (_e478 == 0u));
                    phi_2934_ = _e480;
                    if ((_e203 & 8388608u) != 0u) {
                        phi_2934_ = -(_e480);
                    }
                    let _e485 = phi_2934_;
                    let _e486 = (_e199 + _e485);
                    let _e491 = textureLoad(KC, vec2<i32>((_e486 & 2047i), (_e486 >> bitcast<u32>(11i))), 0i);
                    let _e495 = abs((bitcast<f32>(_e491.z) - _e312));
                    phi_2943_ = _e495;
                    if (_e495 > 3.1415927f) {
                        phi_2943_ = (6.2831855f - _e495);
                    }
                    let _e499 = phi_2943_;
                    let _e504 = ((_e499 * select(0.5f, -0.5f, ((_e478 != 0u) == _e435))) + _e312);
                    let _e508 = vec2<f32>(sin(_e504), -(cos(_e504)));
                    let _e509 = (_e140 * _e508);
                    let _e517 = ((abs(_e509.x) + abs(_e509.y)) * (1f / dot(_e509, _e509)));
                    let _e519 = cos((_e499 * 0.5f));
                    let _e520 = (_e204 == 335544320u);
                    phi_2121_ = _e520;
                    if !(_e520) {
                        phi_2121_ = ((_e204 == 268435456u) && (_e519 >= 0.25f));
                    }
                    let _e526 = phi_2121_;
                    if _e526 {
                        phi_2950_ = (_e463 * (1f / max(_e519, select(0.25f, 1f, ((_e203 & 33554432u) != 0u)))));
                    } else {
                        phi_2950_ = ((_e463 * _e519) + (_e517 * 0.5f));
                    }
                    let _e537 = phi_2950_;
                    let _e539 = (_e537 + (_e517 * 0.5f));
                    phi_2965_ = _e465;
                    if ((_e203 & 2097152u) != 0u) {
                        if (_e464 <= ((_e539 * _e519) + (_e456 * 0.125f))) {
                            phi_2966_ = (_e508 * (_e464 * (1f / _e519)));
                        } else {
                            let _e546 = (_e508 * _e539);
                            phi_2966_ = (vec2<f32>(dot(_e465, _e465), dot(_e546, _e546)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e465, _e546)));
                        }
                        let _e557 = phi_2966_;
                        phi_2965_ = _e557;
                    }
                    let _e559 = phi_2965_;
                    let _e564 = ((_e539 - dot((_e559 * abs(_e443)), _e508)) / _e517);
                    if _e435 {
                        phi_2987_ = vec4<f32>(_e476.x, _e564, _e476.z, _e476.w);
                    } else {
                        phi_2987_ = vec4<f32>(_e564, _e476.y, _e476.z, _e476.w);
                    }
                    let _e576 = phi_2987_;
                    phi_3002_ = _e559;
                    phi_2986_ = _e576;
                }
                let _e578 = phi_3002_;
                let _e580 = phi_2986_;
                let _e582 = (_e580.xy * _e462);
                let _e588 = vec4<f32>(_e582.x, _e580.y, _e580.z, _e580.w);
                let _e595 = vec4<f32>(_e588.x, max(_e582.y, 0.0001f), _e588.z, _e588.w);
                phi_3035_ = _e595;
                if _e444 {
                    phi_3035_ = vec4<f32>((-2f - _e582.x), _e595.y, _e595.z, _e595.w);
                }
                let _e603 = phi_3035_;
                if (_e107 != 0i) {
                    phi_3075_ = _e603;
                    phi_3037_ = vec2<f32>();
                    phi_3036_ = false;
                    break;
                }
                phi_3032_ = _e603;
                phi_3028_ = (_e140 * (_e578 * _e443));
                phi_3004_ = _e318;
            } else {
                let _e327 = vec4<f32>(_e158, -1f, 0f, 0f);
                if (_e325 != 0f) {
                    let _e338 = vec4<f32>(_e327.x, -2f, _e327.z, _e327.w);
                    let _e343 = vec4<f32>(_e338.x, _e338.y, 1000000f, _e338.w);
                    phi_2881_ = vec4<f32>(_e343.x, _e343.y, _e343.z, _e158);
                    if _e207 {
                        phi_2838_ = _e308;
                        phi_2837_ = _e306;
                        if (_e308 < 0f) {
                            phi_2838_ = -(_e308);
                            phi_2837_ = (_e306 + _e308);
                        }
                        let _e353 = phi_2838_;
                        let _e355 = phi_2837_;
                        let _e357 = ((_e312 - _e355) + 1.5707964f);
                        let _e363 = clamp(((_e357 - (floor((_e357 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e353);
                        phi_2839_ = _e363;
                        if (_e363 > (_e353 * 0.5f)) {
                            phi_2839_ = (_e353 - _e363);
                        }
                        let _e368 = phi_2839_;
                        let _e375 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e368), cos(_e368)) * abs(_e310))) * 0.5f);
                        if (abs((_e353 - 1.5707964f)) < 0.001f) {
                            phi_2865_ = 0f;
                            phi_2863_ = 0f;
                        } else {
                            let _e379 = tan(_e353);
                            let _e384 = (sign((1.5707964f - _e353)) / max(abs(_e379), 0.000001f));
                            if (_e384 >= 0f) {
                                phi_2843_ = (_e375.y - ((1f - _e375.x) * _e379));
                            } else {
                                phi_2843_ = (_e375.y + (_e375.x * _e379));
                            }
                            let _e396 = phi_2843_;
                            phi_2865_ = _e396;
                            phi_2863_ = _e384;
                        }
                        let _e398 = phi_2865_;
                        let _e400 = phi_2863_;
                        phi_2881_ = vec4<f32>((max(_e375.x, 0f) + 0.25f), (-2f - _e375.y), _e400, _e398);
                    }
                    let _e408 = phi_2881_;
                    phi_3031_ = (_e140 * (_e316 * (_e310 * _e325)));
                    phi_2880_ = _e408;
                } else {
                    phi_3031_ = (sign(((_e316 * _e310) * _naga_inverse_2x2_f32(_e140))) * 0.5f);
                    phi_2880_ = _e327;
                }
                let _e413 = phi_3031_;
                let _e415 = phi_2880_;
                phi_3034_ = _e415;
                if (((_e203 & 8388608u) != 0u) != ((_e203 & 16777216u) != 0u)) {
                    phi_3034_ = (_e415 * vec4<f32>(-1f, 1f, 1f, 1f));
                }
                let _e423 = phi_3034_;
                if (((_e203 & 2147483648u) != 0u) && (_e107 != 1i)) {
                    phi_3075_ = _e423;
                    phi_3037_ = vec2<f32>();
                    phi_3036_ = false;
                    break;
                }
                phi_3032_ = _e423;
                phi_3028_ = _e413;
                phi_3004_ = select(_e318, _e125, vec2((_e107 == 2i)));
            }
            let _e608 = phi_3032_;
            let _e610 = phi_3028_;
            let _e612 = phi_3004_;
            let _e618 = m.Ig;
            let _e621 = select(_e608.xy, vec2<f32>(1f, -1f), vec2((_e618 != 0u)));
            let _e627 = vec4<f32>(_e621.x, _e608.y, _e608.z, _e608.w);
            phi_3075_ = vec4<f32>(_e627.x, _e621.y, _e627.z, _e627.w);
            phi_3037_ = (((_e140 * _e612) + _e610) + bitcast<vec2<f32>>(_e144.xy));
            phi_3036_ = true;
            break;
        }
    }
    let _e635 = phi_3075_;
    let _e637 = phi_3037_;
    let _e639 = phi_3036_;
    L = _e635;
    let _e642 = local;
    let _e644 = AD.d2_[_e642];
    let _e646 = m.d6_;
    let _e648 = local_1;
    if (_e648 == 0u) {
        phi_3076_ = 0f;
    } else {
        let _e651 = local_2;
        phi_3076_ = unpack2x16float(((_e651 + 1023u) * _e646)).x;
    }
    let _e657 = phi_3076_;
    B0_ = _e657;
    if ((_e644.x & 512u) != 0u) {
        let _e661 = B0_;
        B0_ = -(_e661);
    }
    let _e663 = (_e644.x & 15u);
    if jh {
        let _e664 = (_e663 == 0u);
        if _e664 {
            phi_3077_ = _e644.y;
        } else {
            phi_3077_ = _e644.x;
        }
        let _e667 = phi_3077_;
        let _e669 = (_e667 >> bitcast<u32>(16i));
        if (_e669 == 0u) {
            phi_3078_ = 0f;
        } else {
            phi_3078_ = unpack2x16float(((_e669 + 1023u) * _e646)).x;
        }
        let _e676 = phi_3078_;
        phi_3079_ = _e676;
        if _e664 {
            phi_3079_ = -(_e676);
        }
        let _e679 = phi_3079_;
        V1_[0u] = _e679;
    }
    if lh {
        f2_ = f32(((_e644.x >> bitcast<u32>(4i)) & 15u));
    }
    if kh {
        let _e686 = local_3;
        let _e687 = (_e686 * 8u);
        let _e691 = QB.d2_[(_e687 + 2u)];
        let _e696 = vec2<f32>(_e691.x, _e691.y);
        let _e697 = vec2<f32>(_e691.z, _e691.w);
        let _e702 = QB.d2_[(_e687 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e707 = (abs(_e696) + abs(_e697));
                let _e709 = (_e707.x != 0f);
                phi_2521_ = _e709;
                if _e709 {
                    phi_2521_ = (_e707.y != 0f);
                }
                let _e713 = phi_2521_;
                if _e713 {
                    let _e717 = ((mat2x2<f32>(_e696, _e697) * _e637) + _e702.xy);
                    let _e718 = -(_e717);
                    let _e724 = (vec2<f32>(1f, 1f) / _e707).xyxy;
                    phi_3080_ = (((vec4<f32>(_e717.x, _e717.y, _e718.x, _e718.y) * _e724) + _e724) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3080_ = _e702.xyxy;
                    break;
                }
            }
        }
        let _e729 = phi_3080_;
        M0_ = _e729;
    }
    if (_e663 == 1u) {
        f1_ = unpack4x8unorm(_e644.y);
    } else {
        if (jh && (_e663 == 0u)) {
            let _e774 = (_e644.x >> bitcast<u32>(16i));
            if (_e774 == 0u) {
                phi_3081_ = 0f;
            } else {
                phi_3081_ = unpack2x16float(((_e774 + 1023u) * _e646)).x;
            }
            let _e781 = phi_3081_;
            V1_[1u] = _e781;
        } else {
            let _e734 = local_4;
            let _e735 = (_e734 * 8u);
            let _e738 = QB.d2_[_e735];
            let _e749 = QB.d2_[(_e735 + 1u)];
            let _e752 = ((mat2x2<f32>(vec2<f32>(_e738.x, _e738.y), vec2<f32>(_e738.z, _e738.w)) * _e637) + _e749.xy);
            f1_[3u] = -(bitcast<f32>(_e644.y));
            if (_e749.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e749.w;
            }
            if (_e663 == 2u) {
                f1_[1u] = 0f;
                f1_[0u] = _e752.x;
            } else {
                let _e764 = f1_[2u];
                f1_[2u] = -(_e764);
                f1_[0u] = _e752.x;
                f1_[1u] = _e752.y;
            }
        }
    }
    phi_1376_ = rh;
    if rh {
        phi_1376_ = ((_e644.x & 2048u) != 0u);
    }
    let _e788 = phi_1376_;
    if _e788 {
        let _e790 = local_5;
        let _e791 = (_e790 * 8u);
        let _e795 = QB.d2_[(_e791 + 4u)];
        let _e806 = QB.d2_[(_e791 + 5u)];
        let _e809 = ((mat2x2<f32>(vec2<f32>(_e795.x, _e795.y), vec2<f32>(_e795.z, _e795.w)) * _e637) + _e806.xy);
        A2_ = vec3<f32>(_e809.x, _e809.y, (1f + _e806.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e639 {
        let _e819 = m.of_;
        let _e821 = m.pf;
        let _e831 = local_6;
        let _e835 = PB.d2_[(_e831 + 3u)];
        f3_ = _e835.xy;
        o4_ = (_e637 + bitcast<vec2<f32>>(_e835.zw));
        phi_3099_ = vec4<f32>(((_e637.x * _e819) - 1f), ((_e637.y * _e821) - sign(_e821)), 0f, 1f);
    } else {
        let _e816 = m.R2_;
        phi_3099_ = vec4(_e816);
    }
    let _e841 = phi_3099_;
    unnamed.gl_Position = _e841;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e21 = L;
    let _e22 = B0_;
    let _e23 = V1_;
    let _e24 = f2_;
    let _e25 = M0_;
    let _e26 = f1_;
    let _e27 = A2_;
    let _e28 = f3_;
    let _e29 = o4_;
    let _e30 = unnamed.gl_Position;
    return VertexOutput(_e21, _e22, _e23, _e24, _e25, _e26, _e27, _e28, _e29, _e30);
}

fn _naga_inverse_2x2_f32(m: mat2x2<f32>) -> mat2x2<f32> {
    var adj: mat2x2<f32>;
    adj[0][0] = m[1][1];
    adj[0][1] = -m[0][1];
    adj[1][0] = -m[1][0];
    adj[1][1] = m[0][0];

    let det: f32 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
    return adj * (1 / det);
}
