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

struct Ef {
    v2_: array<u32>,
}

struct m0Ue {
    v2_: array<u32>,
}

struct Ef_1 {
    v2_: array<atomic<u32>>,
}

@id(8) override nj: bool = true;
@id(9) override oj: bool = true;
@id(2) override hj: bool = true;
@id(4) override jj: bool = true;
@id(3) override ij: bool = true;
@id(1) override gj: bool = true;
@id(0) override fj: bool = true;

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Y0_: Ef_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> V0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
var<private> T_1: vec4<f32>;
var<private> N4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> k2_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Ue;
var<private> Q1_: vec4<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_2377_: f32;
    var phi_2378_: f32;
    var phi_2379_: vec2<f32>;
    var phi_2396_: vec4<f32>;
    var phi_2395_: vec4<f32>;
    var phi_1410_: bool;
    var phi_1425_: bool;
    var phi_2380_: f32;
    var phi_2391_: vec4<f32>;
    var phi_2398_: vec4<f32>;
    var phi_2399_: vec4<f32>;
    var phi_1716_: bool;
    var phi_2400_: f32;
    var phi_2427_: f32;
    var phi_2428_: f32;
    var phi_1653_: bool;
    var phi_2429_: f32;
    var phi_2430_: f32;
    var phi_2432_: f32;
    var phi_1157_: bool;
    var phi_2433_: f32;
    var local: bool;
    var phi_2071_: bool;
    var phi_2073_: bool;
    var phi_2461_: f32;
    var phi_2456_: u32;
    var phi_2453_: f32;
    var phi_2460_: f32;
    var phi_2455_: u32;
    var phi_2452_: f32;
    var phi_2457_: f32;
    var phi_2454_: u32;
    var phi_2451_: f32;
    var phi_2462_: f32;
    var phi_2463_: f32;
    var phi_2464_: f32;
    var phi_2465_: f32;
    var phi_2468_: f32;
    var phi_2466_: f32;
    var phi_2482_: f32;
    var phi_2515_: vec3<f32>;

    let _e80 = gl_FragCoord_1;
    let _e84 = bitcast<vec2<u32>>(vec2<i32>(floor(_e80.xy)));
    let _e86 = j.T6_;
    let _e115 = bitcast<i32>((((((_e84.y >> bitcast<u32>(5u)) * (((_e86 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e84.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e84.x & 28u) << bitcast<u32>(5u)) + ((_e84.y & 28u) << bitcast<u32>(2i)))) + (((_e84.y & 3u) << bitcast<u32>(2i)) + (_e84.x & 3u))));
    let _e116 = Q0_1;
    let _e118 = V0_1;
    let _e119 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e122 = (hj && (u32(_e116) != 0u));
            if (_e119.w >= 0f) {
                phi_2395_ = _e119;
            } else {
                let _e126 = j.P8_;
                let _e128 = j.Q8_;
                let _e131 = abs(_e119.z);
                let _e132 = floor(_e131);
                let _e135 = (((_e131 - _e132) * 8f) + 0.0009765625f);
                let _e137 = ((_e132 * _e126) + _e128);
                let _e138 = floor(_e135);
                switch bitcast<i32>(0u) {
                    default: {
                        if (oj && (_e138 == 4f)) {
                            let _e157 = normalize(vec2<f32>(_e119.x, -(_e119.y)));
                            let _e160 = acos(clamp(_e157.x, -1f, 1f));
                            if (_e157.y >= 0f) {
                                phi_2378_ = _e160;
                            } else {
                                phi_2378_ = -(_e160);
                            }
                            let _e165 = phi_2378_;
                            phi_2379_ = vec2<f32>((_e165 * 0.15915494f), _e137);
                            break;
                        } else {
                            if (_e138 == 2f) {
                                phi_2377_ = _e119.x;
                            } else {
                                phi_2377_ = length(_e119.xy);
                            }
                            let _e147 = phi_2377_;
                            phi_2379_ = vec2<f32>(((clamp(_e147, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e119.z < 0f))) + _e135), _e137);
                            break;
                        }
                    }
                }
                let _e169 = phi_2379_;
                let _e170 = textureSampleLevel(ZC, R8_, _e169, 0f);
                phi_2396_ = _e170;
                if !(_e122) {
                    let _e176 = (_e170.xyz * _e170.w);
                    phi_2396_ = vec4<f32>(_e176.x, _e176.y, _e176.z, (_e170.w * (ceil(_e119.w) * -0.003921569f)));
                }
                let _e183 = phi_2396_;
                phi_2395_ = _e183;
            }
            let _e185 = phi_2395_;
            phi_1410_ = jj;
            if jj {
                phi_1410_ = (_e118.z < 0f);
            }
            let _e189 = phi_1410_;
            if _e189 {
                let _e191 = textureSampleLevel(UB, W4_, _e118.xy, 0f);
                phi_2399_ = _e191;
                break;
            }
            phi_1425_ = jj;
            if jj {
                phi_1425_ = (_e118.z > 0f);
            }
            let _e195 = phi_1425_;
            phi_2398_ = _e185;
            if _e195 {
                let _e199 = textureSampleLevel(UB, W4_, _e118.xy, (_e118.z - 1f));
                phi_2391_ = _e199;
                if _e122 {
                    if (_e199.w != 0f) {
                        phi_2380_ = (1f / _e199.w);
                    } else {
                        phi_2380_ = 0f;
                    }
                    let _e205 = phi_2380_;
                    let _e206 = (_e199.xyz * _e205);
                    phi_2391_ = vec4<f32>(_e206.x, _e206.y, _e206.z, _e199.w);
                }
                let _e212 = phi_2391_;
                phi_2398_ = (_e185 * _e212);
            }
            let _e215 = phi_2398_;
            phi_2399_ = _e215;
            break;
        }
    }
    let _e217 = phi_2399_;
    let _e218 = T_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e221 = (_e218.y >= 0f);
            local = _e221;
            if _e221 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1653_ = ij;
                        if ij {
                            phi_1653_ = (_e218.x < -1.5f);
                        }
                        let _e289 = phi_1653_;
                        if _e289 {
                            let _e295 = textureSampleLevel(AD, db, vec2<f32>((3f + _e218.x), 0f), 0f);
                            let _e300 = textureSampleLevel(AD, db, vec2<f32>((1f - _e218.y), 0f), 0f);
                            phi_2429_ = ((1f - _e295.x) - _e300.x);
                            break;
                        } else {
                            phi_2429_ = min(_e218.x, _e218.y);
                            break;
                        }
                    }
                }
                let _e304 = phi_2429_;
                phi_2430_ = _e304;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1716_ = ij;
                        if ij {
                            phi_1716_ = (_e218.y < -1.5f);
                        }
                        let _e225 = phi_1716_;
                        if _e225 {
                            let _e229 = max(_e218.w, 0f);
                            if (_e218.z >= 0f) {
                                let _e232 = textureSampleLevel(AD, db, vec2<f32>(_e229, 0f), 0f);
                                phi_2400_ = _e232.x;
                            } else {
                                phi_2400_ = 0f;
                            }
                            let _e235 = phi_2400_;
                            phi_2427_ = _e235;
                            if (abs(_e218.z) < 1000f) {
                                let _e241 = (-2f - _e218.y);
                                let _e243 = ((_e241 - _e229) * 0.5984134f);
                                let _e246 = (vec4(_e229) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e243));
                                let _e252 = ((_e246 * -(_e218.z)) + vec4(((_e241 * _e218.z) + (abs(_e218.x) - 0.25f))));
                                let _e255 = textureSampleLevel(AD, db, vec2<f32>(_e252.x, 0f), 0f);
                                let _e258 = textureSampleLevel(AD, db, vec2<f32>(_e252.y, 0f), 0f);
                                let _e261 = textureSampleLevel(AD, db, vec2<f32>(_e252.z, 0f), 0f);
                                let _e264 = textureSampleLevel(AD, db, vec2<f32>(_e252.w, 0f), 0f);
                                let _e270 = (_e246 * 5.0959306f);
                                phi_2427_ = (_e235 + (dot(vec4<f32>(_e255.x, _e258.x, _e261.x, _e264.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e270) * (_e270 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e243));
                            }
                            let _e279 = phi_2427_;
                            phi_2428_ = (_e279 * sign(_e218.x));
                            break;
                        } else {
                            phi_2428_ = _e218.x;
                            break;
                        }
                    }
                }
                let _e284 = phi_2428_;
                phi_2430_ = _e284;
                break;
            }
        }
    }
    let _e306 = phi_2430_;
    let _e307 = N4_1;
    let _e310 = A3_1[1u];
    let _e312 = A3_1[0u];
    let _e313 = vec2<u32>(floor(_e307));
    let _e340 = (_e312 + (((((_e313.y >> bitcast<u32>(5u)) * (_e310 << bitcast<u32>(5u))) + ((_e313.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e313.x & 28u) << bitcast<u32>(5u)) + ((_e313.y & 28u) << bitcast<u32>(2i)))) + (((_e313.y & 3u) << bitcast<u32>(2i)) + (_e313.x & 3u))));
    phi_2432_ = 1f;
    if gj {
        let _e341 = W0_1;
        let _e344 = min(_e341.xy, _e341.zw);
        phi_2432_ = min(min(_e344.x, _e344.y), 1f);
    }
    let _e350 = phi_2432_;
    phi_1157_ = fj;
    if fj {
        let _e352 = k2_1[0u];
        phi_1157_ = (_e352 != 0f);
    }
    let _e355 = phi_1157_;
    phi_2433_ = _e350;
    if _e355 {
        let _e358 = m0_.v2_[_e115];
        phi_2433_ = min(unpack4x8unorm(_e358).x, _e350);
    }
    let _e363 = phi_2433_;
    let _e365 = clamp(_e306, 0f, max(_e363, 0f));
    let _e367 = local;
    if _e367 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e217.w, _e365) >= 1f) {
                    phi_2465_ = 1f;
                    break;
                }
                let _e458 = j.r2_;
                let _e460 = atomicMax((&Y0_.v2_[_e340]), (_e458 | u32(((abs(_e365) * 1024f) + 0.5f))));
                if (_e460 < _e458) {
                    phi_2464_ = _e365;
                } else {
                    let _e464 = (f32((_e460 & 524287u)) * 0.0009765625f);
                    phi_2464_ = ((max(_e464, _e365) - _e464) / max((1f - (_e464 * _e217.w)), 0.000062f));
                }
                let _e472 = phi_2464_;
                phi_2465_ = _e472;
                break;
            }
        }
        let _e474 = phi_2465_;
        phi_2468_ = _e474;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e373 = u32(((abs(_e365) * 1024f) + 0.5f));
                let _e376 = atomicLoad((&Y0_.v2_[_e340]));
                let _e378 = (min(_e217.w, _e365) >= 1f);
                phi_2073_ = _e378;
                if _e378 {
                    let _e380 = j.r2_;
                    let _e381 = (_e376 < _e380);
                    phi_2071_ = _e381;
                    if !(_e381) {
                        phi_2071_ = (_e376 >= (_e380 | 262144u));
                    }
                    let _e386 = phi_2071_;
                    phi_2073_ = _e386;
                }
                let _e388 = phi_2073_;
                if _e388 {
                    phi_2463_ = 1f;
                    break;
                }
                let _e390 = j.r2_;
                phi_2457_ = 0f;
                phi_2454_ = _e373;
                phi_2451_ = _e365;
                if (_e376 < _e390) {
                    let _e393 = (_e390 | (262144u + _e373));
                    let _e394 = atomicMax((&Y0_.v2_[_e340]), _e393);
                    if (_e394 <= _e390) {
                        phi_2460_ = _e365;
                        phi_2455_ = _e373;
                        phi_2452_ = 0f;
                    } else {
                        phi_2461_ = 0f;
                        phi_2456_ = _e373;
                        phi_2453_ = _e365;
                        if (_e394 < _e393) {
                            let _e398 = ((_e394 & 524287u) - 262144u);
                            let _e400 = (f32(_e398) * 0.0009765625f);
                            phi_2461_ = ((_e365 - _e400) / max((1f - (_e400 * _e217.w)), 0.000062f));
                            phi_2456_ = _e398;
                            phi_2453_ = _e400;
                        }
                        let _e407 = phi_2461_;
                        let _e409 = phi_2456_;
                        let _e411 = phi_2453_;
                        phi_2460_ = _e407;
                        phi_2455_ = _e409;
                        phi_2452_ = _e411;
                    }
                    let _e413 = phi_2460_;
                    let _e415 = phi_2455_;
                    let _e417 = phi_2452_;
                    phi_2457_ = _e413;
                    phi_2454_ = _e415;
                    phi_2451_ = _e417;
                }
                let _e419 = phi_2457_;
                let _e421 = phi_2454_;
                let _e423 = phi_2451_;
                phi_2462_ = _e419;
                if (_e423 > 0f) {
                    let _e425 = atomicAdd((&Y0_.v2_[_e340]), _e421);
                    let _e430 = (f32(bitcast<i32>(((_e425 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e432 = clamp(_e430, 0f, 1f);
                    phi_2462_ = (_e419 + ((1f - (_e419 * _e217.w)) * ((clamp((_e430 + _e423), 0f, 1f) - _e432) / max((1f - (_e432 * _e217.w)), 0.000062f))));
                }
                let _e444 = phi_2462_;
                phi_2463_ = _e444;
                break;
            }
        }
        let _e446 = phi_2463_;
        phi_2468_ = _e446;
    }
    let _e476 = phi_2468_;
    phi_2482_ = f32();
    if nj {
        let _e478 = j.G3_;
        let _e480 = j.H3_;
        if nj {
            phi_2466_ = ((fract((52.982918f * fract(((0.06711056f * _e80.x) + (0.00583715f * _e80.y))))) * _e478) + _e480);
        } else {
            phi_2466_ = 0f;
        }
        let _e492 = phi_2466_;
        phi_2482_ = _e492;
    }
    let _e494 = phi_2482_;
    let _e495 = (_e217 * _e476);
    let _e496 = _e495.xyz;
    if (nj && (_e495.w != 0f)) {
        phi_2515_ = (vec3(_e494) + _e496);
    } else {
        phi_2515_ = _e496;
    }
    let _e503 = phi_2515_;
    let _e509 = vec4<f32>(_e503.x, _e495.y, _e495.z, _e495.w);
    let _e515 = vec4<f32>(_e509.x, _e503.y, _e509.z, _e509.w);
    m0_.v2_[_e115] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    Q1_ = vec4<f32>(_e515.x, _e515.y, _e503.z, _e515.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(2) T: vec4<f32>, @location(8) N4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    V0_1 = V0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    T_1 = T;
    N4_1 = N4_;
    A3_1 = A3_;
    W0_1 = W0_;
    k2_1 = k2_;
    G0_1 = G0_;
    main_1();
    let _e21 = Q1_;
    return _e21;
}
