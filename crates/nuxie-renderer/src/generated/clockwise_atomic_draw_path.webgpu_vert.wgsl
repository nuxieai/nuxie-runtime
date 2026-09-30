struct Ig {
    g2_: array<vec4<u32>>,
}

struct Hg {
    g2_: array<vec4<u32>>,
}

struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
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

@id(0) override Hh: bool = true;
@id(2) override Jh: bool = true;
@id(1) override Ih: bool = true;
@id(8) override Ph: bool = true;

@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> HD: Ig;
@group(0) @binding(2)
var<storage> OB: Hg;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
var<private> O: vec4<f32>;
@group(0) @binding(3)
var<storage> CD: kf;
var<private> D0_: f32;
var<private> Y1_: vec2<f32>;
var<private> g1_: f32;
@group(0) @binding(4)
var<storage> PB: lf;
var<private> O0_: vec4<f32>;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> k3_: vec2<u32>;
var<private> v4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2865_: f32;
    var phi_2803_: f32;
    var phi_2775_: i32;
    var phi_1695_: bool;
    var phi_2788_: i32;
    var phi_2780_: vec4<u32>;
    var phi_2787_: i32;
    var phi_2779_: vec4<u32>;
    var phi_2786_: i32;
    var phi_2784_: vec4<u32>;
    var phi_2783_: u32;
    var phi_2790_: vec2<i32>;
    var phi_2791_: vec4<u32>;
    var phi_2795_: f32;
    var phi_2809_: f32;
    var phi_2875_: f32;
    var phi_2874_: f32;
    var phi_2817_: f32;
    var phi_2810_: f32;
    var phi_2807_: f32;
    var phi_2821_: f32;
    var phi_2896_: f32;
    var phi_2887_: f32;
    var phi_2872_: f32;
    var phi_2820_: f32;
    var phi_2870_: f32;
    var phi_2906_: f32;
    var phi_2905_: f32;
    var phi_2907_: f32;
    var phi_2911_: f32;
    var phi_2933_: f32;
    var phi_2931_: f32;
    var phi_2949_: vec4<f32>;
    var phi_3103_: vec2<f32>;
    var phi_2948_: vec4<f32>;
    var phi_3109_: vec4<f32>;
    var phi_2953_: f32;
    var phi_2964_: f32;
    var phi_2956_: f32;
    var phi_3045_: f32;
    var phi_3002_: i32;
    var phi_3011_: f32;
    var phi_2034_: bool;
    var phi_3018_: f32;
    var phi_3034_: vec2<f32>;
    var phi_3033_: vec2<f32>;
    var phi_3055_: vec4<f32>;
    var phi_3070_: vec2<f32>;
    var phi_3054_: vec4<f32>;
    var phi_3106_: vec4<f32>;
    var phi_3110_: bool;
    var phi_3104_: vec4<f32>;
    var phi_3098_: vec2<f32>;
    var phi_3072_: vec2<f32>;
    var phi_3144_: f32;
    var phi_3145_: u32;
    var phi_3146_: f32;
    var phi_3147_: f32;
    var phi_2530_: bool;
    var phi_3148_: vec4<f32>;
    var phi_3198_: vec4<f32>;
    var phi_3199_: vec4<f32>;
    var phi_3200_: f32;
    var phi_1391_: bool;
    var phi_3218_: vec4<f32>;

    let _e95 = gl_InstanceIndex_1;
    let _e96 = VB_1;
    let _e97 = WB_1;
    let _e99 = i32(_e96.x);
    let _e103 = bitcast<i32>(_e96.w);
    let _e105 = (_e103 >> bitcast<u32>(2i));
    let _e106 = (_e103 & 3i);
    let _e108 = min(_e99, (_e105 - 1i));
    let _e110 = ((_e95 * _e105) + _e108);
    let _e115 = textureLoad(JC, vec2<i32>((_e110 & 2047i), (_e110 >> bitcast<u32>(11i))), 0i);
    let _e122 = HD.g2_[(max((_e115.w & 65535u), 1u) - 1u)];
    let _e124 = bitcast<vec2<f32>>(_e122.xy);
    let _e126 = (_e122.z & 65535u);
    let _e128 = (_e126 * 4u);
    let _e131 = OB.g2_[_e128];
    let _e132 = bitcast<vec4<f32>>(_e131);
    let _e139 = mat2x2<f32>(vec2<f32>(_e132.x, _e132.y), vec2<f32>(_e132.z, _e132.w));
    let _e143 = OB.g2_[(_e128 + 1u)];
    let _e147 = bitcast<f32>(_e143.z);
    let _e149 = bitcast<f32>(_e143.w);
    let _e150 = (_e115.w & 8388608u);
    phi_2865_ = _e96.z;
    phi_2803_ = _e96.y;
    phi_2775_ = _e99;
    if (_e150 != 0u) {
        phi_2865_ = _e97.z;
        phi_2803_ = _e97.y;
        phi_2775_ = i32(_e97.x);
    }
    let _e157 = phi_2865_;
    let _e159 = phi_2803_;
    let _e161 = phi_2775_;
    phi_2786_ = _e110;
    phi_2784_ = _e115;
    phi_2783_ = _e115.w;
    if (_e161 != _e108) {
        let _e164 = ((_e110 + _e161) - _e108);
        let _e169 = textureLoad(JC, vec2<i32>((_e164 & 2047i), (_e164 >> bitcast<u32>(11i))), 0i);
        if ((_e169.w & 8454143u) != (_e115.w & 8454143u)) {
            let _e174 = (_e147 == 0f);
            phi_1695_ = _e174;
            if !(_e174) {
                phi_1695_ = (_e124.x != 0f);
            }
            let _e179 = phi_1695_;
            phi_2788_ = _e110;
            phi_2780_ = _e115;
            if _e179 {
                let _e180 = bitcast<i32>(_e122.w);
                let _e185 = textureLoad(JC, vec2<i32>((_e180 & 2047i), (_e180 >> bitcast<u32>(11i))), 0i);
                phi_2788_ = _e180;
                phi_2780_ = _e185;
            }
            let _e187 = phi_2788_;
            let _e189 = phi_2780_;
            phi_2787_ = _e187;
            phi_2779_ = _e189;
        } else {
            phi_2787_ = _e164;
            phi_2779_ = _e169;
        }
        let _e191 = phi_2787_;
        let _e193 = phi_2779_;
        phi_2786_ = _e191;
        phi_2784_ = _e193;
        phi_2783_ = ((_e193.w & 4286578687u) | _e150);
    }
    let _e198 = phi_2786_;
    let _e200 = phi_2784_;
    let _e202 = phi_2783_;
    let _e203 = (_e202 & 469762048u);
    let _e206 = ((_e203 == 67108864u) && (_e106 == 0i));
    if _e206 {
        let _e211 = f32((_e200.z & 65535u));
        let _e214 = f32((_e200.z >> bitcast<u32>(16i)));
        let _e220 = vec2<i32>(i32((-1f - _e211)), i32(((_e214 - _e211) + 1f)));
        phi_2790_ = _e220;
        if ((_e202 & 8388608u) != 0u) {
            phi_2790_ = -(_e220);
        }
        let _e225 = phi_2790_;
        let _e227 = (_e198 + _e225.x);
        let _e232 = textureLoad(JC, vec2<i32>((_e227 & 2047i), (_e227 >> bitcast<u32>(11i))), 0i);
        let _e234 = (_e198 + _e225.y);
        let _e239 = textureLoad(JC, vec2<i32>((_e234 & 2047i), (_e234 >> bitcast<u32>(11i))), 0i);
        phi_2791_ = _e239;
        if ((_e239.w & 8454143u) != (_e232.w & 8454143u)) {
            let _e245 = bitcast<i32>(_e122.w);
            let _e250 = textureLoad(JC, vec2<i32>((_e245 & 2047i), (_e245 >> bitcast<u32>(11i))), 0i);
            phi_2791_ = _e250;
        }
        let _e252 = phi_2791_;
        let _e254 = bitcast<f32>(_e232.z);
        let _e256 = bitcast<f32>(_e252.z);
        let _e257 = (_e256 - _e254);
        phi_2795_ = _e257;
        if (abs(_e257) > 3.1415927f) {
            phi_2795_ = (_e257 - (6.2831855f * sign(_e257)));
        }
        let _e264 = phi_2795_;
        let _e265 = (_e214 + -2f);
        let _e271 = clamp(round(((abs(_e264) * 0.31830987f) * _e265)), 1f, (_e214 + -3f));
        let _e272 = (_e265 - _e271);
        if (_e211 <= _e272) {
            phi_2875_ = _e159;
            if (_e211 == _e272) {
                phi_2875_ = -(_e159);
            }
            let _e289 = phi_2875_;
            phi_2874_ = _e289;
            phi_2817_ = -(((3.1415927f * sign(_e264)) - _e264));
            phi_2810_ = _e272;
            phi_2807_ = _e211;
        } else {
            let _e275 = (_e211 == (_e272 + 1f));
            if _e275 {
                phi_2809_ = 0f;
            } else {
                phi_2809_ = (_e211 - (_e272 + 2f));
            }
            let _e279 = phi_2809_;
            phi_2874_ = select(_e159, 0f, _e275);
            phi_2817_ = _e264;
            phi_2810_ = select(_e271, 0f, _e275);
            phi_2807_ = _e279;
        }
        let _e291 = phi_2874_;
        let _e293 = phi_2817_;
        let _e295 = phi_2810_;
        let _e297 = phi_2807_;
        if (_e297 == _e295) {
            phi_2821_ = _e256;
        } else {
            phi_2821_ = (_e254 + (_e293 * (_e297 / _e295)));
        }
        let _e303 = phi_2821_;
        phi_2896_ = _e254;
        phi_2887_ = _e293;
        phi_2872_ = _e291;
        phi_2820_ = _e303;
    } else {
        phi_2896_ = f32();
        phi_2887_ = f32();
        phi_2872_ = _e159;
        phi_2820_ = bitcast<f32>(_e200.z);
    }
    let _e305 = phi_2896_;
    let _e307 = phi_2887_;
    let _e309 = phi_2872_;
    let _e311 = phi_2820_;
    let _e315 = vec2<f32>(sin(_e311), -(cos(_e311)));
    let _e317 = bitcast<vec2<f32>>(_e200.xy);
    phi_2870_ = _e149;
    if (_e149 != 0f) {
        phi_2870_ = max(_e149, (1f / length((_e139 * _e315))));
    }
    let _e324 = phi_2870_;
    if (_e147 != 0f) {
        let _e432 = (_e309 * sign(determinant(_e139)));
        let _e434 = ((_e202 & 1048576u) != 0u);
        phi_2953_ = _e432;
        if _e434 {
            phi_2953_ = min(_e432, 0f);
        }
        let _e437 = phi_2953_;
        phi_2964_ = _e437;
        if ((_e202 & 524288u) != 0u) {
            phi_2964_ = max(_e437, 0f);
        }
        let _e442 = phi_2964_;
        let _e443 = (_e324 != 0f);
        if _e443 {
            phi_2956_ = _e324;
        } else {
            let _e444 = (_e139 * _e315);
            phi_2956_ = (((abs(_e444.x) + abs(_e444.y)) * (1f / dot(_e444, _e444))) * 0.5f);
        }
        let _e455 = phi_2956_;
        let _e458 = ((_e455 > _e147) && (_e324 == 0f));
        phi_3045_ = 1f;
        if _e458 {
            phi_3045_ = (_e147 / _e455);
        }
        let _e461 = phi_3045_;
        let _e462 = select(_e147, _e455, _e458);
        let _e463 = (_e462 + _e455);
        let _e464 = (_e315 * _e463);
        let _e465 = (_e442 * _e463);
        let _e472 = (((vec2<f32>(_e465, -(_e465)) + vec2(_e462)) * (0.5f / _e455)) + vec2<f32>(0.5f, 0.5f));
        let _e475 = vec4<f32>(_e472.x, _e472.y, 0f, 0f);
        phi_3070_ = _e464;
        phi_3054_ = _e475;
        if (_e203 > 134217728u) {
            let _e477 = (_e202 & 4194304u);
            let _e479 = select(2i, -2i, (_e477 == 0u));
            phi_3002_ = _e479;
            if ((_e202 & 8388608u) != 0u) {
                phi_3002_ = -(_e479);
            }
            let _e484 = phi_3002_;
            let _e485 = (_e198 + _e484);
            let _e490 = textureLoad(JC, vec2<i32>((_e485 & 2047i), (_e485 >> bitcast<u32>(11i))), 0i);
            let _e494 = abs((bitcast<f32>(_e490.z) - _e311));
            phi_3011_ = _e494;
            if (_e494 > 3.1415927f) {
                phi_3011_ = (6.2831855f - _e494);
            }
            let _e498 = phi_3011_;
            let _e503 = ((_e498 * select(0.5f, -0.5f, ((_e477 != 0u) == _e434))) + _e311);
            let _e507 = vec2<f32>(sin(_e503), -(cos(_e503)));
            let _e508 = (_e139 * _e507);
            let _e516 = ((abs(_e508.x) + abs(_e508.y)) * (1f / dot(_e508, _e508)));
            let _e518 = cos((_e498 * 0.5f));
            let _e519 = (_e203 == 335544320u);
            phi_2034_ = _e519;
            if !(_e519) {
                phi_2034_ = ((_e203 == 268435456u) && (_e518 >= 0.25f));
            }
            let _e525 = phi_2034_;
            if _e525 {
                phi_3018_ = (_e462 * (1f / max(_e518, select(0.25f, 1f, ((_e202 & 33554432u) != 0u)))));
            } else {
                phi_3018_ = ((_e462 * _e518) + (_e516 * 0.5f));
            }
            let _e536 = phi_3018_;
            let _e538 = (_e536 + (_e516 * 0.5f));
            phi_3033_ = _e464;
            if ((_e202 & 2097152u) != 0u) {
                if (_e463 <= ((_e538 * _e518) + (_e455 * 0.125f))) {
                    phi_3034_ = (_e507 * (_e463 * (1f / _e518)));
                } else {
                    let _e545 = (_e507 * _e538);
                    phi_3034_ = (vec2<f32>(dot(_e464, _e464), dot(_e545, _e545)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e464, _e545)));
                }
                let _e556 = phi_3034_;
                phi_3033_ = _e556;
            }
            let _e558 = phi_3033_;
            let _e563 = ((_e538 - dot((_e558 * abs(_e442)), _e507)) / _e516);
            if _e434 {
                phi_3055_ = vec4<f32>(_e475.x, _e563, _e475.z, _e475.w);
            } else {
                phi_3055_ = vec4<f32>(_e563, _e475.y, _e475.z, _e475.w);
            }
            let _e575 = phi_3055_;
            phi_3070_ = _e558;
            phi_3054_ = _e575;
        }
        let _e577 = phi_3070_;
        let _e579 = phi_3054_;
        let _e581 = (_e579.xy * _e461);
        let _e587 = vec4<f32>(_e581.x, _e579.y, _e579.z, _e579.w);
        let _e594 = vec4<f32>(_e587.x, max(_e581.y, 0.0001f), _e587.z, _e587.w);
        phi_3106_ = _e594;
        if _e443 {
            phi_3106_ = vec4<f32>((-2f - _e581.x), _e594.y, _e594.z, _e594.w);
        }
        let _e602 = phi_3106_;
        phi_3110_ = (_e106 != 0i);
        phi_3104_ = _e602;
        phi_3098_ = (_e139 * (_e577 * _e442));
        phi_3072_ = _e317;
    } else {
        let _e326 = vec4<f32>(_e157, -1f, 0f, 0f);
        if (_e324 != 0f) {
            let _e337 = vec4<f32>(_e326.x, -2f, _e326.z, _e326.w);
            let _e342 = vec4<f32>(_e337.x, _e337.y, 1000000f, _e337.w);
            phi_2949_ = vec4<f32>(_e342.x, _e342.y, _e342.z, _e157);
            if _e206 {
                phi_2906_ = _e307;
                phi_2905_ = _e305;
                if (_e307 < 0f) {
                    phi_2906_ = -(_e307);
                    phi_2905_ = (_e305 + _e307);
                }
                let _e352 = phi_2906_;
                let _e354 = phi_2905_;
                let _e356 = ((_e311 - _e354) + 1.5707964f);
                let _e362 = clamp(((_e356 - (floor((_e356 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e352);
                phi_2907_ = _e362;
                if (_e362 > (_e352 * 0.5f)) {
                    phi_2907_ = (_e352 - _e362);
                }
                let _e367 = phi_2907_;
                let _e374 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e367), cos(_e367)) * abs(_e309))) * 0.5f);
                if (abs((_e352 - 1.5707964f)) < 0.001f) {
                    phi_2933_ = 0f;
                    phi_2931_ = 0f;
                } else {
                    let _e378 = tan(_e352);
                    let _e383 = (sign((1.5707964f - _e352)) / max(abs(_e378), 0.000001f));
                    if (_e383 >= 0f) {
                        phi_2911_ = (_e374.y - ((1f - _e374.x) * _e378));
                    } else {
                        phi_2911_ = (_e374.y + (_e374.x * _e378));
                    }
                    let _e395 = phi_2911_;
                    phi_2933_ = _e395;
                    phi_2931_ = _e383;
                }
                let _e397 = phi_2933_;
                let _e399 = phi_2931_;
                phi_2949_ = vec4<f32>((max(_e374.x, 0f) + 0.25f), (-2f - _e374.y), _e399, _e397);
            }
            let _e407 = phi_2949_;
            phi_3103_ = (_e139 * (_e315 * (_e309 * _e324)));
            phi_2948_ = _e407;
        } else {
            phi_3103_ = (sign(((_e315 * _e309) * _naga_inverse_2x2_f32(_e139))) * 0.5f);
            phi_2948_ = _e326;
        }
        let _e412 = phi_3103_;
        let _e414 = phi_2948_;
        phi_3109_ = _e414;
        if (((_e202 & 8388608u) != 0u) != ((_e202 & 16777216u) != 0u)) {
            phi_3109_ = (_e414 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e422 = phi_3109_;
        phi_3110_ = (((_e202 & 2147483648u) != 0u) && (_e106 != 1i));
        phi_3104_ = _e422;
        phi_3098_ = _e412;
        phi_3072_ = select(_e317, _e124, vec2((_e106 == 2i)));
    }
    let _e607 = phi_3110_;
    let _e609 = phi_3104_;
    let _e611 = phi_3098_;
    let _e613 = phi_3072_;
    let _e616 = (((_e139 * _e613) + _e611) + bitcast<vec2<f32>>(_e143.xy));
    let _e619 = j.eh;
    let _e622 = select(_e609.xy, vec2<f32>(1f, -1f), vec2((_e619 != 0u)));
    let _e628 = vec4<f32>(_e622.x, _e609.y, _e609.z, _e609.w);
    O = vec4<f32>(_e628.x, _e622.y, _e628.z, _e628.w);
    let _e637 = CD.g2_[_e126];
    let _e639 = j.c6_;
    if (_e126 == 0u) {
        phi_3144_ = 0f;
    } else {
        phi_3144_ = unpack2x16float(((_e126 + 1023u) * _e639)).x;
    }
    let _e646 = phi_3144_;
    D0_ = _e646;
    if ((_e637.x & 512u) != 0u) {
        let _e650 = D0_;
        D0_ = -(_e650);
    }
    let _e652 = (_e637.x & 15u);
    if Hh {
        let _e653 = (_e652 == 0u);
        if _e653 {
            phi_3145_ = _e637.y;
        } else {
            phi_3145_ = _e637.x;
        }
        let _e656 = phi_3145_;
        let _e658 = (_e656 >> bitcast<u32>(16i));
        if (_e658 == 0u) {
            phi_3146_ = 0f;
        } else {
            phi_3146_ = unpack2x16float(((_e658 + 1023u) * _e639)).x;
        }
        let _e665 = phi_3146_;
        phi_3147_ = _e665;
        if _e653 {
            phi_3147_ = -(_e665);
        }
        let _e668 = phi_3147_;
        Y1_[0u] = _e668;
    }
    if Jh {
        g1_ = f32(((_e637.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ih {
        let _e674 = (_e126 * 8u);
        let _e678 = PB.g2_[(_e674 + 2u)];
        let _e683 = vec2<f32>(_e678.x, _e678.y);
        let _e684 = vec2<f32>(_e678.z, _e678.w);
        let _e689 = PB.g2_[(_e674 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e694 = (abs(_e683) + abs(_e684));
                let _e696 = (_e694.x != 0f);
                phi_2530_ = _e696;
                if _e696 {
                    phi_2530_ = (_e694.y != 0f);
                }
                let _e700 = phi_2530_;
                if _e700 {
                    let _e704 = ((mat2x2<f32>(_e683, _e684) * _e616) + _e689.xy);
                    let _e705 = -(_e704);
                    let _e711 = (vec2<f32>(1f, 1f) / _e694).xyxy;
                    phi_3148_ = (((vec4<f32>(_e704.x, _e704.y, _e705.x, _e705.y) * _e711) + _e711) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3148_ = _e689.xyxy;
                    break;
                }
            }
        }
        let _e716 = phi_3148_;
        O0_ = _e716;
    }
    if (_e652 == 1u) {
        X1_ = unpack4x8unorm(_e637.y);
    } else {
        if (Hh && (_e652 == 0u)) {
            let _e798 = (_e637.x >> bitcast<u32>(16i));
            if (_e798 == 0u) {
                phi_3200_ = 0f;
            } else {
                phi_3200_ = unpack2x16float(((_e798 + 1023u) * _e639)).x;
            }
            let _e805 = phi_3200_;
            Y1_[1u] = _e805;
        } else {
            let _e720 = (_e126 * 8u);
            let _e723 = PB.g2_[_e720];
            let _e734 = PB.g2_[(_e720 + 1u)];
            let _e743 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e637.y));
            let _e745 = ((mat2x2<f32>(vec2<f32>(_e723.x, _e723.y), vec2<f32>(_e723.z, _e723.w)) * _e616) + _e734.xy);
            if (_e734.z > 0.9f) {
                phi_3198_ = vec4<f32>(_e743.x, _e743.y, 2f, _e743.w);
            } else {
                phi_3198_ = vec4<f32>(_e743.x, _e743.y, _e734.w, _e743.w);
            }
            let _e760 = phi_3198_;
            if (f32(_e652) == 2f) {
                let _e786 = vec4<f32>(_e745.x, _e760.y, _e760.z, _e760.w);
                phi_3199_ = vec4<f32>(_e786.x, 0f, _e786.z, _e786.w);
            } else {
                let _e768 = vec4<f32>(_e760.x, _e760.y, -(_e760.z), _e760.w);
                let _e774 = vec4<f32>(_e745.x, _e768.y, _e768.z, _e768.w);
                phi_3199_ = vec4<f32>(_e774.x, _e745.y, _e774.z, _e774.w);
            }
            let _e793 = phi_3199_;
            X1_ = _e793;
            let _e795 = X1_[3u];
            X1_[3u] = -(_e795);
        }
    }
    phi_1391_ = Ph;
    if Ph {
        phi_1391_ = ((_e637.x & 2048u) != 0u);
    }
    let _e812 = phi_1391_;
    if _e812 {
        let _e813 = (_e126 * 8u);
        let _e817 = PB.g2_[(_e813 + 4u)];
        let _e828 = PB.g2_[(_e813 + 5u)];
        let _e831 = ((mat2x2<f32>(vec2<f32>(_e817.x, _e817.y), vec2<f32>(_e817.z, _e817.w)) * _e616) + _e828.xy);
        C2_ = vec3<f32>(_e831.x, _e831.y, (1f + _e828.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e607) {
        let _e842 = j.Hf;
        let _e844 = j.If;
        let _e856 = OB.g2_[(_e128 + 3u)];
        k3_ = _e856.xy;
        v4_ = (_e616 + bitcast<vec2<f32>>(_e856.zw));
        phi_3218_ = vec4<f32>(((_e616.x * _e842) - 1f), ((_e616.y * _e844) - sign(_e844)), 0f, 1f);
    } else {
        let _e839 = j.W2_;
        phi_3218_ = vec4(_e839);
    }
    let _e862 = phi_3218_;
    unnamed.gl_Position = _e862;
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
    let _e22 = D0_;
    let _e23 = Y1_;
    let _e24 = g1_;
    let _e25 = O0_;
    let _e26 = X1_;
    let _e27 = C2_;
    let _e28 = k3_;
    let _e29 = v4_;
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
