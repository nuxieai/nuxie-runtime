struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct zf {
    r2_: array<u32>,
}

struct m0Pe {
    r2_: array<u32>,
}

struct zf_1 {
    r2_: array<atomic<u32>>,
}

@id(7) override dj: bool = true;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;
@id(3) override Zi: bool = true;
@id(1) override Xi: bool = true;
@id(0) override Wi: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Y0_: zf_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> U0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> L4_1: vec2<f32>;
var<private> z3_1: vec2<u32>;
var<private> V0_1: vec4<f32>;
var<private> i2_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
var<private> L1_: vec4<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_2273_: f32;
    var phi_2288_: vec4<f32>;
    var phi_2287_: vec4<f32>;
    var phi_1356_: bool;
    var phi_1371_: bool;
    var phi_2274_: f32;
    var phi_2283_: vec4<f32>;
    var phi_2290_: vec4<f32>;
    var phi_2291_: vec4<f32>;
    var phi_1617_: bool;
    var phi_2292_: f32;
    var phi_2315_: f32;
    var phi_2316_: f32;
    var phi_1554_: bool;
    var phi_2317_: f32;
    var phi_2318_: f32;
    var phi_2320_: f32;
    var phi_1108_: bool;
    var phi_2321_: f32;
    var local: bool;
    var phi_1972_: bool;
    var phi_1974_: bool;
    var phi_2349_: f32;
    var phi_2344_: u32;
    var phi_2341_: f32;
    var phi_2348_: f32;
    var phi_2343_: u32;
    var phi_2340_: f32;
    var phi_2345_: f32;
    var phi_2342_: u32;
    var phi_2339_: f32;
    var phi_2350_: f32;
    var phi_2351_: f32;
    var phi_2352_: f32;
    var phi_2353_: f32;
    var phi_2356_: f32;
    var phi_2354_: f32;
    var phi_2370_: f32;
    var phi_2401_: vec3<f32>;

    let _e76 = gl_FragCoord_1;
    let _e80 = bitcast<vec2<u32>>(vec2<i32>(floor(_e76.xy)));
    let _e82 = j.P6_;
    let _e111 = bitcast<i32>((((((_e80.y >> bitcast<u32>(5u)) * (((_e82 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e80.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e80.x & 28u) << bitcast<u32>(5u)) + ((_e80.y & 28u) << bitcast<u32>(2i)))) + (((_e80.y & 3u) << bitcast<u32>(2i)) + (_e80.x & 3u))));
    let _e112 = P0_1;
    let _e114 = U0_1;
    let _e115 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e118 = (Yi && (u32(_e112) != 0u));
            if (_e115.w >= 0f) {
                phi_2287_ = _e115;
            } else {
                let _e122 = j.L8_;
                let _e124 = j.M8_;
                let _e127 = abs(_e115.z);
                let _e128 = floor(_e127);
                let _e131 = (((_e127 - _e128) * 8f) + 0.0009765625f);
                if (floor(_e131) == 2f) {
                    phi_2273_ = _e115.x;
                } else {
                    phi_2273_ = length(_e115.xy);
                }
                let _e140 = phi_2273_;
                let _e146 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e140, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e115.z < 0f))) + _e131), ((_e128 * _e122) + _e124)), 0f);
                phi_2288_ = _e146;
                if !(_e118) {
                    let _e152 = (_e146.xyz * _e146.w);
                    phi_2288_ = vec4<f32>(_e152.x, _e152.y, _e152.z, (_e146.w * (ceil(_e115.w) * -0.003921569f)));
                }
                let _e159 = phi_2288_;
                phi_2287_ = _e159;
            }
            let _e161 = phi_2287_;
            phi_1356_ = ej;
            if ej {
                phi_1356_ = (_e114.z < 0f);
            }
            let _e165 = phi_1356_;
            if _e165 {
                let _e167 = textureSampleLevel(TB, U4_, _e114.xy, 0f);
                phi_2291_ = _e167;
                break;
            }
            phi_1371_ = ej;
            if ej {
                phi_1371_ = (_e114.z > 0f);
            }
            let _e171 = phi_1371_;
            phi_2290_ = _e161;
            if _e171 {
                let _e175 = textureSampleLevel(TB, U4_, _e114.xy, (_e114.z - 1f));
                phi_2283_ = _e175;
                if _e118 {
                    if (_e175.w != 0f) {
                        phi_2274_ = (1f / _e175.w);
                    } else {
                        phi_2274_ = 0f;
                    }
                    let _e181 = phi_2274_;
                    let _e182 = (_e175.xyz * _e181);
                    phi_2283_ = vec4<f32>(_e182.x, _e182.y, _e182.z, _e175.w);
                }
                let _e188 = phi_2283_;
                phi_2290_ = (_e161 * _e188);
            }
            let _e191 = phi_2290_;
            phi_2291_ = _e191;
            break;
        }
    }
    let _e193 = phi_2291_;
    let _e194 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e197 = (_e194.y >= 0f);
            local = _e197;
            if _e197 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1554_ = Zi;
                        if Zi {
                            phi_1554_ = (_e194.x < -1.5f);
                        }
                        let _e265 = phi_1554_;
                        if _e265 {
                            let _e271 = textureSampleLevel(YC, ab, vec2<f32>((3f + _e194.x), 0f), 0f);
                            let _e276 = textureSampleLevel(YC, ab, vec2<f32>((1f - _e194.y), 0f), 0f);
                            phi_2317_ = ((1f - _e271.x) - _e276.x);
                            break;
                        } else {
                            phi_2317_ = min(_e194.x, _e194.y);
                            break;
                        }
                    }
                }
                let _e280 = phi_2317_;
                phi_2318_ = _e280;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1617_ = Zi;
                        if Zi {
                            phi_1617_ = (_e194.y < -1.5f);
                        }
                        let _e201 = phi_1617_;
                        if _e201 {
                            let _e205 = max(_e194.w, 0f);
                            if (_e194.z >= 0f) {
                                let _e208 = textureSampleLevel(YC, ab, vec2<f32>(_e205, 0f), 0f);
                                phi_2292_ = _e208.x;
                            } else {
                                phi_2292_ = 0f;
                            }
                            let _e211 = phi_2292_;
                            phi_2315_ = _e211;
                            if (abs(_e194.z) < 1000f) {
                                let _e217 = (-2f - _e194.y);
                                let _e219 = ((_e217 - _e205) * 0.5984134f);
                                let _e222 = (vec4(_e205) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e219));
                                let _e228 = ((_e222 * -(_e194.z)) + vec4(((_e217 * _e194.z) + (abs(_e194.x) - 0.25f))));
                                let _e231 = textureSampleLevel(YC, ab, vec2<f32>(_e228.x, 0f), 0f);
                                let _e234 = textureSampleLevel(YC, ab, vec2<f32>(_e228.y, 0f), 0f);
                                let _e237 = textureSampleLevel(YC, ab, vec2<f32>(_e228.z, 0f), 0f);
                                let _e240 = textureSampleLevel(YC, ab, vec2<f32>(_e228.w, 0f), 0f);
                                let _e246 = (_e222 * 5.0959306f);
                                phi_2315_ = (_e211 + (dot(vec4<f32>(_e231.x, _e234.x, _e237.x, _e240.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e246) * (_e246 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e219));
                            }
                            let _e255 = phi_2315_;
                            phi_2316_ = (_e255 * sign(_e194.x));
                            break;
                        } else {
                            phi_2316_ = _e194.x;
                            break;
                        }
                    }
                }
                let _e260 = phi_2316_;
                phi_2318_ = _e260;
                break;
            }
        }
    }
    let _e282 = phi_2318_;
    let _e283 = L4_1;
    let _e286 = z3_1[1u];
    let _e288 = z3_1[0u];
    let _e289 = vec2<u32>(floor(_e283));
    let _e316 = (_e288 + (((((_e289.y >> bitcast<u32>(5u)) * (_e286 << bitcast<u32>(5u))) + ((_e289.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e289.x & 28u) << bitcast<u32>(5u)) + ((_e289.y & 28u) << bitcast<u32>(2i)))) + (((_e289.y & 3u) << bitcast<u32>(2i)) + (_e289.x & 3u))));
    phi_2320_ = 1f;
    if Xi {
        let _e317 = V0_1;
        let _e320 = min(_e317.xy, _e317.zw);
        phi_2320_ = min(min(_e320.x, _e320.y), 1f);
    }
    let _e326 = phi_2320_;
    phi_1108_ = Wi;
    if Wi {
        let _e328 = i2_1[0u];
        phi_1108_ = (_e328 != 0f);
    }
    let _e331 = phi_1108_;
    phi_2321_ = _e326;
    if _e331 {
        let _e334 = m0_.r2_[_e111];
        phi_2321_ = min(unpack4x8unorm(_e334).x, _e326);
    }
    let _e339 = phi_2321_;
    let _e341 = clamp(_e282, 0f, max(_e339, 0f));
    let _e343 = local;
    if _e343 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e193.w, _e341) >= 1f) {
                    phi_2353_ = 1f;
                    break;
                }
                let _e434 = j.q2_;
                let _e436 = atomicMax((&Y0_.r2_[_e316]), (_e434 | u32(((abs(_e341) * 1024f) + 0.5f))));
                if (_e436 < _e434) {
                    phi_2352_ = _e341;
                } else {
                    let _e440 = (f32((_e436 & 524287u)) * 0.0009765625f);
                    phi_2352_ = ((max(_e440, _e341) - _e440) / max((1f - (_e440 * _e193.w)), 0.000062f));
                }
                let _e448 = phi_2352_;
                phi_2353_ = _e448;
                break;
            }
        }
        let _e450 = phi_2353_;
        phi_2356_ = _e450;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e349 = u32(((abs(_e341) * 1024f) + 0.5f));
                let _e352 = atomicLoad((&Y0_.r2_[_e316]));
                let _e354 = (min(_e193.w, _e341) >= 1f);
                phi_1974_ = _e354;
                if _e354 {
                    let _e356 = j.q2_;
                    let _e357 = (_e352 < _e356);
                    phi_1972_ = _e357;
                    if !(_e357) {
                        phi_1972_ = (_e352 >= (_e356 | 262144u));
                    }
                    let _e362 = phi_1972_;
                    phi_1974_ = _e362;
                }
                let _e364 = phi_1974_;
                if _e364 {
                    phi_2351_ = 1f;
                    break;
                }
                let _e366 = j.q2_;
                phi_2345_ = 0f;
                phi_2342_ = _e349;
                phi_2339_ = _e341;
                if (_e352 < _e366) {
                    let _e369 = (_e366 | (262144u + _e349));
                    let _e370 = atomicMax((&Y0_.r2_[_e316]), _e369);
                    if (_e370 <= _e366) {
                        phi_2348_ = _e341;
                        phi_2343_ = _e349;
                        phi_2340_ = 0f;
                    } else {
                        phi_2349_ = 0f;
                        phi_2344_ = _e349;
                        phi_2341_ = _e341;
                        if (_e370 < _e369) {
                            let _e374 = ((_e370 & 524287u) - 262144u);
                            let _e376 = (f32(_e374) * 0.0009765625f);
                            phi_2349_ = ((_e341 - _e376) / max((1f - (_e376 * _e193.w)), 0.000062f));
                            phi_2344_ = _e374;
                            phi_2341_ = _e376;
                        }
                        let _e383 = phi_2349_;
                        let _e385 = phi_2344_;
                        let _e387 = phi_2341_;
                        phi_2348_ = _e383;
                        phi_2343_ = _e385;
                        phi_2340_ = _e387;
                    }
                    let _e389 = phi_2348_;
                    let _e391 = phi_2343_;
                    let _e393 = phi_2340_;
                    phi_2345_ = _e389;
                    phi_2342_ = _e391;
                    phi_2339_ = _e393;
                }
                let _e395 = phi_2345_;
                let _e397 = phi_2342_;
                let _e399 = phi_2339_;
                phi_2350_ = _e395;
                if (_e399 > 0f) {
                    let _e401 = atomicAdd((&Y0_.r2_[_e316]), _e397);
                    let _e406 = (f32(bitcast<i32>(((_e401 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e408 = clamp(_e406, 0f, 1f);
                    phi_2350_ = (_e395 + ((1f - (_e395 * _e193.w)) * ((clamp((_e406 + _e399), 0f, 1f) - _e408) / max((1f - (_e408 * _e193.w)), 0.000062f))));
                }
                let _e420 = phi_2350_;
                phi_2351_ = _e420;
                break;
            }
        }
        let _e422 = phi_2351_;
        phi_2356_ = _e422;
    }
    let _e452 = phi_2356_;
    phi_2370_ = f32();
    if dj {
        let _e454 = j.F3_;
        let _e456 = j.G3_;
        if dj {
            phi_2354_ = ((fract((52.982918f * fract(((0.06711056f * _e76.x) + (0.00583715f * _e76.y))))) * _e454) + _e456);
        } else {
            phi_2354_ = 0f;
        }
        let _e468 = phi_2354_;
        phi_2370_ = _e468;
    }
    let _e470 = phi_2370_;
    let _e471 = (_e193 * _e452);
    let _e472 = _e471.xyz;
    if (dj && (_e471.w != 0f)) {
        phi_2401_ = (vec3(_e470) + _e472);
    } else {
        phi_2401_ = _e472;
    }
    let _e479 = phi_2401_;
    let _e485 = vec4<f32>(_e479.x, _e471.y, _e471.z, _e471.w);
    let _e491 = vec4<f32>(_e485.x, _e479.y, _e485.z, _e485.w);
    m0_.r2_[_e111] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    L1_ = vec4<f32>(_e491.x, _e491.y, _e479.z, _e491.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) L4_: vec2<f32>, @location(7) @interpolate(flat, either) z3_: vec2<u32>, @location(5) V0_: vec4<f32>, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    U0_1 = U0_;
    P0_1 = P0_;
    O0_1 = O0_;
    S_1 = S;
    L4_1 = L4_;
    z3_1 = z3_;
    V0_1 = V0_;
    i2_1 = i2_;
    G0_1 = G0_;
    main_1();
    let _e21 = L1_;
    return _e21;
}
