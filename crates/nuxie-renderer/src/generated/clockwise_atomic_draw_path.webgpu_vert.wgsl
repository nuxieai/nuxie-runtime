struct Dg {
    e2_: array<vec4<u32>>,
}

struct Cg {
    e2_: array<vec4<u32>>,
}

struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct jf {
    e2_: array<vec2<u32>>,
}

struct kf {
    e2_: array<vec4<f32>>,
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

@id(0) override Ch: bool = true;
@id(2) override Eh: bool = true;
@id(1) override Dh: bool = true;
@id(8) override Kh: bool = true;

@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ID: Dg;
@group(0) @binding(2)
var<storage> PB: Cg;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
var<private> M: vec4<f32>;
@group(0) @binding(3)
var<storage> DD: jf;
var<private> C0_: f32;
var<private> W1_: vec2<f32>;
var<private> g2_: f32;
@group(0) @binding(4)
var<storage> QB: kf;
var<private> M0_: vec4<f32>;
var<private> V1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> g3_: vec2<u32>;
var<private> p4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ea: sampler;

fn main_1() {
    var phi_2881_: f32;
    var phi_2825_: f32;
    var phi_2797_: i32;
    var phi_1720_: bool;
    var phi_2810_: i32;
    var phi_2802_: vec4<u32>;
    var phi_2809_: i32;
    var phi_2801_: vec4<u32>;
    var phi_2808_: i32;
    var phi_2806_: vec4<u32>;
    var phi_2805_: u32;
    var phi_2812_: vec2<i32>;
    var phi_2813_: vec4<u32>;
    var phi_2817_: f32;
    var phi_2830_: f32;
    var phi_2898_: f32;
    var phi_2896_: f32;
    var phi_2839_: f32;
    var phi_2832_: f32;
    var phi_2829_: f32;
    var phi_2843_: f32;
    var phi_2918_: f32;
    var phi_2909_: f32;
    var phi_2894_: f32;
    var phi_2842_: f32;
    var phi_2892_: f32;
    var phi_2928_: f32;
    var phi_2927_: f32;
    var phi_2929_: f32;
    var phi_2933_: f32;
    var phi_2955_: f32;
    var phi_2953_: f32;
    var phi_2971_: vec4<f32>;
    var phi_3121_: vec2<f32>;
    var phi_2970_: vec4<f32>;
    var phi_3124_: vec4<f32>;
    var phi_2975_: f32;
    var phi_2986_: f32;
    var phi_2978_: f32;
    var phi_3067_: f32;
    var phi_3024_: i32;
    var phi_3033_: f32;
    var phi_2152_: bool;
    var phi_3040_: f32;
    var phi_3056_: vec2<f32>;
    var phi_3055_: vec2<f32>;
    var phi_3077_: vec4<f32>;
    var phi_3092_: vec2<f32>;
    var phi_3076_: vec4<f32>;
    var phi_3125_: vec4<f32>;
    var phi_3122_: vec4<f32>;
    var phi_3118_: vec2<f32>;
    var phi_3094_: vec2<f32>;
    var phi_3165_: vec4<f32>;
    var phi_3127_: vec2<f32>;
    var phi_3126_: bool;
    var local: u32;
    var local_1: u32;
    var local_2: u32;
    var phi_3166_: f32;
    var phi_3167_: u32;
    var phi_3168_: f32;
    var phi_3169_: f32;
    var local_3: u32;
    var phi_2552_: bool;
    var phi_3170_: vec4<f32>;
    var local_4: u32;
    var phi_3219_: vec4<f32>;
    var phi_3220_: vec4<f32>;
    var phi_3221_: f32;
    var phi_1407_: bool;
    var local_5: u32;
    var local_6: u32;
    var phi_3239_: vec4<f32>;

    let _e96 = gl_InstanceIndex_1;
    let _e97 = UB_1;
    let _e98 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e101 = i32(_e97.x);
            let _e105 = bitcast<i32>(_e97.w);
            let _e107 = (_e105 >> bitcast<u32>(2i));
            let _e108 = (_e105 & 3i);
            let _e110 = min(_e101, (_e107 - 1i));
            let _e112 = ((_e96 * _e107) + _e110);
            let _e117 = textureLoad(KC, vec2<i32>((_e112 & 2047i), (_e112 >> bitcast<u32>(11i))), 0i);
            let _e124 = ID.e2_[(max((_e117.w & 65535u), 1u) - 1u)];
            let _e126 = bitcast<vec2<f32>>(_e124.xy);
            let _e128 = (_e124.z & 65535u);
            let _e130 = (_e128 * 4u);
            let _e133 = PB.e2_[_e130];
            let _e134 = bitcast<vec4<f32>>(_e133);
            let _e141 = mat2x2<f32>(vec2<f32>(_e134.x, _e134.y), vec2<f32>(_e134.z, _e134.w));
            let _e145 = PB.e2_[(_e130 + 1u)];
            let _e149 = bitcast<f32>(_e145.z);
            let _e151 = bitcast<f32>(_e145.w);
            let _e152 = (_e117.w & 8388608u);
            phi_2881_ = _e97.z;
            phi_2825_ = _e97.y;
            phi_2797_ = _e101;
            local = _e128;
            local_1 = _e128;
            local_2 = _e128;
            local_3 = _e128;
            local_4 = _e128;
            local_5 = _e128;
            local_6 = _e130;
            if (_e152 != 0u) {
                phi_2881_ = _e98.z;
                phi_2825_ = _e98.y;
                phi_2797_ = i32(_e98.x);
            }
            let _e159 = phi_2881_;
            let _e161 = phi_2825_;
            let _e163 = phi_2797_;
            phi_2808_ = _e112;
            phi_2806_ = _e117;
            phi_2805_ = _e117.w;
            if (_e163 != _e110) {
                let _e166 = ((_e112 + _e163) - _e110);
                let _e171 = textureLoad(KC, vec2<i32>((_e166 & 2047i), (_e166 >> bitcast<u32>(11i))), 0i);
                if ((_e171.w & 8454143u) != (_e117.w & 8454143u)) {
                    let _e176 = (_e149 == 0f);
                    phi_1720_ = _e176;
                    if !(_e176) {
                        phi_1720_ = (_e126.x != 0f);
                    }
                    let _e181 = phi_1720_;
                    phi_2810_ = _e112;
                    phi_2802_ = _e117;
                    if _e181 {
                        let _e182 = bitcast<i32>(_e124.w);
                        let _e187 = textureLoad(KC, vec2<i32>((_e182 & 2047i), (_e182 >> bitcast<u32>(11i))), 0i);
                        phi_2810_ = _e182;
                        phi_2802_ = _e187;
                    }
                    let _e189 = phi_2810_;
                    let _e191 = phi_2802_;
                    phi_2809_ = _e189;
                    phi_2801_ = _e191;
                } else {
                    phi_2809_ = _e166;
                    phi_2801_ = _e171;
                }
                let _e193 = phi_2809_;
                let _e195 = phi_2801_;
                phi_2808_ = _e193;
                phi_2806_ = _e195;
                phi_2805_ = ((_e195.w & 4286578687u) | _e152);
            }
            let _e200 = phi_2808_;
            let _e202 = phi_2806_;
            let _e204 = phi_2805_;
            let _e205 = (_e204 & 469762048u);
            let _e208 = ((_e205 == 67108864u) && (_e108 == 0i));
            if _e208 {
                let _e213 = f32((_e202.z & 65535u));
                let _e216 = f32((_e202.z >> bitcast<u32>(16i)));
                let _e222 = vec2<i32>(i32((-1f - _e213)), i32(((_e216 - _e213) + 1f)));
                phi_2812_ = _e222;
                if ((_e204 & 8388608u) != 0u) {
                    phi_2812_ = -(_e222);
                }
                let _e227 = phi_2812_;
                let _e229 = (_e200 + _e227.x);
                let _e234 = textureLoad(KC, vec2<i32>((_e229 & 2047i), (_e229 >> bitcast<u32>(11i))), 0i);
                let _e236 = (_e200 + _e227.y);
                let _e241 = textureLoad(KC, vec2<i32>((_e236 & 2047i), (_e236 >> bitcast<u32>(11i))), 0i);
                phi_2813_ = _e241;
                if ((_e241.w & 8454143u) != (_e234.w & 8454143u)) {
                    let _e247 = bitcast<i32>(_e124.w);
                    let _e252 = textureLoad(KC, vec2<i32>((_e247 & 2047i), (_e247 >> bitcast<u32>(11i))), 0i);
                    phi_2813_ = _e252;
                }
                let _e254 = phi_2813_;
                let _e256 = bitcast<f32>(_e234.z);
                let _e258 = bitcast<f32>(_e254.z);
                let _e259 = (_e258 - _e256);
                phi_2817_ = _e259;
                if (abs(_e259) > 3.1415927f) {
                    phi_2817_ = (_e259 - (6.2831855f * sign(_e259)));
                }
                let _e266 = phi_2817_;
                let _e267 = (_e216 + -2f);
                let _e273 = clamp(round(((abs(_e266) * 0.31830987f) * _e267)), 1f, (_e216 + -3f));
                let _e274 = (_e267 - _e273);
                if (_e213 <= _e274) {
                    phi_2898_ = _e161;
                    if (_e213 == _e274) {
                        phi_2898_ = -(_e161);
                    }
                    let _e291 = phi_2898_;
                    phi_2896_ = _e291;
                    phi_2839_ = -(((3.1415927f * sign(_e266)) - _e266));
                    phi_2832_ = _e274;
                    phi_2829_ = _e213;
                } else {
                    let _e277 = (_e213 == (_e274 + 1f));
                    if _e277 {
                        phi_2830_ = 0f;
                    } else {
                        phi_2830_ = (_e213 - (_e274 + 2f));
                    }
                    let _e281 = phi_2830_;
                    phi_2896_ = select(_e161, 0f, _e277);
                    phi_2839_ = _e266;
                    phi_2832_ = select(_e273, 0f, _e277);
                    phi_2829_ = _e281;
                }
                let _e293 = phi_2896_;
                let _e295 = phi_2839_;
                let _e297 = phi_2832_;
                let _e299 = phi_2829_;
                if (_e299 == _e297) {
                    phi_2843_ = _e258;
                } else {
                    phi_2843_ = (_e256 + (_e295 * (_e299 / _e297)));
                }
                let _e305 = phi_2843_;
                phi_2918_ = _e256;
                phi_2909_ = _e295;
                phi_2894_ = _e293;
                phi_2842_ = _e305;
            } else {
                phi_2918_ = f32();
                phi_2909_ = f32();
                phi_2894_ = _e161;
                phi_2842_ = bitcast<f32>(_e202.z);
            }
            let _e307 = phi_2918_;
            let _e309 = phi_2909_;
            let _e311 = phi_2894_;
            let _e313 = phi_2842_;
            let _e317 = vec2<f32>(sin(_e313), -(cos(_e313)));
            let _e319 = bitcast<vec2<f32>>(_e202.xy);
            phi_2892_ = _e151;
            if (_e151 != 0f) {
                phi_2892_ = max(_e151, (1f / length((_e141 * _e317))));
            }
            let _e326 = phi_2892_;
            if (_e149 != 0f) {
                let _e434 = (_e311 * sign(determinant(_e141)));
                let _e436 = ((_e204 & 1048576u) != 0u);
                phi_2975_ = _e434;
                if _e436 {
                    phi_2975_ = min(_e434, 0f);
                }
                let _e439 = phi_2975_;
                phi_2986_ = _e439;
                if ((_e204 & 524288u) != 0u) {
                    phi_2986_ = max(_e439, 0f);
                }
                let _e444 = phi_2986_;
                let _e445 = (_e326 != 0f);
                if _e445 {
                    phi_2978_ = _e326;
                } else {
                    let _e446 = (_e141 * _e317);
                    phi_2978_ = (((abs(_e446.x) + abs(_e446.y)) * (1f / dot(_e446, _e446))) * 0.5f);
                }
                let _e457 = phi_2978_;
                let _e460 = ((_e457 > _e149) && (_e326 == 0f));
                phi_3067_ = 1f;
                if _e460 {
                    phi_3067_ = (_e149 / _e457);
                }
                let _e463 = phi_3067_;
                let _e464 = select(_e149, _e457, _e460);
                let _e465 = (_e464 + _e457);
                let _e466 = (_e317 * _e465);
                let _e467 = (_e444 * _e465);
                let _e474 = (((vec2<f32>(_e467, -(_e467)) + vec2(_e464)) * (0.5f / _e457)) + vec2<f32>(0.5f, 0.5f));
                let _e477 = vec4<f32>(_e474.x, _e474.y, 0f, 0f);
                phi_3092_ = _e466;
                phi_3076_ = _e477;
                if (_e205 > 134217728u) {
                    let _e479 = (_e204 & 4194304u);
                    let _e481 = select(2i, -2i, (_e479 == 0u));
                    phi_3024_ = _e481;
                    if ((_e204 & 8388608u) != 0u) {
                        phi_3024_ = -(_e481);
                    }
                    let _e486 = phi_3024_;
                    let _e487 = (_e200 + _e486);
                    let _e492 = textureLoad(KC, vec2<i32>((_e487 & 2047i), (_e487 >> bitcast<u32>(11i))), 0i);
                    let _e496 = abs((bitcast<f32>(_e492.z) - _e313));
                    phi_3033_ = _e496;
                    if (_e496 > 3.1415927f) {
                        phi_3033_ = (6.2831855f - _e496);
                    }
                    let _e500 = phi_3033_;
                    let _e505 = ((_e500 * select(0.5f, -0.5f, ((_e479 != 0u) == _e436))) + _e313);
                    let _e509 = vec2<f32>(sin(_e505), -(cos(_e505)));
                    let _e510 = (_e141 * _e509);
                    let _e518 = ((abs(_e510.x) + abs(_e510.y)) * (1f / dot(_e510, _e510)));
                    let _e520 = cos((_e500 * 0.5f));
                    let _e521 = (_e205 == 335544320u);
                    phi_2152_ = _e521;
                    if !(_e521) {
                        phi_2152_ = ((_e205 == 268435456u) && (_e520 >= 0.25f));
                    }
                    let _e527 = phi_2152_;
                    if _e527 {
                        phi_3040_ = (_e464 * (1f / max(_e520, select(0.25f, 1f, ((_e204 & 33554432u) != 0u)))));
                    } else {
                        phi_3040_ = ((_e464 * _e520) + (_e518 * 0.5f));
                    }
                    let _e538 = phi_3040_;
                    let _e540 = (_e538 + (_e518 * 0.5f));
                    phi_3055_ = _e466;
                    if ((_e204 & 2097152u) != 0u) {
                        if (_e465 <= ((_e540 * _e520) + (_e457 * 0.125f))) {
                            phi_3056_ = (_e509 * (_e465 * (1f / _e520)));
                        } else {
                            let _e547 = (_e509 * _e540);
                            phi_3056_ = (vec2<f32>(dot(_e466, _e466), dot(_e547, _e547)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e466, _e547)));
                        }
                        let _e558 = phi_3056_;
                        phi_3055_ = _e558;
                    }
                    let _e560 = phi_3055_;
                    let _e565 = ((_e540 - dot((_e560 * abs(_e444)), _e509)) / _e518);
                    if _e436 {
                        phi_3077_ = vec4<f32>(_e477.x, _e565, _e477.z, _e477.w);
                    } else {
                        phi_3077_ = vec4<f32>(_e565, _e477.y, _e477.z, _e477.w);
                    }
                    let _e577 = phi_3077_;
                    phi_3092_ = _e560;
                    phi_3076_ = _e577;
                }
                let _e579 = phi_3092_;
                let _e581 = phi_3076_;
                let _e583 = (_e581.xy * _e463);
                let _e589 = vec4<f32>(_e583.x, _e581.y, _e581.z, _e581.w);
                let _e596 = vec4<f32>(_e589.x, max(_e583.y, 0.0001f), _e589.z, _e589.w);
                phi_3125_ = _e596;
                if _e445 {
                    phi_3125_ = vec4<f32>((-2f - _e583.x), _e596.y, _e596.z, _e596.w);
                }
                let _e604 = phi_3125_;
                if (_e108 != 0i) {
                    phi_3165_ = _e604;
                    phi_3127_ = vec2<f32>();
                    phi_3126_ = false;
                    break;
                }
                phi_3122_ = _e604;
                phi_3118_ = (_e141 * (_e579 * _e444));
                phi_3094_ = _e319;
            } else {
                let _e328 = vec4<f32>(_e159, -1f, 0f, 0f);
                if (_e326 != 0f) {
                    let _e339 = vec4<f32>(_e328.x, -2f, _e328.z, _e328.w);
                    let _e344 = vec4<f32>(_e339.x, _e339.y, 1000000f, _e339.w);
                    phi_2971_ = vec4<f32>(_e344.x, _e344.y, _e344.z, _e159);
                    if _e208 {
                        phi_2928_ = _e309;
                        phi_2927_ = _e307;
                        if (_e309 < 0f) {
                            phi_2928_ = -(_e309);
                            phi_2927_ = (_e307 + _e309);
                        }
                        let _e354 = phi_2928_;
                        let _e356 = phi_2927_;
                        let _e358 = ((_e313 - _e356) + 1.5707964f);
                        let _e364 = clamp(((_e358 - (floor((_e358 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e354);
                        phi_2929_ = _e364;
                        if (_e364 > (_e354 * 0.5f)) {
                            phi_2929_ = (_e354 - _e364);
                        }
                        let _e369 = phi_2929_;
                        let _e376 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e369), cos(_e369)) * abs(_e311))) * 0.5f);
                        if (abs((_e354 - 1.5707964f)) < 0.001f) {
                            phi_2955_ = 0f;
                            phi_2953_ = 0f;
                        } else {
                            let _e380 = tan(_e354);
                            let _e385 = (sign((1.5707964f - _e354)) / max(abs(_e380), 0.000001f));
                            if (_e385 >= 0f) {
                                phi_2933_ = (_e376.y - ((1f - _e376.x) * _e380));
                            } else {
                                phi_2933_ = (_e376.y + (_e376.x * _e380));
                            }
                            let _e397 = phi_2933_;
                            phi_2955_ = _e397;
                            phi_2953_ = _e385;
                        }
                        let _e399 = phi_2955_;
                        let _e401 = phi_2953_;
                        phi_2971_ = vec4<f32>((max(_e376.x, 0f) + 0.25f), (-2f - _e376.y), _e401, _e399);
                    }
                    let _e409 = phi_2971_;
                    phi_3121_ = (_e141 * (_e317 * (_e311 * _e326)));
                    phi_2970_ = _e409;
                } else {
                    phi_3121_ = (sign(((_e317 * _e311) * _naga_inverse_2x2_f32(_e141))) * 0.5f);
                    phi_2970_ = _e328;
                }
                let _e414 = phi_3121_;
                let _e416 = phi_2970_;
                phi_3124_ = _e416;
                if (((_e204 & 8388608u) != 0u) != ((_e204 & 16777216u) != 0u)) {
                    phi_3124_ = (_e416 * vec4<f32>(-1f, 1f, 1f, 1f));
                }
                let _e424 = phi_3124_;
                if (((_e204 & 2147483648u) != 0u) && (_e108 != 1i)) {
                    phi_3165_ = _e424;
                    phi_3127_ = vec2<f32>();
                    phi_3126_ = false;
                    break;
                }
                phi_3122_ = _e424;
                phi_3118_ = _e414;
                phi_3094_ = select(_e319, _e126, vec2((_e108 == 2i)));
            }
            let _e609 = phi_3122_;
            let _e611 = phi_3118_;
            let _e613 = phi_3094_;
            let _e619 = l.Zg;
            let _e622 = select(_e609.xy, vec2<f32>(1f, -1f), vec2((_e619 != 0u)));
            let _e628 = vec4<f32>(_e622.x, _e609.y, _e609.z, _e609.w);
            phi_3165_ = vec4<f32>(_e628.x, _e622.y, _e628.z, _e628.w);
            phi_3127_ = (((_e141 * _e613) + _e611) + bitcast<vec2<f32>>(_e145.xy));
            phi_3126_ = true;
            break;
        }
    }
    let _e636 = phi_3165_;
    let _e638 = phi_3127_;
    let _e640 = phi_3126_;
    M = _e636;
    let _e643 = local;
    let _e645 = DD.e2_[_e643];
    let _e647 = l.f6_;
    let _e649 = local_1;
    if (_e649 == 0u) {
        phi_3166_ = 0f;
    } else {
        let _e652 = local_2;
        phi_3166_ = unpack2x16float(((_e652 + 1023u) * _e647)).x;
    }
    let _e658 = phi_3166_;
    C0_ = _e658;
    if ((_e645.x & 512u) != 0u) {
        let _e662 = C0_;
        C0_ = -(_e662);
    }
    let _e664 = (_e645.x & 15u);
    if Ch {
        let _e665 = (_e664 == 0u);
        if _e665 {
            phi_3167_ = _e645.y;
        } else {
            phi_3167_ = _e645.x;
        }
        let _e668 = phi_3167_;
        let _e670 = (_e668 >> bitcast<u32>(16i));
        if (_e670 == 0u) {
            phi_3168_ = 0f;
        } else {
            phi_3168_ = unpack2x16float(((_e670 + 1023u) * _e647)).x;
        }
        let _e677 = phi_3168_;
        phi_3169_ = _e677;
        if _e665 {
            phi_3169_ = -(_e677);
        }
        let _e680 = phi_3169_;
        W1_[0u] = _e680;
    }
    if Eh {
        g2_ = f32(((_e645.x >> bitcast<u32>(4i)) & 15u));
    }
    if Dh {
        let _e687 = local_3;
        let _e688 = (_e687 * 8u);
        let _e692 = QB.e2_[(_e688 + 2u)];
        let _e697 = vec2<f32>(_e692.x, _e692.y);
        let _e698 = vec2<f32>(_e692.z, _e692.w);
        let _e703 = QB.e2_[(_e688 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e708 = (abs(_e697) + abs(_e698));
                let _e710 = (_e708.x != 0f);
                phi_2552_ = _e710;
                if _e710 {
                    phi_2552_ = (_e708.y != 0f);
                }
                let _e714 = phi_2552_;
                if _e714 {
                    let _e718 = ((mat2x2<f32>(_e697, _e698) * _e638) + _e703.xy);
                    let _e719 = -(_e718);
                    let _e725 = (vec2<f32>(1f, 1f) / _e708).xyxy;
                    phi_3170_ = (((vec4<f32>(_e718.x, _e718.y, _e719.x, _e719.y) * _e725) + _e725) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_3170_ = _e703.xyxy;
                    break;
                }
            }
        }
        let _e730 = phi_3170_;
        M0_ = _e730;
    }
    if (_e664 == 1u) {
        V1_ = unpack4x8unorm(_e645.y);
    } else {
        if (Ch && (_e664 == 0u)) {
            let _e814 = (_e645.x >> bitcast<u32>(16i));
            if (_e814 == 0u) {
                phi_3221_ = 0f;
            } else {
                phi_3221_ = unpack2x16float(((_e814 + 1023u) * _e647)).x;
            }
            let _e821 = phi_3221_;
            W1_[1u] = _e821;
        } else {
            let _e735 = local_4;
            let _e736 = (_e735 * 8u);
            let _e739 = QB.e2_[_e736];
            let _e750 = QB.e2_[(_e736 + 1u)];
            let _e759 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e645.y));
            let _e761 = ((mat2x2<f32>(vec2<f32>(_e739.x, _e739.y), vec2<f32>(_e739.z, _e739.w)) * _e638) + _e750.xy);
            if (_e750.z > 0.9f) {
                phi_3219_ = vec4<f32>(_e759.x, _e759.y, 2f, _e759.w);
            } else {
                phi_3219_ = vec4<f32>(_e759.x, _e759.y, _e750.w, _e759.w);
            }
            let _e776 = phi_3219_;
            if (f32(_e664) == 2f) {
                let _e802 = vec4<f32>(_e761.x, _e776.y, _e776.z, _e776.w);
                phi_3220_ = vec4<f32>(_e802.x, 0f, _e802.z, _e802.w);
            } else {
                let _e784 = vec4<f32>(_e776.x, _e776.y, -(_e776.z), _e776.w);
                let _e790 = vec4<f32>(_e761.x, _e784.y, _e784.z, _e784.w);
                phi_3220_ = vec4<f32>(_e790.x, _e761.y, _e790.z, _e790.w);
            }
            let _e809 = phi_3220_;
            V1_ = _e809;
            let _e811 = V1_[3u];
            V1_[3u] = -(_e811);
        }
    }
    phi_1407_ = Kh;
    if Kh {
        phi_1407_ = ((_e645.x & 2048u) != 0u);
    }
    let _e828 = phi_1407_;
    if _e828 {
        let _e830 = local_5;
        let _e831 = (_e830 * 8u);
        let _e835 = QB.e2_[(_e831 + 4u)];
        let _e846 = QB.e2_[(_e831 + 5u)];
        let _e849 = ((mat2x2<f32>(vec2<f32>(_e835.x, _e835.y), vec2<f32>(_e835.z, _e835.w)) * _e638) + _e846.xy);
        C2_ = vec3<f32>(_e849.x, _e849.y, (1f + _e846.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e640 {
        let _e859 = l.Ff;
        let _e861 = l.Gf;
        let _e871 = local_6;
        let _e875 = PB.e2_[(_e871 + 3u)];
        g3_ = _e875.xy;
        p4_ = (_e638 + bitcast<vec2<f32>>(_e875.zw));
        phi_3239_ = vec4<f32>(((_e638.x * _e859) - 1f), ((_e638.y * _e861) - sign(_e861)), 0f, 1f);
    } else {
        let _e856 = l.U2_;
        phi_3239_ = vec4(_e856);
    }
    let _e881 = phi_3239_;
    unnamed.gl_Position = _e881;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e21 = M;
    let _e22 = C0_;
    let _e23 = W1_;
    let _e24 = g2_;
    let _e25 = M0_;
    let _e26 = V1_;
    let _e27 = C2_;
    let _e28 = g3_;
    let _e29 = p4_;
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
