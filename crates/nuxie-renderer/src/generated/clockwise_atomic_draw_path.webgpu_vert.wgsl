struct mi {
    v2_: array<vec4<u32>>,
}

struct li {
    v2_: array<vec4<u32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct og {
    v2_: array<vec2<u32>>,
}

struct pg {
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

@id(0) override fj: bool = true;
@id(2) override hj: bool = true;
@id(1) override gj: bool = true;
@id(4) override jj: bool = true;

@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> DD: mi;
@group(0) @binding(2)
var<storage> KB: li;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> ZB_1: vec4<f32>;
var<private> T: vec4<f32>;
@group(0) @binding(3)
var<storage> XC: og;
var<private> G0_: f32;
var<private> k2_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> W0_: vec4<f32>;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
var<private> A3_: vec2<u32>;
var<private> N4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    var phi_2892_: f32;
    var phi_2830_: f32;
    var phi_2802_: i32;
    var phi_1722_: bool;
    var phi_2815_: i32;
    var phi_2807_: vec4<u32>;
    var phi_2814_: i32;
    var phi_2806_: vec4<u32>;
    var phi_2813_: i32;
    var phi_2811_: vec4<u32>;
    var phi_2810_: u32;
    var phi_2817_: vec2<i32>;
    var phi_2818_: vec4<u32>;
    var phi_2822_: f32;
    var phi_2836_: f32;
    var phi_2902_: f32;
    var phi_2901_: f32;
    var phi_2844_: f32;
    var phi_2837_: f32;
    var phi_2834_: f32;
    var phi_2848_: f32;
    var phi_2923_: f32;
    var phi_2914_: f32;
    var phi_2899_: f32;
    var phi_2847_: f32;
    var phi_2897_: f32;
    var phi_2933_: f32;
    var phi_2932_: f32;
    var phi_2934_: f32;
    var phi_2938_: f32;
    var phi_2960_: f32;
    var phi_2958_: f32;
    var phi_2976_: vec4<f32>;
    var phi_3096_: vec2<f32>;
    var phi_2975_: vec4<f32>;
    var phi_3102_: vec4<f32>;
    var phi_2980_: f32;
    var phi_2991_: f32;
    var phi_2983_: f32;
    var phi_3044_: f32;
    var phi_3018_: f32;
    var phi_2037_: bool;
    var phi_3023_: f32;
    var phi_3035_: vec2<f32>;
    var phi_3034_: vec2<f32>;
    var phi_3052_: vec4<f32>;
    var phi_3065_: vec2<f32>;
    var phi_3051_: vec4<f32>;
    var phi_3099_: vec4<f32>;
    var phi_3103_: bool;
    var phi_3097_: vec4<f32>;
    var phi_3091_: vec2<f32>;
    var phi_3067_: vec2<f32>;
    var phi_3135_: f32;
    var phi_3136_: u32;
    var phi_3137_: f32;
    var phi_3138_: f32;
    var phi_2547_: bool;
    var phi_3139_: vec4<f32>;
    var phi_3187_: f32;
    var phi_3189_: f32;
    var phi_1397_: bool;
    var phi_3190_: f32;
    var phi_3208_: vec4<f32>;

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
    let _e127 = DD.v2_[(max((_e120.w & 65535u), 1u) - 1u)];
    let _e129 = bitcast<vec2<f32>>(_e127.xy);
    let _e131 = (_e127.z & 65535u);
    let _e133 = (_e131 * 4u);
    let _e136 = KB.v2_[_e133];
    let _e137 = bitcast<vec4<f32>>(_e136);
    let _e144 = mat2x2<f32>(vec2<f32>(_e137.x, _e137.y), vec2<f32>(_e137.z, _e137.w));
    let _e148 = KB.v2_[(_e133 + 1u)];
    let _e152 = bitcast<f32>(_e148.z);
    let _e154 = bitcast<f32>(_e148.w);
    let _e155 = (_e120.w & 8388608u);
    phi_2892_ = _e101.z;
    phi_2830_ = _e101.y;
    phi_2802_ = _e104;
    if (_e155 != 0u) {
        phi_2892_ = _e102.z;
        phi_2830_ = _e102.y;
        phi_2802_ = i32(_e102.x);
    }
    let _e162 = phi_2892_;
    let _e164 = phi_2830_;
    let _e166 = phi_2802_;
    phi_2813_ = _e115;
    phi_2811_ = _e120;
    phi_2810_ = _e120.w;
    if (_e166 != _e113) {
        let _e169 = ((_e115 + _e166) - _e113);
        let _e174 = textureLoad(VB, vec2<i32>((_e169 & 2047i), (_e169 >> bitcast<u32>(11i))), 0i);
        if ((_e174.w & 8454143u) != (_e120.w & 8454143u)) {
            let _e179 = (_e152 == 0f);
            phi_1722_ = _e179;
            if !(_e179) {
                phi_1722_ = (_e129.x != 0f);
            }
            let _e184 = phi_1722_;
            phi_2815_ = _e115;
            phi_2807_ = _e120;
            if _e184 {
                let _e185 = bitcast<i32>(_e127.w);
                let _e190 = textureLoad(VB, vec2<i32>((_e185 & 2047i), (_e185 >> bitcast<u32>(11i))), 0i);
                phi_2815_ = _e185;
                phi_2807_ = _e190;
            }
            let _e192 = phi_2815_;
            let _e194 = phi_2807_;
            phi_2814_ = _e192;
            phi_2806_ = _e194;
        } else {
            phi_2814_ = _e169;
            phi_2806_ = _e174;
        }
        let _e196 = phi_2814_;
        let _e198 = phi_2806_;
        phi_2813_ = _e196;
        phi_2811_ = _e198;
        phi_2810_ = ((_e198.w & 4286578687u) | _e155);
    }
    let _e203 = phi_2813_;
    let _e205 = phi_2811_;
    let _e207 = phi_2810_;
    let _e208 = (_e207 & 469762048u);
    let _e211 = ((_e208 == 67108864u) && (_e111 == 0i));
    if _e211 {
        let _e217 = f32((_e205.z & 65535u));
        let _e220 = f32((_e205.z >> bitcast<u32>(16i)));
        let _e226 = vec2<i32>(i32((-1f - _e217)), i32(((_e220 - _e217) + 1f)));
        phi_2817_ = _e226;
        if ((_e207 & 8388608u) != 0u) {
            phi_2817_ = -(_e226);
        }
        let _e231 = phi_2817_;
        let _e233 = (_e203 + _e231.x);
        let _e238 = textureLoad(VB, vec2<i32>((_e233 & 2047i), (_e233 >> bitcast<u32>(11i))), 0i);
        let _e240 = (_e203 + _e231.y);
        let _e245 = textureLoad(VB, vec2<i32>((_e240 & 2047i), (_e240 >> bitcast<u32>(11i))), 0i);
        phi_2818_ = _e245;
        if ((_e245.w & 8454143u) != (_e238.w & 8454143u)) {
            let _e251 = bitcast<i32>(_e127.w);
            let _e256 = textureLoad(VB, vec2<i32>((_e251 & 2047i), (_e251 >> bitcast<u32>(11i))), 0i);
            phi_2818_ = _e256;
        }
        let _e258 = phi_2818_;
        let _e261 = (f32(_e238.z) * 0.0000000014629181f);
        let _e264 = (f32(_e258.z) * 0.0000000014629181f);
        let _e265 = (_e264 - _e261);
        phi_2822_ = _e265;
        if (abs(_e265) > 3.1415927f) {
            phi_2822_ = (_e265 - (6.2831855f * sign(_e265)));
        }
        let _e272 = phi_2822_;
        let _e273 = (_e220 + -2f);
        let _e279 = clamp(round(((abs(_e272) * 0.31830987f) * _e273)), 1f, (_e220 + -3f));
        let _e280 = (_e273 - _e279);
        if (_e217 <= _e280) {
            phi_2902_ = _e164;
            if (_e217 == _e280) {
                phi_2902_ = -(_e164);
            }
            let _e297 = phi_2902_;
            phi_2901_ = _e297;
            phi_2844_ = -(((3.1415927f * sign(_e272)) - _e272));
            phi_2837_ = _e280;
            phi_2834_ = _e217;
        } else {
            let _e283 = (_e217 == (_e280 + 1f));
            if _e283 {
                phi_2836_ = 0f;
            } else {
                phi_2836_ = (_e217 - (_e280 + 2f));
            }
            let _e287 = phi_2836_;
            phi_2901_ = select(_e164, 0f, _e283);
            phi_2844_ = _e272;
            phi_2837_ = select(_e279, 0f, _e283);
            phi_2834_ = _e287;
        }
        let _e299 = phi_2901_;
        let _e301 = phi_2844_;
        let _e303 = phi_2837_;
        let _e305 = phi_2834_;
        if (_e305 == _e303) {
            phi_2848_ = _e264;
        } else {
            phi_2848_ = (_e261 + (_e301 * (_e305 / _e303)));
        }
        let _e311 = phi_2848_;
        phi_2923_ = _e261;
        phi_2914_ = _e301;
        phi_2899_ = _e299;
        phi_2847_ = _e311;
    } else {
        phi_2923_ = f32();
        phi_2914_ = f32();
        phi_2899_ = _e164;
        phi_2847_ = (f32(_e205.z) * 0.0000000014629181f);
    }
    let _e313 = phi_2923_;
    let _e315 = phi_2914_;
    let _e317 = phi_2899_;
    let _e319 = phi_2847_;
    let _e323 = vec2<f32>(sin(_e319), -(cos(_e319)));
    let _e325 = bitcast<vec2<f32>>(_e205.xy);
    phi_2897_ = _e154;
    if (_e154 != 0f) {
        phi_2897_ = max(_e154, (1f / length((_e144 * _e323))));
    }
    let _e332 = phi_2897_;
    if (_e152 != 0f) {
        let _e440 = (_e317 * sign(determinant(_e144)));
        let _e442 = ((_e207 & 1048576u) != 0u);
        phi_2980_ = _e440;
        if _e442 {
            phi_2980_ = min(_e440, 0f);
        }
        let _e445 = phi_2980_;
        phi_2991_ = _e445;
        if ((_e207 & 524288u) != 0u) {
            phi_2991_ = max(_e445, 0f);
        }
        let _e450 = phi_2991_;
        let _e451 = (_e332 != 0f);
        if _e451 {
            phi_2983_ = _e332;
        } else {
            let _e452 = (_e144 * _e323);
            phi_2983_ = (((abs(_e452.x) + abs(_e452.y)) * (1f / dot(_e452, _e452))) * 0.5f);
        }
        let _e463 = phi_2983_;
        let _e466 = ((_e463 > _e152) && (_e332 == 0f));
        phi_3044_ = 1f;
        if _e466 {
            phi_3044_ = (_e152 / _e463);
        }
        let _e469 = phi_3044_;
        let _e470 = select(_e152, _e463, _e466);
        let _e471 = (_e470 + _e463);
        let _e472 = (_e323 * _e471);
        let _e473 = (_e450 * _e471);
        let _e480 = (((vec2<f32>(_e473, -(_e473)) + vec2(_e470)) * (0.5f / _e463)) + vec2<f32>(0.5f, 0.5f));
        let _e483 = vec4<f32>(_e480.x, _e480.y, 0f, 0f);
        phi_3065_ = _e472;
        phi_3051_ = _e483;
        if (_e208 > 201326592u) {
            let _e489 = f32((_e205.z & 65535u));
            let _e490 = (_e489 * 0.000015259022f);
            let _e494 = sqrt(max((1f - (_e490 * _e490)), 0f));
            phi_3018_ = _e494;
            if (((_e207 & 4194304u) != 0u) == _e442) {
                phi_3018_ = -(_e494);
            }
            let _e498 = phi_3018_;
            let _e503 = (mat2x2<f32>(vec2<f32>(_e490, _e498), vec2<f32>(-(_e498), _e490)) * _e323);
            let _e504 = (_e144 * _e503);
            let _e512 = ((abs(_e504.x) + abs(_e504.y)) * (1f / dot(_e504, _e504)));
            let _e513 = (_e208 == 402653184u);
            phi_2037_ = _e513;
            if !(_e513) {
                phi_2037_ = ((_e208 == 335544320u) && (_e490 >= 0.25f));
            }
            let _e519 = phi_2037_;
            if _e519 {
                phi_3023_ = (_e470 * (1f / max(_e490, select(0.25f, 1f, ((_e207 & 33554432u) != 0u)))));
            } else {
                phi_3023_ = ((_e470 * _e490) + (_e512 * 0.5f));
            }
            let _e530 = phi_3023_;
            let _e532 = (_e530 + (_e512 * 0.5f));
            phi_3034_ = _e472;
            if ((_e207 & 2097152u) != 0u) {
                if (_e471 <= ((_e532 * _e490) + (_e463 * 0.125f))) {
                    phi_3035_ = (_e503 * (_e471 * (65535f / _e489)));
                } else {
                    let _e539 = (_e503 * _e532);
                    phi_3035_ = (vec2<f32>(dot(_e472, _e472), dot(_e539, _e539)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e472, _e539)));
                }
                let _e550 = phi_3035_;
                phi_3034_ = _e550;
            }
            let _e552 = phi_3034_;
            let _e557 = ((_e532 - dot((_e552 * abs(_e450)), _e503)) / _e512);
            if _e442 {
                phi_3052_ = vec4<f32>(_e483.x, _e557, _e483.z, _e483.w);
            } else {
                phi_3052_ = vec4<f32>(_e557, _e483.y, _e483.z, _e483.w);
            }
            let _e569 = phi_3052_;
            phi_3065_ = _e552;
            phi_3051_ = _e569;
        }
        let _e571 = phi_3065_;
        let _e573 = phi_3051_;
        let _e575 = (_e573.xy * _e469);
        let _e581 = vec4<f32>(_e575.x, _e573.y, _e573.z, _e573.w);
        let _e588 = vec4<f32>(_e581.x, max(_e575.y, 0.0001f), _e581.z, _e581.w);
        phi_3099_ = _e588;
        if _e451 {
            phi_3099_ = vec4<f32>((-2f - _e575.x), _e588.y, _e588.z, _e588.w);
        }
        let _e596 = phi_3099_;
        phi_3103_ = (_e111 != 0i);
        phi_3097_ = _e596;
        phi_3091_ = (_e144 * (_e571 * _e450));
        phi_3067_ = _e325;
    } else {
        let _e334 = vec4<f32>(_e162, -1f, 0f, 0f);
        if (_e332 != 0f) {
            let _e345 = vec4<f32>(_e334.x, -2f, _e334.z, _e334.w);
            let _e350 = vec4<f32>(_e345.x, _e345.y, 1000000f, _e345.w);
            phi_2976_ = vec4<f32>(_e350.x, _e350.y, _e350.z, _e162);
            if _e211 {
                phi_2933_ = _e315;
                phi_2932_ = _e313;
                if (_e315 < 0f) {
                    phi_2933_ = -(_e315);
                    phi_2932_ = (_e313 + _e315);
                }
                let _e360 = phi_2933_;
                let _e362 = phi_2932_;
                let _e364 = ((_e319 - _e362) + 1.5707964f);
                let _e370 = clamp(((_e364 - (floor((_e364 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e360);
                phi_2934_ = _e370;
                if (_e370 > (_e360 * 0.5f)) {
                    phi_2934_ = (_e360 - _e370);
                }
                let _e375 = phi_2934_;
                let _e382 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e375), cos(_e375)) * abs(_e317))) * 0.5f);
                if (abs((_e360 - 1.5707964f)) < 0.001f) {
                    phi_2960_ = 0f;
                    phi_2958_ = 0f;
                } else {
                    let _e386 = tan(_e360);
                    let _e391 = (sign((1.5707964f - _e360)) / max(abs(_e386), 0.000001f));
                    if (_e391 >= 0f) {
                        phi_2938_ = (_e382.y - ((1f - _e382.x) * _e386));
                    } else {
                        phi_2938_ = (_e382.y + (_e382.x * _e386));
                    }
                    let _e403 = phi_2938_;
                    phi_2960_ = _e403;
                    phi_2958_ = _e391;
                }
                let _e405 = phi_2960_;
                let _e407 = phi_2958_;
                phi_2976_ = vec4<f32>((max(_e382.x, 0f) + 0.25f), (-2f - _e382.y), _e407, _e405);
            }
            let _e415 = phi_2976_;
            phi_3096_ = (_e144 * (_e323 * (_e317 * _e332)));
            phi_2975_ = _e415;
        } else {
            phi_3096_ = (sign(((_e323 * _e317) * _naga_inverse_2x2_f32(_e144))) * 0.5f);
            phi_2975_ = _e334;
        }
        let _e420 = phi_3096_;
        let _e422 = phi_2975_;
        phi_3102_ = _e422;
        if (((_e207 & 8388608u) != 0u) != ((_e207 & 16777216u) != 0u)) {
            phi_3102_ = (_e422 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e430 = phi_3102_;
        phi_3103_ = (((_e207 & 2147483648u) != 0u) && (_e111 != 1i));
        phi_3097_ = _e430;
        phi_3091_ = _e420;
        phi_3067_ = select(_e325, _e129, vec2((_e111 == 2i)));
    }
    let _e601 = phi_3103_;
    let _e603 = phi_3097_;
    let _e605 = phi_3091_;
    let _e607 = phi_3067_;
    let _e610 = (((_e144 * _e607) + _e605) + bitcast<vec2<f32>>(_e148.xy));
    let _e613 = j.Di;
    let _e616 = select(_e603.xy, vec2<f32>(1f, -1f), vec2((_e613 != 0u)));
    let _e622 = vec4<f32>(_e616.x, _e603.y, _e603.z, _e603.w);
    T = vec4<f32>(_e622.x, _e616.y, _e622.z, _e622.w);
    let _e631 = XC.v2_[_e131];
    let _e633 = j.z6_;
    if (_e131 == 0u) {
        phi_3135_ = 0f;
    } else {
        phi_3135_ = unpack2x16float(((_e131 + 1023u) * _e633)).x;
    }
    let _e640 = phi_3135_;
    G0_ = _e640;
    if ((_e631.x & 512u) != 0u) {
        let _e644 = G0_;
        G0_ = -(_e644);
    }
    let _e646 = (_e631.x & 15u);
    if fj {
        let _e647 = (_e646 == 0u);
        if _e647 {
            phi_3136_ = _e631.y;
        } else {
            phi_3136_ = _e631.x;
        }
        let _e650 = phi_3136_;
        let _e652 = (_e650 >> bitcast<u32>(16i));
        if (_e652 == 0u) {
            phi_3137_ = 0f;
        } else {
            phi_3137_ = unpack2x16float(((_e652 + 1023u) * _e633)).x;
        }
        let _e659 = phi_3137_;
        phi_3138_ = _e659;
        if _e647 {
            phi_3138_ = -(_e659);
        }
        let _e662 = phi_3138_;
        k2_[0u] = _e662;
    }
    if hj {
        Q0_ = f32(((_e631.x >> bitcast<u32>(4i)) & 15u));
    }
    if gj {
        let _e668 = (_e131 * 8u);
        let _e672 = JB.v2_[(_e668 + 2u)];
        let _e677 = vec2<f32>(_e672.x, _e672.y);
        let _e678 = vec2<f32>(_e672.z, _e672.w);
        let _e683 = JB.v2_[(_e668 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e688 = (abs(_e677) + abs(_e678));
                let _e690 = (_e688.x != 0f);
                phi_2547_ = _e690;
                if _e690 {
                    phi_2547_ = (_e688.y != 0f);
                }
                let _e694 = phi_2547_;
                if _e694 {
                    let _e698 = ((mat2x2<f32>(_e677, _e678) * _e610) + _e683.xy);
                    let _e699 = -(_e698);
                    let _e705 = (vec2<f32>(1f, 1f) / _e688).xyxy;
                    phi_3139_ = (((vec4<f32>(_e698.x, _e698.y, _e699.x, _e699.y) * _e705) + _e705) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3139_ = _e683.xyxy;
                    break;
                }
            }
        }
        let _e710 = phi_3139_;
        W0_ = _e710;
    }
    if (_e646 == 1u) {
        P0_ = unpack4x8unorm(_e631.y);
    } else {
        if (fj && (_e646 == 0u)) {
            let _e753 = (_e631.x >> bitcast<u32>(16i));
            if (_e753 == 0u) {
                phi_3189_ = 0f;
            } else {
                phi_3189_ = unpack2x16float(((_e753 + 1023u) * _e633)).x;
            }
            let _e760 = phi_3189_;
            k2_[1u] = _e760;
        } else {
            let _e714 = (_e131 * 8u);
            let _e717 = JB.v2_[_e714];
            let _e728 = JB.v2_[(_e714 + 1u)];
            let _e733 = ((mat2x2<f32>(vec2<f32>(_e717.x, _e717.y), vec2<f32>(_e717.z, _e717.w)) * _e610) + _e728.xy);
            let _e744 = ((_e728.w + (f32(_e646) * 0.125f)) + (max(_e728.z, 0f) * 0.00024414063f));
            if (_e728.z < 0f) {
                phi_3187_ = -(_e744);
            } else {
                phi_3187_ = _e744;
            }
            let _e747 = phi_3187_;
            P0_ = vec4<f32>(_e733.x, _e733.y, _e747, (-0.75f - round((bitcast<f32>(_e631.y) * 255f))));
        }
    }
    phi_1397_ = jj;
    if jj {
        phi_1397_ = ((_e631.x & 2048u) != 0u);
    }
    let _e767 = phi_1397_;
    if _e767 {
        let _e768 = (_e131 * 8u);
        let _e772 = JB.v2_[(_e768 + 4u)];
        let _e783 = JB.v2_[(_e768 + 5u)];
        let _e786 = ((mat2x2<f32>(vec2<f32>(_e772.x, _e772.y), vec2<f32>(_e772.z, _e772.w)) * _e610) + _e783.xy);
        phi_3190_ = (1f + _e783.z);
        if ((_e631.x & 4096u) != 0u) {
            phi_3190_ = (-1f - f32(((_e631.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e797 = phi_3190_;
        V0_ = vec3<f32>(_e786.x, _e786.y, _e797);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e601) {
        let _e806 = j.Mg;
        let _e808 = j.Ng;
        let _e820 = KB.v2_[(_e133 + 3u)];
        A3_ = _e820.xy;
        N4_ = (_e610 + bitcast<vec2<f32>>(_e820.zw));
        phi_3208_ = vec4<f32>(((_e610.x * _e806) - 1f), ((_e610.y * _e808) - sign(_e808)), 0f, 1f);
    } else {
        let _e803 = j.i3_;
        phi_3208_ = vec4(_e803);
    }
    let _e826 = phi_3208_;
    unnamed.gl_Position = _e826;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) YB: vec4<f32>, @location(1) ZB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    YB_1 = YB;
    ZB_1 = ZB;
    main_1();
    let _e21 = T;
    let _e22 = G0_;
    let _e23 = k2_;
    let _e24 = Q0_;
    let _e25 = W0_;
    let _e26 = P0_;
    let _e27 = V0_;
    let _e28 = A3_;
    let _e29 = N4_;
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
