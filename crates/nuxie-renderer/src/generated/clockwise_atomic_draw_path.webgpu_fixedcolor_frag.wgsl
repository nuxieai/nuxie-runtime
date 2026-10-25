struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct Kf {
    v2_: array<u32>,
}

struct m0cf {
    v2_: array<u32>,
}

struct Kf_1 {
    v2_: array<atomic<u32>>,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;
@id(3) override mj: bool = true;
@id(1) override kj: bool = true;
@id(0) override jj: bool = true;

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Y0_: Kf_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> O0_1: vec3<f32>;
var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
var<private> T_1: vec4<f32>;
var<private> P4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> k2_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
var<private> Q1_: vec4<f32>;
var<private> I0_1: f32;

fn main_1() {
    var phi_2365_: f32;
    var phi_2366_: f32;
    var phi_2367_: vec2<f32>;
    var phi_2384_: vec4<f32>;
    var phi_2383_: vec4<f32>;
    var phi_1405_: bool;
    var phi_1420_: bool;
    var phi_2368_: f32;
    var phi_2379_: vec4<f32>;
    var phi_2386_: vec4<f32>;
    var phi_2387_: vec4<f32>;
    var phi_1706_: bool;
    var phi_2388_: f32;
    var phi_2415_: f32;
    var phi_2416_: f32;
    var phi_1643_: bool;
    var phi_2417_: f32;
    var phi_2418_: f32;
    var phi_2420_: f32;
    var phi_1152_: bool;
    var phi_2421_: f32;
    var local: bool;
    var phi_2061_: bool;
    var phi_2063_: bool;
    var phi_2449_: f32;
    var phi_2444_: u32;
    var phi_2441_: f32;
    var phi_2448_: f32;
    var phi_2443_: u32;
    var phi_2440_: f32;
    var phi_2445_: f32;
    var phi_2442_: u32;
    var phi_2439_: f32;
    var phi_2450_: f32;
    var phi_2451_: f32;
    var phi_2452_: f32;
    var phi_2453_: f32;
    var phi_2456_: f32;
    var phi_2454_: f32;
    var phi_2470_: f32;
    var phi_2503_: vec3<f32>;

    let _e80 = gl_FragCoord_1;
    let _e84 = bitcast<vec2<u32>>(vec2<i32>(floor(_e80.xy)));
    let _e86 = j.T6_;
    let _e115 = bitcast<i32>((((((_e84.y >> bitcast<u32>(5u)) * (((_e86 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e84.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e84.x & 28u) << bitcast<u32>(5u)) + ((_e84.y & 28u) << bitcast<u32>(2i)))) + (((_e84.y & 3u) << bitcast<u32>(2i)) + (_e84.x & 3u))));
    let _e116 = S0_1;
    let _e118 = O0_1;
    let _e119 = N0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e122 = (lj && (u32(_e116) != 0u));
            if (_e119.w >= 0f) {
                phi_2383_ = _e119;
            } else {
                let _e126 = j.Q8_;
                let _e128 = j.R8_;
                let _e131 = abs(_e119.z);
                let _e132 = floor(_e131);
                let _e135 = (((_e131 - _e132) * 8f) + 0.0009765625f);
                let _e137 = ((_e132 * _e126) + _e128);
                let _e138 = floor(_e135);
                let _e139 = _e119.xy;
                switch bitcast<i32>(0u) {
                    default: {
                        if (sj && (_e138 == 4f)) {
                            let _e153 = normalize(_e139);
                            let _e156 = acos(clamp(_e153.x, -1f, 1f));
                            if (_e153.y >= 0f) {
                                phi_2366_ = _e156;
                            } else {
                                phi_2366_ = -(_e156);
                            }
                            let _e161 = phi_2366_;
                            phi_2367_ = vec2<f32>((_e161 * 0.15915494f), _e137);
                            break;
                        } else {
                            if (_e138 == 2f) {
                                phi_2365_ = _e119.x;
                            } else {
                                phi_2365_ = length(_e139);
                            }
                            let _e147 = phi_2365_;
                            phi_2367_ = vec2<f32>(((clamp(_e147, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e119.z < 0f))) + _e135), _e137);
                            break;
                        }
                    }
                }
                let _e165 = phi_2367_;
                let _e166 = textureSampleLevel(ZC, S8_, _e165, 0f);
                phi_2384_ = _e166;
                if !(_e122) {
                    let _e172 = (_e166.xyz * _e166.w);
                    phi_2384_ = vec4<f32>(_e172.x, _e172.y, _e172.z, (_e166.w * (ceil(_e119.w) * -0.003921569f)));
                }
                let _e179 = phi_2384_;
                phi_2383_ = _e179;
            }
            let _e181 = phi_2383_;
            phi_1405_ = nj;
            if nj {
                phi_1405_ = (_e118.z < 0f);
            }
            let _e185 = phi_1405_;
            if _e185 {
                let _e187 = textureSampleLevel(RB, v4_, _e118.xy, 0f);
                phi_2387_ = _e187;
                break;
            }
            phi_1420_ = nj;
            if nj {
                phi_1420_ = (_e118.z > 0f);
            }
            let _e191 = phi_1420_;
            phi_2386_ = _e181;
            if _e191 {
                let _e195 = textureSampleLevel(RB, v4_, _e118.xy, (_e118.z - 1f));
                phi_2379_ = _e195;
                if _e122 {
                    if (_e195.w != 0f) {
                        phi_2368_ = (1f / _e195.w);
                    } else {
                        phi_2368_ = 0f;
                    }
                    let _e201 = phi_2368_;
                    let _e202 = (_e195.xyz * _e201);
                    phi_2379_ = vec4<f32>(_e202.x, _e202.y, _e202.z, _e195.w);
                }
                let _e208 = phi_2379_;
                phi_2386_ = (_e181 * _e208);
            }
            let _e211 = phi_2386_;
            phi_2387_ = _e211;
            break;
        }
    }
    let _e213 = phi_2387_;
    let _e214 = T_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e217 = (_e214.y >= 0f);
            local = _e217;
            if _e217 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1643_ = mj;
                        if mj {
                            phi_1643_ = (_e214.x < -1.5f);
                        }
                        let _e285 = phi_1643_;
                        if _e285 {
                            let _e291 = textureSampleLevel(AD, eb, vec2<f32>((3f + _e214.x), 0f), 0f);
                            let _e296 = textureSampleLevel(AD, eb, vec2<f32>((1f - _e214.y), 0f), 0f);
                            phi_2417_ = ((1f - _e291.x) - _e296.x);
                            break;
                        } else {
                            phi_2417_ = min(_e214.x, _e214.y);
                            break;
                        }
                    }
                }
                let _e300 = phi_2417_;
                phi_2418_ = _e300;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1706_ = mj;
                        if mj {
                            phi_1706_ = (_e214.y < -1.5f);
                        }
                        let _e221 = phi_1706_;
                        if _e221 {
                            let _e225 = max(_e214.w, 0f);
                            if (_e214.z >= 0f) {
                                let _e228 = textureSampleLevel(AD, eb, vec2<f32>(_e225, 0f), 0f);
                                phi_2388_ = _e228.x;
                            } else {
                                phi_2388_ = 0f;
                            }
                            let _e231 = phi_2388_;
                            phi_2415_ = _e231;
                            if (abs(_e214.z) < 1000f) {
                                let _e237 = (-2f - _e214.y);
                                let _e239 = ((_e237 - _e225) * 0.5984134f);
                                let _e242 = (vec4(_e225) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e239));
                                let _e248 = ((_e242 * -(_e214.z)) + vec4(((_e237 * _e214.z) + (abs(_e214.x) - 0.25f))));
                                let _e251 = textureSampleLevel(AD, eb, vec2<f32>(_e248.x, 0f), 0f);
                                let _e254 = textureSampleLevel(AD, eb, vec2<f32>(_e248.y, 0f), 0f);
                                let _e257 = textureSampleLevel(AD, eb, vec2<f32>(_e248.z, 0f), 0f);
                                let _e260 = textureSampleLevel(AD, eb, vec2<f32>(_e248.w, 0f), 0f);
                                let _e266 = (_e242 * 5.0959306f);
                                phi_2415_ = (_e231 + (dot(vec4<f32>(_e251.x, _e254.x, _e257.x, _e260.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e266) * (_e266 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e239));
                            }
                            let _e275 = phi_2415_;
                            phi_2416_ = (_e275 * sign(_e214.x));
                            break;
                        } else {
                            phi_2416_ = _e214.x;
                            break;
                        }
                    }
                }
                let _e280 = phi_2416_;
                phi_2418_ = _e280;
                break;
            }
        }
    }
    let _e302 = phi_2418_;
    let _e303 = P4_1;
    let _e306 = A3_1[1u];
    let _e308 = A3_1[0u];
    let _e309 = vec2<u32>(floor(_e303));
    let _e336 = (_e308 + (((((_e309.y >> bitcast<u32>(5u)) * (_e306 << bitcast<u32>(5u))) + ((_e309.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e309.x & 28u) << bitcast<u32>(5u)) + ((_e309.y & 28u) << bitcast<u32>(2i)))) + (((_e309.y & 3u) << bitcast<u32>(2i)) + (_e309.x & 3u))));
    phi_2420_ = 1f;
    if kj {
        let _e337 = W0_1;
        let _e340 = min(_e337.xy, _e337.zw);
        phi_2420_ = min(min(_e340.x, _e340.y), 1f);
    }
    let _e346 = phi_2420_;
    phi_1152_ = jj;
    if jj {
        let _e348 = k2_1[0u];
        phi_1152_ = (_e348 != 0f);
    }
    let _e351 = phi_1152_;
    phi_2421_ = _e346;
    if _e351 {
        let _e354 = m0_.v2_[_e115];
        phi_2421_ = min(unpack4x8unorm(_e354).x, _e346);
    }
    let _e359 = phi_2421_;
    let _e361 = clamp(_e302, 0f, max(_e359, 0f));
    let _e363 = local;
    if _e363 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e213.w, _e361) >= 1f) {
                    phi_2453_ = 1f;
                    break;
                }
                let _e454 = j.r2_;
                let _e456 = atomicMax((&Y0_.v2_[_e336]), (_e454 | u32(((abs(_e361) * 1024f) + 0.5f))));
                if (_e456 < _e454) {
                    phi_2452_ = _e361;
                } else {
                    let _e460 = (f32((_e456 & 524287u)) * 0.0009765625f);
                    phi_2452_ = ((max(_e460, _e361) - _e460) / max((1f - (_e460 * _e213.w)), 0.000062f));
                }
                let _e468 = phi_2452_;
                phi_2453_ = _e468;
                break;
            }
        }
        let _e470 = phi_2453_;
        phi_2456_ = _e470;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e369 = u32(((abs(_e361) * 1024f) + 0.5f));
                let _e372 = atomicLoad((&Y0_.v2_[_e336]));
                let _e374 = (min(_e213.w, _e361) >= 1f);
                phi_2063_ = _e374;
                if _e374 {
                    let _e376 = j.r2_;
                    let _e377 = (_e372 < _e376);
                    phi_2061_ = _e377;
                    if !(_e377) {
                        phi_2061_ = (_e372 >= (_e376 | 262144u));
                    }
                    let _e382 = phi_2061_;
                    phi_2063_ = _e382;
                }
                let _e384 = phi_2063_;
                if _e384 {
                    phi_2451_ = 1f;
                    break;
                }
                let _e386 = j.r2_;
                phi_2445_ = 0f;
                phi_2442_ = _e369;
                phi_2439_ = _e361;
                if (_e372 < _e386) {
                    let _e389 = (_e386 | (262144u + _e369));
                    let _e390 = atomicMax((&Y0_.v2_[_e336]), _e389);
                    if (_e390 <= _e386) {
                        phi_2448_ = _e361;
                        phi_2443_ = _e369;
                        phi_2440_ = 0f;
                    } else {
                        phi_2449_ = 0f;
                        phi_2444_ = _e369;
                        phi_2441_ = _e361;
                        if (_e390 < _e389) {
                            let _e394 = ((_e390 & 524287u) - 262144u);
                            let _e396 = (f32(_e394) * 0.0009765625f);
                            phi_2449_ = ((_e361 - _e396) / max((1f - (_e396 * _e213.w)), 0.000062f));
                            phi_2444_ = _e394;
                            phi_2441_ = _e396;
                        }
                        let _e403 = phi_2449_;
                        let _e405 = phi_2444_;
                        let _e407 = phi_2441_;
                        phi_2448_ = _e403;
                        phi_2443_ = _e405;
                        phi_2440_ = _e407;
                    }
                    let _e409 = phi_2448_;
                    let _e411 = phi_2443_;
                    let _e413 = phi_2440_;
                    phi_2445_ = _e409;
                    phi_2442_ = _e411;
                    phi_2439_ = _e413;
                }
                let _e415 = phi_2445_;
                let _e417 = phi_2442_;
                let _e419 = phi_2439_;
                phi_2450_ = _e415;
                if (_e419 > 0f) {
                    let _e421 = atomicAdd((&Y0_.v2_[_e336]), _e417);
                    let _e426 = (f32(bitcast<i32>(((_e421 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e428 = clamp(_e426, 0f, 1f);
                    phi_2450_ = (_e415 + ((1f - (_e415 * _e213.w)) * ((clamp((_e426 + _e419), 0f, 1f) - _e428) / max((1f - (_e428 * _e213.w)), 0.000062f))));
                }
                let _e440 = phi_2450_;
                phi_2451_ = _e440;
                break;
            }
        }
        let _e442 = phi_2451_;
        phi_2456_ = _e442;
    }
    let _e472 = phi_2456_;
    phi_2470_ = f32();
    if rj {
        let _e474 = j.H3_;
        let _e476 = j.I3_;
        if rj {
            phi_2454_ = ((fract((52.982918f * fract(((0.06711056f * _e80.x) + (0.00583715f * _e80.y))))) * _e474) + _e476);
        } else {
            phi_2454_ = 0f;
        }
        let _e488 = phi_2454_;
        phi_2470_ = _e488;
    }
    let _e490 = phi_2470_;
    let _e491 = (_e213 * _e472);
    let _e492 = _e491.xyz;
    if (rj && (_e491.w != 0f)) {
        phi_2503_ = (vec3(_e490) + _e492);
    } else {
        phi_2503_ = _e492;
    }
    let _e499 = phi_2503_;
    let _e505 = vec4<f32>(_e499.x, _e491.y, _e491.z, _e491.w);
    let _e511 = vec4<f32>(_e505.x, _e499.y, _e505.z, _e505.w);
    m0_.v2_[_e115] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    Q1_ = vec4<f32>(_e511.x, _e511.y, _e499.z, _e511.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) O0_: vec3<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @location(2) T: vec4<f32>, @location(8) P4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @location(3) @interpolate(flat, either) I0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    O0_1 = O0_;
    S0_1 = S0_;
    N0_1 = N0_;
    T_1 = T;
    P4_1 = P4_;
    A3_1 = A3_;
    W0_1 = W0_;
    k2_1 = k2_;
    I0_1 = I0_;
    main_1();
    let _e21 = Q1_;
    return _e21;
}
