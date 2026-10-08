struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct wf {
    v2_: array<u32>,
}

struct wf_1 {
    v2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(7) override bj: bool = true;
@id(2) override Wi: bool = true;
@id(8) override cj: bool = true;
@id(3) override Xi: bool = true;
@id(1) override Vi: bool = true;
@id(0) override Ui: bool = true;

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Z0_: wf_1;
var<private> V0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> J4_1: vec2<f32>;
var<private> y3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> j2_1: vec2<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> m0_: vec4<f32>;
var<private> N1_: vec4<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_2132_: f32;
    var phi_2133_: f32;
    var phi_2149_: vec4<f32>;
    var phi_2148_: vec4<f32>;
    var phi_1264_: bool;
    var phi_1279_: bool;
    var phi_2134_: f32;
    var phi_2144_: vec4<f32>;
    var phi_2151_: vec4<f32>;
    var phi_2152_: vec4<f32>;
    var phi_1491_: bool;
    var phi_2153_: f32;
    var phi_2178_: f32;
    var phi_2179_: f32;
    var phi_1428_: bool;
    var phi_2180_: f32;
    var phi_2181_: f32;
    var phi_2183_: f32;
    var phi_1052_: bool;
    var phi_2184_: f32;
    var local: bool;
    var phi_1846_: bool;
    var phi_1848_: bool;
    var phi_2212_: f32;
    var phi_2207_: u32;
    var phi_2204_: f32;
    var phi_2211_: f32;
    var phi_2206_: u32;
    var phi_2203_: f32;
    var phi_2208_: f32;
    var phi_2205_: u32;
    var phi_2202_: f32;
    var phi_2213_: f32;
    var phi_2214_: f32;
    var phi_2215_: f32;
    var phi_2216_: f32;
    var phi_2219_: f32;
    var phi_2217_: f32;
    var phi_2233_: f32;
    var phi_2265_: vec3<f32>;

    let _e71 = P0_1;
    let _e73 = V0_1;
    let _e74 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e77 = (Wi && (u32(_e71) != 0u));
            if (_e74.w >= 0f) {
                phi_2148_ = _e74;
            } else {
                let _e80 = -(_e74.w);
                let _e85 = j.ad;
                let _e88 = j.g7_;
                if (_e74.z > 0f) {
                    phi_2132_ = _e74.x;
                } else {
                    phi_2132_ = length(_e74.xy);
                }
                let _e96 = phi_2132_;
                let _e97 = clamp(_e96, 0f, 1f);
                let _e98 = abs(_e74.z);
                if (_e98 > 1f) {
                    phi_2133_ = ((0.9980469f * _e97) + 0.0009765625f);
                } else {
                    phi_2133_ = ((0.001953125f * _e97) + _e98);
                }
                let _e105 = phi_2133_;
                let _e107 = textureSampleLevel(YC, H8_, vec2<f32>(_e105, ((floor(_e80) * _e85) + _e88)), 0f);
                phi_2149_ = _e107;
                if !(_e77) {
                    let _e111 = (_e107.xyz * _e107.w);
                    phi_2149_ = vec4<f32>(_e111.x, _e111.y, _e111.z, (_e107.w * (fract(_e80) * 1.0039216f)));
                }
                let _e118 = phi_2149_;
                phi_2148_ = _e118;
            }
            let _e120 = phi_2148_;
            phi_1264_ = cj;
            if cj {
                phi_1264_ = (_e73.z < 0f);
            }
            let _e124 = phi_1264_;
            if _e124 {
                let _e126 = textureSampleLevel(TB, S4_, _e73.xy, 0f);
                phi_2152_ = _e126;
                break;
            }
            phi_1279_ = cj;
            if cj {
                phi_1279_ = (_e73.z > 0f);
            }
            let _e130 = phi_1279_;
            phi_2151_ = _e120;
            if _e130 {
                let _e134 = textureSampleLevel(TB, S4_, _e73.xy, (_e73.z - 1f));
                phi_2144_ = _e134;
                if _e77 {
                    if (_e134.w != 0f) {
                        phi_2134_ = (1f / _e134.w);
                    } else {
                        phi_2134_ = 0f;
                    }
                    let _e140 = phi_2134_;
                    let _e141 = (_e134.xyz * _e140);
                    phi_2144_ = vec4<f32>(_e141.x, _e141.y, _e141.z, _e134.w);
                }
                let _e147 = phi_2144_;
                phi_2151_ = (_e120 * _e147);
            }
            let _e150 = phi_2151_;
            phi_2152_ = _e150;
            break;
        }
    }
    let _e152 = phi_2152_;
    let _e153 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e156 = (_e153.y >= 0f);
            local = _e156;
            if _e156 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1428_ = Xi;
                        if Xi {
                            phi_1428_ = (_e153.x < -1.5f);
                        }
                        let _e224 = phi_1428_;
                        if _e224 {
                            let _e230 = textureSampleLevel(ZC, Ta, vec2<f32>((3f + _e153.x), 0f), 0f);
                            let _e235 = textureSampleLevel(ZC, Ta, vec2<f32>((1f - _e153.y), 0f), 0f);
                            phi_2180_ = ((1f - _e230.x) - _e235.x);
                            break;
                        } else {
                            phi_2180_ = min(_e153.x, _e153.y);
                            break;
                        }
                    }
                }
                let _e239 = phi_2180_;
                phi_2181_ = _e239;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1491_ = Xi;
                        if Xi {
                            phi_1491_ = (_e153.y < -1.5f);
                        }
                        let _e160 = phi_1491_;
                        if _e160 {
                            let _e164 = max(_e153.w, 0f);
                            if (_e153.z >= 0f) {
                                let _e167 = textureSampleLevel(ZC, Ta, vec2<f32>(_e164, 0f), 0f);
                                phi_2153_ = _e167.x;
                            } else {
                                phi_2153_ = 0f;
                            }
                            let _e170 = phi_2153_;
                            phi_2178_ = _e170;
                            if (abs(_e153.z) < 1000f) {
                                let _e176 = (-2f - _e153.y);
                                let _e178 = ((_e176 - _e164) * 0.5984134f);
                                let _e181 = (vec4(_e164) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e178));
                                let _e187 = ((_e181 * -(_e153.z)) + vec4(((_e176 * _e153.z) + (abs(_e153.x) - 0.25f))));
                                let _e190 = textureSampleLevel(ZC, Ta, vec2<f32>(_e187.x, 0f), 0f);
                                let _e193 = textureSampleLevel(ZC, Ta, vec2<f32>(_e187.y, 0f), 0f);
                                let _e196 = textureSampleLevel(ZC, Ta, vec2<f32>(_e187.z, 0f), 0f);
                                let _e199 = textureSampleLevel(ZC, Ta, vec2<f32>(_e187.w, 0f), 0f);
                                let _e205 = (_e181 * 5.0959306f);
                                phi_2178_ = (_e170 + (dot(vec4<f32>(_e190.x, _e193.x, _e196.x, _e199.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e205) * (_e205 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e178));
                            }
                            let _e214 = phi_2178_;
                            phi_2179_ = (_e214 * sign(_e153.x));
                            break;
                        } else {
                            phi_2179_ = _e153.x;
                            break;
                        }
                    }
                }
                let _e219 = phi_2179_;
                phi_2181_ = _e219;
                break;
            }
        }
    }
    let _e241 = phi_2181_;
    let _e242 = J4_1;
    let _e245 = y3_1[1u];
    let _e247 = y3_1[0u];
    let _e248 = vec2<u32>(floor(_e242));
    let _e275 = (_e247 + (((((_e248.y >> bitcast<u32>(5u)) * (_e245 << bitcast<u32>(5u))) + ((_e248.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e248.x & 28u) << bitcast<u32>(5u)) + ((_e248.y & 28u) << bitcast<u32>(2i)))) + (((_e248.y & 3u) << bitcast<u32>(2i)) + (_e248.x & 3u))));
    phi_2183_ = 1f;
    if Vi {
        let _e276 = W0_1;
        let _e279 = min(_e276.xy, _e276.zw);
        phi_2183_ = min(min(_e279.x, _e279.y), 1f);
    }
    let _e285 = phi_2183_;
    phi_1052_ = Ui;
    if Ui {
        let _e287 = j2_1[0u];
        phi_1052_ = (_e287 != 0f);
    }
    let _e290 = phi_1052_;
    phi_2184_ = _e285;
    if _e290 {
        phi_2184_ = min(0f, _e285);
    }
    let _e293 = phi_2184_;
    let _e295 = clamp(_e241, 0f, max(_e293, 0f));
    let _e297 = local;
    if _e297 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e152.w, _e295) >= 1f) {
                    phi_2216_ = 1f;
                    break;
                }
                let _e388 = j.r2_;
                let _e390 = atomicMax((&Z0_.v2_[_e275]), (_e388 | u32(((abs(_e295) * 1024f) + 0.5f))));
                if (_e390 < _e388) {
                    phi_2215_ = _e295;
                } else {
                    let _e394 = (f32((_e390 & 524287u)) * 0.0009765625f);
                    phi_2215_ = ((max(_e394, _e295) - _e394) / max((1f - (_e394 * _e152.w)), 0.000062f));
                }
                let _e402 = phi_2215_;
                phi_2216_ = _e402;
                break;
            }
        }
        let _e404 = phi_2216_;
        phi_2219_ = _e404;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e303 = u32(((abs(_e295) * 1024f) + 0.5f));
                let _e306 = atomicLoad((&Z0_.v2_[_e275]));
                let _e308 = (min(_e152.w, _e295) >= 1f);
                phi_1848_ = _e308;
                if _e308 {
                    let _e310 = j.r2_;
                    let _e311 = (_e306 < _e310);
                    phi_1846_ = _e311;
                    if !(_e311) {
                        phi_1846_ = (_e306 >= (_e310 | 262144u));
                    }
                    let _e316 = phi_1846_;
                    phi_1848_ = _e316;
                }
                let _e318 = phi_1848_;
                if _e318 {
                    phi_2214_ = 1f;
                    break;
                }
                let _e320 = j.r2_;
                phi_2208_ = 0f;
                phi_2205_ = _e303;
                phi_2202_ = _e295;
                if (_e306 < _e320) {
                    let _e323 = (_e320 | (262144u + _e303));
                    let _e324 = atomicMax((&Z0_.v2_[_e275]), _e323);
                    if (_e324 <= _e320) {
                        phi_2211_ = _e295;
                        phi_2206_ = _e303;
                        phi_2203_ = 0f;
                    } else {
                        phi_2212_ = 0f;
                        phi_2207_ = _e303;
                        phi_2204_ = _e295;
                        if (_e324 < _e323) {
                            let _e328 = ((_e324 & 524287u) - 262144u);
                            let _e330 = (f32(_e328) * 0.0009765625f);
                            phi_2212_ = ((_e295 - _e330) / max((1f - (_e330 * _e152.w)), 0.000062f));
                            phi_2207_ = _e328;
                            phi_2204_ = _e330;
                        }
                        let _e337 = phi_2212_;
                        let _e339 = phi_2207_;
                        let _e341 = phi_2204_;
                        phi_2211_ = _e337;
                        phi_2206_ = _e339;
                        phi_2203_ = _e341;
                    }
                    let _e343 = phi_2211_;
                    let _e345 = phi_2206_;
                    let _e347 = phi_2203_;
                    phi_2208_ = _e343;
                    phi_2205_ = _e345;
                    phi_2202_ = _e347;
                }
                let _e349 = phi_2208_;
                let _e351 = phi_2205_;
                let _e353 = phi_2202_;
                phi_2213_ = _e349;
                if (_e353 > 0f) {
                    let _e355 = atomicAdd((&Z0_.v2_[_e275]), _e351);
                    let _e360 = (f32(bitcast<i32>(((_e355 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e362 = clamp(_e360, 0f, 1f);
                    phi_2213_ = (_e349 + ((1f - (_e349 * _e152.w)) * ((clamp((_e360 + _e353), 0f, 1f) - _e362) / max((1f - (_e362 * _e152.w)), 0.000062f))));
                }
                let _e374 = phi_2213_;
                phi_2214_ = _e374;
                break;
            }
        }
        let _e376 = phi_2214_;
        phi_2219_ = _e376;
    }
    let _e406 = phi_2219_;
    phi_2233_ = f32();
    if bj {
        let _e407 = gl_FragCoord_1;
        let _e409 = j.E3_;
        let _e411 = j.F3_;
        if bj {
            phi_2217_ = ((fract((52.982918f * fract(((0.06711056f * _e407.x) + (0.00583715f * _e407.y))))) * _e409) + _e411);
        } else {
            phi_2217_ = 0f;
        }
        let _e423 = phi_2217_;
        phi_2233_ = _e423;
    }
    let _e425 = phi_2233_;
    let _e426 = (_e152 * _e406);
    let _e427 = _e426.xyz;
    if (bj && (_e426.w != 0f)) {
        phi_2265_ = (vec3(_e425) + _e427);
    } else {
        phi_2265_ = _e427;
    }
    let _e434 = phi_2265_;
    let _e440 = vec4<f32>(_e434.x, _e426.y, _e426.z, _e426.w);
    let _e446 = vec4<f32>(_e440.x, _e434.y, _e440.z, _e440.w);
    m0_ = vec4<f32>(0f, 0f, 0f, 0f);
    N1_ = vec4<f32>(_e446.x, _e446.y, _e434.z, _e446.w);
    return;
}

@fragment
fn main(@location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) J4_: vec2<f32>, @location(7) @interpolate(flat, either) y3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) j2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> FragmentOutput {
    V0_1 = V0_;
    P0_1 = P0_;
    O0_1 = O0_;
    S_1 = S;
    J4_1 = J4_;
    y3_1 = y3_;
    W0_1 = W0_;
    j2_1 = j2_;
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    main_1();
    let _e22 = m0_;
    let _e23 = N1_;
    return FragmentOutput(_e22, _e23);
}
