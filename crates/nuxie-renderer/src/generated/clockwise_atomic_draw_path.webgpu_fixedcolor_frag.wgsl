struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct Ae {
    g2_: array<u32>,
}

struct i0Sd {
    g2_: array<u32>,
}

struct Ae_1 {
    g2_: array<atomic<u32>>,
}

@id(7) override Lh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;
@id(3) override Hh: bool = true;
@id(1) override Fh: bool = true;
@id(0) override Eh: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(0)
var<uniform> j: SB;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var M9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(6)
var<storage, read_write> S0_: Ae_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> O_1: vec4<f32>;
var<private> v4_1: vec2<f32>;
var<private> k3_1: vec2<u32>;
var<private> O0_1: vec4<f32>;
var<private> Y1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Sd;
var<private> F1_: vec4<f32>;
var<private> D0_1: f32;

fn main_1() {
    var phi_2152_: f32;
    var phi_2153_: f32;
    var phi_2167_: vec4<f32>;
    var phi_2166_: vec4<f32>;
    var phi_1300_: bool;
    var phi_2154_: f32;
    var phi_2163_: vec4<f32>;
    var phi_2169_: vec4<f32>;
    var phi_1506_: bool;
    var phi_2170_: f32;
    var phi_2191_: f32;
    var phi_2192_: f32;
    var phi_1443_: bool;
    var phi_2193_: f32;
    var phi_2194_: f32;
    var phi_2196_: f32;
    var phi_1051_: bool;
    var phi_2197_: f32;
    var local: bool;
    var phi_1861_: bool;
    var phi_1863_: bool;
    var phi_2225_: f32;
    var phi_2220_: u32;
    var phi_2217_: f32;
    var phi_2224_: f32;
    var phi_2219_: u32;
    var phi_2216_: f32;
    var phi_2221_: f32;
    var phi_2218_: u32;
    var phi_2215_: f32;
    var phi_2226_: f32;
    var phi_2227_: f32;
    var phi_2228_: f32;
    var phi_2229_: f32;
    var phi_2232_: f32;
    var phi_2230_: f32;
    var phi_2246_: f32;
    var phi_2276_: vec3<f32>;

    let _e74 = gl_FragCoord_1;
    let _e78 = bitcast<vec2<u32>>(vec2<i32>(floor(_e74.xy)));
    let _e80 = j.n6_;
    let _e109 = bitcast<i32>((((((_e78.y >> bitcast<u32>(5u)) * (((_e80 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e78.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e78.x & 28u) << bitcast<u32>(5u)) + ((_e78.y & 28u) << bitcast<u32>(2i)))) + (((_e78.y & 3u) << bitcast<u32>(2i)) + (_e78.x & 3u))));
    let _e110 = g1_1;
    let _e112 = C2_1;
    let _e113 = X1_1;
    let _e115 = (Gh && (u32(_e110) != 0u));
    if (_e113.w >= 0f) {
        phi_2166_ = _e113;
    } else {
        let _e118 = -(_e113.w);
        let _e123 = j.Zb;
        let _e126 = j.ac;
        if (_e113.z > 0f) {
            phi_2152_ = _e113.x;
        } else {
            phi_2152_ = length(_e113.xy);
        }
        let _e134 = phi_2152_;
        let _e135 = clamp(_e134, 0f, 1f);
        let _e136 = abs(_e113.z);
        if (_e136 > 1f) {
            phi_2153_ = ((0.9980469f * _e135) + 0.0009765625f);
        } else {
            phi_2153_ = ((0.001953125f * _e135) + _e136);
        }
        let _e143 = phi_2153_;
        let _e145 = textureSampleLevel(DD, M9_, vec2<f32>(_e143, ((floor(_e118) * _e123) + _e126)), 0f);
        phi_2167_ = _e145;
        if !(_e115) {
            let _e149 = (_e145.xyz * _e145.w);
            phi_2167_ = vec4<f32>(_e149.x, _e149.y, _e149.z, (_e145.w * (fract(_e118) * 1.0039216f)));
        }
        let _e156 = phi_2167_;
        phi_2166_ = _e156;
    }
    let _e158 = phi_2166_;
    phi_1300_ = Mh;
    if Mh {
        phi_1300_ = (_e112.z > 0f);
    }
    let _e162 = phi_1300_;
    phi_2169_ = _e158;
    if _e162 {
        let _e166 = textureSampleLevel(GC, W5_, _e112.xy, (_e112.z - 1f));
        phi_2163_ = _e166;
        if _e115 {
            if (_e166.w != 0f) {
                phi_2154_ = (1f / _e166.w);
            } else {
                phi_2154_ = 0f;
            }
            let _e172 = phi_2154_;
            let _e173 = (_e166.xyz * _e172);
            phi_2163_ = vec4<f32>(_e173.x, _e173.y, _e173.z, _e166.w);
        }
        let _e179 = phi_2163_;
        phi_2169_ = (_e158 * _e179);
    }
    let _e182 = phi_2169_;
    let _e183 = O_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e186 = (_e183.y >= 0f);
            local = _e186;
            if _e186 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1443_ = Hh;
                        if Hh {
                            phi_1443_ = (_e183.x < -1.5f);
                        }
                        let _e254 = phi_1443_;
                        if _e254 {
                            let _e260 = textureSampleLevel(XC, ca, vec2<f32>((3f + _e183.x), 0f), 0f);
                            let _e265 = textureSampleLevel(XC, ca, vec2<f32>((1f - _e183.y), 0f), 0f);
                            phi_2193_ = ((1f - _e260.x) - _e265.x);
                            break;
                        } else {
                            phi_2193_ = min(_e183.x, _e183.y);
                            break;
                        }
                    }
                }
                let _e269 = phi_2193_;
                phi_2194_ = _e269;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1506_ = Hh;
                        if Hh {
                            phi_1506_ = (_e183.y < -1.5f);
                        }
                        let _e190 = phi_1506_;
                        if _e190 {
                            let _e194 = max(_e183.w, 0f);
                            if (_e183.z >= 0f) {
                                let _e197 = textureSampleLevel(XC, ca, vec2<f32>(_e194, 0f), 0f);
                                phi_2170_ = _e197.x;
                            } else {
                                phi_2170_ = 0f;
                            }
                            let _e200 = phi_2170_;
                            phi_2191_ = _e200;
                            if (abs(_e183.z) < 1000f) {
                                let _e206 = (-2f - _e183.y);
                                let _e208 = ((_e206 - _e194) * 0.5984134f);
                                let _e211 = (vec4(_e194) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e208));
                                let _e217 = ((_e211 * -(_e183.z)) + vec4(((_e206 * _e183.z) + (abs(_e183.x) - 0.25f))));
                                let _e220 = textureSampleLevel(XC, ca, vec2<f32>(_e217.x, 0f), 0f);
                                let _e223 = textureSampleLevel(XC, ca, vec2<f32>(_e217.y, 0f), 0f);
                                let _e226 = textureSampleLevel(XC, ca, vec2<f32>(_e217.z, 0f), 0f);
                                let _e229 = textureSampleLevel(XC, ca, vec2<f32>(_e217.w, 0f), 0f);
                                let _e235 = (_e211 * 5.0959306f);
                                phi_2191_ = (_e200 + (dot(vec4<f32>(_e220.x, _e223.x, _e226.x, _e229.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e235) * (_e235 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e208));
                            }
                            let _e244 = phi_2191_;
                            phi_2192_ = (_e244 * sign(_e183.x));
                            break;
                        } else {
                            phi_2192_ = _e183.x;
                            break;
                        }
                    }
                }
                let _e249 = phi_2192_;
                phi_2194_ = _e249;
                break;
            }
        }
    }
    let _e271 = phi_2194_;
    let _e272 = v4_1;
    let _e275 = k3_1[1u];
    let _e277 = k3_1[0u];
    let _e278 = vec2<u32>(floor(_e272));
    let _e305 = (_e277 + (((((_e278.y >> bitcast<u32>(5u)) * (_e275 << bitcast<u32>(5u))) + ((_e278.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e278.x & 28u) << bitcast<u32>(5u)) + ((_e278.y & 28u) << bitcast<u32>(2i)))) + (((_e278.y & 3u) << bitcast<u32>(2i)) + (_e278.x & 3u))));
    phi_2196_ = 1f;
    if Fh {
        let _e306 = O0_1;
        let _e309 = min(_e306.xy, _e306.zw);
        phi_2196_ = min(min(_e309.x, _e309.y), 1f);
    }
    let _e315 = phi_2196_;
    phi_1051_ = Eh;
    if Eh {
        let _e317 = Y1_1[0u];
        phi_1051_ = (_e317 != 0f);
    }
    let _e320 = phi_1051_;
    phi_2197_ = _e315;
    if _e320 {
        let _e323 = i0_.g2_[_e109];
        phi_2197_ = min(unpack4x8unorm(_e323).x, _e315);
    }
    let _e328 = phi_2197_;
    let _e330 = clamp(_e271, 0f, max(_e328, 0f));
    let _e332 = local;
    if _e332 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e182.w, _e330) >= 1f) {
                    phi_2229_ = 1f;
                    break;
                }
                let _e423 = j.f2_;
                let _e425 = atomicMax((&S0_.g2_[_e305]), (_e423 | u32(((abs(_e330) * 1024f) + 0.5f))));
                if (_e425 < _e423) {
                    phi_2228_ = _e330;
                } else {
                    let _e429 = (f32((_e425 & 524287u)) * 0.0009765625f);
                    phi_2228_ = ((max(_e429, _e330) - _e429) / max((1f - (_e429 * _e182.w)), 0.000062f));
                }
                let _e437 = phi_2228_;
                phi_2229_ = _e437;
                break;
            }
        }
        let _e439 = phi_2229_;
        phi_2232_ = _e439;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e338 = u32(((abs(_e330) * 1024f) + 0.5f));
                let _e341 = atomicLoad((&S0_.g2_[_e305]));
                let _e343 = (min(_e182.w, _e330) >= 1f);
                phi_1863_ = _e343;
                if _e343 {
                    let _e345 = j.f2_;
                    let _e346 = (_e341 < _e345);
                    phi_1861_ = _e346;
                    if !(_e346) {
                        phi_1861_ = (_e341 >= (_e345 | 262144u));
                    }
                    let _e351 = phi_1861_;
                    phi_1863_ = _e351;
                }
                let _e353 = phi_1863_;
                if _e353 {
                    phi_2227_ = 1f;
                    break;
                }
                let _e355 = j.f2_;
                phi_2221_ = 0f;
                phi_2218_ = _e338;
                phi_2215_ = _e330;
                if (_e341 < _e355) {
                    let _e358 = (_e355 | (262144u + _e338));
                    let _e359 = atomicMax((&S0_.g2_[_e305]), _e358);
                    if (_e359 <= _e355) {
                        phi_2224_ = _e330;
                        phi_2219_ = _e338;
                        phi_2216_ = 0f;
                    } else {
                        phi_2225_ = 0f;
                        phi_2220_ = _e338;
                        phi_2217_ = _e330;
                        if (_e359 < _e358) {
                            let _e363 = ((_e359 & 524287u) - 262144u);
                            let _e365 = (f32(_e363) * 0.0009765625f);
                            phi_2225_ = ((_e330 - _e365) / max((1f - (_e365 * _e182.w)), 0.000062f));
                            phi_2220_ = _e363;
                            phi_2217_ = _e365;
                        }
                        let _e372 = phi_2225_;
                        let _e374 = phi_2220_;
                        let _e376 = phi_2217_;
                        phi_2224_ = _e372;
                        phi_2219_ = _e374;
                        phi_2216_ = _e376;
                    }
                    let _e378 = phi_2224_;
                    let _e380 = phi_2219_;
                    let _e382 = phi_2216_;
                    phi_2221_ = _e378;
                    phi_2218_ = _e380;
                    phi_2215_ = _e382;
                }
                let _e384 = phi_2221_;
                let _e386 = phi_2218_;
                let _e388 = phi_2215_;
                phi_2226_ = _e384;
                if (_e388 > 0f) {
                    let _e390 = atomicAdd((&S0_.g2_[_e305]), _e386);
                    let _e395 = (f32(bitcast<i32>(((_e390 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e397 = clamp(_e395, 0f, 1f);
                    phi_2226_ = (_e384 + ((1f - (_e384 * _e182.w)) * ((clamp((_e395 + _e388), 0f, 1f) - _e397) / max((1f - (_e397 * _e182.w)), 0.000062f))));
                }
                let _e409 = phi_2226_;
                phi_2227_ = _e409;
                break;
            }
        }
        let _e411 = phi_2227_;
        phi_2232_ = _e411;
    }
    let _e441 = phi_2232_;
    phi_2246_ = f32();
    if Lh {
        let _e443 = j.F3_;
        let _e445 = j.G3_;
        if Lh {
            phi_2230_ = ((fract((52.982918f * fract(((0.06711056f * _e74.x) + (0.00583715f * _e74.y))))) * _e443) + _e445);
        } else {
            phi_2230_ = 0f;
        }
        let _e457 = phi_2230_;
        phi_2246_ = _e457;
    }
    let _e459 = phi_2246_;
    let _e460 = (_e182 * _e441);
    let _e461 = _e460.xyz;
    if (Lh && (_e460.w != 0f)) {
        phi_2276_ = (vec3(_e459) + _e461);
    } else {
        phi_2276_ = _e461;
    }
    let _e468 = phi_2276_;
    let _e474 = vec4<f32>(_e468.x, _e460.y, _e460.z, _e460.w);
    let _e480 = vec4<f32>(_e474.x, _e468.y, _e474.z, _e474.w);
    i0_.g2_[_e109] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    F1_ = vec4<f32>(_e480.x, _e480.y, _e468.z, _e480.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @location(2) O: vec4<f32>, @location(8) v4_: vec2<f32>, @location(7) @interpolate(flat, either) k3_: vec2<u32>, @location(5) O0_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @location(3) @interpolate(flat, either) D0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    O_1 = O;
    v4_1 = v4_;
    k3_1 = k3_;
    O0_1 = O0_;
    Y1_1 = Y1_;
    D0_1 = D0_;
    main_1();
    let _e21 = F1_;
    return _e21;
}
