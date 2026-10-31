struct xi {
    z2_: array<vec4<u32>>,
}

struct nd {
    z2_: array<vec4<u32>>,
}

struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct Cg {
    z2_: array<vec4<f32>>,
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

@id(0) override uj: bool = true;
@id(2) override wj: bool = true;
@id(1) override vj: bool = true;
@id(4) override yj: bool = true;

@group(0) @binding(6)
var WB: texture_2d<u32>;
@group(0) @binding(4)
var<storage> CD: xi;
@group(0) @binding(2)
var<storage> JB: nd;
@group(0) @binding(0)
var<uniform> j: RB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> ZB_1: vec4<f32>;
var<private> S: vec4<f32>;
var<private> J0_: f32;
var<private> l2_: vec2<f32>;
var<private> S0_: f32;
@group(0) @binding(3)
var<storage> KB: Cg;
var<private> Y0_: vec4<f32>;
var<private> N0_: vec4<f32>;
var<private> O0_: vec3<f32>;
var<private> D3_: vec2<u32>;
var<private> V4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var ib: sampler;

fn main_1() {
    var phi_2973_: f32;
    var phi_2923_: f32;
    var phi_2860_: f32;
    var phi_2950_: f32;
    var phi_2886_: f32;
    var phi_2854_: i32;
    var phi_1762_: bool;
    var phi_2871_: i32;
    var phi_2862_: vec4<u32>;
    var phi_2870_: i32;
    var phi_2861_: vec4<u32>;
    var phi_2869_: i32;
    var phi_2867_: vec4<u32>;
    var phi_2866_: u32;
    var phi_2873_: vec2<i32>;
    var phi_2874_: vec4<u32>;
    var phi_2878_: f32;
    var phi_2893_: f32;
    var phi_2961_: f32;
    var phi_2960_: f32;
    var phi_2901_: f32;
    var phi_2894_: f32;
    var phi_2891_: f32;
    var phi_2905_: f32;
    var phi_2998_: f32;
    var phi_2988_: f32;
    var phi_2958_: f32;
    var phi_2904_: f32;
    var phi_2956_: f32;
    var phi_3178_: vec2<f32>;
    var phi_3009_: f32;
    var phi_3008_: f32;
    var phi_3010_: f32;
    var phi_3014_: f32;
    var phi_3037_: f32;
    var phi_3035_: f32;
    var phi_3054_: vec4<f32>;
    var phi_3177_: vec2<f32>;
    var phi_3053_: vec4<f32>;
    var phi_3184_: vec4<f32>;
    var phi_3059_: f32;
    var phi_3070_: f32;
    var phi_3062_: f32;
    var phi_3124_: f32;
    var phi_3098_: f32;
    var phi_2077_: bool;
    var phi_3103_: f32;
    var phi_3115_: vec2<f32>;
    var phi_3114_: vec2<f32>;
    var phi_3132_: vec4<f32>;
    var phi_3145_: vec2<f32>;
    var phi_3131_: vec4<f32>;
    var phi_3181_: vec4<f32>;
    var phi_3185_: bool;
    var phi_3179_: vec4<f32>;
    var phi_3172_: vec2<f32>;
    var phi_3147_: vec2<f32>;
    var phi_3218_: f32;
    var phi_3219_: u32;
    var phi_3220_: f32;
    var phi_3221_: f32;
    var phi_2596_: bool;
    var phi_3222_: vec4<f32>;
    var phi_3272_: f32;
    var phi_3274_: f32;
    var phi_1421_: bool;
    var phi_3275_: f32;
    var phi_3293_: vec4<f32>;

    let _e100 = gl_InstanceIndex_1;
    let _e101 = YB_1;
    let _e102 = ZB_1;
    let _e104 = i32(_e101.x);
    let _e108 = bitcast<i32>(_e101.w);
    let _e110 = (_e108 >> bitcast<u32>(2i));
    let _e111 = (_e108 & 3i);
    let _e113 = min(_e104, (_e110 - 1i));
    let _e115 = ((_e100 * _e110) + _e113);
    let _e120 = textureLoad(WB, vec2<i32>((_e115 & 2047i), (_e115 >> bitcast<u32>(11i))), 0i);
    let _e127 = CD.z2_[(max((_e120.w & 65535u), 1u) - 1u)];
    let _e129 = bitcast<vec2<f32>>(_e127.xy);
    let _e131 = (_e127.z & 65535u);
    let _e133 = (_e131 * 4u);
    let _e136 = JB.z2_[_e133];
    let _e137 = bitcast<vec4<f32>>(_e136);
    let _e144 = mat2x2<f32>(vec2<f32>(_e137.x, _e137.y), vec2<f32>(_e137.z, _e137.w));
    let _e148 = JB.z2_[(_e133 + 1u)];
    phi_2973_ = 0f;
    phi_2923_ = 0f;
    phi_2860_ = 0f;
    if ((_e148.z & 32768u) != 0u) {
        let _e157 = JB.z2_[(_e133 + 2u)];
        phi_2973_ = bitcast<f32>(_e157.z);
        phi_2923_ = bitcast<f32>(_e157.y);
        phi_2860_ = bitcast<f32>(_e157.x);
    }
    let _e165 = phi_2973_;
    let _e167 = phi_2923_;
    let _e169 = phi_2860_;
    let _e170 = (_e120.w & 8388608u);
    phi_2950_ = _e101.z;
    phi_2886_ = _e101.y;
    phi_2854_ = _e104;
    if (_e170 != 0u) {
        phi_2950_ = _e102.z;
        phi_2886_ = _e102.y;
        phi_2854_ = i32(_e102.x);
    }
    let _e177 = phi_2950_;
    let _e179 = phi_2886_;
    let _e181 = phi_2854_;
    phi_2869_ = _e115;
    phi_2867_ = _e120;
    phi_2866_ = _e120.w;
    if (_e181 != _e113) {
        let _e184 = ((_e115 + _e181) - _e113);
        let _e189 = textureLoad(WB, vec2<i32>((_e184 & 2047i), (_e184 >> bitcast<u32>(11i))), 0i);
        if ((_e189.w & 8454143u) != (_e120.w & 8454143u)) {
            let _e194 = (_e169 == 0f);
            phi_1762_ = _e194;
            if !(_e194) {
                phi_1762_ = (_e129.x != 0f);
            }
            let _e199 = phi_1762_;
            phi_2871_ = _e115;
            phi_2862_ = _e120;
            if _e199 {
                let _e200 = bitcast<i32>(_e127.w);
                let _e205 = textureLoad(WB, vec2<i32>((_e200 & 2047i), (_e200 >> bitcast<u32>(11i))), 0i);
                phi_2871_ = _e200;
                phi_2862_ = _e205;
            }
            let _e207 = phi_2871_;
            let _e209 = phi_2862_;
            phi_2870_ = _e207;
            phi_2861_ = _e209;
        } else {
            phi_2870_ = _e184;
            phi_2861_ = _e189;
        }
        let _e211 = phi_2870_;
        let _e213 = phi_2861_;
        phi_2869_ = _e211;
        phi_2867_ = _e213;
        phi_2866_ = ((_e213.w & 4286578687u) | _e170);
    }
    let _e218 = phi_2869_;
    let _e220 = phi_2867_;
    let _e222 = phi_2866_;
    let _e223 = (_e222 & 469762048u);
    let _e226 = ((_e223 == 67108864u) && (_e111 == 0i));
    if _e226 {
        let _e232 = f32((_e220.z & 65535u));
        let _e235 = f32((_e220.z >> bitcast<u32>(16i)));
        let _e241 = vec2<i32>(i32((-1f - _e232)), i32(((_e235 - _e232) + 1f)));
        phi_2873_ = _e241;
        if ((_e222 & 8388608u) != 0u) {
            phi_2873_ = -(_e241);
        }
        let _e246 = phi_2873_;
        let _e248 = (_e218 + _e246.x);
        let _e253 = textureLoad(WB, vec2<i32>((_e248 & 2047i), (_e248 >> bitcast<u32>(11i))), 0i);
        let _e255 = (_e218 + _e246.y);
        let _e260 = textureLoad(WB, vec2<i32>((_e255 & 2047i), (_e255 >> bitcast<u32>(11i))), 0i);
        phi_2874_ = _e260;
        if ((_e260.w & 8454143u) != (_e253.w & 8454143u)) {
            let _e266 = bitcast<i32>(_e127.w);
            let _e271 = textureLoad(WB, vec2<i32>((_e266 & 2047i), (_e266 >> bitcast<u32>(11i))), 0i);
            phi_2874_ = _e271;
        }
        let _e273 = phi_2874_;
        let _e276 = (f32(_e253.z) * 0.0000000014629181f);
        let _e279 = (f32(_e273.z) * 0.0000000014629181f);
        let _e280 = (_e279 - _e276);
        phi_2878_ = _e280;
        if (abs(_e280) > 3.1415927f) {
            phi_2878_ = (_e280 - (6.2831855f * sign(_e280)));
        }
        let _e287 = phi_2878_;
        let _e288 = (_e235 + -2f);
        let _e294 = clamp(round(((abs(_e287) * 0.31830987f) * _e288)), 1f, (_e235 + -3f));
        let _e295 = (_e288 - _e294);
        if (_e232 <= _e295) {
            phi_2961_ = _e179;
            if (_e232 == _e295) {
                phi_2961_ = -(_e179);
            }
            let _e312 = phi_2961_;
            phi_2960_ = _e312;
            phi_2901_ = -(((3.1415927f * sign(_e287)) - _e287));
            phi_2894_ = _e295;
            phi_2891_ = _e232;
        } else {
            let _e298 = (_e232 == (_e295 + 1f));
            if _e298 {
                phi_2893_ = 0f;
            } else {
                phi_2893_ = (_e232 - (_e295 + 2f));
            }
            let _e302 = phi_2893_;
            phi_2960_ = select(_e179, 0f, _e298);
            phi_2901_ = _e287;
            phi_2894_ = select(_e294, 0f, _e298);
            phi_2891_ = _e302;
        }
        let _e314 = phi_2960_;
        let _e316 = phi_2901_;
        let _e318 = phi_2894_;
        let _e320 = phi_2891_;
        if (_e320 == _e318) {
            phi_2905_ = _e279;
        } else {
            phi_2905_ = (_e276 + (_e316 * (_e320 / _e318)));
        }
        let _e326 = phi_2905_;
        phi_2998_ = _e276;
        phi_2988_ = _e316;
        phi_2958_ = _e314;
        phi_2904_ = _e326;
    } else {
        phi_2998_ = f32();
        phi_2988_ = f32();
        phi_2958_ = _e179;
        phi_2904_ = (f32(_e220.z) * 0.0000000014629181f);
    }
    let _e328 = phi_2998_;
    let _e330 = phi_2988_;
    let _e332 = phi_2958_;
    let _e334 = phi_2904_;
    let _e338 = vec2<f32>(sin(_e334), -(cos(_e334)));
    let _e340 = bitcast<vec2<f32>>(_e220.xy);
    phi_2956_ = _e167;
    if (_e167 != 0f) {
        phi_2956_ = max(_e167, (1f / length((_e144 * _e338))));
    }
    let _e347 = phi_2956_;
    if (_e169 != 0f) {
        let _e460 = (_e332 * sign(determinant(_e144)));
        let _e462 = ((_e222 & 1048576u) != 0u);
        phi_3059_ = _e460;
        if _e462 {
            phi_3059_ = min(_e460, 0f);
        }
        let _e465 = phi_3059_;
        phi_3070_ = _e465;
        if ((_e222 & 524288u) != 0u) {
            phi_3070_ = max(_e465, 0f);
        }
        let _e470 = phi_3070_;
        let _e471 = (_e347 != 0f);
        if _e471 {
            phi_3062_ = _e347;
        } else {
            let _e472 = (_e144 * _e338);
            phi_3062_ = (((abs(_e472.x) + abs(_e472.y)) * (1f / dot(_e472, _e472))) * 0.5f);
        }
        let _e483 = phi_3062_;
        let _e486 = ((_e483 > _e169) && (_e347 == 0f));
        phi_3124_ = 1f;
        if _e486 {
            phi_3124_ = (_e169 / _e483);
        }
        let _e489 = phi_3124_;
        let _e490 = select(_e169, _e483, _e486);
        let _e491 = (_e490 + _e483);
        let _e492 = (_e338 * _e491);
        let _e493 = (_e470 * _e491);
        let _e500 = (((vec2<f32>(_e493, -(_e493)) + vec2(_e490)) * (0.5f / _e483)) + vec2<f32>(0.5f, 0.5f));
        let _e503 = vec4<f32>(_e500.x, _e500.y, 0f, 0f);
        phi_3145_ = _e492;
        phi_3131_ = _e503;
        if (_e223 > 201326592u) {
            let _e509 = f32((_e220.z & 65535u));
            let _e510 = (_e509 * 0.000015259022f);
            let _e514 = sqrt(max((1f - (_e510 * _e510)), 0f));
            phi_3098_ = _e514;
            if (((_e222 & 4194304u) != 0u) == _e462) {
                phi_3098_ = -(_e514);
            }
            let _e518 = phi_3098_;
            let _e523 = (mat2x2<f32>(vec2<f32>(_e510, _e518), vec2<f32>(-(_e518), _e510)) * _e338);
            let _e524 = (_e144 * _e523);
            let _e532 = ((abs(_e524.x) + abs(_e524.y)) * (1f / dot(_e524, _e524)));
            let _e533 = (_e223 == 402653184u);
            phi_2077_ = _e533;
            if !(_e533) {
                phi_2077_ = ((_e223 == 335544320u) && (_e510 >= 0.25f));
            }
            let _e539 = phi_2077_;
            if _e539 {
                phi_3103_ = (_e490 * (1f / max(_e510, select(0.25f, 1f, ((_e222 & 33554432u) != 0u)))));
            } else {
                phi_3103_ = ((_e490 * _e510) + (_e532 * 0.5f));
            }
            let _e550 = phi_3103_;
            let _e552 = (_e550 + (_e532 * 0.5f));
            phi_3114_ = _e492;
            if ((_e222 & 2097152u) != 0u) {
                if (_e491 <= ((_e552 * _e510) + (_e483 * 0.125f))) {
                    phi_3115_ = (_e523 * (_e491 * (65535f / _e509)));
                } else {
                    let _e559 = (_e523 * _e552);
                    phi_3115_ = (vec2<f32>(dot(_e492, _e492), dot(_e559, _e559)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e492, _e559)));
                }
                let _e570 = phi_3115_;
                phi_3114_ = _e570;
            }
            let _e572 = phi_3114_;
            let _e577 = ((_e552 - dot((_e572 * abs(_e470)), _e523)) / _e532);
            if _e462 {
                phi_3132_ = vec4<f32>(_e503.x, _e577, _e503.z, _e503.w);
            } else {
                phi_3132_ = vec4<f32>(_e577, _e503.y, _e503.z, _e503.w);
            }
            let _e589 = phi_3132_;
            phi_3145_ = _e572;
            phi_3131_ = _e589;
        }
        let _e591 = phi_3145_;
        let _e593 = phi_3131_;
        let _e595 = (_e593.xy * _e489);
        let _e601 = vec4<f32>(_e595.x, _e593.y, _e593.z, _e593.w);
        let _e608 = vec4<f32>(_e601.x, max(_e595.y, 0.0001f), _e601.z, _e601.w);
        phi_3181_ = _e608;
        if _e471 {
            phi_3181_ = vec4<f32>((-2f - _e595.x), _e608.y, _e608.z, _e608.w);
        }
        let _e616 = phi_3181_;
        phi_3185_ = (_e111 != 0i);
        phi_3179_ = _e616;
        phi_3172_ = (_e144 * (_e591 * _e470));
        phi_3147_ = _e340;
    } else {
        let _e349 = vec4<f32>(_e177, -1f, 0f, 0f);
        if (_e347 != 0f) {
            let _e365 = vec4<f32>(_e349.x, -2f, _e349.z, _e349.w);
            let _e370 = vec4<f32>(_e365.x, _e365.y, 1000000f, _e365.w);
            phi_3054_ = vec4<f32>(_e370.x, _e370.y, _e370.z, _e177);
            if _e226 {
                phi_3009_ = _e330;
                phi_3008_ = _e328;
                if (_e330 < 0f) {
                    phi_3009_ = -(_e330);
                    phi_3008_ = (_e328 + _e330);
                }
                let _e380 = phi_3009_;
                let _e382 = phi_3008_;
                let _e384 = ((_e334 - _e382) + 1.5707964f);
                let _e390 = clamp(((_e384 - (floor((_e384 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e380);
                phi_3010_ = _e390;
                if (_e390 > (_e380 * 0.5f)) {
                    phi_3010_ = (_e380 - _e390);
                }
                let _e395 = phi_3010_;
                let _e402 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e395), cos(_e395)) * abs(_e332))) * 0.5f);
                if (abs((_e380 - 1.5707964f)) < 0.001f) {
                    phi_3037_ = 0f;
                    phi_3035_ = 0f;
                } else {
                    let _e406 = tan(_e380);
                    let _e411 = (sign((1.5707964f - _e380)) / max(abs(_e406), 0.000001f));
                    if (_e411 >= 0f) {
                        phi_3014_ = (_e402.y - ((1f - _e402.x) * _e406));
                    } else {
                        phi_3014_ = (_e402.y + (_e402.x * _e406));
                    }
                    let _e423 = phi_3014_;
                    phi_3037_ = _e423;
                    phi_3035_ = _e411;
                }
                let _e425 = phi_3037_;
                let _e427 = phi_3035_;
                phi_3054_ = vec4<f32>((max(_e402.x, 0f) + 0.25f), (-2f - _e402.y), _e427, _e425);
            }
            let _e435 = phi_3054_;
            phi_3177_ = (_e144 * (_e338 * (_e332 * _e347)));
            phi_3053_ = _e435;
        } else {
            let _e355 = (sign(((_e338 * _e332) * _naga_inverse_2x2_f32(_e144))) * 0.5f);
            phi_3178_ = _e355;
            if (_e111 != 2i) {
                phi_3178_ = (_e355 + (_e338 * _e165));
            }
            let _e360 = phi_3178_;
            phi_3177_ = _e360;
            phi_3053_ = _e349;
        }
        let _e440 = phi_3177_;
        let _e442 = phi_3053_;
        phi_3184_ = _e442;
        if (((_e222 & 8388608u) != 0u) != ((_e222 & 16777216u) != 0u)) {
            phi_3184_ = (_e442 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e450 = phi_3184_;
        phi_3185_ = (((_e222 & 2147483648u) != 0u) && (_e111 != 1i));
        phi_3179_ = _e450;
        phi_3172_ = _e440;
        phi_3147_ = select(_e340, _e129, vec2((_e111 == 2i)));
    }
    let _e621 = phi_3185_;
    let _e623 = phi_3179_;
    let _e625 = phi_3172_;
    let _e627 = phi_3147_;
    let _e630 = (((_e144 * _e627) + _e625) + bitcast<vec2<f32>>(_e148.xy));
    let _e633 = j.Oi;
    let _e636 = select(_e623.xy, vec2<f32>(1f, -1f), vec2((_e633 != 0u)));
    let _e642 = vec4<f32>(_e636.x, _e623.y, _e623.z, _e623.w);
    S = vec4<f32>(_e642.x, _e636.y, _e642.z, _e642.w);
    let _e650 = j.z6_;
    if (_e131 == 0u) {
        phi_3218_ = 0f;
    } else {
        phi_3218_ = unpack2x16float(((_e131 + 1023u) * _e650)).x;
    }
    let _e657 = phi_3218_;
    J0_ = _e657;
    if ((_e148.z & 512u) != 0u) {
        let _e660 = J0_;
        J0_ = -(_e660);
    }
    let _e662 = (_e148.z & 15u);
    if uj {
        let _e663 = (_e662 == 0u);
        if _e663 {
            phi_3219_ = _e148.w;
        } else {
            phi_3219_ = _e148.z;
        }
        let _e666 = phi_3219_;
        let _e668 = (_e666 >> bitcast<u32>(16i));
        if (_e668 == 0u) {
            phi_3220_ = 0f;
        } else {
            phi_3220_ = unpack2x16float(((_e668 + 1023u) * _e650)).x;
        }
        let _e675 = phi_3220_;
        phi_3221_ = _e675;
        if _e663 {
            phi_3221_ = -(_e675);
        }
        let _e678 = phi_3221_;
        l2_[0u] = _e678;
    }
    if wj {
        S0_ = f32(((_e148.z >> bitcast<u32>(4i)) & 15u));
    }
    if vj {
        let _e684 = (_e131 * 8u);
        let _e688 = KB.z2_[(_e684 + 2u)];
        let _e693 = vec2<f32>(_e688.x, _e688.y);
        let _e694 = vec2<f32>(_e688.z, _e688.w);
        let _e699 = KB.z2_[(_e684 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e704 = (abs(_e693) + abs(_e694));
                let _e706 = (_e704.x != 0f);
                phi_2596_ = _e706;
                if _e706 {
                    phi_2596_ = (_e704.y != 0f);
                }
                let _e710 = phi_2596_;
                if _e710 {
                    let _e714 = ((mat2x2<f32>(_e693, _e694) * _e630) + _e699.xy);
                    let _e715 = -(_e714);
                    let _e721 = (vec2<f32>(1f, 1f) / _e704).xyxy;
                    phi_3222_ = (((vec4<f32>(_e714.x, _e714.y, _e715.x, _e715.y) * _e721) + _e721) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3222_ = _e699.xyxy;
                    break;
                }
            }
        }
        let _e726 = phi_3222_;
        Y0_ = _e726;
    }
    if (_e662 == 1u) {
        N0_ = unpack4x8unorm(_e148.w);
    } else {
        if (uj && (_e662 == 0u)) {
            let _e769 = (_e148.z >> bitcast<u32>(16i));
            if (_e769 == 0u) {
                phi_3274_ = 0f;
            } else {
                phi_3274_ = unpack2x16float(((_e769 + 1023u) * _e650)).x;
            }
            let _e776 = phi_3274_;
            l2_[1u] = _e776;
        } else {
            let _e730 = (_e131 * 8u);
            let _e733 = KB.z2_[_e730];
            let _e744 = KB.z2_[(_e730 + 1u)];
            let _e749 = ((mat2x2<f32>(vec2<f32>(_e733.x, _e733.y), vec2<f32>(_e733.z, _e733.w)) * _e630) + _e744.xy);
            let _e760 = ((_e744.w + (f32(_e662) * 0.125f)) + (max(_e744.z, 0f) * 0.00024414063f));
            if (_e744.z < 0f) {
                phi_3272_ = -(_e760);
            } else {
                phi_3272_ = _e760;
            }
            let _e763 = phi_3272_;
            N0_ = vec4<f32>(_e749.x, _e749.y, _e763, (-0.75f - round((bitcast<f32>(_e148.w) * 255f))));
        }
    }
    phi_1421_ = yj;
    if yj {
        phi_1421_ = ((_e148.z & 2048u) != 0u);
    }
    let _e783 = phi_1421_;
    if _e783 {
        let _e784 = (_e131 * 8u);
        let _e788 = KB.z2_[(_e784 + 4u)];
        let _e799 = KB.z2_[(_e784 + 5u)];
        let _e802 = ((mat2x2<f32>(vec2<f32>(_e788.x, _e788.y), vec2<f32>(_e788.z, _e788.w)) * _e630) + _e799.xy);
        phi_3275_ = (1f + _e799.z);
        if ((_e148.z & 4096u) != 0u) {
            phi_3275_ = (-1f - f32(((_e148.z & 24576u) >> bitcast<u32>(13u))));
        }
        let _e813 = phi_3275_;
        O0_ = vec3<f32>(_e802.x, _e802.y, _e813);
    } else {
        O0_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e621) {
        let _e822 = j.ah;
        let _e824 = j.bh;
        let _e836 = JB.z2_[(_e133 + 3u)];
        D3_ = _e836.xy;
        V4_ = (_e630 + bitcast<vec2<f32>>(_e836.zw));
        phi_3293_ = vec4<f32>(((_e630.x * _e822) - 1f), ((_e630.y * _e824) - sign(_e824)), 0f, 1f);
    } else {
        let _e819 = j.n3_;
        phi_3293_ = vec4(_e819);
    }
    let _e842 = phi_3293_;
    unnamed.gl_Position = _e842;
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
    let _e22 = J0_;
    let _e23 = l2_;
    let _e24 = S0_;
    let _e25 = Y0_;
    let _e26 = N0_;
    let _e27 = O0_;
    let _e28 = D3_;
    let _e29 = V4_;
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
