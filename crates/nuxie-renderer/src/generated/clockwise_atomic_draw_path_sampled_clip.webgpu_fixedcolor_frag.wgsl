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
var<private> V0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
var<private> T_1: vec4<f32>;
var<private> N4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> k2_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_2270_: f32;
    var phi_2271_: f32;
    var phi_2272_: vec2<f32>;
    var phi_2289_: vec4<f32>;
    var phi_2288_: vec4<f32>;
    var phi_1330_: bool;
    var phi_1345_: bool;
    var phi_2273_: f32;
    var phi_2284_: vec4<f32>;
    var phi_2291_: vec4<f32>;
    var phi_2292_: vec4<f32>;
    var phi_1635_: bool;
    var phi_2293_: f32;
    var phi_2320_: f32;
    var phi_2321_: f32;
    var phi_1572_: bool;
    var phi_2322_: f32;
    var phi_2323_: f32;
    var phi_2325_: f32;
    var phi_1117_: bool;
    var phi_2326_: f32;
    var local: bool;
    var phi_1990_: bool;
    var phi_1992_: bool;
    var phi_2354_: f32;
    var phi_2349_: u32;
    var phi_2346_: f32;
    var phi_2353_: f32;
    var phi_2348_: u32;
    var phi_2345_: f32;
    var phi_2350_: f32;
    var phi_2347_: u32;
    var phi_2344_: f32;
    var phi_2355_: f32;
    var phi_2356_: f32;
    var phi_2357_: f32;
    var phi_2358_: f32;
    var phi_2361_: f32;
    var phi_2359_: f32;
    var phi_2375_: f32;
    var phi_2408_: vec3<f32>;

    let _e75 = Q0_1;
    let _e77 = V0_1;
    let _e78 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e81 = (gj && (u32(_e75) != 0u));
            if (_e78.w >= 0f) {
                phi_2288_ = _e78;
            } else {
                let _e85 = j.O8_;
                let _e87 = j.P8_;
                let _e90 = abs(_e78.z);
                let _e91 = floor(_e90);
                let _e94 = (((_e90 - _e91) * 8f) + 0.0009765625f);
                let _e96 = ((_e91 * _e85) + _e87);
                let _e97 = floor(_e94);
                switch bitcast<i32>(0u) {
                    default: {
                        if (_e97 == 4f) {
                            let _e115 = normalize(vec2<f32>(_e78.x, -(_e78.y)));
                            let _e118 = acos(clamp(_e115.x, -1f, 1f));
                            if (_e115.y >= 0f) {
                                phi_2271_ = _e118;
                            } else {
                                phi_2271_ = -(_e118);
                            }
                            let _e123 = phi_2271_;
                            phi_2272_ = vec2<f32>((_e123 * 0.15915507f), _e96);
                            break;
                        } else {
                            if (_e97 == 2f) {
                                phi_2270_ = _e78.x;
                            } else {
                                phi_2270_ = length(_e78.xy);
                            }
                            let _e105 = phi_2270_;
                            phi_2272_ = vec2<f32>(((clamp(_e105, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e78.z < 0f))) + _e94), _e96);
                            break;
                        }
                    }
                }
                let _e127 = phi_2272_;
                let _e128 = textureSampleLevel(ZC, Q8_, _e127, 0f);
                phi_2289_ = _e128;
                if !(_e81) {
                    let _e134 = (_e128.xyz * _e128.w);
                    phi_2289_ = vec4<f32>(_e134.x, _e134.y, _e134.z, (_e128.w * (ceil(_e78.w) * -0.003921569f)));
                }
                let _e141 = phi_2289_;
                phi_2288_ = _e141;
            }
            let _e143 = phi_2288_;
            phi_1330_ = mj;
            if mj {
                phi_1330_ = (_e77.z < 0f);
            }
            let _e147 = phi_1330_;
            if _e147 {
                let _e149 = textureSampleLevel(UB, W4_, _e77.xy, 0f);
                phi_2292_ = _e149;
                break;
            }
            phi_1345_ = mj;
            if mj {
                phi_1345_ = (_e77.z > 0f);
            }
            let _e153 = phi_1345_;
            phi_2291_ = _e143;
            if _e153 {
                let _e157 = textureSampleLevel(UB, W4_, _e77.xy, (_e77.z - 1f));
                phi_2284_ = _e157;
                if _e81 {
                    if (_e157.w != 0f) {
                        phi_2273_ = (1f / _e157.w);
                    } else {
                        phi_2273_ = 0f;
                    }
                    let _e163 = phi_2273_;
                    let _e164 = (_e157.xyz * _e163);
                    phi_2284_ = vec4<f32>(_e164.x, _e164.y, _e164.z, _e157.w);
                }
                let _e170 = phi_2284_;
                phi_2291_ = (_e143 * _e170);
            }
            let _e173 = phi_2291_;
            phi_2292_ = _e173;
            break;
        }
    }
    let _e175 = phi_2292_;
    let _e176 = T_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e179 = (_e176.y >= 0f);
            local = _e179;
            if _e179 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1572_ = hj;
                        if hj {
                            phi_1572_ = (_e176.x < -1.5f);
                        }
                        let _e247 = phi_1572_;
                        if _e247 {
                            let _e253 = textureSampleLevel(AD, db, vec2<f32>((3f + _e176.x), 0f), 0f);
                            let _e258 = textureSampleLevel(AD, db, vec2<f32>((1f - _e176.y), 0f), 0f);
                            phi_2322_ = ((1f - _e253.x) - _e258.x);
                            break;
                        } else {
                            phi_2322_ = min(_e176.x, _e176.y);
                            break;
                        }
                    }
                }
                let _e262 = phi_2322_;
                phi_2323_ = _e262;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1635_ = hj;
                        if hj {
                            phi_1635_ = (_e176.y < -1.5f);
                        }
                        let _e183 = phi_1635_;
                        if _e183 {
                            let _e187 = max(_e176.w, 0f);
                            if (_e176.z >= 0f) {
                                let _e190 = textureSampleLevel(AD, db, vec2<f32>(_e187, 0f), 0f);
                                phi_2293_ = _e190.x;
                            } else {
                                phi_2293_ = 0f;
                            }
                            let _e193 = phi_2293_;
                            phi_2320_ = _e193;
                            if (abs(_e176.z) < 1000f) {
                                let _e199 = (-2f - _e176.y);
                                let _e201 = ((_e199 - _e187) * 0.5984134f);
                                let _e204 = (vec4(_e187) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e201));
                                let _e210 = ((_e204 * -(_e176.z)) + vec4(((_e199 * _e176.z) + (abs(_e176.x) - 0.25f))));
                                let _e213 = textureSampleLevel(AD, db, vec2<f32>(_e210.x, 0f), 0f);
                                let _e216 = textureSampleLevel(AD, db, vec2<f32>(_e210.y, 0f), 0f);
                                let _e219 = textureSampleLevel(AD, db, vec2<f32>(_e210.z, 0f), 0f);
                                let _e222 = textureSampleLevel(AD, db, vec2<f32>(_e210.w, 0f), 0f);
                                let _e228 = (_e204 * 5.0959306f);
                                phi_2320_ = (_e193 + (dot(vec4<f32>(_e213.x, _e216.x, _e219.x, _e222.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e228) * (_e228 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e201));
                            }
                            let _e237 = phi_2320_;
                            phi_2321_ = (_e237 * sign(_e176.x));
                            break;
                        } else {
                            phi_2321_ = _e176.x;
                            break;
                        }
                    }
                }
                let _e242 = phi_2321_;
                phi_2323_ = _e242;
                break;
            }
        }
    }
    let _e264 = phi_2323_;
    let _e265 = N4_1;
    let _e268 = A3_1[1u];
    let _e270 = A3_1[0u];
    let _e271 = vec2<u32>(floor(_e265));
    let _e298 = (_e270 + (((((_e271.y >> bitcast<u32>(5u)) * (_e268 << bitcast<u32>(5u))) + ((_e271.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e271.x & 28u) << bitcast<u32>(5u)) + ((_e271.y & 28u) << bitcast<u32>(2i)))) + (((_e271.y & 3u) << bitcast<u32>(2i)) + (_e271.x & 3u))));
    phi_2325_ = 1f;
    if fj {
        let _e299 = W0_1;
        let _e302 = min(_e299.xy, _e299.zw);
        phi_2325_ = min(min(_e302.x, _e302.y), 1f);
    }
    let _e308 = phi_2325_;
    phi_1117_ = ej;
    if ej {
        let _e310 = k2_1[0u];
        phi_1117_ = (_e310 != 0f);
    }
    let _e313 = phi_1117_;
    phi_2326_ = _e308;
    if _e313 {
        let _e314 = gl_FragCoord_1;
        let _e318 = textureLoad(m0_, vec2<i32>(floor(_e314.xy)), 0i);
        phi_2326_ = min(_e318.x, _e308);
    }
    let _e322 = phi_2326_;
    let _e324 = clamp(_e264, 0f, max(_e322, 0f));
    let _e326 = local;
    if _e326 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e175.w, _e324) >= 1f) {
                    phi_2358_ = 1f;
                    break;
                }
                let _e417 = j.r2_;
                let _e419 = atomicMax((&Y0_.v2_[_e298]), (_e417 | u32(((abs(_e324) * 1024f) + 0.5f))));
                if (_e419 < _e417) {
                    phi_2357_ = _e324;
                } else {
                    let _e423 = (f32((_e419 & 524287u)) * 0.0009765625f);
                    phi_2357_ = ((max(_e423, _e324) - _e423) / max((1f - (_e423 * _e175.w)), 0.000062f));
                }
                let _e431 = phi_2357_;
                phi_2358_ = _e431;
                break;
            }
        }
        let _e433 = phi_2358_;
        phi_2361_ = _e433;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e332 = u32(((abs(_e324) * 1024f) + 0.5f));
                let _e335 = atomicLoad((&Y0_.v2_[_e298]));
                let _e337 = (min(_e175.w, _e324) >= 1f);
                phi_1992_ = _e337;
                if _e337 {
                    let _e339 = j.r2_;
                    let _e340 = (_e335 < _e339);
                    phi_1990_ = _e340;
                    if !(_e340) {
                        phi_1990_ = (_e335 >= (_e339 | 262144u));
                    }
                    let _e345 = phi_1990_;
                    phi_1992_ = _e345;
                }
                let _e347 = phi_1992_;
                if _e347 {
                    phi_2356_ = 1f;
                    break;
                }
                let _e349 = j.r2_;
                phi_2350_ = 0f;
                phi_2347_ = _e332;
                phi_2344_ = _e324;
                if (_e335 < _e349) {
                    let _e352 = (_e349 | (262144u + _e332));
                    let _e353 = atomicMax((&Y0_.v2_[_e298]), _e352);
                    if (_e353 <= _e349) {
                        phi_2353_ = _e324;
                        phi_2348_ = _e332;
                        phi_2345_ = 0f;
                    } else {
                        phi_2354_ = 0f;
                        phi_2349_ = _e332;
                        phi_2346_ = _e324;
                        if (_e353 < _e352) {
                            let _e357 = ((_e353 & 524287u) - 262144u);
                            let _e359 = (f32(_e357) * 0.0009765625f);
                            phi_2354_ = ((_e324 - _e359) / max((1f - (_e359 * _e175.w)), 0.000062f));
                            phi_2349_ = _e357;
                            phi_2346_ = _e359;
                        }
                        let _e366 = phi_2354_;
                        let _e368 = phi_2349_;
                        let _e370 = phi_2346_;
                        phi_2353_ = _e366;
                        phi_2348_ = _e368;
                        phi_2345_ = _e370;
                    }
                    let _e372 = phi_2353_;
                    let _e374 = phi_2348_;
                    let _e376 = phi_2345_;
                    phi_2350_ = _e372;
                    phi_2347_ = _e374;
                    phi_2344_ = _e376;
                }
                let _e378 = phi_2350_;
                let _e380 = phi_2347_;
                let _e382 = phi_2344_;
                phi_2355_ = _e378;
                if (_e382 > 0f) {
                    let _e384 = atomicAdd((&Y0_.v2_[_e298]), _e380);
                    let _e389 = (f32(bitcast<i32>(((_e384 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e391 = clamp(_e389, 0f, 1f);
                    phi_2355_ = (_e378 + ((1f - (_e378 * _e175.w)) * ((clamp((_e389 + _e382), 0f, 1f) - _e391) / max((1f - (_e391 * _e175.w)), 0.000062f))));
                }
                let _e403 = phi_2355_;
                phi_2356_ = _e403;
                break;
            }
        }
        let _e405 = phi_2356_;
        phi_2361_ = _e405;
    }
    let _e435 = phi_2361_;
    phi_2375_ = f32();
    if lj {
        let _e436 = gl_FragCoord_1;
        let _e438 = j.G3_;
        let _e440 = j.H3_;
        if lj {
            phi_2359_ = ((fract((52.982918f * fract(((0.06711056f * _e436.x) + (0.00583715f * _e436.y))))) * _e438) + _e440);
        } else {
            phi_2359_ = 0f;
        }
        let _e452 = phi_2359_;
        phi_2375_ = _e452;
    }
    let _e454 = phi_2375_;
    let _e455 = (_e175 * _e435);
    let _e456 = _e455.xyz;
    if (lj && (_e455.w != 0f)) {
        phi_2408_ = (vec3(_e454) + _e456);
    } else {
        phi_2408_ = _e456;
    }
    let _e463 = phi_2408_;
    let _e469 = vec4<f32>(_e463.x, _e455.y, _e455.z, _e455.w);
    let _e475 = vec4<f32>(_e469.x, _e463.y, _e469.z, _e469.w);
    Q1_ = vec4<f32>(_e475.x, _e475.y, _e463.z, _e475.w);
    return;
}

@fragment
fn main(@location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(2) T: vec4<f32>, @location(8) N4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> @location(0) vec4<f32> {
    V0_1 = V0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    T_1 = T;
    N4_1 = N4_;
    A3_1 = A3_;
    W0_1 = W0_;
    k2_1 = k2_;
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    main_1();
    let _e21 = Q1_;
    return _e21;
}
