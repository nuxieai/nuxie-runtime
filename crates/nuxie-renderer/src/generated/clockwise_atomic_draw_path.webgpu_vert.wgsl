struct pi {
    x2_: array<vec4<u32>>,
}

struct jd {
    x2_: array<vec4<u32>>,
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

struct ug {
    x2_: array<vec4<f32>>,
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

@id(0) override jj: bool = true;
@id(2) override lj: bool = true;
@id(1) override kj: bool = true;
@id(4) override nj: bool = true;

@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(4)
var<storage> CD: pi;
@group(0) @binding(2)
var<storage> JB: jd;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> ZB_1: vec4<f32>;
var<private> S: vec4<f32>;
var<private> I0_: f32;
var<private> l2_: vec2<f32>;
var<private> R0_: f32;
@group(0) @binding(3)
var<storage> KB: ug;
var<private> V0_: vec4<f32>;
var<private> M0_: vec4<f32>;
var<private> N0_: vec3<f32>;
var<private> B3_: vec2<u32>;
var<private> P4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_2896_: f32;
    var phi_2833_: f32;
    var phi_2923_: f32;
    var phi_2859_: f32;
    var phi_2827_: i32;
    var phi_1745_: bool;
    var phi_2844_: i32;
    var phi_2835_: vec4<u32>;
    var phi_2843_: i32;
    var phi_2834_: vec4<u32>;
    var phi_2842_: i32;
    var phi_2840_: vec4<u32>;
    var phi_2839_: u32;
    var phi_2846_: vec2<i32>;
    var phi_2847_: vec4<u32>;
    var phi_2851_: f32;
    var phi_2866_: f32;
    var phi_2934_: f32;
    var phi_2933_: f32;
    var phi_2874_: f32;
    var phi_2867_: f32;
    var phi_2864_: f32;
    var phi_2878_: f32;
    var phi_2956_: f32;
    var phi_2946_: f32;
    var phi_2931_: f32;
    var phi_2877_: f32;
    var phi_2929_: f32;
    var phi_2967_: f32;
    var phi_2966_: f32;
    var phi_2968_: f32;
    var phi_2972_: f32;
    var phi_2995_: f32;
    var phi_2993_: f32;
    var phi_3011_: vec4<f32>;
    var phi_3132_: vec2<f32>;
    var phi_3010_: vec4<f32>;
    var phi_3138_: vec4<f32>;
    var phi_3015_: f32;
    var phi_3026_: f32;
    var phi_3018_: f32;
    var phi_3080_: f32;
    var phi_3054_: f32;
    var phi_2060_: bool;
    var phi_3059_: f32;
    var phi_3071_: vec2<f32>;
    var phi_3070_: vec2<f32>;
    var phi_3088_: vec4<f32>;
    var phi_3101_: vec2<f32>;
    var phi_3087_: vec4<f32>;
    var phi_3135_: vec4<f32>;
    var phi_3139_: bool;
    var phi_3133_: vec4<f32>;
    var phi_3127_: vec2<f32>;
    var phi_3103_: vec2<f32>;
    var phi_3171_: f32;
    var phi_3172_: u32;
    var phi_3173_: f32;
    var phi_3174_: f32;
    var phi_2570_: bool;
    var phi_3175_: vec4<f32>;
    var phi_3224_: f32;
    var phi_3226_: f32;
    var phi_1408_: bool;
    var phi_3227_: f32;
    var phi_3245_: vec4<f32>;

    let _e100 = gl_InstanceIndex_1;
    let _e101 = YB_1;
    let _e102 = ZB_1;
    let _e104 = i32(_e101.x);
    let _e108 = bitcast<i32>(_e101.w);
    let _e110 = (_e108 >> bitcast<u32>(2i));
    let _e111 = (_e108 & 3i);
    let _e113 = min(_e104, (_e110 - 1i));
    let _e115 = ((_e100 * _e110) + _e113);
    let _e120 = textureLoad(VB, vec2<i32>((_e115 & 2047i), (_e115 >> bitcast<u32>(11i))), 0i);
    let _e127 = CD.x2_[(max((_e120.w & 65535u), 1u) - 1u)];
    let _e129 = bitcast<vec2<f32>>(_e127.xy);
    let _e131 = (_e127.z & 65535u);
    let _e133 = (_e131 * 4u);
    let _e136 = JB.x2_[_e133];
    let _e137 = bitcast<vec4<f32>>(_e136);
    let _e144 = mat2x2<f32>(vec2<f32>(_e137.x, _e137.y), vec2<f32>(_e137.z, _e137.w));
    let _e148 = JB.x2_[(_e133 + 1u)];
    phi_2896_ = 0f;
    phi_2833_ = 0f;
    if ((_e148.z & 32768u) != 0u) {
        let _e157 = JB.x2_[(_e133 + 2u)];
        phi_2896_ = bitcast<f32>(_e157.y);
        phi_2833_ = bitcast<f32>(_e157.x);
    }
    let _e163 = phi_2896_;
    let _e165 = phi_2833_;
    let _e166 = (_e120.w & 8388608u);
    phi_2923_ = _e101.z;
    phi_2859_ = _e101.y;
    phi_2827_ = _e104;
    if (_e166 != 0u) {
        phi_2923_ = _e102.z;
        phi_2859_ = _e102.y;
        phi_2827_ = i32(_e102.x);
    }
    let _e173 = phi_2923_;
    let _e175 = phi_2859_;
    let _e177 = phi_2827_;
    phi_2842_ = _e115;
    phi_2840_ = _e120;
    phi_2839_ = _e120.w;
    if (_e177 != _e113) {
        let _e180 = ((_e115 + _e177) - _e113);
        let _e185 = textureLoad(VB, vec2<i32>((_e180 & 2047i), (_e180 >> bitcast<u32>(11i))), 0i);
        if ((_e185.w & 8454143u) != (_e120.w & 8454143u)) {
            let _e190 = (_e165 == 0f);
            phi_1745_ = _e190;
            if !(_e190) {
                phi_1745_ = (_e129.x != 0f);
            }
            let _e195 = phi_1745_;
            phi_2844_ = _e115;
            phi_2835_ = _e120;
            if _e195 {
                let _e196 = bitcast<i32>(_e127.w);
                let _e201 = textureLoad(VB, vec2<i32>((_e196 & 2047i), (_e196 >> bitcast<u32>(11i))), 0i);
                phi_2844_ = _e196;
                phi_2835_ = _e201;
            }
            let _e203 = phi_2844_;
            let _e205 = phi_2835_;
            phi_2843_ = _e203;
            phi_2834_ = _e205;
        } else {
            phi_2843_ = _e180;
            phi_2834_ = _e185;
        }
        let _e207 = phi_2843_;
        let _e209 = phi_2834_;
        phi_2842_ = _e207;
        phi_2840_ = _e209;
        phi_2839_ = ((_e209.w & 4286578687u) | _e166);
    }
    let _e214 = phi_2842_;
    let _e216 = phi_2840_;
    let _e218 = phi_2839_;
    let _e219 = (_e218 & 469762048u);
    let _e222 = ((_e219 == 67108864u) && (_e111 == 0i));
    if _e222 {
        let _e228 = f32((_e216.z & 65535u));
        let _e231 = f32((_e216.z >> bitcast<u32>(16i)));
        let _e237 = vec2<i32>(i32((-1f - _e228)), i32(((_e231 - _e228) + 1f)));
        phi_2846_ = _e237;
        if ((_e218 & 8388608u) != 0u) {
            phi_2846_ = -(_e237);
        }
        let _e242 = phi_2846_;
        let _e244 = (_e214 + _e242.x);
        let _e249 = textureLoad(VB, vec2<i32>((_e244 & 2047i), (_e244 >> bitcast<u32>(11i))), 0i);
        let _e251 = (_e214 + _e242.y);
        let _e256 = textureLoad(VB, vec2<i32>((_e251 & 2047i), (_e251 >> bitcast<u32>(11i))), 0i);
        phi_2847_ = _e256;
        if ((_e256.w & 8454143u) != (_e249.w & 8454143u)) {
            let _e262 = bitcast<i32>(_e127.w);
            let _e267 = textureLoad(VB, vec2<i32>((_e262 & 2047i), (_e262 >> bitcast<u32>(11i))), 0i);
            phi_2847_ = _e267;
        }
        let _e269 = phi_2847_;
        let _e272 = (f32(_e249.z) * 0.0000000014629181f);
        let _e275 = (f32(_e269.z) * 0.0000000014629181f);
        let _e276 = (_e275 - _e272);
        phi_2851_ = _e276;
        if (abs(_e276) > 3.1415927f) {
            phi_2851_ = (_e276 - (6.2831855f * sign(_e276)));
        }
        let _e283 = phi_2851_;
        let _e284 = (_e231 + -2f);
        let _e290 = clamp(round(((abs(_e283) * 0.31830987f) * _e284)), 1f, (_e231 + -3f));
        let _e291 = (_e284 - _e290);
        if (_e228 <= _e291) {
            phi_2934_ = _e175;
            if (_e228 == _e291) {
                phi_2934_ = -(_e175);
            }
            let _e308 = phi_2934_;
            phi_2933_ = _e308;
            phi_2874_ = -(((3.1415927f * sign(_e283)) - _e283));
            phi_2867_ = _e291;
            phi_2864_ = _e228;
        } else {
            let _e294 = (_e228 == (_e291 + 1f));
            if _e294 {
                phi_2866_ = 0f;
            } else {
                phi_2866_ = (_e228 - (_e291 + 2f));
            }
            let _e298 = phi_2866_;
            phi_2933_ = select(_e175, 0f, _e294);
            phi_2874_ = _e283;
            phi_2867_ = select(_e290, 0f, _e294);
            phi_2864_ = _e298;
        }
        let _e310 = phi_2933_;
        let _e312 = phi_2874_;
        let _e314 = phi_2867_;
        let _e316 = phi_2864_;
        if (_e316 == _e314) {
            phi_2878_ = _e275;
        } else {
            phi_2878_ = (_e272 + (_e312 * (_e316 / _e314)));
        }
        let _e322 = phi_2878_;
        phi_2956_ = _e272;
        phi_2946_ = _e312;
        phi_2931_ = _e310;
        phi_2877_ = _e322;
    } else {
        phi_2956_ = f32();
        phi_2946_ = f32();
        phi_2931_ = _e175;
        phi_2877_ = (f32(_e216.z) * 0.0000000014629181f);
    }
    let _e324 = phi_2956_;
    let _e326 = phi_2946_;
    let _e328 = phi_2931_;
    let _e330 = phi_2877_;
    let _e334 = vec2<f32>(sin(_e330), -(cos(_e330)));
    let _e336 = bitcast<vec2<f32>>(_e216.xy);
    phi_2929_ = _e163;
    if (_e163 != 0f) {
        phi_2929_ = max(_e163, (1f / length((_e144 * _e334))));
    }
    let _e343 = phi_2929_;
    if (_e165 != 0f) {
        let _e451 = (_e328 * sign(determinant(_e144)));
        let _e453 = ((_e218 & 1048576u) != 0u);
        phi_3015_ = _e451;
        if _e453 {
            phi_3015_ = min(_e451, 0f);
        }
        let _e456 = phi_3015_;
        phi_3026_ = _e456;
        if ((_e218 & 524288u) != 0u) {
            phi_3026_ = max(_e456, 0f);
        }
        let _e461 = phi_3026_;
        let _e462 = (_e343 != 0f);
        if _e462 {
            phi_3018_ = _e343;
        } else {
            let _e463 = (_e144 * _e334);
            phi_3018_ = (((abs(_e463.x) + abs(_e463.y)) * (1f / dot(_e463, _e463))) * 0.5f);
        }
        let _e474 = phi_3018_;
        let _e477 = ((_e474 > _e165) && (_e343 == 0f));
        phi_3080_ = 1f;
        if _e477 {
            phi_3080_ = (_e165 / _e474);
        }
        let _e480 = phi_3080_;
        let _e481 = select(_e165, _e474, _e477);
        let _e482 = (_e481 + _e474);
        let _e483 = (_e334 * _e482);
        let _e484 = (_e461 * _e482);
        let _e491 = (((vec2<f32>(_e484, -(_e484)) + vec2(_e481)) * (0.5f / _e474)) + vec2<f32>(0.5f, 0.5f));
        let _e494 = vec4<f32>(_e491.x, _e491.y, 0f, 0f);
        phi_3101_ = _e483;
        phi_3087_ = _e494;
        if (_e219 > 201326592u) {
            let _e500 = f32((_e216.z & 65535u));
            let _e501 = (_e500 * 0.000015259022f);
            let _e505 = sqrt(max((1f - (_e501 * _e501)), 0f));
            phi_3054_ = _e505;
            if (((_e218 & 4194304u) != 0u) == _e453) {
                phi_3054_ = -(_e505);
            }
            let _e509 = phi_3054_;
            let _e514 = (mat2x2<f32>(vec2<f32>(_e501, _e509), vec2<f32>(-(_e509), _e501)) * _e334);
            let _e515 = (_e144 * _e514);
            let _e523 = ((abs(_e515.x) + abs(_e515.y)) * (1f / dot(_e515, _e515)));
            let _e524 = (_e219 == 402653184u);
            phi_2060_ = _e524;
            if !(_e524) {
                phi_2060_ = ((_e219 == 335544320u) && (_e501 >= 0.25f));
            }
            let _e530 = phi_2060_;
            if _e530 {
                phi_3059_ = (_e481 * (1f / max(_e501, select(0.25f, 1f, ((_e218 & 33554432u) != 0u)))));
            } else {
                phi_3059_ = ((_e481 * _e501) + (_e523 * 0.5f));
            }
            let _e541 = phi_3059_;
            let _e543 = (_e541 + (_e523 * 0.5f));
            phi_3070_ = _e483;
            if ((_e218 & 2097152u) != 0u) {
                if (_e482 <= ((_e543 * _e501) + (_e474 * 0.125f))) {
                    phi_3071_ = (_e514 * (_e482 * (65535f / _e500)));
                } else {
                    let _e550 = (_e514 * _e543);
                    phi_3071_ = (vec2<f32>(dot(_e483, _e483), dot(_e550, _e550)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e483, _e550)));
                }
                let _e561 = phi_3071_;
                phi_3070_ = _e561;
            }
            let _e563 = phi_3070_;
            let _e568 = ((_e543 - dot((_e563 * abs(_e461)), _e514)) / _e523);
            if _e453 {
                phi_3088_ = vec4<f32>(_e494.x, _e568, _e494.z, _e494.w);
            } else {
                phi_3088_ = vec4<f32>(_e568, _e494.y, _e494.z, _e494.w);
            }
            let _e580 = phi_3088_;
            phi_3101_ = _e563;
            phi_3087_ = _e580;
        }
        let _e582 = phi_3101_;
        let _e584 = phi_3087_;
        let _e586 = (_e584.xy * _e480);
        let _e592 = vec4<f32>(_e586.x, _e584.y, _e584.z, _e584.w);
        let _e599 = vec4<f32>(_e592.x, max(_e586.y, 0.0001f), _e592.z, _e592.w);
        phi_3135_ = _e599;
        if _e462 {
            phi_3135_ = vec4<f32>((-2f - _e586.x), _e599.y, _e599.z, _e599.w);
        }
        let _e607 = phi_3135_;
        phi_3139_ = (_e111 != 0i);
        phi_3133_ = _e607;
        phi_3127_ = (_e144 * (_e582 * _e461));
        phi_3103_ = _e336;
    } else {
        let _e345 = vec4<f32>(_e173, -1f, 0f, 0f);
        if (_e343 != 0f) {
            let _e356 = vec4<f32>(_e345.x, -2f, _e345.z, _e345.w);
            let _e361 = vec4<f32>(_e356.x, _e356.y, 1000000f, _e356.w);
            phi_3011_ = vec4<f32>(_e361.x, _e361.y, _e361.z, _e173);
            if _e222 {
                phi_2967_ = _e326;
                phi_2966_ = _e324;
                if (_e326 < 0f) {
                    phi_2967_ = -(_e326);
                    phi_2966_ = (_e324 + _e326);
                }
                let _e371 = phi_2967_;
                let _e373 = phi_2966_;
                let _e375 = ((_e330 - _e373) + 1.5707964f);
                let _e381 = clamp(((_e375 - (floor((_e375 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e371);
                phi_2968_ = _e381;
                if (_e381 > (_e371 * 0.5f)) {
                    phi_2968_ = (_e371 - _e381);
                }
                let _e386 = phi_2968_;
                let _e393 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e386), cos(_e386)) * abs(_e328))) * 0.5f);
                if (abs((_e371 - 1.5707964f)) < 0.001f) {
                    phi_2995_ = 0f;
                    phi_2993_ = 0f;
                } else {
                    let _e397 = tan(_e371);
                    let _e402 = (sign((1.5707964f - _e371)) / max(abs(_e397), 0.000001f));
                    if (_e402 >= 0f) {
                        phi_2972_ = (_e393.y - ((1f - _e393.x) * _e397));
                    } else {
                        phi_2972_ = (_e393.y + (_e393.x * _e397));
                    }
                    let _e414 = phi_2972_;
                    phi_2995_ = _e414;
                    phi_2993_ = _e402;
                }
                let _e416 = phi_2995_;
                let _e418 = phi_2993_;
                phi_3011_ = vec4<f32>((max(_e393.x, 0f) + 0.25f), (-2f - _e393.y), _e418, _e416);
            }
            let _e426 = phi_3011_;
            phi_3132_ = (_e144 * (_e334 * (_e328 * _e343)));
            phi_3010_ = _e426;
        } else {
            phi_3132_ = (sign(((_e334 * _e328) * _naga_inverse_2x2_f32(_e144))) * 0.5f);
            phi_3010_ = _e345;
        }
        let _e431 = phi_3132_;
        let _e433 = phi_3010_;
        phi_3138_ = _e433;
        if (((_e218 & 8388608u) != 0u) != ((_e218 & 16777216u) != 0u)) {
            phi_3138_ = (_e433 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e441 = phi_3138_;
        phi_3139_ = (((_e218 & 2147483648u) != 0u) && (_e111 != 1i));
        phi_3133_ = _e441;
        phi_3127_ = _e431;
        phi_3103_ = select(_e336, _e129, vec2((_e111 == 2i)));
    }
    let _e612 = phi_3139_;
    let _e614 = phi_3133_;
    let _e616 = phi_3127_;
    let _e618 = phi_3103_;
    let _e621 = (((_e144 * _e618) + _e616) + bitcast<vec2<f32>>(_e148.xy));
    let _e624 = j.Gi;
    let _e627 = select(_e614.xy, vec2<f32>(1f, -1f), vec2((_e624 != 0u)));
    let _e633 = vec4<f32>(_e627.x, _e614.y, _e614.z, _e614.w);
    S = vec4<f32>(_e633.x, _e627.y, _e633.z, _e633.w);
    let _e641 = j.w6_;
    if (_e131 == 0u) {
        phi_3171_ = 0f;
    } else {
        phi_3171_ = unpack2x16float(((_e131 + 1023u) * _e641)).x;
    }
    let _e648 = phi_3171_;
    I0_ = _e648;
    if ((_e148.z & 512u) != 0u) {
        let _e651 = I0_;
        I0_ = -(_e651);
    }
    let _e653 = (_e148.z & 15u);
    if jj {
        let _e654 = (_e653 == 0u);
        if _e654 {
            phi_3172_ = _e148.w;
        } else {
            phi_3172_ = _e148.z;
        }
        let _e657 = phi_3172_;
        let _e659 = (_e657 >> bitcast<u32>(16i));
        if (_e659 == 0u) {
            phi_3173_ = 0f;
        } else {
            phi_3173_ = unpack2x16float(((_e659 + 1023u) * _e641)).x;
        }
        let _e666 = phi_3173_;
        phi_3174_ = _e666;
        if _e654 {
            phi_3174_ = -(_e666);
        }
        let _e669 = phi_3174_;
        l2_[0u] = _e669;
    }
    if lj {
        R0_ = f32(((_e148.z >> bitcast<u32>(4i)) & 15u));
    }
    if kj {
        let _e675 = (_e131 * 8u);
        let _e679 = KB.x2_[(_e675 + 2u)];
        let _e684 = vec2<f32>(_e679.x, _e679.y);
        let _e685 = vec2<f32>(_e679.z, _e679.w);
        let _e690 = KB.x2_[(_e675 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e695 = (abs(_e684) + abs(_e685));
                let _e697 = (_e695.x != 0f);
                phi_2570_ = _e697;
                if _e697 {
                    phi_2570_ = (_e695.y != 0f);
                }
                let _e701 = phi_2570_;
                if _e701 {
                    let _e705 = ((mat2x2<f32>(_e684, _e685) * _e621) + _e690.xy);
                    let _e706 = -(_e705);
                    let _e712 = (vec2<f32>(1f, 1f) / _e695).xyxy;
                    phi_3175_ = (((vec4<f32>(_e705.x, _e705.y, _e706.x, _e706.y) * _e712) + _e712) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3175_ = _e690.xyxy;
                    break;
                }
            }
        }
        let _e717 = phi_3175_;
        V0_ = _e717;
    }
    if (_e653 == 1u) {
        M0_ = unpack4x8unorm(_e148.w);
    } else {
        if (jj && (_e653 == 0u)) {
            let _e760 = (_e148.z >> bitcast<u32>(16i));
            if (_e760 == 0u) {
                phi_3226_ = 0f;
            } else {
                phi_3226_ = unpack2x16float(((_e760 + 1023u) * _e641)).x;
            }
            let _e767 = phi_3226_;
            l2_[1u] = _e767;
        } else {
            let _e721 = (_e131 * 8u);
            let _e724 = KB.x2_[_e721];
            let _e735 = KB.x2_[(_e721 + 1u)];
            let _e740 = ((mat2x2<f32>(vec2<f32>(_e724.x, _e724.y), vec2<f32>(_e724.z, _e724.w)) * _e621) + _e735.xy);
            let _e751 = ((_e735.w + (f32(_e653) * 0.125f)) + (max(_e735.z, 0f) * 0.00024414063f));
            if (_e735.z < 0f) {
                phi_3224_ = -(_e751);
            } else {
                phi_3224_ = _e751;
            }
            let _e754 = phi_3224_;
            M0_ = vec4<f32>(_e740.x, _e740.y, _e754, (-0.75f - round((bitcast<f32>(_e148.w) * 255f))));
        }
    }
    phi_1408_ = nj;
    if nj {
        phi_1408_ = ((_e148.z & 2048u) != 0u);
    }
    let _e774 = phi_1408_;
    if _e774 {
        let _e775 = (_e131 * 8u);
        let _e779 = KB.x2_[(_e775 + 4u)];
        let _e790 = KB.x2_[(_e775 + 5u)];
        let _e793 = ((mat2x2<f32>(vec2<f32>(_e779.x, _e779.y), vec2<f32>(_e779.z, _e779.w)) * _e621) + _e790.xy);
        phi_3227_ = (1f + _e790.z);
        if ((_e148.z & 4096u) != 0u) {
            phi_3227_ = (-1f - f32(((_e148.z & 24576u) >> bitcast<u32>(13u))));
        }
        let _e804 = phi_3227_;
        N0_ = vec3<f32>(_e793.x, _e793.y, _e804);
    } else {
        N0_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e612) {
        let _e813 = j.Rg;
        let _e815 = j.Sg;
        let _e827 = JB.x2_[(_e133 + 3u)];
        B3_ = _e827.xy;
        P4_ = (_e621 + bitcast<vec2<f32>>(_e827.zw));
        phi_3245_ = vec4<f32>(((_e621.x * _e813) - 1f), ((_e621.y * _e815) - sign(_e815)), 0f, 1f);
    } else {
        let _e810 = j.j3_;
        phi_3245_ = vec4(_e810);
    }
    let _e833 = phi_3245_;
    unnamed.gl_Position = _e833;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) YB: vec4<f32>, @location(1) ZB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    YB_1 = YB;
    ZB_1 = ZB;
    main_1();
    let _e21 = S;
    let _e22 = I0_;
    let _e23 = l2_;
    let _e24 = R0_;
    let _e25 = V0_;
    let _e26 = M0_;
    let _e27 = N0_;
    let _e28 = B3_;
    let _e29 = P4_;
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
