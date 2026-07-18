struct Yf {
    c2_: array<vec4<u32>>,
}

struct Xf {
    c2_: array<vec4<u32>>,
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

struct Fe {
    c2_: array<vec2<u32>>,
}

struct Ge {
    c2_: array<vec4<f32>>,
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
    @location(7) @interpolate(flat, either) member_6: vec2<u32>,
    @location(8) member_7: vec2<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Ug: bool = true;
@id(2) override Wg: bool = true;
@id(1) override Vg: bool = true;

@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ED: Yf;
@group(0) @binding(2)
var<storage> PB: Xf;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
var<private> O: vec4<f32>;
@group(0) @binding(3)
var<storage> AD: Fe;
var<private> A0_: f32;
var<private> U1_: vec2<f32>;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ge;
var<private> L0_: vec4<f32>;
var<private> f1_: vec4<f32>;
var<private> a3_: vec2<u32>;
var<private> l4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_2754_: f32;
    var phi_2698_: f32;
    var phi_2670_: i32;
    var phi_1665_: bool;
    var phi_2683_: i32;
    var phi_2675_: vec4<u32>;
    var phi_2682_: i32;
    var phi_2674_: vec4<u32>;
    var phi_2681_: i32;
    var phi_2679_: vec4<u32>;
    var phi_2678_: u32;
    var phi_2685_: vec2<i32>;
    var phi_2686_: vec4<u32>;
    var phi_2690_: f32;
    var phi_2703_: f32;
    var phi_2771_: f32;
    var phi_2769_: f32;
    var phi_2712_: f32;
    var phi_2705_: f32;
    var phi_2702_: f32;
    var phi_2716_: f32;
    var phi_2791_: f32;
    var phi_2782_: f32;
    var phi_2767_: f32;
    var phi_2715_: f32;
    var phi_2765_: f32;
    var phi_2801_: f32;
    var phi_2800_: f32;
    var phi_2802_: f32;
    var phi_2806_: f32;
    var phi_2828_: f32;
    var phi_2826_: f32;
    var phi_2844_: vec4<f32>;
    var phi_2994_: vec2<f32>;
    var phi_2843_: vec4<f32>;
    var phi_2997_: vec4<f32>;
    var phi_2848_: f32;
    var phi_2859_: f32;
    var phi_2851_: f32;
    var phi_2940_: f32;
    var phi_2897_: i32;
    var phi_2906_: f32;
    var phi_2097_: bool;
    var phi_2913_: f32;
    var phi_2929_: vec2<f32>;
    var phi_2928_: vec2<f32>;
    var phi_2950_: vec4<f32>;
    var phi_2965_: vec2<f32>;
    var phi_2949_: vec4<f32>;
    var phi_2998_: vec4<f32>;
    var phi_2995_: vec4<f32>;
    var phi_2991_: vec2<f32>;
    var phi_2967_: vec2<f32>;
    var phi_3038_: vec4<f32>;
    var phi_3000_: vec2<f32>;
    var phi_2999_: bool;
    var local: u32;
    var local_1: u32;
    var local_2: u32;
    var phi_3039_: f32;
    var phi_3040_: u32;
    var phi_3041_: f32;
    var phi_3042_: f32;
    var local_3: u32;
    var local_4: u32;
    var phi_2497_: bool;
    var phi_3043_: vec4<f32>;
    var local_5: u32;
    var local_6: u32;
    var phi_3044_: f32;
    var local_7: u32;
    var phi_3061_: vec4<f32>;

    let _e89 = gl_InstanceIndex_1;
    let _e90 = UB_1;
    let _e91 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e94 = i32(_e90.x);
            let _e98 = bitcast<i32>(_e90.w);
            let _e100 = (_e98 >> bitcast<u32>(2i));
            let _e101 = (_e98 & 3i);
            let _e103 = min(_e94, (_e100 - 1i));
            let _e105 = ((_e89 * _e100) + _e103);
            let _e110 = textureLoad(LC, vec2<i32>((_e105 & 2047i), (_e105 >> bitcast<u32>(11i))), 0i);
            let _e117 = ED.c2_[(max((_e110.w & 65535u), 1u) - 1u)];
            let _e119 = bitcast<vec2<f32>>(_e117.xy);
            let _e121 = (_e117.z & 65535u);
            let _e123 = (_e121 * 4u);
            let _e126 = PB.c2_[_e123];
            let _e127 = bitcast<vec4<f32>>(_e126);
            let _e134 = mat2x2<f32>(vec2<f32>(_e127.x, _e127.y), vec2<f32>(_e127.z, _e127.w));
            let _e135 = (_e123 + 1u);
            let _e138 = PB.c2_[_e135];
            let _e142 = bitcast<f32>(_e138.z);
            let _e144 = bitcast<f32>(_e138.w);
            let _e145 = (_e110.w & 8388608u);
            phi_2754_ = _e90.z;
            phi_2698_ = _e90.y;
            phi_2670_ = _e94;
            local = _e121;
            local_1 = _e121;
            local_2 = _e121;
            local_3 = _e123;
            local_4 = _e123;
            local_5 = _e123;
            local_6 = _e135;
            local_7 = _e123;
            if (_e145 != 0u) {
                phi_2754_ = _e91.z;
                phi_2698_ = _e91.y;
                phi_2670_ = i32(_e91.x);
            }
            let _e152 = phi_2754_;
            let _e154 = phi_2698_;
            let _e156 = phi_2670_;
            phi_2681_ = _e105;
            phi_2679_ = _e110;
            phi_2678_ = _e110.w;
            if (_e156 != _e103) {
                let _e159 = ((_e105 + _e156) - _e103);
                let _e164 = textureLoad(LC, vec2<i32>((_e159 & 2047i), (_e159 >> bitcast<u32>(11i))), 0i);
                if ((_e164.w & 8454143u) != (_e110.w & 8454143u)) {
                    let _e169 = (_e142 == 0f);
                    phi_1665_ = _e169;
                    if !(_e169) {
                        phi_1665_ = (_e119.x != 0f);
                    }
                    let _e174 = phi_1665_;
                    phi_2683_ = _e105;
                    phi_2675_ = _e110;
                    if _e174 {
                        let _e175 = bitcast<i32>(_e117.w);
                        let _e180 = textureLoad(LC, vec2<i32>((_e175 & 2047i), (_e175 >> bitcast<u32>(11i))), 0i);
                        phi_2683_ = _e175;
                        phi_2675_ = _e180;
                    }
                    let _e182 = phi_2683_;
                    let _e184 = phi_2675_;
                    phi_2682_ = _e182;
                    phi_2674_ = _e184;
                } else {
                    phi_2682_ = _e159;
                    phi_2674_ = _e164;
                }
                let _e186 = phi_2682_;
                let _e188 = phi_2674_;
                phi_2681_ = _e186;
                phi_2679_ = _e188;
                phi_2678_ = ((_e188.w & 4286578687u) | _e145);
            }
            let _e193 = phi_2681_;
            let _e195 = phi_2679_;
            let _e197 = phi_2678_;
            let _e198 = (_e197 & 469762048u);
            let _e201 = ((_e198 == 67108864u) && (_e101 == 0i));
            if _e201 {
                let _e206 = f32((_e195.z & 65535u));
                let _e209 = f32((_e195.z >> bitcast<u32>(16i)));
                let _e215 = vec2<i32>(i32((-1f - _e206)), i32(((_e209 - _e206) + 1f)));
                phi_2685_ = _e215;
                if ((_e197 & 8388608u) != 0u) {
                    phi_2685_ = -(_e215);
                }
                let _e220 = phi_2685_;
                let _e222 = (_e193 + _e220.x);
                let _e227 = textureLoad(LC, vec2<i32>((_e222 & 2047i), (_e222 >> bitcast<u32>(11i))), 0i);
                let _e229 = (_e193 + _e220.y);
                let _e234 = textureLoad(LC, vec2<i32>((_e229 & 2047i), (_e229 >> bitcast<u32>(11i))), 0i);
                phi_2686_ = _e234;
                if ((_e234.w & 8454143u) != (_e227.w & 8454143u)) {
                    let _e240 = bitcast<i32>(_e117.w);
                    let _e245 = textureLoad(LC, vec2<i32>((_e240 & 2047i), (_e240 >> bitcast<u32>(11i))), 0i);
                    phi_2686_ = _e245;
                }
                let _e247 = phi_2686_;
                let _e249 = bitcast<f32>(_e227.z);
                let _e251 = bitcast<f32>(_e247.z);
                let _e252 = (_e251 - _e249);
                phi_2690_ = _e252;
                if (abs(_e252) > 3.1415927f) {
                    phi_2690_ = (_e252 - (6.2831855f * sign(_e252)));
                }
                let _e259 = phi_2690_;
                let _e260 = (_e209 + -2f);
                let _e266 = clamp(round(((abs(_e259) * 0.31830987f) * _e260)), 1f, (_e209 + -3f));
                let _e267 = (_e260 - _e266);
                if (_e206 <= _e267) {
                    phi_2771_ = _e154;
                    if (_e206 == _e267) {
                        phi_2771_ = -(_e154);
                    }
                    let _e284 = phi_2771_;
                    phi_2769_ = _e284;
                    phi_2712_ = -(((3.1415927f * sign(_e259)) - _e259));
                    phi_2705_ = _e267;
                    phi_2702_ = _e206;
                } else {
                    let _e270 = (_e206 == (_e267 + 1f));
                    if _e270 {
                        phi_2703_ = 0f;
                    } else {
                        phi_2703_ = (_e206 - (_e267 + 2f));
                    }
                    let _e274 = phi_2703_;
                    phi_2769_ = select(_e154, 0f, _e270);
                    phi_2712_ = _e259;
                    phi_2705_ = select(_e266, 0f, _e270);
                    phi_2702_ = _e274;
                }
                let _e286 = phi_2769_;
                let _e288 = phi_2712_;
                let _e290 = phi_2705_;
                let _e292 = phi_2702_;
                if (_e292 == _e290) {
                    phi_2716_ = _e251;
                } else {
                    phi_2716_ = (_e249 + (_e288 * (_e292 / _e290)));
                }
                let _e298 = phi_2716_;
                phi_2791_ = _e249;
                phi_2782_ = _e288;
                phi_2767_ = _e286;
                phi_2715_ = _e298;
            } else {
                phi_2791_ = f32();
                phi_2782_ = f32();
                phi_2767_ = _e154;
                phi_2715_ = bitcast<f32>(_e195.z);
            }
            let _e300 = phi_2791_;
            let _e302 = phi_2782_;
            let _e304 = phi_2767_;
            let _e306 = phi_2715_;
            let _e310 = vec2<f32>(sin(_e306), -(cos(_e306)));
            let _e312 = bitcast<vec2<f32>>(_e195.xy);
            phi_2765_ = _e144;
            if (_e144 != 0f) {
                phi_2765_ = max(_e144, (1f / length((_e134 * _e310))));
            }
            let _e319 = phi_2765_;
            if (_e142 != 0f) {
                let _e427 = (_e304 * sign(determinant(_e134)));
                let _e429 = ((_e197 & 1048576u) != 0u);
                phi_2848_ = _e427;
                if _e429 {
                    phi_2848_ = min(_e427, 0f);
                }
                let _e432 = phi_2848_;
                phi_2859_ = _e432;
                if ((_e197 & 524288u) != 0u) {
                    phi_2859_ = max(_e432, 0f);
                }
                let _e437 = phi_2859_;
                let _e438 = (_e319 != 0f);
                if _e438 {
                    phi_2851_ = _e319;
                } else {
                    let _e439 = (_e134 * _e310);
                    phi_2851_ = (((abs(_e439.x) + abs(_e439.y)) * (1f / dot(_e439, _e439))) * 0.5f);
                }
                let _e450 = phi_2851_;
                let _e453 = ((_e450 > _e142) && (_e319 == 0f));
                phi_2940_ = 1f;
                if _e453 {
                    phi_2940_ = (_e142 / _e450);
                }
                let _e456 = phi_2940_;
                let _e457 = select(_e142, _e450, _e453);
                let _e458 = (_e457 + _e450);
                let _e459 = (_e310 * _e458);
                let _e460 = (_e437 * _e458);
                let _e467 = (((vec2<f32>(_e460, -(_e460)) + vec2(_e457)) * (0.5f / _e450)) + vec2<f32>(0.5f, 0.5f));
                let _e470 = vec4<f32>(_e467.x, _e467.y, 0f, 0f);
                phi_2965_ = _e459;
                phi_2949_ = _e470;
                if (_e198 > 134217728u) {
                    let _e472 = (_e197 & 4194304u);
                    let _e474 = select(2i, -2i, (_e472 == 0u));
                    phi_2897_ = _e474;
                    if ((_e197 & 8388608u) != 0u) {
                        phi_2897_ = -(_e474);
                    }
                    let _e479 = phi_2897_;
                    let _e480 = (_e193 + _e479);
                    let _e485 = textureLoad(LC, vec2<i32>((_e480 & 2047i), (_e480 >> bitcast<u32>(11i))), 0i);
                    let _e489 = abs((bitcast<f32>(_e485.z) - _e306));
                    phi_2906_ = _e489;
                    if (_e489 > 3.1415927f) {
                        phi_2906_ = (6.2831855f - _e489);
                    }
                    let _e493 = phi_2906_;
                    let _e498 = ((_e493 * select(0.5f, -0.5f, ((_e472 != 0u) == _e429))) + _e306);
                    let _e502 = vec2<f32>(sin(_e498), -(cos(_e498)));
                    let _e503 = (_e134 * _e502);
                    let _e511 = ((abs(_e503.x) + abs(_e503.y)) * (1f / dot(_e503, _e503)));
                    let _e513 = cos((_e493 * 0.5f));
                    let _e514 = (_e198 == 335544320u);
                    phi_2097_ = _e514;
                    if !(_e514) {
                        phi_2097_ = ((_e198 == 268435456u) && (_e513 >= 0.25f));
                    }
                    let _e520 = phi_2097_;
                    if _e520 {
                        phi_2913_ = (_e457 * (1f / max(_e513, select(0.25f, 1f, ((_e197 & 33554432u) != 0u)))));
                    } else {
                        phi_2913_ = ((_e457 * _e513) + (_e511 * 0.5f));
                    }
                    let _e531 = phi_2913_;
                    let _e533 = (_e531 + (_e511 * 0.5f));
                    phi_2928_ = _e459;
                    if ((_e197 & 2097152u) != 0u) {
                        if (_e458 <= ((_e533 * _e513) + (_e450 * 0.125f))) {
                            phi_2929_ = (_e502 * (_e458 * (1f / _e513)));
                        } else {
                            let _e540 = (_e502 * _e533);
                            phi_2929_ = (vec2<f32>(dot(_e459, _e459), dot(_e540, _e540)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e459, _e540)));
                        }
                        let _e551 = phi_2929_;
                        phi_2928_ = _e551;
                    }
                    let _e553 = phi_2928_;
                    let _e558 = ((_e533 - dot((_e553 * abs(_e437)), _e502)) / _e511);
                    if _e429 {
                        phi_2950_ = vec4<f32>(_e470.x, _e558, _e470.z, _e470.w);
                    } else {
                        phi_2950_ = vec4<f32>(_e558, _e470.y, _e470.z, _e470.w);
                    }
                    let _e570 = phi_2950_;
                    phi_2965_ = _e553;
                    phi_2949_ = _e570;
                }
                let _e572 = phi_2965_;
                let _e574 = phi_2949_;
                let _e576 = (_e574.xy * _e456);
                let _e582 = vec4<f32>(_e576.x, _e574.y, _e574.z, _e574.w);
                let _e589 = vec4<f32>(_e582.x, max(_e576.y, 0.0001f), _e582.z, _e582.w);
                phi_2998_ = _e589;
                if _e438 {
                    phi_2998_ = vec4<f32>((-2f - _e576.x), _e589.y, _e589.z, _e589.w);
                }
                let _e597 = phi_2998_;
                if (_e101 != 0i) {
                    phi_3038_ = _e597;
                    phi_3000_ = vec2<f32>();
                    phi_2999_ = false;
                    break;
                }
                phi_2995_ = _e597;
                phi_2991_ = (_e134 * (_e572 * _e437));
                phi_2967_ = _e312;
            } else {
                let _e321 = vec4<f32>(_e152, -1f, 0f, 0f);
                if (_e319 != 0f) {
                    let _e332 = vec4<f32>(_e321.x, -2f, _e321.z, _e321.w);
                    let _e337 = vec4<f32>(_e332.x, _e332.y, 1000000f, _e332.w);
                    phi_2844_ = vec4<f32>(_e337.x, _e337.y, _e337.z, _e152);
                    if _e201 {
                        phi_2801_ = _e302;
                        phi_2800_ = _e300;
                        if (_e302 < 0f) {
                            phi_2801_ = -(_e302);
                            phi_2800_ = (_e300 + _e302);
                        }
                        let _e347 = phi_2801_;
                        let _e349 = phi_2800_;
                        let _e351 = ((_e306 - _e349) + 1.5707964f);
                        let _e357 = clamp(((_e351 - (floor((_e351 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e347);
                        phi_2802_ = _e357;
                        if (_e357 > (_e347 * 0.5f)) {
                            phi_2802_ = (_e347 - _e357);
                        }
                        let _e362 = phi_2802_;
                        let _e369 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e362), cos(_e362)) * abs(_e304))) * 0.5f);
                        if (abs((_e347 - 1.5707964f)) < 0.001f) {
                            phi_2828_ = 0f;
                            phi_2826_ = 0f;
                        } else {
                            let _e373 = tan(_e347);
                            let _e378 = (sign((1.5707964f - _e347)) / max(abs(_e373), 0.000001f));
                            if (_e378 >= 0f) {
                                phi_2806_ = (_e369.y - ((1f - _e369.x) * _e373));
                            } else {
                                phi_2806_ = (_e369.y + (_e369.x * _e373));
                            }
                            let _e390 = phi_2806_;
                            phi_2828_ = _e390;
                            phi_2826_ = _e378;
                        }
                        let _e392 = phi_2828_;
                        let _e394 = phi_2826_;
                        phi_2844_ = vec4<f32>((max(_e369.x, 0f) + 0.25f), (-2f - _e369.y), _e394, _e392);
                    }
                    let _e402 = phi_2844_;
                    phi_2994_ = (_e134 * (_e310 * (_e304 * _e319)));
                    phi_2843_ = _e402;
                } else {
                    phi_2994_ = (sign(((_e310 * _e304) * _naga_inverse_2x2_f32(_e134))) * 0.5f);
                    phi_2843_ = _e321;
                }
                let _e407 = phi_2994_;
                let _e409 = phi_2843_;
                phi_2997_ = _e409;
                if (((_e197 & 8388608u) != 0u) != ((_e197 & 16777216u) != 0u)) {
                    phi_2997_ = (_e409 * vec4<f32>(-1f, 1f, 1f, 1f));
                }
                let _e417 = phi_2997_;
                if (((_e197 & 2147483648u) != 0u) && (_e101 != 1i)) {
                    phi_3038_ = _e417;
                    phi_3000_ = vec2<f32>();
                    phi_2999_ = false;
                    break;
                }
                phi_2995_ = _e417;
                phi_2991_ = _e407;
                phi_2967_ = select(_e312, _e119, vec2((_e101 == 2i)));
            }
            let _e602 = phi_2995_;
            let _e604 = phi_2991_;
            let _e606 = phi_2967_;
            let _e612 = m.ug;
            let _e615 = select(_e602.xy, vec2<f32>(1f, -1f), vec2((_e612 != 0u)));
            let _e621 = vec4<f32>(_e615.x, _e602.y, _e602.z, _e602.w);
            phi_3038_ = vec4<f32>(_e621.x, _e615.y, _e621.z, _e621.w);
            phi_3000_ = (((_e134 * _e606) + _e604) + bitcast<vec2<f32>>(_e138.xy));
            phi_2999_ = true;
            break;
        }
    }
    let _e629 = phi_3038_;
    let _e631 = phi_3000_;
    let _e633 = phi_2999_;
    O = _e629;
    let _e636 = local;
    let _e638 = AD.c2_[_e636];
    let _e640 = m.Z5_;
    let _e642 = local_1;
    if (_e642 == 0u) {
        phi_3039_ = 0f;
    } else {
        let _e645 = local_2;
        phi_3039_ = unpack2x16float(((_e645 + 1023u) * _e640)).x;
    }
    let _e651 = phi_3039_;
    A0_ = _e651;
    if ((_e638.x & 512u) != 0u) {
        let _e655 = A0_;
        A0_ = -(_e655);
    }
    let _e657 = (_e638.x & 15u);
    if Ug {
        let _e658 = (_e657 == 0u);
        if _e658 {
            phi_3040_ = _e638.y;
        } else {
            phi_3040_ = _e638.x;
        }
        let _e661 = phi_3040_;
        let _e663 = (_e661 >> bitcast<u32>(16i));
        if (_e663 == 0u) {
            phi_3041_ = 0f;
        } else {
            phi_3041_ = unpack2x16float(((_e663 + 1023u) * _e640)).x;
        }
        let _e670 = phi_3041_;
        phi_3042_ = _e670;
        if _e658 {
            phi_3042_ = -(_e670);
        }
        let _e673 = phi_3042_;
        U1_[0u] = _e673;
    }
    if Wg {
        e2_ = f32(((_e638.x >> bitcast<u32>(4i)) & 15u));
    }
    if Vg {
        let _e680 = local_3;
        let _e684 = RB.c2_[(_e680 + 2u)];
        let _e689 = vec2<f32>(_e684.x, _e684.y);
        let _e690 = vec2<f32>(_e684.z, _e684.w);
        let _e693 = local_4;
        let _e697 = RB.c2_[(_e693 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e702 = (abs(_e689) + abs(_e690));
                let _e704 = (_e702.x != 0f);
                phi_2497_ = _e704;
                if _e704 {
                    phi_2497_ = (_e702.y != 0f);
                }
                let _e708 = phi_2497_;
                if _e708 {
                    let _e712 = ((mat2x2<f32>(_e689, _e690) * _e631) + _e697.xy);
                    let _e713 = -(_e712);
                    let _e719 = (vec2<f32>(1f, 1f) / _e702).xyxy;
                    phi_3043_ = (((vec4<f32>(_e712.x, _e712.y, _e713.x, _e713.y) * _e719) + _e719) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3043_ = _e697.xyxy;
                    break;
                }
            }
        }
        let _e724 = phi_3043_;
        L0_ = _e724;
    }
    if (_e657 == 1u) {
        f1_ = unpack4x8unorm(_e638.y);
    } else {
        if (Ug && (_e657 == 0u)) {
            let _e778 = (_e638.x >> bitcast<u32>(16i));
            if (_e778 == 0u) {
                phi_3044_ = 0f;
            } else {
                phi_3044_ = unpack2x16float(((_e778 + 1023u) * _e640)).x;
            }
            let _e785 = phi_3044_;
            U1_[1u] = _e785;
        } else {
            let _e730 = local_5;
            let _e732 = RB.c2_[_e730];
            let _e742 = local_6;
            let _e744 = RB.c2_[_e742];
            let _e747 = ((mat2x2<f32>(vec2<f32>(_e732.x, _e732.y), vec2<f32>(_e732.z, _e732.w)) * _e631) + _e744.xy);
            let _e748 = (_e657 == 2u);
            if (_e748 || (_e657 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e638.y));
                if (_e744.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e744.w;
                }
                if _e748 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e747.x;
                } else {
                    let _e768 = f1_[2u];
                    f1_[2u] = -(_e768);
                    f1_[0u] = _e747.x;
                    f1_[1u] = _e747.y;
                }
            } else {
                f1_ = vec4<f32>(_e747.x, _e747.y, bitcast<f32>(_e638.y), (-2f - _e744.z));
            }
        }
    }
    if _e633 {
        let _e793 = m.bf;
        let _e795 = m.cf;
        let _e805 = local_7;
        let _e809 = PB.c2_[(_e805 + 3u)];
        a3_ = _e809.xy;
        l4_ = (_e631 + bitcast<vec2<f32>>(_e809.zw));
        phi_3061_ = vec4<f32>(((_e631.x * _e793) - 1f), ((_e631.y * _e795) - sign(_e795)), 0f, 1f);
    } else {
        let _e790 = m.N2_;
        phi_3061_ = vec4(_e790);
    }
    let _e815 = phi_3061_;
    unnamed.gl_Position = _e815;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e20 = O;
    let _e21 = A0_;
    let _e22 = U1_;
    let _e23 = e2_;
    let _e24 = L0_;
    let _e25 = f1_;
    let _e26 = a3_;
    let _e27 = l4_;
    let _e28 = unnamed.gl_Position;
    return VertexOutput(_e20, _e21, _e22, _e23, _e24, _e25, _e26, _e27, _e28);
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
