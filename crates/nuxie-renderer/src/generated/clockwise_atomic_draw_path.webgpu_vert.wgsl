struct eh {
    j2_: array<vec4<u32>>,
}

struct dh {
    j2_: array<vec4<u32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Bf {
    j2_: array<vec2<u32>>,
}

struct Cf {
    j2_: array<vec4<f32>>,
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

@id(0) override bi: bool = true;
@id(2) override di: bool = true;
@id(1) override ci: bool = true;
@id(8) override ji: bool = true;

@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> AD: eh;
@group(0) @binding(2)
var<storage> LB: dh;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> XB_1: vec4<f32>;
var<private> S: vec4<f32>;
@group(0) @binding(3)
var<storage> XC: Bf;
var<private> F0_: f32;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: Cf;
var<private> R0_: vec4<f32>;
var<private> a1_: vec4<f32>;
var<private> F1_: vec3<f32>;
var<private> q3_: vec2<u32>;
var<private> F4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_2847_: f32;
    var phi_2785_: f32;
    var phi_2757_: i32;
    var phi_1687_: bool;
    var phi_2770_: i32;
    var phi_2762_: vec4<u32>;
    var phi_2769_: i32;
    var phi_2761_: vec4<u32>;
    var phi_2768_: i32;
    var phi_2766_: vec4<u32>;
    var phi_2765_: u32;
    var phi_2772_: vec2<i32>;
    var phi_2773_: vec4<u32>;
    var phi_2777_: f32;
    var phi_2791_: f32;
    var phi_2857_: f32;
    var phi_2856_: f32;
    var phi_2799_: f32;
    var phi_2792_: f32;
    var phi_2789_: f32;
    var phi_2803_: f32;
    var phi_2878_: f32;
    var phi_2869_: f32;
    var phi_2854_: f32;
    var phi_2802_: f32;
    var phi_2852_: f32;
    var phi_2888_: f32;
    var phi_2887_: f32;
    var phi_2889_: f32;
    var phi_2893_: f32;
    var phi_2915_: f32;
    var phi_2913_: f32;
    var phi_2931_: vec4<f32>;
    var phi_3051_: vec2<f32>;
    var phi_2930_: vec4<f32>;
    var phi_3057_: vec4<f32>;
    var phi_2935_: f32;
    var phi_2946_: f32;
    var phi_2938_: f32;
    var phi_2999_: f32;
    var phi_2973_: f32;
    var phi_2002_: bool;
    var phi_2978_: f32;
    var phi_2990_: vec2<f32>;
    var phi_2989_: vec2<f32>;
    var phi_3007_: vec4<f32>;
    var phi_3020_: vec2<f32>;
    var phi_3006_: vec4<f32>;
    var phi_3054_: vec4<f32>;
    var phi_3058_: bool;
    var phi_3052_: vec4<f32>;
    var phi_3046_: vec2<f32>;
    var phi_3022_: vec2<f32>;
    var phi_3090_: f32;
    var phi_3091_: u32;
    var phi_3092_: f32;
    var phi_3093_: f32;
    var phi_2512_: bool;
    var phi_3094_: vec4<f32>;
    var phi_3142_: vec4<f32>;
    var phi_3143_: vec4<f32>;
    var phi_3144_: f32;
    var phi_1385_: bool;
    var phi_3162_: vec4<f32>;

    let _e96 = gl_InstanceIndex_1;
    let _e97 = WB_1;
    let _e98 = XB_1;
    let _e100 = i32(_e97.x);
    let _e104 = bitcast<i32>(_e97.w);
    let _e106 = (_e104 >> bitcast<u32>(2i));
    let _e107 = (_e104 & 3i);
    let _e109 = min(_e100, (_e106 - 1i));
    let _e111 = ((_e96 * _e106) + _e109);
    let _e116 = textureLoad(TB, vec2<i32>((_e111 & 2047i), (_e111 >> bitcast<u32>(11i))), 0i);
    let _e123 = AD.j2_[(max((_e116.w & 65535u), 1u) - 1u)];
    let _e125 = bitcast<vec2<f32>>(_e123.xy);
    let _e127 = (_e123.z & 65535u);
    let _e129 = (_e127 * 4u);
    let _e132 = LB.j2_[_e129];
    let _e133 = bitcast<vec4<f32>>(_e132);
    let _e140 = mat2x2<f32>(vec2<f32>(_e133.x, _e133.y), vec2<f32>(_e133.z, _e133.w));
    let _e144 = LB.j2_[(_e129 + 1u)];
    let _e148 = bitcast<f32>(_e144.z);
    let _e150 = bitcast<f32>(_e144.w);
    let _e151 = (_e116.w & 8388608u);
    phi_2847_ = _e97.z;
    phi_2785_ = _e97.y;
    phi_2757_ = _e100;
    if (_e151 != 0u) {
        phi_2847_ = _e98.z;
        phi_2785_ = _e98.y;
        phi_2757_ = i32(_e98.x);
    }
    let _e158 = phi_2847_;
    let _e160 = phi_2785_;
    let _e162 = phi_2757_;
    phi_2768_ = _e111;
    phi_2766_ = _e116;
    phi_2765_ = _e116.w;
    if (_e162 != _e109) {
        let _e165 = ((_e111 + _e162) - _e109);
        let _e170 = textureLoad(TB, vec2<i32>((_e165 & 2047i), (_e165 >> bitcast<u32>(11i))), 0i);
        if ((_e170.w & 8454143u) != (_e116.w & 8454143u)) {
            let _e175 = (_e148 == 0f);
            phi_1687_ = _e175;
            if !(_e175) {
                phi_1687_ = (_e125.x != 0f);
            }
            let _e180 = phi_1687_;
            phi_2770_ = _e111;
            phi_2762_ = _e116;
            if _e180 {
                let _e181 = bitcast<i32>(_e123.w);
                let _e186 = textureLoad(TB, vec2<i32>((_e181 & 2047i), (_e181 >> bitcast<u32>(11i))), 0i);
                phi_2770_ = _e181;
                phi_2762_ = _e186;
            }
            let _e188 = phi_2770_;
            let _e190 = phi_2762_;
            phi_2769_ = _e188;
            phi_2761_ = _e190;
        } else {
            phi_2769_ = _e165;
            phi_2761_ = _e170;
        }
        let _e192 = phi_2769_;
        let _e194 = phi_2761_;
        phi_2768_ = _e192;
        phi_2766_ = _e194;
        phi_2765_ = ((_e194.w & 4286578687u) | _e151);
    }
    let _e199 = phi_2768_;
    let _e201 = phi_2766_;
    let _e203 = phi_2765_;
    let _e204 = (_e203 & 469762048u);
    let _e207 = ((_e204 == 67108864u) && (_e107 == 0i));
    if _e207 {
        let _e213 = f32((_e201.z & 65535u));
        let _e216 = f32((_e201.z >> bitcast<u32>(16i)));
        let _e222 = vec2<i32>(i32((-1f - _e213)), i32(((_e216 - _e213) + 1f)));
        phi_2772_ = _e222;
        if ((_e203 & 8388608u) != 0u) {
            phi_2772_ = -(_e222);
        }
        let _e227 = phi_2772_;
        let _e229 = (_e199 + _e227.x);
        let _e234 = textureLoad(TB, vec2<i32>((_e229 & 2047i), (_e229 >> bitcast<u32>(11i))), 0i);
        let _e236 = (_e199 + _e227.y);
        let _e241 = textureLoad(TB, vec2<i32>((_e236 & 2047i), (_e236 >> bitcast<u32>(11i))), 0i);
        phi_2773_ = _e241;
        if ((_e241.w & 8454143u) != (_e234.w & 8454143u)) {
            let _e247 = bitcast<i32>(_e123.w);
            let _e252 = textureLoad(TB, vec2<i32>((_e247 & 2047i), (_e247 >> bitcast<u32>(11i))), 0i);
            phi_2773_ = _e252;
        }
        let _e254 = phi_2773_;
        let _e257 = (f32(_e234.z) * 0.0000000014629181f);
        let _e260 = (f32(_e254.z) * 0.0000000014629181f);
        let _e261 = (_e260 - _e257);
        phi_2777_ = _e261;
        if (abs(_e261) > 3.1415927f) {
            phi_2777_ = (_e261 - (6.2831855f * sign(_e261)));
        }
        let _e268 = phi_2777_;
        let _e269 = (_e216 + -2f);
        let _e275 = clamp(round(((abs(_e268) * 0.31830987f) * _e269)), 1f, (_e216 + -3f));
        let _e276 = (_e269 - _e275);
        if (_e213 <= _e276) {
            phi_2857_ = _e160;
            if (_e213 == _e276) {
                phi_2857_ = -(_e160);
            }
            let _e293 = phi_2857_;
            phi_2856_ = _e293;
            phi_2799_ = -(((3.1415927f * sign(_e268)) - _e268));
            phi_2792_ = _e276;
            phi_2789_ = _e213;
        } else {
            let _e279 = (_e213 == (_e276 + 1f));
            if _e279 {
                phi_2791_ = 0f;
            } else {
                phi_2791_ = (_e213 - (_e276 + 2f));
            }
            let _e283 = phi_2791_;
            phi_2856_ = select(_e160, 0f, _e279);
            phi_2799_ = _e268;
            phi_2792_ = select(_e275, 0f, _e279);
            phi_2789_ = _e283;
        }
        let _e295 = phi_2856_;
        let _e297 = phi_2799_;
        let _e299 = phi_2792_;
        let _e301 = phi_2789_;
        if (_e301 == _e299) {
            phi_2803_ = _e260;
        } else {
            phi_2803_ = (_e257 + (_e297 * (_e301 / _e299)));
        }
        let _e307 = phi_2803_;
        phi_2878_ = _e257;
        phi_2869_ = _e297;
        phi_2854_ = _e295;
        phi_2802_ = _e307;
    } else {
        phi_2878_ = f32();
        phi_2869_ = f32();
        phi_2854_ = _e160;
        phi_2802_ = (f32(_e201.z) * 0.0000000014629181f);
    }
    let _e309 = phi_2878_;
    let _e311 = phi_2869_;
    let _e313 = phi_2854_;
    let _e315 = phi_2802_;
    let _e319 = vec2<f32>(sin(_e315), -(cos(_e315)));
    let _e321 = bitcast<vec2<f32>>(_e201.xy);
    phi_2852_ = _e150;
    if (_e150 != 0f) {
        phi_2852_ = max(_e150, (1f / length((_e140 * _e319))));
    }
    let _e328 = phi_2852_;
    if (_e148 != 0f) {
        let _e436 = (_e313 * sign(determinant(_e140)));
        let _e438 = ((_e203 & 1048576u) != 0u);
        phi_2935_ = _e436;
        if _e438 {
            phi_2935_ = min(_e436, 0f);
        }
        let _e441 = phi_2935_;
        phi_2946_ = _e441;
        if ((_e203 & 524288u) != 0u) {
            phi_2946_ = max(_e441, 0f);
        }
        let _e446 = phi_2946_;
        let _e447 = (_e328 != 0f);
        if _e447 {
            phi_2938_ = _e328;
        } else {
            let _e448 = (_e140 * _e319);
            phi_2938_ = (((abs(_e448.x) + abs(_e448.y)) * (1f / dot(_e448, _e448))) * 0.5f);
        }
        let _e459 = phi_2938_;
        let _e462 = ((_e459 > _e148) && (_e328 == 0f));
        phi_2999_ = 1f;
        if _e462 {
            phi_2999_ = (_e148 / _e459);
        }
        let _e465 = phi_2999_;
        let _e466 = select(_e148, _e459, _e462);
        let _e467 = (_e466 + _e459);
        let _e468 = (_e319 * _e467);
        let _e469 = (_e446 * _e467);
        let _e476 = (((vec2<f32>(_e469, -(_e469)) + vec2(_e466)) * (0.5f / _e459)) + vec2<f32>(0.5f, 0.5f));
        let _e479 = vec4<f32>(_e476.x, _e476.y, 0f, 0f);
        phi_3020_ = _e468;
        phi_3006_ = _e479;
        if (_e204 > 134217728u) {
            let _e485 = f32((_e201.z & 65535u));
            let _e486 = (_e485 * 0.000015259022f);
            let _e490 = sqrt(max((1f - (_e486 * _e486)), 0f));
            phi_2973_ = _e490;
            if (((_e203 & 4194304u) != 0u) == _e438) {
                phi_2973_ = -(_e490);
            }
            let _e494 = phi_2973_;
            let _e499 = (mat2x2<f32>(vec2<f32>(_e486, _e494), vec2<f32>(-(_e494), _e486)) * _e319);
            let _e500 = (_e140 * _e499);
            let _e508 = ((abs(_e500.x) + abs(_e500.y)) * (1f / dot(_e500, _e500)));
            let _e509 = (_e204 == 335544320u);
            phi_2002_ = _e509;
            if !(_e509) {
                phi_2002_ = ((_e204 == 268435456u) && (_e486 >= 0.25f));
            }
            let _e515 = phi_2002_;
            if _e515 {
                phi_2978_ = (_e466 * (1f / max(_e486, select(0.25f, 1f, ((_e203 & 33554432u) != 0u)))));
            } else {
                phi_2978_ = ((_e466 * _e486) + (_e508 * 0.5f));
            }
            let _e526 = phi_2978_;
            let _e528 = (_e526 + (_e508 * 0.5f));
            phi_2989_ = _e468;
            if ((_e203 & 2097152u) != 0u) {
                if (_e467 <= ((_e528 * _e486) + (_e459 * 0.125f))) {
                    phi_2990_ = (_e499 * (_e467 * (65535f / _e485)));
                } else {
                    let _e535 = (_e499 * _e528);
                    phi_2990_ = (vec2<f32>(dot(_e468, _e468), dot(_e535, _e535)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e468, _e535)));
                }
                let _e546 = phi_2990_;
                phi_2989_ = _e546;
            }
            let _e548 = phi_2989_;
            let _e553 = ((_e528 - dot((_e548 * abs(_e446)), _e499)) / _e508);
            if _e438 {
                phi_3007_ = vec4<f32>(_e479.x, _e553, _e479.z, _e479.w);
            } else {
                phi_3007_ = vec4<f32>(_e553, _e479.y, _e479.z, _e479.w);
            }
            let _e565 = phi_3007_;
            phi_3020_ = _e548;
            phi_3006_ = _e565;
        }
        let _e567 = phi_3020_;
        let _e569 = phi_3006_;
        let _e571 = (_e569.xy * _e465);
        let _e577 = vec4<f32>(_e571.x, _e569.y, _e569.z, _e569.w);
        let _e584 = vec4<f32>(_e577.x, max(_e571.y, 0.0001f), _e577.z, _e577.w);
        phi_3054_ = _e584;
        if _e447 {
            phi_3054_ = vec4<f32>((-2f - _e571.x), _e584.y, _e584.z, _e584.w);
        }
        let _e592 = phi_3054_;
        phi_3058_ = (_e107 != 0i);
        phi_3052_ = _e592;
        phi_3046_ = (_e140 * (_e567 * _e446));
        phi_3022_ = _e321;
    } else {
        let _e330 = vec4<f32>(_e158, -1f, 0f, 0f);
        if (_e328 != 0f) {
            let _e341 = vec4<f32>(_e330.x, -2f, _e330.z, _e330.w);
            let _e346 = vec4<f32>(_e341.x, _e341.y, 1000000f, _e341.w);
            phi_2931_ = vec4<f32>(_e346.x, _e346.y, _e346.z, _e158);
            if _e207 {
                phi_2888_ = _e311;
                phi_2887_ = _e309;
                if (_e311 < 0f) {
                    phi_2888_ = -(_e311);
                    phi_2887_ = (_e309 + _e311);
                }
                let _e356 = phi_2888_;
                let _e358 = phi_2887_;
                let _e360 = ((_e315 - _e358) + 1.5707964f);
                let _e366 = clamp(((_e360 - (floor((_e360 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e356);
                phi_2889_ = _e366;
                if (_e366 > (_e356 * 0.5f)) {
                    phi_2889_ = (_e356 - _e366);
                }
                let _e371 = phi_2889_;
                let _e378 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e371), cos(_e371)) * abs(_e313))) * 0.5f);
                if (abs((_e356 - 1.5707964f)) < 0.001f) {
                    phi_2915_ = 0f;
                    phi_2913_ = 0f;
                } else {
                    let _e382 = tan(_e356);
                    let _e387 = (sign((1.5707964f - _e356)) / max(abs(_e382), 0.000001f));
                    if (_e387 >= 0f) {
                        phi_2893_ = (_e378.y - ((1f - _e378.x) * _e382));
                    } else {
                        phi_2893_ = (_e378.y + (_e378.x * _e382));
                    }
                    let _e399 = phi_2893_;
                    phi_2915_ = _e399;
                    phi_2913_ = _e387;
                }
                let _e401 = phi_2915_;
                let _e403 = phi_2913_;
                phi_2931_ = vec4<f32>((max(_e378.x, 0f) + 0.25f), (-2f - _e378.y), _e403, _e401);
            }
            let _e411 = phi_2931_;
            phi_3051_ = (_e140 * (_e319 * (_e313 * _e328)));
            phi_2930_ = _e411;
        } else {
            phi_3051_ = (sign(((_e319 * _e313) * _naga_inverse_2x2_f32(_e140))) * 0.5f);
            phi_2930_ = _e330;
        }
        let _e416 = phi_3051_;
        let _e418 = phi_2930_;
        phi_3057_ = _e418;
        if (((_e203 & 8388608u) != 0u) != ((_e203 & 16777216u) != 0u)) {
            phi_3057_ = (_e418 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e426 = phi_3057_;
        phi_3058_ = (((_e203 & 2147483648u) != 0u) && (_e107 != 1i));
        phi_3052_ = _e426;
        phi_3046_ = _e416;
        phi_3022_ = select(_e321, _e125, vec2((_e107 == 2i)));
    }
    let _e597 = phi_3058_;
    let _e599 = phi_3052_;
    let _e601 = phi_3046_;
    let _e603 = phi_3022_;
    let _e606 = (((_e140 * _e603) + _e601) + bitcast<vec2<f32>>(_e144.xy));
    let _e609 = j.zh;
    let _e612 = select(_e599.xy, vec2<f32>(1f, -1f), vec2((_e609 != 0u)));
    let _e618 = vec4<f32>(_e612.x, _e599.y, _e599.z, _e599.w);
    S = vec4<f32>(_e618.x, _e612.y, _e618.z, _e618.w);
    let _e627 = XC.j2_[_e127];
    let _e629 = j.T4_;
    if (_e127 == 0u) {
        phi_3090_ = 0f;
    } else {
        phi_3090_ = unpack2x16float(((_e127 + 1023u) * _e629)).x;
    }
    let _e636 = phi_3090_;
    F0_ = _e636;
    if ((_e627.x & 512u) != 0u) {
        let _e640 = F0_;
        F0_ = -(_e640);
    }
    let _e642 = (_e627.x & 15u);
    if bi {
        let _e643 = (_e642 == 0u);
        if _e643 {
            phi_3091_ = _e627.y;
        } else {
            phi_3091_ = _e627.x;
        }
        let _e646 = phi_3091_;
        let _e648 = (_e646 >> bitcast<u32>(16i));
        if (_e648 == 0u) {
            phi_3092_ = 0f;
        } else {
            phi_3092_ = unpack2x16float(((_e648 + 1023u) * _e629)).x;
        }
        let _e655 = phi_3092_;
        phi_3093_ = _e655;
        if _e643 {
            phi_3093_ = -(_e655);
        }
        let _e658 = phi_3093_;
        l1_[0u] = _e658;
    }
    if di {
        Q0_ = f32(((_e627.x >> bitcast<u32>(4i)) & 15u));
    }
    if ci {
        let _e664 = (_e127 * 8u);
        let _e668 = JB.j2_[(_e664 + 2u)];
        let _e673 = vec2<f32>(_e668.x, _e668.y);
        let _e674 = vec2<f32>(_e668.z, _e668.w);
        let _e679 = JB.j2_[(_e664 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e684 = (abs(_e673) + abs(_e674));
                let _e686 = (_e684.x != 0f);
                phi_2512_ = _e686;
                if _e686 {
                    phi_2512_ = (_e684.y != 0f);
                }
                let _e690 = phi_2512_;
                if _e690 {
                    let _e694 = ((mat2x2<f32>(_e673, _e674) * _e606) + _e679.xy);
                    let _e695 = -(_e694);
                    let _e701 = (vec2<f32>(1f, 1f) / _e684).xyxy;
                    phi_3094_ = (((vec4<f32>(_e694.x, _e694.y, _e695.x, _e695.y) * _e701) + _e701) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3094_ = _e679.xyxy;
                    break;
                }
            }
        }
        let _e706 = phi_3094_;
        R0_ = _e706;
    }
    if (_e642 == 1u) {
        a1_ = unpack4x8unorm(_e627.y);
    } else {
        if (bi && (_e642 == 0u)) {
            let _e788 = (_e627.x >> bitcast<u32>(16i));
            if (_e788 == 0u) {
                phi_3144_ = 0f;
            } else {
                phi_3144_ = unpack2x16float(((_e788 + 1023u) * _e629)).x;
            }
            let _e795 = phi_3144_;
            l1_[1u] = _e795;
        } else {
            let _e710 = (_e127 * 8u);
            let _e713 = JB.j2_[_e710];
            let _e724 = JB.j2_[(_e710 + 1u)];
            let _e733 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e627.y));
            let _e735 = ((mat2x2<f32>(vec2<f32>(_e713.x, _e713.y), vec2<f32>(_e713.z, _e713.w)) * _e606) + _e724.xy);
            if (_e724.z > 0.9f) {
                phi_3142_ = vec4<f32>(_e733.x, _e733.y, 2f, _e733.w);
            } else {
                phi_3142_ = vec4<f32>(_e733.x, _e733.y, _e724.w, _e733.w);
            }
            let _e750 = phi_3142_;
            if (f32(_e642) == 2f) {
                let _e776 = vec4<f32>(_e735.x, _e750.y, _e750.z, _e750.w);
                phi_3143_ = vec4<f32>(_e776.x, 0f, _e776.z, _e776.w);
            } else {
                let _e758 = vec4<f32>(_e750.x, _e750.y, -(_e750.z), _e750.w);
                let _e764 = vec4<f32>(_e735.x, _e758.y, _e758.z, _e758.w);
                phi_3143_ = vec4<f32>(_e764.x, _e735.y, _e764.z, _e764.w);
            }
            let _e783 = phi_3143_;
            a1_ = _e783;
            let _e785 = a1_[3u];
            a1_[3u] = -(_e785);
        }
    }
    phi_1385_ = ji;
    if ji {
        phi_1385_ = ((_e627.x & 2048u) != 0u);
    }
    let _e802 = phi_1385_;
    if _e802 {
        let _e803 = (_e127 * 8u);
        let _e807 = JB.j2_[(_e803 + 4u)];
        let _e818 = JB.j2_[(_e803 + 5u)];
        let _e821 = ((mat2x2<f32>(vec2<f32>(_e807.x, _e807.y), vec2<f32>(_e807.z, _e807.w)) * _e606) + _e818.xy);
        F1_ = vec3<f32>(_e821.x, _e821.y, (1f + _e818.z));
    } else {
        F1_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e597) {
        let _e832 = j.Yf;
        let _e834 = j.Zf;
        let _e846 = LB.j2_[(_e129 + 3u)];
        q3_ = _e846.xy;
        F4_ = (_e606 + bitcast<vec2<f32>>(_e846.zw));
        phi_3162_ = vec4<f32>(((_e606.x * _e832) - 1f), ((_e606.y * _e834) - sign(_e834)), 0f, 1f);
    } else {
        let _e829 = j.c3_;
        phi_3162_ = vec4(_e829);
    }
    let _e852 = phi_3162_;
    unnamed.gl_Position = _e852;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) WB: vec4<f32>, @location(1) XB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    WB_1 = WB;
    XB_1 = XB;
    main_1();
    let _e21 = S;
    let _e22 = F0_;
    let _e23 = l1_;
    let _e24 = Q0_;
    let _e25 = R0_;
    let _e26 = a1_;
    let _e27 = F1_;
    let _e28 = q3_;
    let _e29 = F4_;
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
