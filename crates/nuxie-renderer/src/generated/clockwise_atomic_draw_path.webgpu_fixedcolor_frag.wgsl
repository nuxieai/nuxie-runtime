struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct zf {
    v2_: array<u32>,
}

struct m0Pe {
    v2_: array<u32>,
}

struct zf_1 {
    v2_: array<atomic<u32>>,
}

@id(7) override fj: bool = true;
@id(2) override aj: bool = true;
@id(8) override gj: bool = true;
@id(3) override bj: bool = true;
@id(1) override Zi: bool = true;
@id(0) override Yi: bool = true;

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var I8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Z0_: zf_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> V0_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> P0_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> J4_1: vec2<f32>;
var<private> y3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> j2_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
var<private> N1_: vec4<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_2207_: f32;
    var phi_2208_: f32;
    var phi_2224_: vec4<f32>;
    var phi_2223_: vec4<f32>;
    var phi_1333_: bool;
    var phi_1348_: bool;
    var phi_2209_: f32;
    var phi_2219_: vec4<f32>;
    var phi_2226_: vec4<f32>;
    var phi_2227_: vec4<f32>;
    var phi_1560_: bool;
    var phi_2228_: f32;
    var phi_2253_: f32;
    var phi_2254_: f32;
    var phi_1497_: bool;
    var phi_2255_: f32;
    var phi_2256_: f32;
    var phi_2258_: f32;
    var phi_1074_: bool;
    var phi_2259_: f32;
    var local: bool;
    var phi_1915_: bool;
    var phi_1917_: bool;
    var phi_2287_: f32;
    var phi_2282_: u32;
    var phi_2279_: f32;
    var phi_2286_: f32;
    var phi_2281_: u32;
    var phi_2278_: f32;
    var phi_2283_: f32;
    var phi_2280_: u32;
    var phi_2277_: f32;
    var phi_2288_: f32;
    var phi_2289_: f32;
    var phi_2290_: f32;
    var phi_2291_: f32;
    var phi_2294_: f32;
    var phi_2292_: f32;
    var phi_2308_: f32;
    var phi_2340_: vec3<f32>;

    let _e74 = gl_FragCoord_1;
    let _e78 = bitcast<vec2<u32>>(vec2<i32>(floor(_e74.xy)));
    let _e80 = j.L6_;
    let _e109 = bitcast<i32>((((((_e78.y >> bitcast<u32>(5u)) * (((_e80 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e78.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e78.x & 28u) << bitcast<u32>(5u)) + ((_e78.y & 28u) << bitcast<u32>(2i)))) + (((_e78.y & 3u) << bitcast<u32>(2i)) + (_e78.x & 3u))));
    let _e110 = Q0_1;
    let _e112 = V0_1;
    let _e113 = P0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e116 = (aj && (u32(_e110) != 0u));
            if (_e113.w >= 0f) {
                phi_2223_ = _e113;
            } else {
                let _e119 = -(_e113.w);
                let _e124 = j.cd;
                let _e127 = j.g7_;
                if (_e113.z > 0f) {
                    phi_2207_ = _e113.x;
                } else {
                    phi_2207_ = length(_e113.xy);
                }
                let _e135 = phi_2207_;
                let _e136 = clamp(_e135, 0f, 1f);
                let _e137 = abs(_e113.z);
                if (_e137 > 1f) {
                    phi_2208_ = ((0.9980469f * _e136) + 0.0009765625f);
                } else {
                    phi_2208_ = ((0.001953125f * _e136) + _e137);
                }
                let _e144 = phi_2208_;
                let _e146 = textureSampleLevel(YC, I8_, vec2<f32>(_e144, ((floor(_e119) * _e124) + _e127)), 0f);
                phi_2224_ = _e146;
                if !(_e116) {
                    let _e150 = (_e146.xyz * _e146.w);
                    phi_2224_ = vec4<f32>(_e150.x, _e150.y, _e150.z, (_e146.w * (fract(_e119) * 1.0039216f)));
                }
                let _e157 = phi_2224_;
                phi_2223_ = _e157;
            }
            let _e159 = phi_2223_;
            phi_1333_ = gj;
            if gj {
                phi_1333_ = (_e112.z < 0f);
            }
            let _e163 = phi_1333_;
            if _e163 {
                let _e165 = textureSampleLevel(TB, S4_, _e112.xy, 0f);
                phi_2227_ = _e165;
                break;
            }
            phi_1348_ = gj;
            if gj {
                phi_1348_ = (_e112.z > 0f);
            }
            let _e169 = phi_1348_;
            phi_2226_ = _e159;
            if _e169 {
                let _e173 = textureSampleLevel(TB, S4_, _e112.xy, (_e112.z - 1f));
                phi_2219_ = _e173;
                if _e116 {
                    if (_e173.w != 0f) {
                        phi_2209_ = (1f / _e173.w);
                    } else {
                        phi_2209_ = 0f;
                    }
                    let _e179 = phi_2209_;
                    let _e180 = (_e173.xyz * _e179);
                    phi_2219_ = vec4<f32>(_e180.x, _e180.y, _e180.z, _e173.w);
                }
                let _e186 = phi_2219_;
                phi_2226_ = (_e159 * _e186);
            }
            let _e189 = phi_2226_;
            phi_2227_ = _e189;
            break;
        }
    }
    let _e191 = phi_2227_;
    let _e192 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e195 = (_e192.y >= 0f);
            local = _e195;
            if _e195 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1497_ = bj;
                        if bj {
                            phi_1497_ = (_e192.x < -1.5f);
                        }
                        let _e263 = phi_1497_;
                        if _e263 {
                            let _e269 = textureSampleLevel(ZC, Va, vec2<f32>((3f + _e192.x), 0f), 0f);
                            let _e274 = textureSampleLevel(ZC, Va, vec2<f32>((1f - _e192.y), 0f), 0f);
                            phi_2255_ = ((1f - _e269.x) - _e274.x);
                            break;
                        } else {
                            phi_2255_ = min(_e192.x, _e192.y);
                            break;
                        }
                    }
                }
                let _e278 = phi_2255_;
                phi_2256_ = _e278;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1560_ = bj;
                        if bj {
                            phi_1560_ = (_e192.y < -1.5f);
                        }
                        let _e199 = phi_1560_;
                        if _e199 {
                            let _e203 = max(_e192.w, 0f);
                            if (_e192.z >= 0f) {
                                let _e206 = textureSampleLevel(ZC, Va, vec2<f32>(_e203, 0f), 0f);
                                phi_2228_ = _e206.x;
                            } else {
                                phi_2228_ = 0f;
                            }
                            let _e209 = phi_2228_;
                            phi_2253_ = _e209;
                            if (abs(_e192.z) < 1000f) {
                                let _e215 = (-2f - _e192.y);
                                let _e217 = ((_e215 - _e203) * 0.5984134f);
                                let _e220 = (vec4(_e203) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e217));
                                let _e226 = ((_e220 * -(_e192.z)) + vec4(((_e215 * _e192.z) + (abs(_e192.x) - 0.25f))));
                                let _e229 = textureSampleLevel(ZC, Va, vec2<f32>(_e226.x, 0f), 0f);
                                let _e232 = textureSampleLevel(ZC, Va, vec2<f32>(_e226.y, 0f), 0f);
                                let _e235 = textureSampleLevel(ZC, Va, vec2<f32>(_e226.z, 0f), 0f);
                                let _e238 = textureSampleLevel(ZC, Va, vec2<f32>(_e226.w, 0f), 0f);
                                let _e244 = (_e220 * 5.0959306f);
                                phi_2253_ = (_e209 + (dot(vec4<f32>(_e229.x, _e232.x, _e235.x, _e238.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e244) * (_e244 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e217));
                            }
                            let _e253 = phi_2253_;
                            phi_2254_ = (_e253 * sign(_e192.x));
                            break;
                        } else {
                            phi_2254_ = _e192.x;
                            break;
                        }
                    }
                }
                let _e258 = phi_2254_;
                phi_2256_ = _e258;
                break;
            }
        }
    }
    let _e280 = phi_2256_;
    let _e281 = J4_1;
    let _e284 = y3_1[1u];
    let _e286 = y3_1[0u];
    let _e287 = vec2<u32>(floor(_e281));
    let _e314 = (_e286 + (((((_e287.y >> bitcast<u32>(5u)) * (_e284 << bitcast<u32>(5u))) + ((_e287.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e287.x & 28u) << bitcast<u32>(5u)) + ((_e287.y & 28u) << bitcast<u32>(2i)))) + (((_e287.y & 3u) << bitcast<u32>(2i)) + (_e287.x & 3u))));
    phi_2258_ = 1f;
    if Zi {
        let _e315 = W0_1;
        let _e318 = min(_e315.xy, _e315.zw);
        phi_2258_ = min(min(_e318.x, _e318.y), 1f);
    }
    let _e324 = phi_2258_;
    phi_1074_ = Yi;
    if Yi {
        let _e326 = j2_1[0u];
        phi_1074_ = (_e326 != 0f);
    }
    let _e329 = phi_1074_;
    phi_2259_ = _e324;
    if _e329 {
        let _e332 = m0_.v2_[_e109];
        phi_2259_ = min(unpack4x8unorm(_e332).x, _e324);
    }
    let _e337 = phi_2259_;
    let _e339 = clamp(_e280, 0f, max(_e337, 0f));
    let _e341 = local;
    if _e341 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e191.w, _e339) >= 1f) {
                    phi_2291_ = 1f;
                    break;
                }
                let _e432 = j.r2_;
                let _e434 = atomicMax((&Z0_.v2_[_e314]), (_e432 | u32(((abs(_e339) * 1024f) + 0.5f))));
                if (_e434 < _e432) {
                    phi_2290_ = _e339;
                } else {
                    let _e438 = (f32((_e434 & 524287u)) * 0.0009765625f);
                    phi_2290_ = ((max(_e438, _e339) - _e438) / max((1f - (_e438 * _e191.w)), 0.000062f));
                }
                let _e446 = phi_2290_;
                phi_2291_ = _e446;
                break;
            }
        }
        let _e448 = phi_2291_;
        phi_2294_ = _e448;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e347 = u32(((abs(_e339) * 1024f) + 0.5f));
                let _e350 = atomicLoad((&Z0_.v2_[_e314]));
                let _e352 = (min(_e191.w, _e339) >= 1f);
                phi_1917_ = _e352;
                if _e352 {
                    let _e354 = j.r2_;
                    let _e355 = (_e350 < _e354);
                    phi_1915_ = _e355;
                    if !(_e355) {
                        phi_1915_ = (_e350 >= (_e354 | 262144u));
                    }
                    let _e360 = phi_1915_;
                    phi_1917_ = _e360;
                }
                let _e362 = phi_1917_;
                if _e362 {
                    phi_2289_ = 1f;
                    break;
                }
                let _e364 = j.r2_;
                phi_2283_ = 0f;
                phi_2280_ = _e347;
                phi_2277_ = _e339;
                if (_e350 < _e364) {
                    let _e367 = (_e364 | (262144u + _e347));
                    let _e368 = atomicMax((&Z0_.v2_[_e314]), _e367);
                    if (_e368 <= _e364) {
                        phi_2286_ = _e339;
                        phi_2281_ = _e347;
                        phi_2278_ = 0f;
                    } else {
                        phi_2287_ = 0f;
                        phi_2282_ = _e347;
                        phi_2279_ = _e339;
                        if (_e368 < _e367) {
                            let _e372 = ((_e368 & 524287u) - 262144u);
                            let _e374 = (f32(_e372) * 0.0009765625f);
                            phi_2287_ = ((_e339 - _e374) / max((1f - (_e374 * _e191.w)), 0.000062f));
                            phi_2282_ = _e372;
                            phi_2279_ = _e374;
                        }
                        let _e381 = phi_2287_;
                        let _e383 = phi_2282_;
                        let _e385 = phi_2279_;
                        phi_2286_ = _e381;
                        phi_2281_ = _e383;
                        phi_2278_ = _e385;
                    }
                    let _e387 = phi_2286_;
                    let _e389 = phi_2281_;
                    let _e391 = phi_2278_;
                    phi_2283_ = _e387;
                    phi_2280_ = _e389;
                    phi_2277_ = _e391;
                }
                let _e393 = phi_2283_;
                let _e395 = phi_2280_;
                let _e397 = phi_2277_;
                phi_2288_ = _e393;
                if (_e397 > 0f) {
                    let _e399 = atomicAdd((&Z0_.v2_[_e314]), _e395);
                    let _e404 = (f32(bitcast<i32>(((_e399 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e406 = clamp(_e404, 0f, 1f);
                    phi_2288_ = (_e393 + ((1f - (_e393 * _e191.w)) * ((clamp((_e404 + _e397), 0f, 1f) - _e406) / max((1f - (_e406 * _e191.w)), 0.000062f))));
                }
                let _e418 = phi_2288_;
                phi_2289_ = _e418;
                break;
            }
        }
        let _e420 = phi_2289_;
        phi_2294_ = _e420;
    }
    let _e450 = phi_2294_;
    phi_2308_ = f32();
    if fj {
        let _e452 = j.E3_;
        let _e454 = j.F3_;
        if fj {
            phi_2292_ = ((fract((52.982918f * fract(((0.06711056f * _e74.x) + (0.00583715f * _e74.y))))) * _e452) + _e454);
        } else {
            phi_2292_ = 0f;
        }
        let _e466 = phi_2292_;
        phi_2308_ = _e466;
    }
    let _e468 = phi_2308_;
    let _e469 = (_e191 * _e450);
    let _e470 = _e469.xyz;
    if (fj && (_e469.w != 0f)) {
        phi_2340_ = (vec3(_e468) + _e470);
    } else {
        phi_2340_ = _e470;
    }
    let _e477 = phi_2340_;
    let _e483 = vec4<f32>(_e477.x, _e469.y, _e469.z, _e469.w);
    let _e489 = vec4<f32>(_e483.x, _e477.y, _e483.z, _e483.w);
    m0_.v2_[_e109] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    N1_ = vec4<f32>(_e489.x, _e489.y, _e477.z, _e489.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) P0_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) J4_: vec2<f32>, @location(7) @interpolate(flat, either) y3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) j2_: vec2<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    V0_1 = V0_;
    Q0_1 = Q0_;
    P0_1 = P0_;
    S_1 = S;
    J4_1 = J4_;
    y3_1 = y3_;
    W0_1 = W0_;
    j2_1 = j2_;
    G0_1 = G0_;
    main_1();
    let _e21 = N1_;
    return _e21;
}
