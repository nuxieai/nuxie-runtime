struct Ae {
    e2_: array<u32>,
}

struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

struct h0Sd {
    e2_: array<u32>,
}

struct Ae_1 {
    e2_: array<atomic<u32>>,
}

@id(7) override Kh: bool = true;
@id(2) override Fh: bool = true;
@id(8) override Lh: bool = true;
@id(3) override Gh: bool = true;
@id(1) override Eh: bool = true;
@id(0) override Dh: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var fa: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var O9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;
@group(0) @binding(6)
var<storage, read_write> Q0_: Ae_1;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> V1_1: vec4<f32>;
var<private> B2_1: vec3<f32>;
var<private> M_1: vec4<f32>;
var<private> p4_1: vec2<f32>;
var<private> g3_1: vec2<u32>;
var<private> M0_1: vec4<f32>;
var<private> W1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Sd;
var<private> C1_: vec4<f32>;
var<private> C0_1: f32;
var<private> g2_1: f32;

fn main_1() {
    var phi_2096_: f32;
    var phi_2097_: f32;
    var phi_2113_: vec4<f32>;
    var phi_2112_: vec4<f32>;
    var phi_2111_: vec4<f32>;
    var phi_1251_: bool;
    var phi_2098_: f32;
    var phi_2108_: vec4<f32>;
    var phi_2115_: vec4<f32>;
    var phi_1451_: bool;
    var phi_2116_: f32;
    var phi_2139_: f32;
    var phi_2140_: f32;
    var phi_1388_: bool;
    var phi_2141_: f32;
    var phi_2142_: f32;
    var phi_2144_: f32;
    var phi_1014_: bool;
    var phi_2145_: f32;
    var local: bool;
    var phi_1806_: bool;
    var phi_1808_: bool;
    var phi_2173_: f32;
    var phi_2168_: u32;
    var phi_2165_: f32;
    var phi_2172_: f32;
    var phi_2167_: u32;
    var phi_2164_: f32;
    var phi_2169_: f32;
    var phi_2166_: u32;
    var phi_2163_: f32;
    var phi_2174_: f32;
    var phi_2175_: f32;
    var phi_2176_: f32;
    var phi_2177_: f32;
    var phi_2180_: f32;
    var phi_2178_: f32;
    var phi_2194_: f32;
    var phi_2225_: vec3<f32>;

    let _e71 = gl_FragCoord_1;
    let _e75 = bitcast<vec2<u32>>(vec2<i32>(floor(_e71.xy)));
    let _e77 = l.o6_;
    let _e106 = bitcast<i32>((((((_e75.y >> bitcast<u32>(5u)) * (((_e77 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e75.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e75.x & 28u) << bitcast<u32>(5u)) + ((_e75.y & 28u) << bitcast<u32>(2i)))) + (((_e75.y & 3u) << bitcast<u32>(2i)) + (_e75.x & 3u))));
    let _e107 = V1_1;
    let _e108 = B2_1;
    if (_e107.w >= 0f) {
        if Fh {
            phi_2112_ = vec4<f32>(_e107.x, _e107.y, _e107.z, _e107.w);
        } else {
            phi_2112_ = (_e107 * 1f);
        }
        let _e152 = phi_2112_;
        phi_2111_ = _e152;
    } else {
        if (_e107.z > 0f) {
            phi_2096_ = _e107.x;
        } else {
            phi_2096_ = length(_e107.xy);
        }
        let _e118 = phi_2096_;
        let _e119 = clamp(_e118, 0f, 1f);
        let _e120 = abs(_e107.z);
        if (_e120 > 1f) {
            phi_2097_ = ((0.9980469f * _e119) + 0.0009765625f);
        } else {
            phi_2097_ = ((0.001953125f * _e119) + _e120);
        }
        let _e127 = phi_2097_;
        let _e129 = textureSampleLevel(ED, O9_, vec2<f32>(_e127, -(_e107.w)), 0f);
        let _e135 = vec4<f32>(_e129.x, _e129.y, _e129.z, _e129.w);
        if Fh {
            phi_2113_ = _e135;
        } else {
            let _e137 = (_e135.xyz * _e129.w);
            phi_2113_ = vec4<f32>(_e137.x, _e137.y, _e137.z, _e129.w);
        }
        let _e143 = phi_2113_;
        phi_2111_ = _e143;
    }
    let _e154 = phi_2111_;
    phi_1251_ = Lh;
    if Lh {
        phi_1251_ = (_e108.z > 0f);
    }
    let _e158 = phi_1251_;
    phi_2115_ = _e154;
    if _e158 {
        let _e162 = textureSampleLevel(HC, V5_, _e108.xy, (_e108.z - 1f));
        phi_2108_ = _e162;
        if Fh {
            if (_e162.w != 0f) {
                phi_2098_ = (1f / _e162.w);
            } else {
                phi_2098_ = 0f;
            }
            let _e168 = phi_2098_;
            let _e169 = (_e162.xyz * _e168);
            phi_2108_ = vec4<f32>(_e169.x, _e169.y, _e169.z, _e162.w);
        }
        let _e175 = phi_2108_;
        phi_2115_ = (_e154 * _e175);
    }
    let _e178 = phi_2115_;
    let _e179 = M_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e182 = (_e179.y >= 0f);
            local = _e182;
            if _e182 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1388_ = Gh;
                        if Gh {
                            phi_1388_ = (_e179.x < -1.5f);
                        }
                        let _e250 = phi_1388_;
                        if _e250 {
                            let _e256 = textureSampleLevel(YC, fa, vec2<f32>((3f + _e179.x), 0f), 0f);
                            let _e261 = textureSampleLevel(YC, fa, vec2<f32>((1f - _e179.y), 0f), 0f);
                            phi_2141_ = ((1f - _e256.x) - _e261.x);
                            break;
                        } else {
                            phi_2141_ = min(_e179.x, _e179.y);
                            break;
                        }
                    }
                }
                let _e265 = phi_2141_;
                phi_2142_ = _e265;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1451_ = Gh;
                        if Gh {
                            phi_1451_ = (_e179.y < -1.5f);
                        }
                        let _e186 = phi_1451_;
                        if _e186 {
                            let _e190 = max(_e179.w, 0f);
                            if (_e179.z >= 0f) {
                                let _e193 = textureSampleLevel(YC, fa, vec2<f32>(_e190, 0f), 0f);
                                phi_2116_ = _e193.x;
                            } else {
                                phi_2116_ = 0f;
                            }
                            let _e196 = phi_2116_;
                            phi_2139_ = _e196;
                            if (abs(_e179.z) < 1000f) {
                                let _e202 = (-2f - _e179.y);
                                let _e204 = ((_e202 - _e190) * 0.5984134f);
                                let _e207 = (vec4(_e190) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e204));
                                let _e213 = ((_e207 * -(_e179.z)) + vec4(((_e202 * _e179.z) + (abs(_e179.x) - 0.25f))));
                                let _e216 = textureSampleLevel(YC, fa, vec2<f32>(_e213.x, 0f), 0f);
                                let _e219 = textureSampleLevel(YC, fa, vec2<f32>(_e213.y, 0f), 0f);
                                let _e222 = textureSampleLevel(YC, fa, vec2<f32>(_e213.z, 0f), 0f);
                                let _e225 = textureSampleLevel(YC, fa, vec2<f32>(_e213.w, 0f), 0f);
                                let _e231 = (_e207 * 5.0959306f);
                                phi_2139_ = (_e196 + (dot(vec4<f32>(_e216.x, _e219.x, _e222.x, _e225.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e231) * (_e231 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e204));
                            }
                            let _e240 = phi_2139_;
                            phi_2140_ = (_e240 * sign(_e179.x));
                            break;
                        } else {
                            phi_2140_ = _e179.x;
                            break;
                        }
                    }
                }
                let _e245 = phi_2140_;
                phi_2142_ = _e245;
                break;
            }
        }
    }
    let _e267 = phi_2142_;
    let _e268 = p4_1;
    let _e271 = g3_1[1u];
    let _e273 = g3_1[0u];
    let _e274 = vec2<u32>(floor(_e268));
    let _e301 = (_e273 + (((((_e274.y >> bitcast<u32>(5u)) * (_e271 << bitcast<u32>(5u))) + ((_e274.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e274.x & 28u) << bitcast<u32>(5u)) + ((_e274.y & 28u) << bitcast<u32>(2i)))) + (((_e274.y & 3u) << bitcast<u32>(2i)) + (_e274.x & 3u))));
    phi_2144_ = 1f;
    if Eh {
        let _e302 = M0_1;
        let _e305 = min(_e302.xy, _e302.zw);
        phi_2144_ = min(min(_e305.x, _e305.y), 1f);
    }
    let _e311 = phi_2144_;
    phi_1014_ = Dh;
    if Dh {
        let _e313 = W1_1[0u];
        phi_1014_ = (_e313 != 0f);
    }
    let _e316 = phi_1014_;
    phi_2145_ = _e311;
    if _e316 {
        let _e319 = h0_.e2_[_e106];
        phi_2145_ = min(unpack4x8unorm(_e319).x, _e311);
    }
    let _e324 = phi_2145_;
    let _e326 = clamp(_e267, 0f, max(_e324, 0f));
    let _e328 = local;
    if _e328 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e178.w, _e326) >= 1f) {
                    phi_2177_ = 1f;
                    break;
                }
                let _e419 = l.d2_;
                let _e421 = atomicMax((&Q0_.e2_[_e301]), (_e419 | u32(((abs(_e326) * 1024f) + 0.5f))));
                if (_e421 < _e419) {
                    phi_2176_ = _e326;
                } else {
                    let _e425 = (f32((_e421 & 524287u)) * 0.0009765625f);
                    phi_2176_ = ((max(_e425, _e326) - _e425) / max((1f - (_e425 * _e178.w)), 0.000062f));
                }
                let _e433 = phi_2176_;
                phi_2177_ = _e433;
                break;
            }
        }
        let _e435 = phi_2177_;
        phi_2180_ = _e435;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e334 = u32(((abs(_e326) * 1024f) + 0.5f));
                let _e337 = atomicLoad((&Q0_.e2_[_e301]));
                let _e339 = (min(_e178.w, _e326) >= 1f);
                phi_1808_ = _e339;
                if _e339 {
                    let _e341 = l.d2_;
                    let _e342 = (_e337 < _e341);
                    phi_1806_ = _e342;
                    if !(_e342) {
                        phi_1806_ = (_e337 >= (_e341 | 262144u));
                    }
                    let _e347 = phi_1806_;
                    phi_1808_ = _e347;
                }
                let _e349 = phi_1808_;
                if _e349 {
                    phi_2175_ = 1f;
                    break;
                }
                let _e351 = l.d2_;
                phi_2169_ = 0f;
                phi_2166_ = _e334;
                phi_2163_ = _e326;
                if (_e337 < _e351) {
                    let _e354 = (_e351 | (262144u + _e334));
                    let _e355 = atomicMax((&Q0_.e2_[_e301]), _e354);
                    if (_e355 <= _e351) {
                        phi_2172_ = _e326;
                        phi_2167_ = _e334;
                        phi_2164_ = 0f;
                    } else {
                        phi_2173_ = 0f;
                        phi_2168_ = _e334;
                        phi_2165_ = _e326;
                        if (_e355 < _e354) {
                            let _e359 = ((_e355 & 524287u) - 262144u);
                            let _e361 = (f32(_e359) * 0.0009765625f);
                            phi_2173_ = ((_e326 - _e361) / max((1f - (_e361 * _e178.w)), 0.000062f));
                            phi_2168_ = _e359;
                            phi_2165_ = _e361;
                        }
                        let _e368 = phi_2173_;
                        let _e370 = phi_2168_;
                        let _e372 = phi_2165_;
                        phi_2172_ = _e368;
                        phi_2167_ = _e370;
                        phi_2164_ = _e372;
                    }
                    let _e374 = phi_2172_;
                    let _e376 = phi_2167_;
                    let _e378 = phi_2164_;
                    phi_2169_ = _e374;
                    phi_2166_ = _e376;
                    phi_2163_ = _e378;
                }
                let _e380 = phi_2169_;
                let _e382 = phi_2166_;
                let _e384 = phi_2163_;
                phi_2174_ = _e380;
                if (_e384 > 0f) {
                    let _e386 = atomicAdd((&Q0_.e2_[_e301]), _e382);
                    let _e391 = (f32(bitcast<i32>(((_e386 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e393 = clamp(_e391, 0f, 1f);
                    phi_2174_ = (_e380 + ((1f - (_e380 * _e178.w)) * ((clamp((_e391 + _e384), 0f, 1f) - _e393) / max((1f - (_e393 * _e178.w)), 0.000062f))));
                }
                let _e405 = phi_2174_;
                phi_2175_ = _e405;
                break;
            }
        }
        let _e407 = phi_2175_;
        phi_2180_ = _e407;
    }
    let _e437 = phi_2180_;
    phi_2194_ = f32();
    if Kh {
        let _e439 = l.C3_;
        let _e441 = l.D3_;
        if Kh {
            phi_2178_ = ((fract((52.982918f * fract(((0.06711056f * _e71.x) + (0.00583715f * _e71.y))))) * _e439) + _e441);
        } else {
            phi_2178_ = 0f;
        }
        let _e453 = phi_2178_;
        phi_2194_ = _e453;
    }
    let _e455 = phi_2194_;
    let _e456 = (_e178 * _e437);
    let _e457 = _e456.xyz;
    if (Kh && (_e456.w != 0f)) {
        phi_2225_ = (vec3(_e455) + _e457);
    } else {
        phi_2225_ = _e457;
    }
    let _e464 = phi_2225_;
    let _e470 = vec4<f32>(_e464.x, _e456.y, _e456.z, _e456.w);
    let _e476 = vec4<f32>(_e470.x, _e464.y, _e470.z, _e470.w);
    h0_.e2_[_e106] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    C1_ = vec4<f32>(_e476.x, _e476.y, _e464.z, _e476.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) V1_: vec4<f32>, @location(9) B2_: vec3<f32>, @location(2) M: vec4<f32>, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    V1_1 = V1_;
    B2_1 = B2_;
    M_1 = M;
    p4_1 = p4_;
    g3_1 = g3_;
    M0_1 = M0_;
    W1_1 = W1_;
    C0_1 = C0_;
    g2_1 = g2_;
    main_1();
    let _e21 = C1_;
    return _e21;
}
