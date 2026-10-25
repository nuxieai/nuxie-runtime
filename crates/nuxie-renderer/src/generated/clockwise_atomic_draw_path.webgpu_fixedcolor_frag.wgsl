struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    H8_: vec4<i32>,
    Fi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ji: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Ci: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
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

@id(7) override lj: bool = true;
@id(2) override gj: bool = true;
@id(8) override mj: bool = true;
@id(3) override hj: bool = true;
@id(1) override fj: bool = true;
@id(0) override ej: bool = true;

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
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
    var phi_2374_: f32;
    var phi_2375_: f32;
    var phi_2376_: vec2<f32>;
    var phi_2393_: vec4<f32>;
    var phi_2392_: vec4<f32>;
    var phi_1408_: bool;
    var phi_1423_: bool;
    var phi_2377_: f32;
    var phi_2388_: vec4<f32>;
    var phi_2395_: vec4<f32>;
    var phi_2396_: vec4<f32>;
    var phi_1713_: bool;
    var phi_2397_: f32;
    var phi_2424_: f32;
    var phi_2425_: f32;
    var phi_1650_: bool;
    var phi_2426_: f32;
    var phi_2427_: f32;
    var phi_2429_: f32;
    var phi_1155_: bool;
    var phi_2430_: f32;
    var local: bool;
    var phi_2068_: bool;
    var phi_2070_: bool;
    var phi_2458_: f32;
    var phi_2453_: u32;
    var phi_2450_: f32;
    var phi_2457_: f32;
    var phi_2452_: u32;
    var phi_2449_: f32;
    var phi_2454_: f32;
    var phi_2451_: u32;
    var phi_2448_: f32;
    var phi_2459_: f32;
    var phi_2460_: f32;
    var phi_2461_: f32;
    var phi_2462_: f32;
    var phi_2465_: f32;
    var phi_2463_: f32;
    var phi_2479_: f32;
    var phi_2512_: vec3<f32>;

    let _e79 = gl_FragCoord_1;
    let _e83 = bitcast<vec2<u32>>(vec2<i32>(floor(_e79.xy)));
    let _e85 = j.T6_;
    let _e114 = bitcast<i32>((((((_e83.y >> bitcast<u32>(5u)) * (((_e85 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e83.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e83.x & 28u) << bitcast<u32>(5u)) + ((_e83.y & 28u) << bitcast<u32>(2i)))) + (((_e83.y & 3u) << bitcast<u32>(2i)) + (_e83.x & 3u))));
    let _e115 = Q0_1;
    let _e117 = V0_1;
    let _e118 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e121 = (gj && (u32(_e115) != 0u));
            if (_e118.w >= 0f) {
                phi_2392_ = _e118;
            } else {
                let _e125 = j.O8_;
                let _e127 = j.P8_;
                let _e130 = abs(_e118.z);
                let _e131 = floor(_e130);
                let _e134 = (((_e130 - _e131) * 8f) + 0.0009765625f);
                let _e136 = ((_e131 * _e125) + _e127);
                let _e137 = floor(_e134);
                switch bitcast<i32>(0u) {
                    default: {
                        if (_e137 == 4f) {
                            let _e155 = normalize(vec2<f32>(_e118.x, -(_e118.y)));
                            let _e158 = acos(clamp(_e155.x, -1f, 1f));
                            if (_e155.y >= 0f) {
                                phi_2375_ = _e158;
                            } else {
                                phi_2375_ = -(_e158);
                            }
                            let _e163 = phi_2375_;
                            phi_2376_ = vec2<f32>((_e163 * 0.15915507f), _e136);
                            break;
                        } else {
                            if (_e137 == 2f) {
                                phi_2374_ = _e118.x;
                            } else {
                                phi_2374_ = length(_e118.xy);
                            }
                            let _e145 = phi_2374_;
                            phi_2376_ = vec2<f32>(((clamp(_e145, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e118.z < 0f))) + _e134), _e136);
                            break;
                        }
                    }
                }
                let _e167 = phi_2376_;
                let _e168 = textureSampleLevel(ZC, Q8_, _e167, 0f);
                phi_2393_ = _e168;
                if !(_e121) {
                    let _e174 = (_e168.xyz * _e168.w);
                    phi_2393_ = vec4<f32>(_e174.x, _e174.y, _e174.z, (_e168.w * (ceil(_e118.w) * -0.003921569f)));
                }
                let _e181 = phi_2393_;
                phi_2392_ = _e181;
            }
            let _e183 = phi_2392_;
            phi_1408_ = mj;
            if mj {
                phi_1408_ = (_e117.z < 0f);
            }
            let _e187 = phi_1408_;
            if _e187 {
                let _e189 = textureSampleLevel(UB, W4_, _e117.xy, 0f);
                phi_2396_ = _e189;
                break;
            }
            phi_1423_ = mj;
            if mj {
                phi_1423_ = (_e117.z > 0f);
            }
            let _e193 = phi_1423_;
            phi_2395_ = _e183;
            if _e193 {
                let _e197 = textureSampleLevel(UB, W4_, _e117.xy, (_e117.z - 1f));
                phi_2388_ = _e197;
                if _e121 {
                    if (_e197.w != 0f) {
                        phi_2377_ = (1f / _e197.w);
                    } else {
                        phi_2377_ = 0f;
                    }
                    let _e203 = phi_2377_;
                    let _e204 = (_e197.xyz * _e203);
                    phi_2388_ = vec4<f32>(_e204.x, _e204.y, _e204.z, _e197.w);
                }
                let _e210 = phi_2388_;
                phi_2395_ = (_e183 * _e210);
            }
            let _e213 = phi_2395_;
            phi_2396_ = _e213;
            break;
        }
    }
    let _e215 = phi_2396_;
    let _e216 = T_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e219 = (_e216.y >= 0f);
            local = _e219;
            if _e219 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1650_ = hj;
                        if hj {
                            phi_1650_ = (_e216.x < -1.5f);
                        }
                        let _e287 = phi_1650_;
                        if _e287 {
                            let _e293 = textureSampleLevel(AD, db, vec2<f32>((3f + _e216.x), 0f), 0f);
                            let _e298 = textureSampleLevel(AD, db, vec2<f32>((1f - _e216.y), 0f), 0f);
                            phi_2426_ = ((1f - _e293.x) - _e298.x);
                            break;
                        } else {
                            phi_2426_ = min(_e216.x, _e216.y);
                            break;
                        }
                    }
                }
                let _e302 = phi_2426_;
                phi_2427_ = _e302;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1713_ = hj;
                        if hj {
                            phi_1713_ = (_e216.y < -1.5f);
                        }
                        let _e223 = phi_1713_;
                        if _e223 {
                            let _e227 = max(_e216.w, 0f);
                            if (_e216.z >= 0f) {
                                let _e230 = textureSampleLevel(AD, db, vec2<f32>(_e227, 0f), 0f);
                                phi_2397_ = _e230.x;
                            } else {
                                phi_2397_ = 0f;
                            }
                            let _e233 = phi_2397_;
                            phi_2424_ = _e233;
                            if (abs(_e216.z) < 1000f) {
                                let _e239 = (-2f - _e216.y);
                                let _e241 = ((_e239 - _e227) * 0.5984134f);
                                let _e244 = (vec4(_e227) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e241));
                                let _e250 = ((_e244 * -(_e216.z)) + vec4(((_e239 * _e216.z) + (abs(_e216.x) - 0.25f))));
                                let _e253 = textureSampleLevel(AD, db, vec2<f32>(_e250.x, 0f), 0f);
                                let _e256 = textureSampleLevel(AD, db, vec2<f32>(_e250.y, 0f), 0f);
                                let _e259 = textureSampleLevel(AD, db, vec2<f32>(_e250.z, 0f), 0f);
                                let _e262 = textureSampleLevel(AD, db, vec2<f32>(_e250.w, 0f), 0f);
                                let _e268 = (_e244 * 5.0959306f);
                                phi_2424_ = (_e233 + (dot(vec4<f32>(_e253.x, _e256.x, _e259.x, _e262.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e268) * (_e268 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e241));
                            }
                            let _e277 = phi_2424_;
                            phi_2425_ = (_e277 * sign(_e216.x));
                            break;
                        } else {
                            phi_2425_ = _e216.x;
                            break;
                        }
                    }
                }
                let _e282 = phi_2425_;
                phi_2427_ = _e282;
                break;
            }
        }
    }
    let _e304 = phi_2427_;
    let _e305 = N4_1;
    let _e308 = A3_1[1u];
    let _e310 = A3_1[0u];
    let _e311 = vec2<u32>(floor(_e305));
    let _e338 = (_e310 + (((((_e311.y >> bitcast<u32>(5u)) * (_e308 << bitcast<u32>(5u))) + ((_e311.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e311.x & 28u) << bitcast<u32>(5u)) + ((_e311.y & 28u) << bitcast<u32>(2i)))) + (((_e311.y & 3u) << bitcast<u32>(2i)) + (_e311.x & 3u))));
    phi_2429_ = 1f;
    if fj {
        let _e339 = W0_1;
        let _e342 = min(_e339.xy, _e339.zw);
        phi_2429_ = min(min(_e342.x, _e342.y), 1f);
    }
    let _e348 = phi_2429_;
    phi_1155_ = ej;
    if ej {
        let _e350 = k2_1[0u];
        phi_1155_ = (_e350 != 0f);
    }
    let _e353 = phi_1155_;
    phi_2430_ = _e348;
    if _e353 {
        let _e356 = m0_.v2_[_e114];
        phi_2430_ = min(unpack4x8unorm(_e356).x, _e348);
    }
    let _e361 = phi_2430_;
    let _e363 = clamp(_e304, 0f, max(_e361, 0f));
    let _e365 = local;
    if _e365 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e215.w, _e363) >= 1f) {
                    phi_2462_ = 1f;
                    break;
                }
                let _e456 = j.r2_;
                let _e458 = atomicMax((&Y0_.v2_[_e338]), (_e456 | u32(((abs(_e363) * 1024f) + 0.5f))));
                if (_e458 < _e456) {
                    phi_2461_ = _e363;
                } else {
                    let _e462 = (f32((_e458 & 524287u)) * 0.0009765625f);
                    phi_2461_ = ((max(_e462, _e363) - _e462) / max((1f - (_e462 * _e215.w)), 0.000062f));
                }
                let _e470 = phi_2461_;
                phi_2462_ = _e470;
                break;
            }
        }
        let _e472 = phi_2462_;
        phi_2465_ = _e472;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e371 = u32(((abs(_e363) * 1024f) + 0.5f));
                let _e374 = atomicLoad((&Y0_.v2_[_e338]));
                let _e376 = (min(_e215.w, _e363) >= 1f);
                phi_2070_ = _e376;
                if _e376 {
                    let _e378 = j.r2_;
                    let _e379 = (_e374 < _e378);
                    phi_2068_ = _e379;
                    if !(_e379) {
                        phi_2068_ = (_e374 >= (_e378 | 262144u));
                    }
                    let _e384 = phi_2068_;
                    phi_2070_ = _e384;
                }
                let _e386 = phi_2070_;
                if _e386 {
                    phi_2460_ = 1f;
                    break;
                }
                let _e388 = j.r2_;
                phi_2454_ = 0f;
                phi_2451_ = _e371;
                phi_2448_ = _e363;
                if (_e374 < _e388) {
                    let _e391 = (_e388 | (262144u + _e371));
                    let _e392 = atomicMax((&Y0_.v2_[_e338]), _e391);
                    if (_e392 <= _e388) {
                        phi_2457_ = _e363;
                        phi_2452_ = _e371;
                        phi_2449_ = 0f;
                    } else {
                        phi_2458_ = 0f;
                        phi_2453_ = _e371;
                        phi_2450_ = _e363;
                        if (_e392 < _e391) {
                            let _e396 = ((_e392 & 524287u) - 262144u);
                            let _e398 = (f32(_e396) * 0.0009765625f);
                            phi_2458_ = ((_e363 - _e398) / max((1f - (_e398 * _e215.w)), 0.000062f));
                            phi_2453_ = _e396;
                            phi_2450_ = _e398;
                        }
                        let _e405 = phi_2458_;
                        let _e407 = phi_2453_;
                        let _e409 = phi_2450_;
                        phi_2457_ = _e405;
                        phi_2452_ = _e407;
                        phi_2449_ = _e409;
                    }
                    let _e411 = phi_2457_;
                    let _e413 = phi_2452_;
                    let _e415 = phi_2449_;
                    phi_2454_ = _e411;
                    phi_2451_ = _e413;
                    phi_2448_ = _e415;
                }
                let _e417 = phi_2454_;
                let _e419 = phi_2451_;
                let _e421 = phi_2448_;
                phi_2459_ = _e417;
                if (_e421 > 0f) {
                    let _e423 = atomicAdd((&Y0_.v2_[_e338]), _e419);
                    let _e428 = (f32(bitcast<i32>(((_e423 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e430 = clamp(_e428, 0f, 1f);
                    phi_2459_ = (_e417 + ((1f - (_e417 * _e215.w)) * ((clamp((_e428 + _e421), 0f, 1f) - _e430) / max((1f - (_e430 * _e215.w)), 0.000062f))));
                }
                let _e442 = phi_2459_;
                phi_2460_ = _e442;
                break;
            }
        }
        let _e444 = phi_2460_;
        phi_2465_ = _e444;
    }
    let _e474 = phi_2465_;
    phi_2479_ = f32();
    if lj {
        let _e476 = j.G3_;
        let _e478 = j.H3_;
        if lj {
            phi_2463_ = ((fract((52.982918f * fract(((0.06711056f * _e79.x) + (0.00583715f * _e79.y))))) * _e476) + _e478);
        } else {
            phi_2463_ = 0f;
        }
        let _e490 = phi_2463_;
        phi_2479_ = _e490;
    }
    let _e492 = phi_2479_;
    let _e493 = (_e215 * _e474);
    let _e494 = _e493.xyz;
    if (lj && (_e493.w != 0f)) {
        phi_2512_ = (vec3(_e492) + _e494);
    } else {
        phi_2512_ = _e494;
    }
    let _e501 = phi_2512_;
    let _e507 = vec4<f32>(_e501.x, _e493.y, _e493.z, _e493.w);
    let _e513 = vec4<f32>(_e507.x, _e501.y, _e507.z, _e507.w);
    m0_.v2_[_e114] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    Q1_ = vec4<f32>(_e513.x, _e513.y, _e501.z, _e513.w);
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
