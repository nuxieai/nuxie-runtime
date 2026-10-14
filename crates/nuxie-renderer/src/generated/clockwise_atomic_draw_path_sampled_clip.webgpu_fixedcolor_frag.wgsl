struct VB {
    xd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    F8_: vec4<i32>,
    zi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Di: f32,
    w6_: u32,
    i3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    wi: u32,
    Ba: u32,
    M8_: f32,
    N8_: f32,
}

struct zf {
    r2_: array<u32>,
}

struct zf_1 {
    r2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(0) member: vec4<f32>,
    @location(1) member_1: vec4<f32>,
}

@id(7) override ej: bool = true;
@id(2) override Zi: bool = true;
@id(8) override fj: bool = true;
@id(3) override aj: bool = true;
@id(1) override Yi: bool = true;
@id(0) override Xi: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var O8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Y0_: zf_1;
var<private> U0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> L4_1: vec2<f32>;
var<private> z3_1: vec2<u32>;
var<private> V0_1: vec4<f32>;
var<private> i2_1: vec2<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> L1_: vec4<f32>;
var<private> G0_1: f32;
var<private> m0_: vec4<f32>;

fn main_1() {
    var phi_2262_: f32;
    var phi_2263_: f32;
    var phi_2264_: vec2<f32>;
    var phi_2281_: vec4<f32>;
    var phi_2280_: vec4<f32>;
    var phi_1322_: bool;
    var phi_1337_: bool;
    var phi_2265_: f32;
    var phi_2276_: vec4<f32>;
    var phi_2283_: vec4<f32>;
    var phi_2284_: vec4<f32>;
    var phi_1627_: bool;
    var phi_2285_: f32;
    var phi_2312_: f32;
    var phi_2313_: f32;
    var phi_1564_: bool;
    var phi_2314_: f32;
    var phi_2315_: f32;
    var phi_2317_: f32;
    var phi_1117_: bool;
    var phi_2318_: f32;
    var local: bool;
    var phi_1982_: bool;
    var phi_1984_: bool;
    var phi_2346_: f32;
    var phi_2341_: u32;
    var phi_2338_: f32;
    var phi_2345_: f32;
    var phi_2340_: u32;
    var phi_2337_: f32;
    var phi_2342_: f32;
    var phi_2339_: u32;
    var phi_2336_: f32;
    var phi_2347_: f32;
    var phi_2348_: f32;
    var phi_2349_: f32;
    var phi_2350_: f32;
    var phi_2353_: f32;
    var phi_2351_: f32;
    var phi_2367_: f32;
    var phi_2400_: vec3<f32>;

    let _e75 = P0_1;
    let _e77 = U0_1;
    let _e78 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e81 = (Zi && (u32(_e75) != 0u));
            if (_e78.w >= 0f) {
                phi_2280_ = _e78;
            } else {
                let _e85 = j.M8_;
                let _e87 = j.N8_;
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
                                phi_2263_ = _e118;
                            } else {
                                phi_2263_ = -(_e118);
                            }
                            let _e123 = phi_2263_;
                            phi_2264_ = vec2<f32>((_e123 * 0.15915507f), _e96);
                            break;
                        } else {
                            if (_e97 == 2f) {
                                phi_2262_ = _e78.x;
                            } else {
                                phi_2262_ = length(_e78.xy);
                            }
                            let _e105 = phi_2262_;
                            phi_2264_ = vec2<f32>(((clamp(_e105, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e78.z < 0f))) + _e94), _e96);
                            break;
                        }
                    }
                }
                let _e127 = phi_2264_;
                let _e128 = textureSampleLevel(XC, O8_, _e127, 0f);
                phi_2281_ = _e128;
                if !(_e81) {
                    let _e134 = (_e128.xyz * _e128.w);
                    phi_2281_ = vec4<f32>(_e134.x, _e134.y, _e134.z, (_e128.w * (ceil(_e78.w) * -0.003921569f)));
                }
                let _e141 = phi_2281_;
                phi_2280_ = _e141;
            }
            let _e143 = phi_2280_;
            phi_1322_ = fj;
            if fj {
                phi_1322_ = (_e77.z < 0f);
            }
            let _e147 = phi_1322_;
            if _e147 {
                let _e149 = textureSampleLevel(TB, U4_, _e77.xy, 0f);
                phi_2284_ = _e149;
                break;
            }
            phi_1337_ = fj;
            if fj {
                phi_1337_ = (_e77.z > 0f);
            }
            let _e153 = phi_1337_;
            phi_2283_ = _e143;
            if _e153 {
                let _e157 = textureSampleLevel(TB, U4_, _e77.xy, (_e77.z - 1f));
                phi_2276_ = _e157;
                if _e81 {
                    if (_e157.w != 0f) {
                        phi_2265_ = (1f / _e157.w);
                    } else {
                        phi_2265_ = 0f;
                    }
                    let _e163 = phi_2265_;
                    let _e164 = (_e157.xyz * _e163);
                    phi_2276_ = vec4<f32>(_e164.x, _e164.y, _e164.z, _e157.w);
                }
                let _e170 = phi_2276_;
                phi_2283_ = (_e143 * _e170);
            }
            let _e173 = phi_2283_;
            phi_2284_ = _e173;
            break;
        }
    }
    let _e175 = phi_2284_;
    let _e176 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e179 = (_e176.y >= 0f);
            local = _e179;
            if _e179 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1564_ = aj;
                        if aj {
                            phi_1564_ = (_e176.x < -1.5f);
                        }
                        let _e247 = phi_1564_;
                        if _e247 {
                            let _e253 = textureSampleLevel(YC, ab, vec2<f32>((3f + _e176.x), 0f), 0f);
                            let _e258 = textureSampleLevel(YC, ab, vec2<f32>((1f - _e176.y), 0f), 0f);
                            phi_2314_ = ((1f - _e253.x) - _e258.x);
                            break;
                        } else {
                            phi_2314_ = min(_e176.x, _e176.y);
                            break;
                        }
                    }
                }
                let _e262 = phi_2314_;
                phi_2315_ = _e262;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1627_ = aj;
                        if aj {
                            phi_1627_ = (_e176.y < -1.5f);
                        }
                        let _e183 = phi_1627_;
                        if _e183 {
                            let _e187 = max(_e176.w, 0f);
                            if (_e176.z >= 0f) {
                                let _e190 = textureSampleLevel(YC, ab, vec2<f32>(_e187, 0f), 0f);
                                phi_2285_ = _e190.x;
                            } else {
                                phi_2285_ = 0f;
                            }
                            let _e193 = phi_2285_;
                            phi_2312_ = _e193;
                            if (abs(_e176.z) < 1000f) {
                                let _e199 = (-2f - _e176.y);
                                let _e201 = ((_e199 - _e187) * 0.5984134f);
                                let _e204 = (vec4(_e187) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e201));
                                let _e210 = ((_e204 * -(_e176.z)) + vec4(((_e199 * _e176.z) + (abs(_e176.x) - 0.25f))));
                                let _e213 = textureSampleLevel(YC, ab, vec2<f32>(_e210.x, 0f), 0f);
                                let _e216 = textureSampleLevel(YC, ab, vec2<f32>(_e210.y, 0f), 0f);
                                let _e219 = textureSampleLevel(YC, ab, vec2<f32>(_e210.z, 0f), 0f);
                                let _e222 = textureSampleLevel(YC, ab, vec2<f32>(_e210.w, 0f), 0f);
                                let _e228 = (_e204 * 5.0959306f);
                                phi_2312_ = (_e193 + (dot(vec4<f32>(_e213.x, _e216.x, _e219.x, _e222.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e228) * (_e228 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e201));
                            }
                            let _e237 = phi_2312_;
                            phi_2313_ = (_e237 * sign(_e176.x));
                            break;
                        } else {
                            phi_2313_ = _e176.x;
                            break;
                        }
                    }
                }
                let _e242 = phi_2313_;
                phi_2315_ = _e242;
                break;
            }
        }
    }
    let _e264 = phi_2315_;
    let _e265 = L4_1;
    let _e268 = z3_1[1u];
    let _e270 = z3_1[0u];
    let _e271 = vec2<u32>(floor(_e265));
    let _e298 = (_e270 + (((((_e271.y >> bitcast<u32>(5u)) * (_e268 << bitcast<u32>(5u))) + ((_e271.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e271.x & 28u) << bitcast<u32>(5u)) + ((_e271.y & 28u) << bitcast<u32>(2i)))) + (((_e271.y & 3u) << bitcast<u32>(2i)) + (_e271.x & 3u))));
    phi_2317_ = 1f;
    if Yi {
        let _e299 = V0_1;
        let _e302 = min(_e299.xy, _e299.zw);
        phi_2317_ = min(min(_e302.x, _e302.y), 1f);
    }
    let _e308 = phi_2317_;
    phi_1117_ = Xi;
    if Xi {
        let _e310 = i2_1[0u];
        phi_1117_ = (_e310 != 0f);
    }
    let _e313 = phi_1117_;
    phi_2318_ = _e308;
    if _e313 {
        phi_2318_ = min(0f, _e308);
    }
    let _e316 = phi_2318_;
    let _e318 = clamp(_e264, 0f, max(_e316, 0f));
    let _e320 = local;
    if _e320 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e175.w, _e318) >= 1f) {
                    phi_2350_ = 1f;
                    break;
                }
                let _e411 = j.q2_;
                let _e413 = atomicMax((&Y0_.r2_[_e298]), (_e411 | u32(((abs(_e318) * 1024f) + 0.5f))));
                if (_e413 < _e411) {
                    phi_2349_ = _e318;
                } else {
                    let _e417 = (f32((_e413 & 524287u)) * 0.0009765625f);
                    phi_2349_ = ((max(_e417, _e318) - _e417) / max((1f - (_e417 * _e175.w)), 0.000062f));
                }
                let _e425 = phi_2349_;
                phi_2350_ = _e425;
                break;
            }
        }
        let _e427 = phi_2350_;
        phi_2353_ = _e427;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e326 = u32(((abs(_e318) * 1024f) + 0.5f));
                let _e329 = atomicLoad((&Y0_.r2_[_e298]));
                let _e331 = (min(_e175.w, _e318) >= 1f);
                phi_1984_ = _e331;
                if _e331 {
                    let _e333 = j.q2_;
                    let _e334 = (_e329 < _e333);
                    phi_1982_ = _e334;
                    if !(_e334) {
                        phi_1982_ = (_e329 >= (_e333 | 262144u));
                    }
                    let _e339 = phi_1982_;
                    phi_1984_ = _e339;
                }
                let _e341 = phi_1984_;
                if _e341 {
                    phi_2348_ = 1f;
                    break;
                }
                let _e343 = j.q2_;
                phi_2342_ = 0f;
                phi_2339_ = _e326;
                phi_2336_ = _e318;
                if (_e329 < _e343) {
                    let _e346 = (_e343 | (262144u + _e326));
                    let _e347 = atomicMax((&Y0_.r2_[_e298]), _e346);
                    if (_e347 <= _e343) {
                        phi_2345_ = _e318;
                        phi_2340_ = _e326;
                        phi_2337_ = 0f;
                    } else {
                        phi_2346_ = 0f;
                        phi_2341_ = _e326;
                        phi_2338_ = _e318;
                        if (_e347 < _e346) {
                            let _e351 = ((_e347 & 524287u) - 262144u);
                            let _e353 = (f32(_e351) * 0.0009765625f);
                            phi_2346_ = ((_e318 - _e353) / max((1f - (_e353 * _e175.w)), 0.000062f));
                            phi_2341_ = _e351;
                            phi_2338_ = _e353;
                        }
                        let _e360 = phi_2346_;
                        let _e362 = phi_2341_;
                        let _e364 = phi_2338_;
                        phi_2345_ = _e360;
                        phi_2340_ = _e362;
                        phi_2337_ = _e364;
                    }
                    let _e366 = phi_2345_;
                    let _e368 = phi_2340_;
                    let _e370 = phi_2337_;
                    phi_2342_ = _e366;
                    phi_2339_ = _e368;
                    phi_2336_ = _e370;
                }
                let _e372 = phi_2342_;
                let _e374 = phi_2339_;
                let _e376 = phi_2336_;
                phi_2347_ = _e372;
                if (_e376 > 0f) {
                    let _e378 = atomicAdd((&Y0_.r2_[_e298]), _e374);
                    let _e383 = (f32(bitcast<i32>(((_e378 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e385 = clamp(_e383, 0f, 1f);
                    phi_2347_ = (_e372 + ((1f - (_e372 * _e175.w)) * ((clamp((_e383 + _e376), 0f, 1f) - _e385) / max((1f - (_e385 * _e175.w)), 0.000062f))));
                }
                let _e397 = phi_2347_;
                phi_2348_ = _e397;
                break;
            }
        }
        let _e399 = phi_2348_;
        phi_2353_ = _e399;
    }
    let _e429 = phi_2353_;
    phi_2367_ = f32();
    if ej {
        let _e430 = gl_FragCoord_1;
        let _e432 = j.F3_;
        let _e434 = j.G3_;
        if ej {
            phi_2351_ = ((fract((52.982918f * fract(((0.06711056f * _e430.x) + (0.00583715f * _e430.y))))) * _e432) + _e434);
        } else {
            phi_2351_ = 0f;
        }
        let _e446 = phi_2351_;
        phi_2367_ = _e446;
    }
    let _e448 = phi_2367_;
    let _e449 = (_e175 * _e429);
    let _e450 = _e449.xyz;
    if (ej && (_e449.w != 0f)) {
        phi_2400_ = (vec3(_e448) + _e450);
    } else {
        phi_2400_ = _e450;
    }
    let _e457 = phi_2400_;
    let _e463 = vec4<f32>(_e457.x, _e449.y, _e449.z, _e449.w);
    let _e469 = vec4<f32>(_e463.x, _e457.y, _e463.z, _e463.w);
    L1_ = vec4<f32>(_e469.x, _e469.y, _e457.z, _e469.w);
    return;
}

@fragment
fn main(@location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) L4_: vec2<f32>, @location(7) @interpolate(flat, either) z3_: vec2<u32>, @location(5) V0_: vec4<f32>, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> FragmentOutput {
    U0_1 = U0_;
    P0_1 = P0_;
    O0_1 = O0_;
    S_1 = S;
    L4_1 = L4_;
    z3_1 = z3_;
    V0_1 = V0_;
    i2_1 = i2_;
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    main_1();
    let _e22 = L1_;
    let _e23 = m0_;
    return FragmentOutput(_e22, _e23);
}
