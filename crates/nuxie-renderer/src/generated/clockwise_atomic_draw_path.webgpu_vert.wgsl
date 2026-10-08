struct hi {
    v2_: array<vec4<u32>>,
}

struct gi {
    v2_: array<vec4<u32>>,
}

struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct jg {
    v2_: array<vec2<u32>>,
}

struct kg {
    v2_: array<vec4<f32>>,
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

@id(0) override Yi: bool = true;
@id(2) override aj: bool = true;
@id(1) override Zi: bool = true;
@id(8) override gj: bool = true;

@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> CD: hi;
@group(0) @binding(2)
var<storage> KB: gi;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> XB_1: vec4<f32>;
var<private> YB_1: vec4<f32>;
var<private> S: vec4<f32>;
@group(0) @binding(3)
var<storage> WC: jg;
var<private> G0_: f32;
var<private> j2_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> W0_: vec4<f32>;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
var<private> y3_: vec2<u32>;
var<private> J4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Va: sampler;

fn main_1() {
    var phi_2871_: f32;
    var phi_2809_: f32;
    var phi_2781_: i32;
    var phi_1709_: bool;
    var phi_2794_: i32;
    var phi_2786_: vec4<u32>;
    var phi_2793_: i32;
    var phi_2785_: vec4<u32>;
    var phi_2792_: i32;
    var phi_2790_: vec4<u32>;
    var phi_2789_: u32;
    var phi_2796_: vec2<i32>;
    var phi_2797_: vec4<u32>;
    var phi_2801_: f32;
    var phi_2815_: f32;
    var phi_2881_: f32;
    var phi_2880_: f32;
    var phi_2823_: f32;
    var phi_2816_: f32;
    var phi_2813_: f32;
    var phi_2827_: f32;
    var phi_2902_: f32;
    var phi_2893_: f32;
    var phi_2878_: f32;
    var phi_2826_: f32;
    var phi_2876_: f32;
    var phi_2912_: f32;
    var phi_2911_: f32;
    var phi_2913_: f32;
    var phi_2917_: f32;
    var phi_2939_: f32;
    var phi_2937_: f32;
    var phi_2955_: vec4<f32>;
    var phi_3075_: vec2<f32>;
    var phi_2954_: vec4<f32>;
    var phi_3081_: vec4<f32>;
    var phi_2959_: f32;
    var phi_2970_: f32;
    var phi_2962_: f32;
    var phi_3023_: f32;
    var phi_2997_: f32;
    var phi_2024_: bool;
    var phi_3002_: f32;
    var phi_3014_: vec2<f32>;
    var phi_3013_: vec2<f32>;
    var phi_3031_: vec4<f32>;
    var phi_3044_: vec2<f32>;
    var phi_3030_: vec4<f32>;
    var phi_3078_: vec4<f32>;
    var phi_3082_: bool;
    var phi_3076_: vec4<f32>;
    var phi_3070_: vec2<f32>;
    var phi_3046_: vec2<f32>;
    var phi_3114_: f32;
    var phi_3115_: u32;
    var phi_3116_: f32;
    var phi_3117_: f32;
    var phi_2534_: bool;
    var phi_3118_: vec4<f32>;
    var phi_3166_: vec4<f32>;
    var phi_3167_: vec4<f32>;
    var phi_3168_: f32;
    var phi_1385_: bool;
    var phi_3169_: f32;
    var phi_3188_: vec4<f32>;

    let _e99 = gl_InstanceIndex_1;
    let _e100 = XB_1;
    let _e101 = YB_1;
    let _e103 = i32(_e100.x);
    let _e107 = bitcast<i32>(_e100.w);
    let _e109 = (_e107 >> bitcast<u32>(2i));
    let _e110 = (_e107 & 3i);
    let _e112 = min(_e103, (_e109 - 1i));
    let _e114 = ((_e99 * _e109) + _e112);
    let _e119 = textureLoad(UB, vec2<i32>((_e114 & 2047i), (_e114 >> bitcast<u32>(11i))), 0i);
    let _e126 = CD.v2_[(max((_e119.w & 65535u), 1u) - 1u)];
    let _e128 = bitcast<vec2<f32>>(_e126.xy);
    let _e130 = (_e126.z & 65535u);
    let _e132 = (_e130 * 4u);
    let _e135 = KB.v2_[_e132];
    let _e136 = bitcast<vec4<f32>>(_e135);
    let _e143 = mat2x2<f32>(vec2<f32>(_e136.x, _e136.y), vec2<f32>(_e136.z, _e136.w));
    let _e147 = KB.v2_[(_e132 + 1u)];
    let _e151 = bitcast<f32>(_e147.z);
    let _e153 = bitcast<f32>(_e147.w);
    let _e154 = (_e119.w & 8388608u);
    phi_2871_ = _e100.z;
    phi_2809_ = _e100.y;
    phi_2781_ = _e103;
    if (_e154 != 0u) {
        phi_2871_ = _e101.z;
        phi_2809_ = _e101.y;
        phi_2781_ = i32(_e101.x);
    }
    let _e161 = phi_2871_;
    let _e163 = phi_2809_;
    let _e165 = phi_2781_;
    phi_2792_ = _e114;
    phi_2790_ = _e119;
    phi_2789_ = _e119.w;
    if (_e165 != _e112) {
        let _e168 = ((_e114 + _e165) - _e112);
        let _e173 = textureLoad(UB, vec2<i32>((_e168 & 2047i), (_e168 >> bitcast<u32>(11i))), 0i);
        if ((_e173.w & 8454143u) != (_e119.w & 8454143u)) {
            let _e178 = (_e151 == 0f);
            phi_1709_ = _e178;
            if !(_e178) {
                phi_1709_ = (_e128.x != 0f);
            }
            let _e183 = phi_1709_;
            phi_2794_ = _e114;
            phi_2786_ = _e119;
            if _e183 {
                let _e184 = bitcast<i32>(_e126.w);
                let _e189 = textureLoad(UB, vec2<i32>((_e184 & 2047i), (_e184 >> bitcast<u32>(11i))), 0i);
                phi_2794_ = _e184;
                phi_2786_ = _e189;
            }
            let _e191 = phi_2794_;
            let _e193 = phi_2786_;
            phi_2793_ = _e191;
            phi_2785_ = _e193;
        } else {
            phi_2793_ = _e168;
            phi_2785_ = _e173;
        }
        let _e195 = phi_2793_;
        let _e197 = phi_2785_;
        phi_2792_ = _e195;
        phi_2790_ = _e197;
        phi_2789_ = ((_e197.w & 4286578687u) | _e154);
    }
    let _e202 = phi_2792_;
    let _e204 = phi_2790_;
    let _e206 = phi_2789_;
    let _e207 = (_e206 & 469762048u);
    let _e210 = ((_e207 == 67108864u) && (_e110 == 0i));
    if _e210 {
        let _e216 = f32((_e204.z & 65535u));
        let _e219 = f32((_e204.z >> bitcast<u32>(16i)));
        let _e225 = vec2<i32>(i32((-1f - _e216)), i32(((_e219 - _e216) + 1f)));
        phi_2796_ = _e225;
        if ((_e206 & 8388608u) != 0u) {
            phi_2796_ = -(_e225);
        }
        let _e230 = phi_2796_;
        let _e232 = (_e202 + _e230.x);
        let _e237 = textureLoad(UB, vec2<i32>((_e232 & 2047i), (_e232 >> bitcast<u32>(11i))), 0i);
        let _e239 = (_e202 + _e230.y);
        let _e244 = textureLoad(UB, vec2<i32>((_e239 & 2047i), (_e239 >> bitcast<u32>(11i))), 0i);
        phi_2797_ = _e244;
        if ((_e244.w & 8454143u) != (_e237.w & 8454143u)) {
            let _e250 = bitcast<i32>(_e126.w);
            let _e255 = textureLoad(UB, vec2<i32>((_e250 & 2047i), (_e250 >> bitcast<u32>(11i))), 0i);
            phi_2797_ = _e255;
        }
        let _e257 = phi_2797_;
        let _e260 = (f32(_e237.z) * 0.0000000014629181f);
        let _e263 = (f32(_e257.z) * 0.0000000014629181f);
        let _e264 = (_e263 - _e260);
        phi_2801_ = _e264;
        if (abs(_e264) > 3.1415927f) {
            phi_2801_ = (_e264 - (6.2831855f * sign(_e264)));
        }
        let _e271 = phi_2801_;
        let _e272 = (_e219 + -2f);
        let _e278 = clamp(round(((abs(_e271) * 0.31830987f) * _e272)), 1f, (_e219 + -3f));
        let _e279 = (_e272 - _e278);
        if (_e216 <= _e279) {
            phi_2881_ = _e163;
            if (_e216 == _e279) {
                phi_2881_ = -(_e163);
            }
            let _e296 = phi_2881_;
            phi_2880_ = _e296;
            phi_2823_ = -(((3.1415927f * sign(_e271)) - _e271));
            phi_2816_ = _e279;
            phi_2813_ = _e216;
        } else {
            let _e282 = (_e216 == (_e279 + 1f));
            if _e282 {
                phi_2815_ = 0f;
            } else {
                phi_2815_ = (_e216 - (_e279 + 2f));
            }
            let _e286 = phi_2815_;
            phi_2880_ = select(_e163, 0f, _e282);
            phi_2823_ = _e271;
            phi_2816_ = select(_e278, 0f, _e282);
            phi_2813_ = _e286;
        }
        let _e298 = phi_2880_;
        let _e300 = phi_2823_;
        let _e302 = phi_2816_;
        let _e304 = phi_2813_;
        if (_e304 == _e302) {
            phi_2827_ = _e263;
        } else {
            phi_2827_ = (_e260 + (_e300 * (_e304 / _e302)));
        }
        let _e310 = phi_2827_;
        phi_2902_ = _e260;
        phi_2893_ = _e300;
        phi_2878_ = _e298;
        phi_2826_ = _e310;
    } else {
        phi_2902_ = f32();
        phi_2893_ = f32();
        phi_2878_ = _e163;
        phi_2826_ = (f32(_e204.z) * 0.0000000014629181f);
    }
    let _e312 = phi_2902_;
    let _e314 = phi_2893_;
    let _e316 = phi_2878_;
    let _e318 = phi_2826_;
    let _e322 = vec2<f32>(sin(_e318), -(cos(_e318)));
    let _e324 = bitcast<vec2<f32>>(_e204.xy);
    phi_2876_ = _e153;
    if (_e153 != 0f) {
        phi_2876_ = max(_e153, (1f / length((_e143 * _e322))));
    }
    let _e331 = phi_2876_;
    if (_e151 != 0f) {
        let _e439 = (_e316 * sign(determinant(_e143)));
        let _e441 = ((_e206 & 1048576u) != 0u);
        phi_2959_ = _e439;
        if _e441 {
            phi_2959_ = min(_e439, 0f);
        }
        let _e444 = phi_2959_;
        phi_2970_ = _e444;
        if ((_e206 & 524288u) != 0u) {
            phi_2970_ = max(_e444, 0f);
        }
        let _e449 = phi_2970_;
        let _e450 = (_e331 != 0f);
        if _e450 {
            phi_2962_ = _e331;
        } else {
            let _e451 = (_e143 * _e322);
            phi_2962_ = (((abs(_e451.x) + abs(_e451.y)) * (1f / dot(_e451, _e451))) * 0.5f);
        }
        let _e462 = phi_2962_;
        let _e465 = ((_e462 > _e151) && (_e331 == 0f));
        phi_3023_ = 1f;
        if _e465 {
            phi_3023_ = (_e151 / _e462);
        }
        let _e468 = phi_3023_;
        let _e469 = select(_e151, _e462, _e465);
        let _e470 = (_e469 + _e462);
        let _e471 = (_e322 * _e470);
        let _e472 = (_e449 * _e470);
        let _e479 = (((vec2<f32>(_e472, -(_e472)) + vec2(_e469)) * (0.5f / _e462)) + vec2<f32>(0.5f, 0.5f));
        let _e482 = vec4<f32>(_e479.x, _e479.y, 0f, 0f);
        phi_3044_ = _e471;
        phi_3030_ = _e482;
        if (_e207 > 134217728u) {
            let _e488 = f32((_e204.z & 65535u));
            let _e489 = (_e488 * 0.000015259022f);
            let _e493 = sqrt(max((1f - (_e489 * _e489)), 0f));
            phi_2997_ = _e493;
            if (((_e206 & 4194304u) != 0u) == _e441) {
                phi_2997_ = -(_e493);
            }
            let _e497 = phi_2997_;
            let _e502 = (mat2x2<f32>(vec2<f32>(_e489, _e497), vec2<f32>(-(_e497), _e489)) * _e322);
            let _e503 = (_e143 * _e502);
            let _e511 = ((abs(_e503.x) + abs(_e503.y)) * (1f / dot(_e503, _e503)));
            let _e512 = (_e207 == 335544320u);
            phi_2024_ = _e512;
            if !(_e512) {
                phi_2024_ = ((_e207 == 268435456u) && (_e489 >= 0.25f));
            }
            let _e518 = phi_2024_;
            if _e518 {
                phi_3002_ = (_e469 * (1f / max(_e489, select(0.25f, 1f, ((_e206 & 33554432u) != 0u)))));
            } else {
                phi_3002_ = ((_e469 * _e489) + (_e511 * 0.5f));
            }
            let _e529 = phi_3002_;
            let _e531 = (_e529 + (_e511 * 0.5f));
            phi_3013_ = _e471;
            if ((_e206 & 2097152u) != 0u) {
                if (_e470 <= ((_e531 * _e489) + (_e462 * 0.125f))) {
                    phi_3014_ = (_e502 * (_e470 * (65535f / _e488)));
                } else {
                    let _e538 = (_e502 * _e531);
                    phi_3014_ = (vec2<f32>(dot(_e471, _e471), dot(_e538, _e538)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e471, _e538)));
                }
                let _e549 = phi_3014_;
                phi_3013_ = _e549;
            }
            let _e551 = phi_3013_;
            let _e556 = ((_e531 - dot((_e551 * abs(_e449)), _e502)) / _e511);
            if _e441 {
                phi_3031_ = vec4<f32>(_e482.x, _e556, _e482.z, _e482.w);
            } else {
                phi_3031_ = vec4<f32>(_e556, _e482.y, _e482.z, _e482.w);
            }
            let _e568 = phi_3031_;
            phi_3044_ = _e551;
            phi_3030_ = _e568;
        }
        let _e570 = phi_3044_;
        let _e572 = phi_3030_;
        let _e574 = (_e572.xy * _e468);
        let _e580 = vec4<f32>(_e574.x, _e572.y, _e572.z, _e572.w);
        let _e587 = vec4<f32>(_e580.x, max(_e574.y, 0.0001f), _e580.z, _e580.w);
        phi_3078_ = _e587;
        if _e450 {
            phi_3078_ = vec4<f32>((-2f - _e574.x), _e587.y, _e587.z, _e587.w);
        }
        let _e595 = phi_3078_;
        phi_3082_ = (_e110 != 0i);
        phi_3076_ = _e595;
        phi_3070_ = (_e143 * (_e570 * _e449));
        phi_3046_ = _e324;
    } else {
        let _e333 = vec4<f32>(_e161, -1f, 0f, 0f);
        if (_e331 != 0f) {
            let _e344 = vec4<f32>(_e333.x, -2f, _e333.z, _e333.w);
            let _e349 = vec4<f32>(_e344.x, _e344.y, 1000000f, _e344.w);
            phi_2955_ = vec4<f32>(_e349.x, _e349.y, _e349.z, _e161);
            if _e210 {
                phi_2912_ = _e314;
                phi_2911_ = _e312;
                if (_e314 < 0f) {
                    phi_2912_ = -(_e314);
                    phi_2911_ = (_e312 + _e314);
                }
                let _e359 = phi_2912_;
                let _e361 = phi_2911_;
                let _e363 = ((_e318 - _e361) + 1.5707964f);
                let _e369 = clamp(((_e363 - (floor((_e363 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e359);
                phi_2913_ = _e369;
                if (_e369 > (_e359 * 0.5f)) {
                    phi_2913_ = (_e359 - _e369);
                }
                let _e374 = phi_2913_;
                let _e381 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e374), cos(_e374)) * abs(_e316))) * 0.5f);
                if (abs((_e359 - 1.5707964f)) < 0.001f) {
                    phi_2939_ = 0f;
                    phi_2937_ = 0f;
                } else {
                    let _e385 = tan(_e359);
                    let _e390 = (sign((1.5707964f - _e359)) / max(abs(_e385), 0.000001f));
                    if (_e390 >= 0f) {
                        phi_2917_ = (_e381.y - ((1f - _e381.x) * _e385));
                    } else {
                        phi_2917_ = (_e381.y + (_e381.x * _e385));
                    }
                    let _e402 = phi_2917_;
                    phi_2939_ = _e402;
                    phi_2937_ = _e390;
                }
                let _e404 = phi_2939_;
                let _e406 = phi_2937_;
                phi_2955_ = vec4<f32>((max(_e381.x, 0f) + 0.25f), (-2f - _e381.y), _e406, _e404);
            }
            let _e414 = phi_2955_;
            phi_3075_ = (_e143 * (_e322 * (_e316 * _e331)));
            phi_2954_ = _e414;
        } else {
            phi_3075_ = (sign(((_e322 * _e316) * _naga_inverse_2x2_f32(_e143))) * 0.5f);
            phi_2954_ = _e333;
        }
        let _e419 = phi_3075_;
        let _e421 = phi_2954_;
        phi_3081_ = _e421;
        if (((_e206 & 8388608u) != 0u) != ((_e206 & 16777216u) != 0u)) {
            phi_3081_ = (_e421 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e429 = phi_3081_;
        phi_3082_ = (((_e206 & 2147483648u) != 0u) && (_e110 != 1i));
        phi_3076_ = _e429;
        phi_3070_ = _e419;
        phi_3046_ = select(_e324, _e128, vec2((_e110 == 2i)));
    }
    let _e600 = phi_3082_;
    let _e602 = phi_3076_;
    let _e604 = phi_3070_;
    let _e606 = phi_3046_;
    let _e609 = (((_e143 * _e606) + _e604) + bitcast<vec2<f32>>(_e147.xy));
    let _e612 = j.yi;
    let _e615 = select(_e602.xy, vec2<f32>(1f, -1f), vec2((_e612 != 0u)));
    let _e621 = vec4<f32>(_e615.x, _e602.y, _e602.z, _e602.w);
    S = vec4<f32>(_e621.x, _e615.y, _e621.z, _e621.w);
    let _e630 = WC.v2_[_e130];
    let _e632 = j.p6_;
    if (_e130 == 0u) {
        phi_3114_ = 0f;
    } else {
        phi_3114_ = unpack2x16float(((_e130 + 1023u) * _e632)).x;
    }
    let _e639 = phi_3114_;
    G0_ = _e639;
    if ((_e630.x & 512u) != 0u) {
        let _e643 = G0_;
        G0_ = -(_e643);
    }
    let _e645 = (_e630.x & 15u);
    if Yi {
        let _e646 = (_e645 == 0u);
        if _e646 {
            phi_3115_ = _e630.y;
        } else {
            phi_3115_ = _e630.x;
        }
        let _e649 = phi_3115_;
        let _e651 = (_e649 >> bitcast<u32>(16i));
        if (_e651 == 0u) {
            phi_3116_ = 0f;
        } else {
            phi_3116_ = unpack2x16float(((_e651 + 1023u) * _e632)).x;
        }
        let _e658 = phi_3116_;
        phi_3117_ = _e658;
        if _e646 {
            phi_3117_ = -(_e658);
        }
        let _e661 = phi_3117_;
        j2_[0u] = _e661;
    }
    if aj {
        Q0_ = f32(((_e630.x >> bitcast<u32>(4i)) & 15u));
    }
    if Zi {
        let _e667 = (_e130 * 8u);
        let _e671 = JB.v2_[(_e667 + 2u)];
        let _e676 = vec2<f32>(_e671.x, _e671.y);
        let _e677 = vec2<f32>(_e671.z, _e671.w);
        let _e682 = JB.v2_[(_e667 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e687 = (abs(_e676) + abs(_e677));
                let _e689 = (_e687.x != 0f);
                phi_2534_ = _e689;
                if _e689 {
                    phi_2534_ = (_e687.y != 0f);
                }
                let _e693 = phi_2534_;
                if _e693 {
                    let _e697 = ((mat2x2<f32>(_e676, _e677) * _e609) + _e682.xy);
                    let _e698 = -(_e697);
                    let _e704 = (vec2<f32>(1f, 1f) / _e687).xyxy;
                    phi_3118_ = (((vec4<f32>(_e697.x, _e697.y, _e698.x, _e698.y) * _e704) + _e704) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3118_ = _e682.xyxy;
                    break;
                }
            }
        }
        let _e709 = phi_3118_;
        W0_ = _e709;
    }
    if (_e645 == 1u) {
        P0_ = unpack4x8unorm(_e630.y);
    } else {
        if (Yi && (_e645 == 0u)) {
            let _e791 = (_e630.x >> bitcast<u32>(16i));
            if (_e791 == 0u) {
                phi_3168_ = 0f;
            } else {
                phi_3168_ = unpack2x16float(((_e791 + 1023u) * _e632)).x;
            }
            let _e798 = phi_3168_;
            j2_[1u] = _e798;
        } else {
            let _e713 = (_e130 * 8u);
            let _e716 = JB.v2_[_e713];
            let _e727 = JB.v2_[(_e713 + 1u)];
            let _e736 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e630.y));
            let _e738 = ((mat2x2<f32>(vec2<f32>(_e716.x, _e716.y), vec2<f32>(_e716.z, _e716.w)) * _e609) + _e727.xy);
            if (_e727.z > 0.9f) {
                phi_3166_ = vec4<f32>(_e736.x, _e736.y, 2f, _e736.w);
            } else {
                phi_3166_ = vec4<f32>(_e736.x, _e736.y, _e727.w, _e736.w);
            }
            let _e753 = phi_3166_;
            if (f32(_e645) == 2f) {
                let _e779 = vec4<f32>(_e738.x, _e753.y, _e753.z, _e753.w);
                phi_3167_ = vec4<f32>(_e779.x, 0f, _e779.z, _e779.w);
            } else {
                let _e761 = vec4<f32>(_e753.x, _e753.y, -(_e753.z), _e753.w);
                let _e767 = vec4<f32>(_e738.x, _e761.y, _e761.z, _e761.w);
                phi_3167_ = vec4<f32>(_e767.x, _e738.y, _e767.z, _e767.w);
            }
            let _e786 = phi_3167_;
            P0_ = _e786;
            let _e788 = P0_[3u];
            P0_[3u] = -(_e788);
        }
    }
    phi_1385_ = gj;
    if gj {
        phi_1385_ = ((_e630.x & 2048u) != 0u);
    }
    let _e805 = phi_1385_;
    if _e805 {
        let _e806 = (_e130 * 8u);
        let _e810 = JB.v2_[(_e806 + 4u)];
        let _e821 = JB.v2_[(_e806 + 5u)];
        let _e824 = ((mat2x2<f32>(vec2<f32>(_e810.x, _e810.y), vec2<f32>(_e810.z, _e810.w)) * _e609) + _e821.xy);
        phi_3169_ = (1f + _e821.z);
        if ((_e630.x & 4096u) != 0u) {
            phi_3169_ = (-1f - f32(((_e630.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e835 = phi_3169_;
        V0_ = vec3<f32>(_e824.x, _e824.y, _e835);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e600) {
        let _e844 = j.Gg;
        let _e846 = j.Hg;
        let _e858 = KB.v2_[(_e132 + 3u)];
        y3_ = _e858.xy;
        J4_ = (_e609 + bitcast<vec2<f32>>(_e858.zw));
        phi_3188_ = vec4<f32>(((_e609.x * _e844) - 1f), ((_e609.y * _e846) - sign(_e846)), 0f, 1f);
    } else {
        let _e841 = j.h3_;
        phi_3188_ = vec4(_e841);
    }
    let _e864 = phi_3188_;
    unnamed.gl_Position = _e864;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) XB: vec4<f32>, @location(1) YB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    XB_1 = XB;
    YB_1 = YB;
    main_1();
    let _e21 = S;
    let _e22 = G0_;
    let _e23 = j2_;
    let _e24 = Q0_;
    let _e25 = W0_;
    let _e26 = P0_;
    let _e27 = V0_;
    let _e28 = y3_;
    let _e29 = J4_;
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
