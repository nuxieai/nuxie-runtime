struct jg {
    d2_: array<vec4<u32>>,
}

struct ig {
    d2_: array<vec4<u32>>,
}

struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct Ne {
    d2_: array<vec2<u32>>,
}

struct Oe {
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

@id(0) override fh: bool = true;
@id(2) override hh: bool = true;
@id(1) override gh: bool = true;
@id(8) override nh: bool = true;

@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> FD: jg;
@group(0) @binding(2)
var<storage> QB: ig;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
var<private> O: vec4<f32>;
@group(0) @binding(3)
var<storage> BD: Ne;
var<private> B0_: f32;
var<private> V1_: vec2<f32>;
var<private> f2_: f32;
@group(0) @binding(4)
var<storage> RB: Oe;
var<private> M0_: vec4<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
var<private> f3_: vec2<u32>;
var<private> n4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_2798_: f32;
    var phi_2742_: f32;
    var phi_2714_: i32;
    var phi_1696_: bool;
    var phi_2727_: i32;
    var phi_2719_: vec4<u32>;
    var phi_2726_: i32;
    var phi_2718_: vec4<u32>;
    var phi_2725_: i32;
    var phi_2723_: vec4<u32>;
    var phi_2722_: u32;
    var phi_2729_: vec2<i32>;
    var phi_2730_: vec4<u32>;
    var phi_2734_: f32;
    var phi_2747_: f32;
    var phi_2815_: f32;
    var phi_2813_: f32;
    var phi_2756_: f32;
    var phi_2749_: f32;
    var phi_2746_: f32;
    var phi_2760_: f32;
    var phi_2835_: f32;
    var phi_2826_: f32;
    var phi_2811_: f32;
    var phi_2759_: f32;
    var phi_2809_: f32;
    var phi_2845_: f32;
    var phi_2844_: f32;
    var phi_2846_: f32;
    var phi_2850_: f32;
    var phi_2872_: f32;
    var phi_2870_: f32;
    var phi_2888_: vec4<f32>;
    var phi_3038_: vec2<f32>;
    var phi_2887_: vec4<f32>;
    var phi_3041_: vec4<f32>;
    var phi_2892_: f32;
    var phi_2903_: f32;
    var phi_2895_: f32;
    var phi_2984_: f32;
    var phi_2941_: i32;
    var phi_2950_: f32;
    var phi_2128_: bool;
    var phi_2957_: f32;
    var phi_2973_: vec2<f32>;
    var phi_2972_: vec2<f32>;
    var phi_2994_: vec4<f32>;
    var phi_3009_: vec2<f32>;
    var phi_2993_: vec4<f32>;
    var phi_3042_: vec4<f32>;
    var phi_3039_: vec4<f32>;
    var phi_3035_: vec2<f32>;
    var phi_3011_: vec2<f32>;
    var phi_3082_: vec4<f32>;
    var phi_3044_: vec2<f32>;
    var phi_3043_: bool;
    var local: u32;
    var local_1: u32;
    var local_2: u32;
    var phi_3083_: f32;
    var phi_3084_: u32;
    var phi_3085_: f32;
    var phi_3086_: f32;
    var local_3: u32;
    var phi_2528_: bool;
    var phi_3087_: vec4<f32>;
    var local_4: u32;
    var phi_3088_: f32;
    var phi_1383_: bool;
    var local_5: u32;
    var local_6: u32;
    var phi_3107_: vec4<f32>;

    let _e95 = gl_InstanceIndex_1;
    let _e96 = VB_1;
    let _e97 = WB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e100 = i32(_e96.x);
            let _e104 = bitcast<i32>(_e96.w);
            let _e106 = (_e104 >> bitcast<u32>(2i));
            let _e107 = (_e104 & 3i);
            let _e109 = min(_e100, (_e106 - 1i));
            let _e111 = ((_e95 * _e106) + _e109);
            let _e116 = textureLoad(MC, vec2<i32>((_e111 & 2047i), (_e111 >> bitcast<u32>(11i))), 0i);
            let _e123 = FD.d2_[(max((_e116.w & 65535u), 1u) - 1u)];
            let _e125 = bitcast<vec2<f32>>(_e123.xy);
            let _e127 = (_e123.z & 65535u);
            let _e129 = (_e127 * 4u);
            let _e132 = QB.d2_[_e129];
            let _e133 = bitcast<vec4<f32>>(_e132);
            let _e140 = mat2x2<f32>(vec2<f32>(_e133.x, _e133.y), vec2<f32>(_e133.z, _e133.w));
            let _e144 = QB.d2_[(_e129 + 1u)];
            let _e148 = bitcast<f32>(_e144.z);
            let _e150 = bitcast<f32>(_e144.w);
            let _e151 = (_e116.w & 8388608u);
            phi_2798_ = _e96.z;
            phi_2742_ = _e96.y;
            phi_2714_ = _e100;
            local = _e127;
            local_1 = _e127;
            local_2 = _e127;
            local_3 = _e127;
            local_4 = _e127;
            local_5 = _e127;
            local_6 = _e129;
            if (_e151 != 0u) {
                phi_2798_ = _e97.z;
                phi_2742_ = _e97.y;
                phi_2714_ = i32(_e97.x);
            }
            let _e158 = phi_2798_;
            let _e160 = phi_2742_;
            let _e162 = phi_2714_;
            phi_2725_ = _e111;
            phi_2723_ = _e116;
            phi_2722_ = _e116.w;
            if (_e162 != _e109) {
                let _e165 = ((_e111 + _e162) - _e109);
                let _e170 = textureLoad(MC, vec2<i32>((_e165 & 2047i), (_e165 >> bitcast<u32>(11i))), 0i);
                if ((_e170.w & 8454143u) != (_e116.w & 8454143u)) {
                    let _e175 = (_e148 == 0f);
                    phi_1696_ = _e175;
                    if !(_e175) {
                        phi_1696_ = (_e125.x != 0f);
                    }
                    let _e180 = phi_1696_;
                    phi_2727_ = _e111;
                    phi_2719_ = _e116;
                    if _e180 {
                        let _e181 = bitcast<i32>(_e123.w);
                        let _e186 = textureLoad(MC, vec2<i32>((_e181 & 2047i), (_e181 >> bitcast<u32>(11i))), 0i);
                        phi_2727_ = _e181;
                        phi_2719_ = _e186;
                    }
                    let _e188 = phi_2727_;
                    let _e190 = phi_2719_;
                    phi_2726_ = _e188;
                    phi_2718_ = _e190;
                } else {
                    phi_2726_ = _e165;
                    phi_2718_ = _e170;
                }
                let _e192 = phi_2726_;
                let _e194 = phi_2718_;
                phi_2725_ = _e192;
                phi_2723_ = _e194;
                phi_2722_ = ((_e194.w & 4286578687u) | _e151);
            }
            let _e199 = phi_2725_;
            let _e201 = phi_2723_;
            let _e203 = phi_2722_;
            let _e204 = (_e203 & 469762048u);
            let _e207 = ((_e204 == 67108864u) && (_e107 == 0i));
            if _e207 {
                let _e212 = f32((_e201.z & 65535u));
                let _e215 = f32((_e201.z >> bitcast<u32>(16i)));
                let _e221 = vec2<i32>(i32((-1f - _e212)), i32(((_e215 - _e212) + 1f)));
                phi_2729_ = _e221;
                if ((_e203 & 8388608u) != 0u) {
                    phi_2729_ = -(_e221);
                }
                let _e226 = phi_2729_;
                let _e228 = (_e199 + _e226.x);
                let _e233 = textureLoad(MC, vec2<i32>((_e228 & 2047i), (_e228 >> bitcast<u32>(11i))), 0i);
                let _e235 = (_e199 + _e226.y);
                let _e240 = textureLoad(MC, vec2<i32>((_e235 & 2047i), (_e235 >> bitcast<u32>(11i))), 0i);
                phi_2730_ = _e240;
                if ((_e240.w & 8454143u) != (_e233.w & 8454143u)) {
                    let _e246 = bitcast<i32>(_e123.w);
                    let _e251 = textureLoad(MC, vec2<i32>((_e246 & 2047i), (_e246 >> bitcast<u32>(11i))), 0i);
                    phi_2730_ = _e251;
                }
                let _e253 = phi_2730_;
                let _e255 = bitcast<f32>(_e233.z);
                let _e257 = bitcast<f32>(_e253.z);
                let _e258 = (_e257 - _e255);
                phi_2734_ = _e258;
                if (abs(_e258) > 3.1415927f) {
                    phi_2734_ = (_e258 - (6.2831855f * sign(_e258)));
                }
                let _e265 = phi_2734_;
                let _e266 = (_e215 + -2f);
                let _e272 = clamp(round(((abs(_e265) * 0.31830987f) * _e266)), 1f, (_e215 + -3f));
                let _e273 = (_e266 - _e272);
                if (_e212 <= _e273) {
                    phi_2815_ = _e160;
                    if (_e212 == _e273) {
                        phi_2815_ = -(_e160);
                    }
                    let _e290 = phi_2815_;
                    phi_2813_ = _e290;
                    phi_2756_ = -(((3.1415927f * sign(_e265)) - _e265));
                    phi_2749_ = _e273;
                    phi_2746_ = _e212;
                } else {
                    let _e276 = (_e212 == (_e273 + 1f));
                    if _e276 {
                        phi_2747_ = 0f;
                    } else {
                        phi_2747_ = (_e212 - (_e273 + 2f));
                    }
                    let _e280 = phi_2747_;
                    phi_2813_ = select(_e160, 0f, _e276);
                    phi_2756_ = _e265;
                    phi_2749_ = select(_e272, 0f, _e276);
                    phi_2746_ = _e280;
                }
                let _e292 = phi_2813_;
                let _e294 = phi_2756_;
                let _e296 = phi_2749_;
                let _e298 = phi_2746_;
                if (_e298 == _e296) {
                    phi_2760_ = _e257;
                } else {
                    phi_2760_ = (_e255 + (_e294 * (_e298 / _e296)));
                }
                let _e304 = phi_2760_;
                phi_2835_ = _e255;
                phi_2826_ = _e294;
                phi_2811_ = _e292;
                phi_2759_ = _e304;
            } else {
                phi_2835_ = f32();
                phi_2826_ = f32();
                phi_2811_ = _e160;
                phi_2759_ = bitcast<f32>(_e201.z);
            }
            let _e306 = phi_2835_;
            let _e308 = phi_2826_;
            let _e310 = phi_2811_;
            let _e312 = phi_2759_;
            let _e316 = vec2<f32>(sin(_e312), -(cos(_e312)));
            let _e318 = bitcast<vec2<f32>>(_e201.xy);
            phi_2809_ = _e150;
            if (_e150 != 0f) {
                phi_2809_ = max(_e150, (1f / length((_e140 * _e316))));
            }
            let _e325 = phi_2809_;
            if (_e148 != 0f) {
                let _e433 = (_e310 * sign(determinant(_e140)));
                let _e435 = ((_e203 & 1048576u) != 0u);
                phi_2892_ = _e433;
                if _e435 {
                    phi_2892_ = min(_e433, 0f);
                }
                let _e438 = phi_2892_;
                phi_2903_ = _e438;
                if ((_e203 & 524288u) != 0u) {
                    phi_2903_ = max(_e438, 0f);
                }
                let _e443 = phi_2903_;
                let _e444 = (_e325 != 0f);
                if _e444 {
                    phi_2895_ = _e325;
                } else {
                    let _e445 = (_e140 * _e316);
                    phi_2895_ = (((abs(_e445.x) + abs(_e445.y)) * (1f / dot(_e445, _e445))) * 0.5f);
                }
                let _e456 = phi_2895_;
                let _e459 = ((_e456 > _e148) && (_e325 == 0f));
                phi_2984_ = 1f;
                if _e459 {
                    phi_2984_ = (_e148 / _e456);
                }
                let _e462 = phi_2984_;
                let _e463 = select(_e148, _e456, _e459);
                let _e464 = (_e463 + _e456);
                let _e465 = (_e316 * _e464);
                let _e466 = (_e443 * _e464);
                let _e473 = (((vec2<f32>(_e466, -(_e466)) + vec2(_e463)) * (0.5f / _e456)) + vec2<f32>(0.5f, 0.5f));
                let _e476 = vec4<f32>(_e473.x, _e473.y, 0f, 0f);
                phi_3009_ = _e465;
                phi_2993_ = _e476;
                if (_e204 > 134217728u) {
                    let _e478 = (_e203 & 4194304u);
                    let _e480 = select(2i, -2i, (_e478 == 0u));
                    phi_2941_ = _e480;
                    if ((_e203 & 8388608u) != 0u) {
                        phi_2941_ = -(_e480);
                    }
                    let _e485 = phi_2941_;
                    let _e486 = (_e199 + _e485);
                    let _e491 = textureLoad(MC, vec2<i32>((_e486 & 2047i), (_e486 >> bitcast<u32>(11i))), 0i);
                    let _e495 = abs((bitcast<f32>(_e491.z) - _e312));
                    phi_2950_ = _e495;
                    if (_e495 > 3.1415927f) {
                        phi_2950_ = (6.2831855f - _e495);
                    }
                    let _e499 = phi_2950_;
                    let _e504 = ((_e499 * select(0.5f, -0.5f, ((_e478 != 0u) == _e435))) + _e312);
                    let _e508 = vec2<f32>(sin(_e504), -(cos(_e504)));
                    let _e509 = (_e140 * _e508);
                    let _e517 = ((abs(_e509.x) + abs(_e509.y)) * (1f / dot(_e509, _e509)));
                    let _e519 = cos((_e499 * 0.5f));
                    let _e520 = (_e204 == 335544320u);
                    phi_2128_ = _e520;
                    if !(_e520) {
                        phi_2128_ = ((_e204 == 268435456u) && (_e519 >= 0.25f));
                    }
                    let _e526 = phi_2128_;
                    if _e526 {
                        phi_2957_ = (_e463 * (1f / max(_e519, select(0.25f, 1f, ((_e203 & 33554432u) != 0u)))));
                    } else {
                        phi_2957_ = ((_e463 * _e519) + (_e517 * 0.5f));
                    }
                    let _e537 = phi_2957_;
                    let _e539 = (_e537 + (_e517 * 0.5f));
                    phi_2972_ = _e465;
                    if ((_e203 & 2097152u) != 0u) {
                        if (_e464 <= ((_e539 * _e519) + (_e456 * 0.125f))) {
                            phi_2973_ = (_e508 * (_e464 * (1f / _e519)));
                        } else {
                            let _e546 = (_e508 * _e539);
                            phi_2973_ = (vec2<f32>(dot(_e465, _e465), dot(_e546, _e546)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e465, _e546)));
                        }
                        let _e557 = phi_2973_;
                        phi_2972_ = _e557;
                    }
                    let _e559 = phi_2972_;
                    let _e564 = ((_e539 - dot((_e559 * abs(_e443)), _e508)) / _e517);
                    if _e435 {
                        phi_2994_ = vec4<f32>(_e476.x, _e564, _e476.z, _e476.w);
                    } else {
                        phi_2994_ = vec4<f32>(_e564, _e476.y, _e476.z, _e476.w);
                    }
                    let _e576 = phi_2994_;
                    phi_3009_ = _e559;
                    phi_2993_ = _e576;
                }
                let _e578 = phi_3009_;
                let _e580 = phi_2993_;
                let _e582 = (_e580.xy * _e462);
                let _e588 = vec4<f32>(_e582.x, _e580.y, _e580.z, _e580.w);
                let _e595 = vec4<f32>(_e588.x, max(_e582.y, 0.0001f), _e588.z, _e588.w);
                phi_3042_ = _e595;
                if _e444 {
                    phi_3042_ = vec4<f32>((-2f - _e582.x), _e595.y, _e595.z, _e595.w);
                }
                let _e603 = phi_3042_;
                if (_e107 != 0i) {
                    phi_3082_ = _e603;
                    phi_3044_ = vec2<f32>();
                    phi_3043_ = false;
                    break;
                }
                phi_3039_ = _e603;
                phi_3035_ = (_e140 * (_e578 * _e443));
                phi_3011_ = _e318;
            } else {
                let _e327 = vec4<f32>(_e158, -1f, 0f, 0f);
                if (_e325 != 0f) {
                    let _e338 = vec4<f32>(_e327.x, -2f, _e327.z, _e327.w);
                    let _e343 = vec4<f32>(_e338.x, _e338.y, 1000000f, _e338.w);
                    phi_2888_ = vec4<f32>(_e343.x, _e343.y, _e343.z, _e158);
                    if _e207 {
                        phi_2845_ = _e308;
                        phi_2844_ = _e306;
                        if (_e308 < 0f) {
                            phi_2845_ = -(_e308);
                            phi_2844_ = (_e306 + _e308);
                        }
                        let _e353 = phi_2845_;
                        let _e355 = phi_2844_;
                        let _e357 = ((_e312 - _e355) + 1.5707964f);
                        let _e363 = clamp(((_e357 - (floor((_e357 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e353);
                        phi_2846_ = _e363;
                        if (_e363 > (_e353 * 0.5f)) {
                            phi_2846_ = (_e353 - _e363);
                        }
                        let _e368 = phi_2846_;
                        let _e375 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e368), cos(_e368)) * abs(_e310))) * 0.5f);
                        if (abs((_e353 - 1.5707964f)) < 0.001f) {
                            phi_2872_ = 0f;
                            phi_2870_ = 0f;
                        } else {
                            let _e379 = tan(_e353);
                            let _e384 = (sign((1.5707964f - _e353)) / max(abs(_e379), 0.000001f));
                            if (_e384 >= 0f) {
                                phi_2850_ = (_e375.y - ((1f - _e375.x) * _e379));
                            } else {
                                phi_2850_ = (_e375.y + (_e375.x * _e379));
                            }
                            let _e396 = phi_2850_;
                            phi_2872_ = _e396;
                            phi_2870_ = _e384;
                        }
                        let _e398 = phi_2872_;
                        let _e400 = phi_2870_;
                        phi_2888_ = vec4<f32>((max(_e375.x, 0f) + 0.25f), (-2f - _e375.y), _e400, _e398);
                    }
                    let _e408 = phi_2888_;
                    phi_3038_ = (_e140 * (_e316 * (_e310 * _e325)));
                    phi_2887_ = _e408;
                } else {
                    phi_3038_ = (sign(((_e316 * _e310) * _naga_inverse_2x2_f32(_e140))) * 0.5f);
                    phi_2887_ = _e327;
                }
                let _e413 = phi_3038_;
                let _e415 = phi_2887_;
                phi_3041_ = _e415;
                if (((_e203 & 8388608u) != 0u) != ((_e203 & 16777216u) != 0u)) {
                    phi_3041_ = (_e415 * vec4<f32>(-1f, 1f, 1f, 1f));
                }
                let _e423 = phi_3041_;
                if (((_e203 & 2147483648u) != 0u) && (_e107 != 1i)) {
                    phi_3082_ = _e423;
                    phi_3044_ = vec2<f32>();
                    phi_3043_ = false;
                    break;
                }
                phi_3039_ = _e423;
                phi_3035_ = _e413;
                phi_3011_ = select(_e318, _e125, vec2((_e107 == 2i)));
            }
            let _e608 = phi_3039_;
            let _e610 = phi_3035_;
            let _e612 = phi_3011_;
            let _e618 = m.Fg;
            let _e621 = select(_e608.xy, vec2<f32>(1f, -1f), vec2((_e618 != 0u)));
            let _e627 = vec4<f32>(_e621.x, _e608.y, _e608.z, _e608.w);
            phi_3082_ = vec4<f32>(_e627.x, _e621.y, _e627.z, _e627.w);
            phi_3044_ = (((_e140 * _e612) + _e610) + bitcast<vec2<f32>>(_e144.xy));
            phi_3043_ = true;
            break;
        }
    }
    let _e635 = phi_3082_;
    let _e637 = phi_3044_;
    let _e639 = phi_3043_;
    O = _e635;
    let _e642 = local;
    let _e644 = BD.d2_[_e642];
    let _e646 = m.d6_;
    let _e648 = local_1;
    if (_e648 == 0u) {
        phi_3083_ = 0f;
    } else {
        let _e651 = local_2;
        phi_3083_ = unpack2x16float(((_e651 + 1023u) * _e646)).x;
    }
    let _e657 = phi_3083_;
    B0_ = _e657;
    if ((_e644.x & 512u) != 0u) {
        let _e661 = B0_;
        B0_ = -(_e661);
    }
    let _e663 = (_e644.x & 15u);
    if fh {
        let _e664 = (_e663 == 0u);
        if _e664 {
            phi_3084_ = _e644.y;
        } else {
            phi_3084_ = _e644.x;
        }
        let _e667 = phi_3084_;
        let _e669 = (_e667 >> bitcast<u32>(16i));
        if (_e669 == 0u) {
            phi_3085_ = 0f;
        } else {
            phi_3085_ = unpack2x16float(((_e669 + 1023u) * _e646)).x;
        }
        let _e676 = phi_3085_;
        phi_3086_ = _e676;
        if _e664 {
            phi_3086_ = -(_e676);
        }
        let _e679 = phi_3086_;
        V1_[0u] = _e679;
    }
    if hh {
        f2_ = f32(((_e644.x >> bitcast<u32>(4i)) & 15u));
    }
    if gh {
        let _e686 = local_3;
        let _e687 = (_e686 * 8u);
        let _e691 = RB.d2_[(_e687 + 2u)];
        let _e696 = vec2<f32>(_e691.x, _e691.y);
        let _e697 = vec2<f32>(_e691.z, _e691.w);
        let _e702 = RB.d2_[(_e687 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e707 = (abs(_e696) + abs(_e697));
                let _e709 = (_e707.x != 0f);
                phi_2528_ = _e709;
                if _e709 {
                    phi_2528_ = (_e707.y != 0f);
                }
                let _e713 = phi_2528_;
                if _e713 {
                    let _e717 = ((mat2x2<f32>(_e696, _e697) * _e637) + _e702.xy);
                    let _e718 = -(_e717);
                    let _e724 = (vec2<f32>(1f, 1f) / _e707).xyxy;
                    phi_3087_ = (((vec4<f32>(_e717.x, _e717.y, _e718.x, _e718.y) * _e724) + _e724) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3087_ = _e702.xyxy;
                    break;
                }
            }
        }
        let _e729 = phi_3087_;
        M0_ = _e729;
    }
    if (_e663 == 1u) {
        f1_ = unpack4x8unorm(_e644.y);
    } else {
        if (fh && (_e663 == 0u)) {
            let _e776 = (_e644.x >> bitcast<u32>(16i));
            if (_e776 == 0u) {
                phi_3088_ = 0f;
            } else {
                phi_3088_ = unpack2x16float(((_e776 + 1023u) * _e646)).x;
            }
            let _e783 = phi_3088_;
            V1_[1u] = _e783;
        } else {
            let _e734 = local_4;
            let _e735 = (_e734 * 8u);
            let _e738 = RB.d2_[_e735];
            let _e749 = RB.d2_[(_e735 + 1u)];
            let _e752 = ((mat2x2<f32>(vec2<f32>(_e738.x, _e738.y), vec2<f32>(_e738.z, _e738.w)) * _e637) + _e749.xy);
            let _e753 = (_e663 == 2u);
            if (_e753 || (_e663 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e644.y));
                if (_e749.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e749.w;
                }
                if _e753 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e752.x;
                } else {
                    let _e766 = f1_[2u];
                    f1_[2u] = -(_e766);
                    f1_[0u] = _e752.x;
                    f1_[1u] = _e752.y;
                }
            }
        }
    }
    phi_1383_ = nh;
    if nh {
        phi_1383_ = ((_e644.x & 2048u) != 0u);
    }
    let _e790 = phi_1383_;
    if _e790 {
        let _e792 = local_5;
        let _e793 = (_e792 * 8u);
        let _e797 = RB.d2_[(_e793 + 4u)];
        let _e808 = RB.d2_[(_e793 + 5u)];
        let _e811 = ((mat2x2<f32>(vec2<f32>(_e797.x, _e797.y), vec2<f32>(_e797.z, _e797.w)) * _e637) + _e808.xy);
        A2_ = vec3<f32>(_e811.x, _e811.y, (1f + _e808.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e639 {
        let _e821 = m.kf;
        let _e823 = m.lf;
        let _e833 = local_6;
        let _e837 = QB.d2_[(_e833 + 3u)];
        f3_ = _e837.xy;
        n4_ = (_e637 + bitcast<vec2<f32>>(_e837.zw));
        phi_3107_ = vec4<f32>(((_e637.x * _e821) - 1f), ((_e637.y * _e823) - sign(_e823)), 0f, 1f);
    } else {
        let _e818 = m.R2_;
        phi_3107_ = vec4(_e818);
    }
    let _e843 = phi_3107_;
    unnamed.gl_Position = _e843;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) VB: vec4<f32>, @location(1) WB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    VB_1 = VB;
    WB_1 = WB;
    main_1();
    let _e21 = O;
    let _e22 = B0_;
    let _e23 = V1_;
    let _e24 = f2_;
    let _e25 = M0_;
    let _e26 = f1_;
    let _e27 = A2_;
    let _e28 = f3_;
    let _e29 = n4_;
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
