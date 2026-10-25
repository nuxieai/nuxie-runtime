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
var<private> O0_1: vec3<f32>;
var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
var<private> T_1: vec4<f32>;
var<private> P4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> k2_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
var<private> I0_1: f32;

fn main_1() {
    var phi_2273_: f32;
    var phi_2274_: f32;
    var phi_2275_: vec2<f32>;
    var phi_2292_: vec4<f32>;
    var phi_2291_: vec4<f32>;
    var phi_1332_: bool;
    var phi_1347_: bool;
    var phi_2276_: f32;
    var phi_2287_: vec4<f32>;
    var phi_2294_: vec4<f32>;
    var phi_2295_: vec4<f32>;
    var phi_1638_: bool;
    var phi_2296_: f32;
    var phi_2323_: f32;
    var phi_2324_: f32;
    var phi_1575_: bool;
    var phi_2325_: f32;
    var phi_2326_: f32;
    var phi_2328_: f32;
    var phi_1119_: bool;
    var phi_2329_: f32;
    var local: bool;
    var phi_1993_: bool;
    var phi_1995_: bool;
    var phi_2357_: f32;
    var phi_2352_: u32;
    var phi_2349_: f32;
    var phi_2356_: f32;
    var phi_2351_: u32;
    var phi_2348_: f32;
    var phi_2353_: f32;
    var phi_2350_: u32;
    var phi_2347_: f32;
    var phi_2358_: f32;
    var phi_2359_: f32;
    var phi_2360_: f32;
    var phi_2361_: f32;
    var phi_2364_: f32;
    var phi_2362_: f32;
    var phi_2378_: f32;
    var phi_2411_: vec3<f32>;

    let _e76 = S0_1;
    let _e78 = O0_1;
    let _e79 = N0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e82 = (lj && (u32(_e76) != 0u));
            if (_e79.w >= 0f) {
                phi_2291_ = _e79;
            } else {
                let _e86 = j.Q8_;
                let _e88 = j.R8_;
                let _e91 = abs(_e79.z);
                let _e92 = floor(_e91);
                let _e95 = (((_e91 - _e92) * 8f) + 0.0009765625f);
                let _e97 = ((_e92 * _e86) + _e88);
                let _e98 = floor(_e95);
                switch bitcast<i32>(0u) {
                    default: {
                        if (sj && (_e98 == 4f)) {
                            let _e117 = normalize(vec2<f32>(_e79.x, -(_e79.y)));
                            let _e120 = acos(clamp(_e117.x, -1f, 1f));
                            if (_e117.y >= 0f) {
                                phi_2274_ = _e120;
                            } else {
                                phi_2274_ = -(_e120);
                            }
                            let _e125 = phi_2274_;
                            phi_2275_ = vec2<f32>((_e125 * 0.15915494f), _e97);
                            break;
                        } else {
                            if (_e98 == 2f) {
                                phi_2273_ = _e79.x;
                            } else {
                                phi_2273_ = length(_e79.xy);
                            }
                            let _e107 = phi_2273_;
                            phi_2275_ = vec2<f32>(((clamp(_e107, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e79.z < 0f))) + _e95), _e97);
                            break;
                        }
                    }
                }
                let _e129 = phi_2275_;
                let _e130 = textureSampleLevel(ZC, S8_, _e129, 0f);
                phi_2292_ = _e130;
                if !(_e82) {
                    let _e136 = (_e130.xyz * _e130.w);
                    phi_2292_ = vec4<f32>(_e136.x, _e136.y, _e136.z, (_e130.w * (ceil(_e79.w) * -0.003921569f)));
                }
                let _e143 = phi_2292_;
                phi_2291_ = _e143;
            }
            let _e145 = phi_2291_;
            phi_1332_ = nj;
            if nj {
                phi_1332_ = (_e78.z < 0f);
            }
            let _e149 = phi_1332_;
            if _e149 {
                let _e151 = textureSampleLevel(RB, v4_, _e78.xy, 0f);
                phi_2295_ = _e151;
                break;
            }
            phi_1347_ = nj;
            if nj {
                phi_1347_ = (_e78.z > 0f);
            }
            let _e155 = phi_1347_;
            phi_2294_ = _e145;
            if _e155 {
                let _e159 = textureSampleLevel(RB, v4_, _e78.xy, (_e78.z - 1f));
                phi_2287_ = _e159;
                if _e82 {
                    if (_e159.w != 0f) {
                        phi_2276_ = (1f / _e159.w);
                    } else {
                        phi_2276_ = 0f;
                    }
                    let _e165 = phi_2276_;
                    let _e166 = (_e159.xyz * _e165);
                    phi_2287_ = vec4<f32>(_e166.x, _e166.y, _e166.z, _e159.w);
                }
                let _e172 = phi_2287_;
                phi_2294_ = (_e145 * _e172);
            }
            let _e175 = phi_2294_;
            phi_2295_ = _e175;
            break;
        }
    }
    let _e177 = phi_2295_;
    let _e178 = T_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e181 = (_e178.y >= 0f);
            local = _e181;
            if _e181 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1575_ = mj;
                        if mj {
                            phi_1575_ = (_e178.x < -1.5f);
                        }
                        let _e249 = phi_1575_;
                        if _e249 {
                            let _e255 = textureSampleLevel(AD, eb, vec2<f32>((3f + _e178.x), 0f), 0f);
                            let _e260 = textureSampleLevel(AD, eb, vec2<f32>((1f - _e178.y), 0f), 0f);
                            phi_2325_ = ((1f - _e255.x) - _e260.x);
                            break;
                        } else {
                            phi_2325_ = min(_e178.x, _e178.y);
                            break;
                        }
                    }
                }
                let _e264 = phi_2325_;
                phi_2326_ = _e264;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1638_ = mj;
                        if mj {
                            phi_1638_ = (_e178.y < -1.5f);
                        }
                        let _e185 = phi_1638_;
                        if _e185 {
                            let _e189 = max(_e178.w, 0f);
                            if (_e178.z >= 0f) {
                                let _e192 = textureSampleLevel(AD, eb, vec2<f32>(_e189, 0f), 0f);
                                phi_2296_ = _e192.x;
                            } else {
                                phi_2296_ = 0f;
                            }
                            let _e195 = phi_2296_;
                            phi_2323_ = _e195;
                            if (abs(_e178.z) < 1000f) {
                                let _e201 = (-2f - _e178.y);
                                let _e203 = ((_e201 - _e189) * 0.5984134f);
                                let _e206 = (vec4(_e189) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e203));
                                let _e212 = ((_e206 * -(_e178.z)) + vec4(((_e201 * _e178.z) + (abs(_e178.x) - 0.25f))));
                                let _e215 = textureSampleLevel(AD, eb, vec2<f32>(_e212.x, 0f), 0f);
                                let _e218 = textureSampleLevel(AD, eb, vec2<f32>(_e212.y, 0f), 0f);
                                let _e221 = textureSampleLevel(AD, eb, vec2<f32>(_e212.z, 0f), 0f);
                                let _e224 = textureSampleLevel(AD, eb, vec2<f32>(_e212.w, 0f), 0f);
                                let _e230 = (_e206 * 5.0959306f);
                                phi_2323_ = (_e195 + (dot(vec4<f32>(_e215.x, _e218.x, _e221.x, _e224.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e230) * (_e230 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e203));
                            }
                            let _e239 = phi_2323_;
                            phi_2324_ = (_e239 * sign(_e178.x));
                            break;
                        } else {
                            phi_2324_ = _e178.x;
                            break;
                        }
                    }
                }
                let _e244 = phi_2324_;
                phi_2326_ = _e244;
                break;
            }
        }
    }
    let _e266 = phi_2326_;
    let _e267 = P4_1;
    let _e270 = A3_1[1u];
    let _e272 = A3_1[0u];
    let _e273 = vec2<u32>(floor(_e267));
    let _e300 = (_e272 + (((((_e273.y >> bitcast<u32>(5u)) * (_e270 << bitcast<u32>(5u))) + ((_e273.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e273.x & 28u) << bitcast<u32>(5u)) + ((_e273.y & 28u) << bitcast<u32>(2i)))) + (((_e273.y & 3u) << bitcast<u32>(2i)) + (_e273.x & 3u))));
    phi_2328_ = 1f;
    if kj {
        let _e301 = W0_1;
        let _e304 = min(_e301.xy, _e301.zw);
        phi_2328_ = min(min(_e304.x, _e304.y), 1f);
    }
    let _e310 = phi_2328_;
    phi_1119_ = jj;
    if jj {
        let _e312 = k2_1[0u];
        phi_1119_ = (_e312 != 0f);
    }
    let _e315 = phi_1119_;
    phi_2329_ = _e310;
    if _e315 {
        let _e316 = gl_FragCoord_1;
        let _e320 = textureLoad(m0_, vec2<i32>(floor(_e316.xy)), 0i);
        phi_2329_ = min(_e320.x, _e310);
    }
    let _e324 = phi_2329_;
    let _e326 = clamp(_e266, 0f, max(_e324, 0f));
    let _e328 = local;
    if _e328 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e177.w, _e326) >= 1f) {
                    phi_2361_ = 1f;
                    break;
                }
                let _e419 = j.r2_;
                let _e421 = atomicMax((&Y0_.v2_[_e300]), (_e419 | u32(((abs(_e326) * 1024f) + 0.5f))));
                if (_e421 < _e419) {
                    phi_2360_ = _e326;
                } else {
                    let _e425 = (f32((_e421 & 524287u)) * 0.0009765625f);
                    phi_2360_ = ((max(_e425, _e326) - _e425) / max((1f - (_e425 * _e177.w)), 0.000062f));
                }
                let _e433 = phi_2360_;
                phi_2361_ = _e433;
                break;
            }
        }
        let _e435 = phi_2361_;
        phi_2364_ = _e435;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e334 = u32(((abs(_e326) * 1024f) + 0.5f));
                let _e337 = atomicLoad((&Y0_.v2_[_e300]));
                let _e339 = (min(_e177.w, _e326) >= 1f);
                phi_1995_ = _e339;
                if _e339 {
                    let _e341 = j.r2_;
                    let _e342 = (_e337 < _e341);
                    phi_1993_ = _e342;
                    if !(_e342) {
                        phi_1993_ = (_e337 >= (_e341 | 262144u));
                    }
                    let _e347 = phi_1993_;
                    phi_1995_ = _e347;
                }
                let _e349 = phi_1995_;
                if _e349 {
                    phi_2359_ = 1f;
                    break;
                }
                let _e351 = j.r2_;
                phi_2353_ = 0f;
                phi_2350_ = _e334;
                phi_2347_ = _e326;
                if (_e337 < _e351) {
                    let _e354 = (_e351 | (262144u + _e334));
                    let _e355 = atomicMax((&Y0_.v2_[_e300]), _e354);
                    if (_e355 <= _e351) {
                        phi_2356_ = _e326;
                        phi_2351_ = _e334;
                        phi_2348_ = 0f;
                    } else {
                        phi_2357_ = 0f;
                        phi_2352_ = _e334;
                        phi_2349_ = _e326;
                        if (_e355 < _e354) {
                            let _e359 = ((_e355 & 524287u) - 262144u);
                            let _e361 = (f32(_e359) * 0.0009765625f);
                            phi_2357_ = ((_e326 - _e361) / max((1f - (_e361 * _e177.w)), 0.000062f));
                            phi_2352_ = _e359;
                            phi_2349_ = _e361;
                        }
                        let _e368 = phi_2357_;
                        let _e370 = phi_2352_;
                        let _e372 = phi_2349_;
                        phi_2356_ = _e368;
                        phi_2351_ = _e370;
                        phi_2348_ = _e372;
                    }
                    let _e374 = phi_2356_;
                    let _e376 = phi_2351_;
                    let _e378 = phi_2348_;
                    phi_2353_ = _e374;
                    phi_2350_ = _e376;
                    phi_2347_ = _e378;
                }
                let _e380 = phi_2353_;
                let _e382 = phi_2350_;
                let _e384 = phi_2347_;
                phi_2358_ = _e380;
                if (_e384 > 0f) {
                    let _e386 = atomicAdd((&Y0_.v2_[_e300]), _e382);
                    let _e391 = (f32(bitcast<i32>(((_e386 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e393 = clamp(_e391, 0f, 1f);
                    phi_2358_ = (_e380 + ((1f - (_e380 * _e177.w)) * ((clamp((_e391 + _e384), 0f, 1f) - _e393) / max((1f - (_e393 * _e177.w)), 0.000062f))));
                }
                let _e405 = phi_2358_;
                phi_2359_ = _e405;
                break;
            }
        }
        let _e407 = phi_2359_;
        phi_2364_ = _e407;
    }
    let _e437 = phi_2364_;
    phi_2378_ = f32();
    if rj {
        let _e438 = gl_FragCoord_1;
        let _e440 = j.H3_;
        let _e442 = j.I3_;
        if rj {
            phi_2362_ = ((fract((52.982918f * fract(((0.06711056f * _e438.x) + (0.00583715f * _e438.y))))) * _e440) + _e442);
        } else {
            phi_2362_ = 0f;
        }
        let _e454 = phi_2362_;
        phi_2378_ = _e454;
    }
    let _e456 = phi_2378_;
    let _e457 = (_e177 * _e437);
    let _e458 = _e457.xyz;
    if (rj && (_e457.w != 0f)) {
        phi_2411_ = (vec3(_e456) + _e458);
    } else {
        phi_2411_ = _e458;
    }
    let _e465 = phi_2411_;
    let _e471 = vec4<f32>(_e465.x, _e457.y, _e457.z, _e457.w);
    let _e477 = vec4<f32>(_e471.x, _e465.y, _e471.z, _e471.w);
    Q1_ = vec4<f32>(_e477.x, _e477.y, _e465.z, _e477.w);
    return;
}

@fragment
fn main(@location(9) O0_: vec3<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @location(2) T: vec4<f32>, @location(8) P4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) I0_: f32) -> @location(0) vec4<f32> {
    O0_1 = O0_;
    S0_1 = S0_;
    N0_1 = N0_;
    T_1 = T;
    P4_1 = P4_;
    A3_1 = A3_;
    W0_1 = W0_;
    k2_1 = k2_;
    gl_FragCoord_1 = gl_FragCoord;
    I0_1 = I0_;
    main_1();
    let _e21 = Q1_;
    return _e21;
}
