struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct wf {
    v2_: array<u32>,
}

struct m0Me {
    v2_: array<u32>,
}

struct wf_1 {
    v2_: array<atomic<u32>>,
}

@id(7) override bj: bool = true;
@id(2) override Wi: bool = true;
@id(8) override cj: bool = true;
@id(3) override Xi: bool = true;
@id(1) override Vi: bool = true;
@id(0) override Ui: bool = true;

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Z0_: wf_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> V0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> J4_1: vec2<f32>;
var<private> y3_1: vec2<u32>;
var<private> W0_1: vec4<f32>;
var<private> j2_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Me;
var<private> N1_: vec4<f32>;
var<private> G0_1: f32;

fn main_1() {
    var phi_2206_: f32;
    var phi_2207_: f32;
    var phi_2223_: vec4<f32>;
    var phi_2222_: vec4<f32>;
    var phi_1332_: bool;
    var phi_1347_: bool;
    var phi_2208_: f32;
    var phi_2218_: vec4<f32>;
    var phi_2225_: vec4<f32>;
    var phi_2226_: vec4<f32>;
    var phi_1559_: bool;
    var phi_2227_: f32;
    var phi_2252_: f32;
    var phi_2253_: f32;
    var phi_1496_: bool;
    var phi_2254_: f32;
    var phi_2255_: f32;
    var phi_2257_: f32;
    var phi_1074_: bool;
    var phi_2258_: f32;
    var local: bool;
    var phi_1914_: bool;
    var phi_1916_: bool;
    var phi_2286_: f32;
    var phi_2281_: u32;
    var phi_2278_: f32;
    var phi_2285_: f32;
    var phi_2280_: u32;
    var phi_2277_: f32;
    var phi_2282_: f32;
    var phi_2279_: u32;
    var phi_2276_: f32;
    var phi_2287_: f32;
    var phi_2288_: f32;
    var phi_2289_: f32;
    var phi_2290_: f32;
    var phi_2293_: f32;
    var phi_2291_: f32;
    var phi_2307_: f32;
    var phi_2339_: vec3<f32>;

    let _e74 = gl_FragCoord_1;
    let _e78 = bitcast<vec2<u32>>(vec2<i32>(floor(_e74.xy)));
    let _e80 = j.L6_;
    let _e109 = bitcast<i32>((((((_e78.y >> bitcast<u32>(5u)) * (((_e80 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e78.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e78.x & 28u) << bitcast<u32>(5u)) + ((_e78.y & 28u) << bitcast<u32>(2i)))) + (((_e78.y & 3u) << bitcast<u32>(2i)) + (_e78.x & 3u))));
    let _e110 = P0_1;
    let _e112 = V0_1;
    let _e113 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e116 = (Wi && (u32(_e110) != 0u));
            if (_e113.w >= 0f) {
                phi_2222_ = _e113;
            } else {
                let _e119 = -(_e113.w);
                let _e124 = j.ad;
                let _e127 = j.g7_;
                if (_e113.z > 0f) {
                    phi_2206_ = _e113.x;
                } else {
                    phi_2206_ = length(_e113.xy);
                }
                let _e135 = phi_2206_;
                let _e136 = clamp(_e135, 0f, 1f);
                let _e137 = abs(_e113.z);
                if (_e137 > 1f) {
                    phi_2207_ = ((0.9980469f * _e136) + 0.0009765625f);
                } else {
                    phi_2207_ = ((0.001953125f * _e136) + _e137);
                }
                let _e144 = phi_2207_;
                let _e146 = textureSampleLevel(YC, H8_, vec2<f32>(_e144, ((floor(_e119) * _e124) + _e127)), 0f);
                phi_2223_ = _e146;
                if !(_e116) {
                    let _e150 = (_e146.xyz * _e146.w);
                    phi_2223_ = vec4<f32>(_e150.x, _e150.y, _e150.z, (_e146.w * (fract(_e119) * 1.0039216f)));
                }
                let _e157 = phi_2223_;
                phi_2222_ = _e157;
            }
            let _e159 = phi_2222_;
            phi_1332_ = cj;
            if cj {
                phi_1332_ = (_e112.z < 0f);
            }
            let _e163 = phi_1332_;
            if _e163 {
                let _e165 = textureSampleLevel(TB, S4_, _e112.xy, 0f);
                phi_2226_ = _e165;
                break;
            }
            phi_1347_ = cj;
            if cj {
                phi_1347_ = (_e112.z > 0f);
            }
            let _e169 = phi_1347_;
            phi_2225_ = _e159;
            if _e169 {
                let _e173 = textureSampleLevel(TB, S4_, _e112.xy, (_e112.z - 1f));
                phi_2218_ = _e173;
                if _e116 {
                    if (_e173.w != 0f) {
                        phi_2208_ = (1f / _e173.w);
                    } else {
                        phi_2208_ = 0f;
                    }
                    let _e179 = phi_2208_;
                    let _e180 = (_e173.xyz * _e179);
                    phi_2218_ = vec4<f32>(_e180.x, _e180.y, _e180.z, _e173.w);
                }
                let _e186 = phi_2218_;
                phi_2225_ = (_e159 * _e186);
            }
            let _e189 = phi_2225_;
            phi_2226_ = _e189;
            break;
        }
    }
    let _e191 = phi_2226_;
    let _e192 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e195 = (_e192.y >= 0f);
            local = _e195;
            if _e195 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1496_ = Xi;
                        if Xi {
                            phi_1496_ = (_e192.x < -1.5f);
                        }
                        let _e263 = phi_1496_;
                        if _e263 {
                            let _e269 = textureSampleLevel(ZC, Ta, vec2<f32>((3f + _e192.x), 0f), 0f);
                            let _e274 = textureSampleLevel(ZC, Ta, vec2<f32>((1f - _e192.y), 0f), 0f);
                            phi_2254_ = ((1f - _e269.x) - _e274.x);
                            break;
                        } else {
                            phi_2254_ = min(_e192.x, _e192.y);
                            break;
                        }
                    }
                }
                let _e278 = phi_2254_;
                phi_2255_ = _e278;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1559_ = Xi;
                        if Xi {
                            phi_1559_ = (_e192.y < -1.5f);
                        }
                        let _e199 = phi_1559_;
                        if _e199 {
                            let _e203 = max(_e192.w, 0f);
                            if (_e192.z >= 0f) {
                                let _e206 = textureSampleLevel(ZC, Ta, vec2<f32>(_e203, 0f), 0f);
                                phi_2227_ = _e206.x;
                            } else {
                                phi_2227_ = 0f;
                            }
                            let _e209 = phi_2227_;
                            phi_2252_ = _e209;
                            if (abs(_e192.z) < 1000f) {
                                let _e215 = (-2f - _e192.y);
                                let _e217 = ((_e215 - _e203) * 0.5984134f);
                                let _e220 = (vec4(_e203) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e217));
                                let _e226 = ((_e220 * -(_e192.z)) + vec4(((_e215 * _e192.z) + (abs(_e192.x) - 0.25f))));
                                let _e229 = textureSampleLevel(ZC, Ta, vec2<f32>(_e226.x, 0f), 0f);
                                let _e232 = textureSampleLevel(ZC, Ta, vec2<f32>(_e226.y, 0f), 0f);
                                let _e235 = textureSampleLevel(ZC, Ta, vec2<f32>(_e226.z, 0f), 0f);
                                let _e238 = textureSampleLevel(ZC, Ta, vec2<f32>(_e226.w, 0f), 0f);
                                let _e244 = (_e220 * 5.0959306f);
                                phi_2252_ = (_e209 + (dot(vec4<f32>(_e229.x, _e232.x, _e235.x, _e238.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e244) * (_e244 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e217));
                            }
                            let _e253 = phi_2252_;
                            phi_2253_ = (_e253 * sign(_e192.x));
                            break;
                        } else {
                            phi_2253_ = _e192.x;
                            break;
                        }
                    }
                }
                let _e258 = phi_2253_;
                phi_2255_ = _e258;
                break;
            }
        }
    }
    let _e280 = phi_2255_;
    let _e281 = J4_1;
    let _e284 = y3_1[1u];
    let _e286 = y3_1[0u];
    let _e287 = vec2<u32>(floor(_e281));
    let _e314 = (_e286 + (((((_e287.y >> bitcast<u32>(5u)) * (_e284 << bitcast<u32>(5u))) + ((_e287.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e287.x & 28u) << bitcast<u32>(5u)) + ((_e287.y & 28u) << bitcast<u32>(2i)))) + (((_e287.y & 3u) << bitcast<u32>(2i)) + (_e287.x & 3u))));
    phi_2257_ = 1f;
    if Vi {
        let _e315 = W0_1;
        let _e318 = min(_e315.xy, _e315.zw);
        phi_2257_ = min(min(_e318.x, _e318.y), 1f);
    }
    let _e324 = phi_2257_;
    phi_1074_ = Ui;
    if Ui {
        let _e326 = j2_1[0u];
        phi_1074_ = (_e326 != 0f);
    }
    let _e329 = phi_1074_;
    phi_2258_ = _e324;
    if _e329 {
        let _e332 = m0_.v2_[_e109];
        phi_2258_ = min(unpack4x8unorm(_e332).x, _e324);
    }
    let _e337 = phi_2258_;
    let _e339 = clamp(_e280, 0f, max(_e337, 0f));
    let _e341 = local;
    if _e341 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e191.w, _e339) >= 1f) {
                    phi_2290_ = 1f;
                    break;
                }
                let _e432 = j.r2_;
                let _e434 = atomicMax((&Z0_.v2_[_e314]), (_e432 | u32(((abs(_e339) * 1024f) + 0.5f))));
                if (_e434 < _e432) {
                    phi_2289_ = _e339;
                } else {
                    let _e438 = (f32((_e434 & 524287u)) * 0.0009765625f);
                    phi_2289_ = ((max(_e438, _e339) - _e438) / max((1f - (_e438 * _e191.w)), 0.000062f));
                }
                let _e446 = phi_2289_;
                phi_2290_ = _e446;
                break;
            }
        }
        let _e448 = phi_2290_;
        phi_2293_ = _e448;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e347 = u32(((abs(_e339) * 1024f) + 0.5f));
                let _e350 = atomicLoad((&Z0_.v2_[_e314]));
                let _e352 = (min(_e191.w, _e339) >= 1f);
                phi_1916_ = _e352;
                if _e352 {
                    let _e354 = j.r2_;
                    let _e355 = (_e350 < _e354);
                    phi_1914_ = _e355;
                    if !(_e355) {
                        phi_1914_ = (_e350 >= (_e354 | 262144u));
                    }
                    let _e360 = phi_1914_;
                    phi_1916_ = _e360;
                }
                let _e362 = phi_1916_;
                if _e362 {
                    phi_2288_ = 1f;
                    break;
                }
                let _e364 = j.r2_;
                phi_2282_ = 0f;
                phi_2279_ = _e347;
                phi_2276_ = _e339;
                if (_e350 < _e364) {
                    let _e367 = (_e364 | (262144u + _e347));
                    let _e368 = atomicMax((&Z0_.v2_[_e314]), _e367);
                    if (_e368 <= _e364) {
                        phi_2285_ = _e339;
                        phi_2280_ = _e347;
                        phi_2277_ = 0f;
                    } else {
                        phi_2286_ = 0f;
                        phi_2281_ = _e347;
                        phi_2278_ = _e339;
                        if (_e368 < _e367) {
                            let _e372 = ((_e368 & 524287u) - 262144u);
                            let _e374 = (f32(_e372) * 0.0009765625f);
                            phi_2286_ = ((_e339 - _e374) / max((1f - (_e374 * _e191.w)), 0.000062f));
                            phi_2281_ = _e372;
                            phi_2278_ = _e374;
                        }
                        let _e381 = phi_2286_;
                        let _e383 = phi_2281_;
                        let _e385 = phi_2278_;
                        phi_2285_ = _e381;
                        phi_2280_ = _e383;
                        phi_2277_ = _e385;
                    }
                    let _e387 = phi_2285_;
                    let _e389 = phi_2280_;
                    let _e391 = phi_2277_;
                    phi_2282_ = _e387;
                    phi_2279_ = _e389;
                    phi_2276_ = _e391;
                }
                let _e393 = phi_2282_;
                let _e395 = phi_2279_;
                let _e397 = phi_2276_;
                phi_2287_ = _e393;
                if (_e397 > 0f) {
                    let _e399 = atomicAdd((&Z0_.v2_[_e314]), _e395);
                    let _e404 = (f32(bitcast<i32>(((_e399 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e406 = clamp(_e404, 0f, 1f);
                    phi_2287_ = (_e393 + ((1f - (_e393 * _e191.w)) * ((clamp((_e404 + _e397), 0f, 1f) - _e406) / max((1f - (_e406 * _e191.w)), 0.000062f))));
                }
                let _e418 = phi_2287_;
                phi_2288_ = _e418;
                break;
            }
        }
        let _e420 = phi_2288_;
        phi_2293_ = _e420;
    }
    let _e450 = phi_2293_;
    phi_2307_ = f32();
    if bj {
        let _e452 = j.E3_;
        let _e454 = j.F3_;
        if bj {
            phi_2291_ = ((fract((52.982918f * fract(((0.06711056f * _e74.x) + (0.00583715f * _e74.y))))) * _e452) + _e454);
        } else {
            phi_2291_ = 0f;
        }
        let _e466 = phi_2291_;
        phi_2307_ = _e466;
    }
    let _e468 = phi_2307_;
    let _e469 = (_e191 * _e450);
    let _e470 = _e469.xyz;
    if (bj && (_e469.w != 0f)) {
        phi_2339_ = (vec3(_e468) + _e470);
    } else {
        phi_2339_ = _e470;
    }
    let _e477 = phi_2339_;
    let _e483 = vec4<f32>(_e477.x, _e469.y, _e469.z, _e469.w);
    let _e489 = vec4<f32>(_e483.x, _e477.y, _e483.z, _e483.w);
    m0_.v2_[_e109] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    N1_ = vec4<f32>(_e489.x, _e489.y, _e477.z, _e489.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) V0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) J4_: vec2<f32>, @location(7) @interpolate(flat, either) y3_: vec2<u32>, @location(5) W0_: vec4<f32>, @location(4) @interpolate(flat, either) j2_: vec2<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    V0_1 = V0_;
    P0_1 = P0_;
    O0_1 = O0_;
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
