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

struct Re {
    j2_: array<u32>,
}

struct m0ge {
    j2_: array<u32>,
}

struct Re_1 {
    j2_: array<atomic<u32>>,
}

@id(7) override ii: bool = true;
@id(2) override di: bool = true;
@id(8) override ji: bool = true;
@id(3) override ei: bool = true;
@id(1) override ci: bool = true;
@id(0) override bi: bool = true;

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var f6_: sampler;
@group(0) @binding(6)
var<storage, read_write> V0_: Re_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> F1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> F4_1: vec2<f32>;
var<private> q3_1: vec2<u32>;
var<private> R0_1: vec4<f32>;
var<private> l1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0ge;
var<private> J1_: vec4<f32>;
var<private> F0_1: f32;

fn main_1() {
    var phi_2151_: f32;
    var phi_2152_: f32;
    var phi_2166_: vec4<f32>;
    var phi_2165_: vec4<f32>;
    var phi_1299_: bool;
    var phi_2153_: f32;
    var phi_2162_: vec4<f32>;
    var phi_2168_: vec4<f32>;
    var phi_1505_: bool;
    var phi_2169_: f32;
    var phi_2190_: f32;
    var phi_2191_: f32;
    var phi_1442_: bool;
    var phi_2192_: f32;
    var phi_2193_: f32;
    var phi_2195_: f32;
    var phi_1051_: bool;
    var phi_2196_: f32;
    var local: bool;
    var phi_1860_: bool;
    var phi_1862_: bool;
    var phi_2224_: f32;
    var phi_2219_: u32;
    var phi_2216_: f32;
    var phi_2223_: f32;
    var phi_2218_: u32;
    var phi_2215_: f32;
    var phi_2220_: f32;
    var phi_2217_: u32;
    var phi_2214_: f32;
    var phi_2225_: f32;
    var phi_2226_: f32;
    var phi_2227_: f32;
    var phi_2228_: f32;
    var phi_2231_: f32;
    var phi_2229_: f32;
    var phi_2245_: f32;
    var phi_2275_: vec3<f32>;

    let _e74 = gl_FragCoord_1;
    let _e78 = bitcast<vec2<u32>>(vec2<i32>(floor(_e74.xy)));
    let _e80 = j.z6_;
    let _e109 = bitcast<i32>((((((_e78.y >> bitcast<u32>(5u)) * (((_e80 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e78.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e78.x & 28u) << bitcast<u32>(5u)) + ((_e78.y & 28u) << bitcast<u32>(2i)))) + (((_e78.y & 3u) << bitcast<u32>(2i)) + (_e78.x & 3u))));
    let _e110 = Q0_1;
    let _e112 = F1_1;
    let _e113 = a1_1;
    let _e115 = (di && (u32(_e110) != 0u));
    if (_e113.w >= 0f) {
        phi_2165_ = _e113;
    } else {
        let _e118 = -(_e113.w);
        let _e123 = j.wc;
        let _e126 = j.xc;
        if (_e113.z > 0f) {
            phi_2151_ = _e113.x;
        } else {
            phi_2151_ = length(_e113.xy);
        }
        let _e134 = phi_2151_;
        let _e135 = clamp(_e134, 0f, 1f);
        let _e136 = abs(_e113.z);
        if (_e136 > 1f) {
            phi_2152_ = ((0.9980469f * _e135) + 0.0009765625f);
        } else {
            phi_2152_ = ((0.001953125f * _e135) + _e136);
        }
        let _e143 = phi_2152_;
        let _e145 = textureSampleLevel(FD, ha, vec2<f32>(_e143, ((floor(_e118) * _e123) + _e126)), 0f);
        phi_2166_ = _e145;
        if !(_e115) {
            let _e149 = (_e145.xyz * _e145.w);
            phi_2166_ = vec4<f32>(_e149.x, _e149.y, _e149.z, (_e145.w * (fract(_e118) * 1.0039216f)));
        }
        let _e156 = phi_2166_;
        phi_2165_ = _e156;
    }
    let _e158 = phi_2165_;
    phi_1299_ = ji;
    if ji {
        phi_1299_ = (_e112.z > 0f);
    }
    let _e162 = phi_1299_;
    phi_2168_ = _e158;
    if _e162 {
        let _e166 = textureSampleLevel(IC, f6_, _e112.xy, (_e112.z - 1f));
        phi_2162_ = _e166;
        if _e115 {
            if (_e166.w != 0f) {
                phi_2153_ = (1f / _e166.w);
            } else {
                phi_2153_ = 0f;
            }
            let _e172 = phi_2153_;
            let _e173 = (_e166.xyz * _e172);
            phi_2162_ = vec4<f32>(_e173.x, _e173.y, _e173.z, _e166.w);
        }
        let _e179 = phi_2162_;
        phi_2168_ = (_e158 * _e179);
    }
    let _e182 = phi_2168_;
    let _e183 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e186 = (_e183.y >= 0f);
            local = _e186;
            if _e186 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1442_ = ei;
                        if ei {
                            phi_1442_ = (_e183.x < -1.5f);
                        }
                        let _e254 = phi_1442_;
                        if _e254 {
                            let _e260 = textureSampleLevel(ZC, wa, vec2<f32>((3f + _e183.x), 0f), 0f);
                            let _e265 = textureSampleLevel(ZC, wa, vec2<f32>((1f - _e183.y), 0f), 0f);
                            phi_2192_ = ((1f - _e260.x) - _e265.x);
                            break;
                        } else {
                            phi_2192_ = min(_e183.x, _e183.y);
                            break;
                        }
                    }
                }
                let _e269 = phi_2192_;
                phi_2193_ = _e269;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1505_ = ei;
                        if ei {
                            phi_1505_ = (_e183.y < -1.5f);
                        }
                        let _e190 = phi_1505_;
                        if _e190 {
                            let _e194 = max(_e183.w, 0f);
                            if (_e183.z >= 0f) {
                                let _e197 = textureSampleLevel(ZC, wa, vec2<f32>(_e194, 0f), 0f);
                                phi_2169_ = _e197.x;
                            } else {
                                phi_2169_ = 0f;
                            }
                            let _e200 = phi_2169_;
                            phi_2190_ = _e200;
                            if (abs(_e183.z) < 1000f) {
                                let _e206 = (-2f - _e183.y);
                                let _e208 = ((_e206 - _e194) * 0.5984134f);
                                let _e211 = (vec4(_e194) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e208));
                                let _e217 = ((_e211 * -(_e183.z)) + vec4(((_e206 * _e183.z) + (abs(_e183.x) - 0.25f))));
                                let _e220 = textureSampleLevel(ZC, wa, vec2<f32>(_e217.x, 0f), 0f);
                                let _e223 = textureSampleLevel(ZC, wa, vec2<f32>(_e217.y, 0f), 0f);
                                let _e226 = textureSampleLevel(ZC, wa, vec2<f32>(_e217.z, 0f), 0f);
                                let _e229 = textureSampleLevel(ZC, wa, vec2<f32>(_e217.w, 0f), 0f);
                                let _e235 = (_e211 * 5.0959306f);
                                phi_2190_ = (_e200 + (dot(vec4<f32>(_e220.x, _e223.x, _e226.x, _e229.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e235) * (_e235 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e208));
                            }
                            let _e244 = phi_2190_;
                            phi_2191_ = (_e244 * sign(_e183.x));
                            break;
                        } else {
                            phi_2191_ = _e183.x;
                            break;
                        }
                    }
                }
                let _e249 = phi_2191_;
                phi_2193_ = _e249;
                break;
            }
        }
    }
    let _e271 = phi_2193_;
    let _e272 = F4_1;
    let _e275 = q3_1[1u];
    let _e277 = q3_1[0u];
    let _e278 = vec2<u32>(floor(_e272));
    let _e305 = (_e277 + (((((_e278.y >> bitcast<u32>(5u)) * (_e275 << bitcast<u32>(5u))) + ((_e278.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e278.x & 28u) << bitcast<u32>(5u)) + ((_e278.y & 28u) << bitcast<u32>(2i)))) + (((_e278.y & 3u) << bitcast<u32>(2i)) + (_e278.x & 3u))));
    phi_2195_ = 1f;
    if ci {
        let _e306 = R0_1;
        let _e309 = min(_e306.xy, _e306.zw);
        phi_2195_ = min(min(_e309.x, _e309.y), 1f);
    }
    let _e315 = phi_2195_;
    phi_1051_ = bi;
    if bi {
        let _e317 = l1_1[0u];
        phi_1051_ = (_e317 != 0f);
    }
    let _e320 = phi_1051_;
    phi_2196_ = _e315;
    if _e320 {
        let _e323 = m0_.j2_[_e109];
        phi_2196_ = min(unpack4x8unorm(_e323).x, _e315);
    }
    let _e328 = phi_2196_;
    let _e330 = clamp(_e271, 0f, max(_e328, 0f));
    let _e332 = local;
    if _e332 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e182.w, _e330) >= 1f) {
                    phi_2228_ = 1f;
                    break;
                }
                let _e423 = j.i2_;
                let _e425 = atomicMax((&V0_.j2_[_e305]), (_e423 | u32(((abs(_e330) * 1024f) + 0.5f))));
                if (_e425 < _e423) {
                    phi_2227_ = _e330;
                } else {
                    let _e429 = (f32((_e425 & 524287u)) * 0.0009765625f);
                    phi_2227_ = ((max(_e429, _e330) - _e429) / max((1f - (_e429 * _e182.w)), 0.000062f));
                }
                let _e437 = phi_2227_;
                phi_2228_ = _e437;
                break;
            }
        }
        let _e439 = phi_2228_;
        phi_2231_ = _e439;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e338 = u32(((abs(_e330) * 1024f) + 0.5f));
                let _e341 = atomicLoad((&V0_.j2_[_e305]));
                let _e343 = (min(_e182.w, _e330) >= 1f);
                phi_1862_ = _e343;
                if _e343 {
                    let _e345 = j.i2_;
                    let _e346 = (_e341 < _e345);
                    phi_1860_ = _e346;
                    if !(_e346) {
                        phi_1860_ = (_e341 >= (_e345 | 262144u));
                    }
                    let _e351 = phi_1860_;
                    phi_1862_ = _e351;
                }
                let _e353 = phi_1862_;
                if _e353 {
                    phi_2226_ = 1f;
                    break;
                }
                let _e355 = j.i2_;
                phi_2220_ = 0f;
                phi_2217_ = _e338;
                phi_2214_ = _e330;
                if (_e341 < _e355) {
                    let _e358 = (_e355 | (262144u + _e338));
                    let _e359 = atomicMax((&V0_.j2_[_e305]), _e358);
                    if (_e359 <= _e355) {
                        phi_2223_ = _e330;
                        phi_2218_ = _e338;
                        phi_2215_ = 0f;
                    } else {
                        phi_2224_ = 0f;
                        phi_2219_ = _e338;
                        phi_2216_ = _e330;
                        if (_e359 < _e358) {
                            let _e363 = ((_e359 & 524287u) - 262144u);
                            let _e365 = (f32(_e363) * 0.0009765625f);
                            phi_2224_ = ((_e330 - _e365) / max((1f - (_e365 * _e182.w)), 0.000062f));
                            phi_2219_ = _e363;
                            phi_2216_ = _e365;
                        }
                        let _e372 = phi_2224_;
                        let _e374 = phi_2219_;
                        let _e376 = phi_2216_;
                        phi_2223_ = _e372;
                        phi_2218_ = _e374;
                        phi_2215_ = _e376;
                    }
                    let _e378 = phi_2223_;
                    let _e380 = phi_2218_;
                    let _e382 = phi_2215_;
                    phi_2220_ = _e378;
                    phi_2217_ = _e380;
                    phi_2214_ = _e382;
                }
                let _e384 = phi_2220_;
                let _e386 = phi_2217_;
                let _e388 = phi_2214_;
                phi_2225_ = _e384;
                if (_e388 > 0f) {
                    let _e390 = atomicAdd((&V0_.j2_[_e305]), _e386);
                    let _e395 = (f32(bitcast<i32>(((_e390 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e397 = clamp(_e395, 0f, 1f);
                    phi_2225_ = (_e384 + ((1f - (_e384 * _e182.w)) * ((clamp((_e395 + _e388), 0f, 1f) - _e397) / max((1f - (_e397 * _e182.w)), 0.000062f))));
                }
                let _e409 = phi_2225_;
                phi_2226_ = _e409;
                break;
            }
        }
        let _e411 = phi_2226_;
        phi_2231_ = _e411;
    }
    let _e441 = phi_2231_;
    phi_2245_ = f32();
    if ii {
        let _e443 = j.M3_;
        let _e445 = j.N3_;
        if ii {
            phi_2229_ = ((fract((52.982918f * fract(((0.06711056f * _e74.x) + (0.00583715f * _e74.y))))) * _e443) + _e445);
        } else {
            phi_2229_ = 0f;
        }
        let _e457 = phi_2229_;
        phi_2245_ = _e457;
    }
    let _e459 = phi_2245_;
    let _e460 = (_e182 * _e441);
    let _e461 = _e460.xyz;
    if (ii && (_e460.w != 0f)) {
        phi_2275_ = (vec3(_e459) + _e461);
    } else {
        phi_2275_ = _e461;
    }
    let _e468 = phi_2275_;
    let _e474 = vec4<f32>(_e468.x, _e460.y, _e460.z, _e460.w);
    let _e480 = vec4<f32>(_e474.x, _e468.y, _e474.z, _e474.w);
    m0_.j2_[_e109] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    J1_ = vec4<f32>(_e480.x, _e480.y, _e468.z, _e480.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) F1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) F4_: vec2<f32>, @location(7) @interpolate(flat, either) q3_: vec2<u32>, @location(5) R0_: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @location(3) @interpolate(flat, either) F0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    F1_1 = F1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    S_1 = S;
    F4_1 = F4_;
    q3_1 = q3_;
    R0_1 = R0_;
    l1_1 = l1_;
    F0_1 = F0_;
    main_1();
    let _e21 = J1_;
    return _e21;
}
