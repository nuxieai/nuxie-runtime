struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Re {
    j2_: array<u32>,
}

struct Re_1 {
    j2_: array<atomic<u32>>,
}

@id(7) override ii: bool = true;
@id(2) override di: bool = true;
@id(8) override ji: bool = true;
@id(3) override ei: bool = true;
@id(1) override ci: bool = true;
@id(0) override bi: bool = true;

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var f6_: sampler;
@group(0) @binding(6)
var<storage, read_write> V0_: Re_1;
var<private> F1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> F4_1: vec2<f32>;
var<private> q3_1: vec2<u32>;
var<private> R0_1: vec4<f32>;
var<private> l1_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> J1_: vec4<f32>;
var<private> F0_1: f32;

fn main_1() {
    var phi_2047_: f32;
    var phi_2048_: f32;
    var phi_2062_: vec4<f32>;
    var phi_2061_: vec4<f32>;
    var phi_1221_: bool;
    var phi_2049_: f32;
    var phi_2058_: vec4<f32>;
    var phi_2064_: vec4<f32>;
    var phi_1427_: bool;
    var phi_2065_: f32;
    var phi_2086_: f32;
    var phi_2087_: f32;
    var phi_1364_: bool;
    var phi_2088_: f32;
    var phi_2089_: f32;
    var phi_2091_: f32;
    var phi_1013_: bool;
    var phi_2092_: f32;
    var local: bool;
    var phi_1782_: bool;
    var phi_1784_: bool;
    var phi_2120_: f32;
    var phi_2115_: u32;
    var phi_2112_: f32;
    var phi_2119_: f32;
    var phi_2114_: u32;
    var phi_2111_: f32;
    var phi_2116_: f32;
    var phi_2113_: u32;
    var phi_2110_: f32;
    var phi_2121_: f32;
    var phi_2122_: f32;
    var phi_2123_: f32;
    var phi_2124_: f32;
    var phi_2127_: f32;
    var phi_2125_: f32;
    var phi_2141_: f32;
    var phi_2171_: vec3<f32>;

    let _e70 = Q0_1;
    let _e72 = F1_1;
    let _e73 = a1_1;
    let _e75 = (di && (u32(_e70) != 0u));
    if (_e73.w >= 0f) {
        phi_2061_ = _e73;
    } else {
        let _e78 = -(_e73.w);
        let _e83 = j.wc;
        let _e86 = j.xc;
        if (_e73.z > 0f) {
            phi_2047_ = _e73.x;
        } else {
            phi_2047_ = length(_e73.xy);
        }
        let _e94 = phi_2047_;
        let _e95 = clamp(_e94, 0f, 1f);
        let _e96 = abs(_e73.z);
        if (_e96 > 1f) {
            phi_2048_ = ((0.9980469f * _e95) + 0.0009765625f);
        } else {
            phi_2048_ = ((0.001953125f * _e95) + _e96);
        }
        let _e103 = phi_2048_;
        let _e105 = textureSampleLevel(FD, ha, vec2<f32>(_e103, ((floor(_e78) * _e83) + _e86)), 0f);
        phi_2062_ = _e105;
        if !(_e75) {
            let _e109 = (_e105.xyz * _e105.w);
            phi_2062_ = vec4<f32>(_e109.x, _e109.y, _e109.z, (_e105.w * (fract(_e78) * 1.0039216f)));
        }
        let _e116 = phi_2062_;
        phi_2061_ = _e116;
    }
    let _e118 = phi_2061_;
    phi_1221_ = ji;
    if ji {
        phi_1221_ = (_e72.z > 0f);
    }
    let _e122 = phi_1221_;
    phi_2064_ = _e118;
    if _e122 {
        let _e126 = textureSampleLevel(IC, f6_, _e72.xy, (_e72.z - 1f));
        phi_2058_ = _e126;
        if _e75 {
            if (_e126.w != 0f) {
                phi_2049_ = (1f / _e126.w);
            } else {
                phi_2049_ = 0f;
            }
            let _e132 = phi_2049_;
            let _e133 = (_e126.xyz * _e132);
            phi_2058_ = vec4<f32>(_e133.x, _e133.y, _e133.z, _e126.w);
        }
        let _e139 = phi_2058_;
        phi_2064_ = (_e118 * _e139);
    }
    let _e142 = phi_2064_;
    let _e143 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e146 = (_e143.y >= 0f);
            local = _e146;
            if _e146 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1364_ = ei;
                        if ei {
                            phi_1364_ = (_e143.x < -1.5f);
                        }
                        let _e214 = phi_1364_;
                        if _e214 {
                            let _e220 = textureSampleLevel(ZC, wa, vec2<f32>((3f + _e143.x), 0f), 0f);
                            let _e225 = textureSampleLevel(ZC, wa, vec2<f32>((1f - _e143.y), 0f), 0f);
                            phi_2088_ = ((1f - _e220.x) - _e225.x);
                            break;
                        } else {
                            phi_2088_ = min(_e143.x, _e143.y);
                            break;
                        }
                    }
                }
                let _e229 = phi_2088_;
                phi_2089_ = _e229;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1427_ = ei;
                        if ei {
                            phi_1427_ = (_e143.y < -1.5f);
                        }
                        let _e150 = phi_1427_;
                        if _e150 {
                            let _e154 = max(_e143.w, 0f);
                            if (_e143.z >= 0f) {
                                let _e157 = textureSampleLevel(ZC, wa, vec2<f32>(_e154, 0f), 0f);
                                phi_2065_ = _e157.x;
                            } else {
                                phi_2065_ = 0f;
                            }
                            let _e160 = phi_2065_;
                            phi_2086_ = _e160;
                            if (abs(_e143.z) < 1000f) {
                                let _e166 = (-2f - _e143.y);
                                let _e168 = ((_e166 - _e154) * 0.5984134f);
                                let _e171 = (vec4(_e154) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e168));
                                let _e177 = ((_e171 * -(_e143.z)) + vec4(((_e166 * _e143.z) + (abs(_e143.x) - 0.25f))));
                                let _e180 = textureSampleLevel(ZC, wa, vec2<f32>(_e177.x, 0f), 0f);
                                let _e183 = textureSampleLevel(ZC, wa, vec2<f32>(_e177.y, 0f), 0f);
                                let _e186 = textureSampleLevel(ZC, wa, vec2<f32>(_e177.z, 0f), 0f);
                                let _e189 = textureSampleLevel(ZC, wa, vec2<f32>(_e177.w, 0f), 0f);
                                let _e195 = (_e171 * 5.0959306f);
                                phi_2086_ = (_e160 + (dot(vec4<f32>(_e180.x, _e183.x, _e186.x, _e189.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e195) * (_e195 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e168));
                            }
                            let _e204 = phi_2086_;
                            phi_2087_ = (_e204 * sign(_e143.x));
                            break;
                        } else {
                            phi_2087_ = _e143.x;
                            break;
                        }
                    }
                }
                let _e209 = phi_2087_;
                phi_2089_ = _e209;
                break;
            }
        }
    }
    let _e231 = phi_2089_;
    let _e232 = F4_1;
    let _e235 = q3_1[1u];
    let _e237 = q3_1[0u];
    let _e238 = vec2<u32>(floor(_e232));
    let _e265 = (_e237 + (((((_e238.y >> bitcast<u32>(5u)) * (_e235 << bitcast<u32>(5u))) + ((_e238.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e238.x & 28u) << bitcast<u32>(5u)) + ((_e238.y & 28u) << bitcast<u32>(2i)))) + (((_e238.y & 3u) << bitcast<u32>(2i)) + (_e238.x & 3u))));
    phi_2091_ = 1f;
    if ci {
        let _e266 = R0_1;
        let _e269 = min(_e266.xy, _e266.zw);
        phi_2091_ = min(min(_e269.x, _e269.y), 1f);
    }
    let _e275 = phi_2091_;
    phi_1013_ = bi;
    if bi {
        let _e277 = l1_1[0u];
        phi_1013_ = (_e277 != 0f);
    }
    let _e280 = phi_1013_;
    phi_2092_ = _e275;
    if _e280 {
        let _e281 = gl_FragCoord_1;
        let _e285 = textureLoad(m0_, vec2<i32>(floor(_e281.xy)), 0i);
        phi_2092_ = min(_e285.x, _e275);
    }
    let _e289 = phi_2092_;
    let _e291 = clamp(_e231, 0f, max(_e289, 0f));
    let _e293 = local;
    if _e293 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e142.w, _e291) >= 1f) {
                    phi_2124_ = 1f;
                    break;
                }
                let _e384 = j.i2_;
                let _e386 = atomicMax((&V0_.j2_[_e265]), (_e384 | u32(((abs(_e291) * 1024f) + 0.5f))));
                if (_e386 < _e384) {
                    phi_2123_ = _e291;
                } else {
                    let _e390 = (f32((_e386 & 524287u)) * 0.0009765625f);
                    phi_2123_ = ((max(_e390, _e291) - _e390) / max((1f - (_e390 * _e142.w)), 0.000062f));
                }
                let _e398 = phi_2123_;
                phi_2124_ = _e398;
                break;
            }
        }
        let _e400 = phi_2124_;
        phi_2127_ = _e400;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e299 = u32(((abs(_e291) * 1024f) + 0.5f));
                let _e302 = atomicLoad((&V0_.j2_[_e265]));
                let _e304 = (min(_e142.w, _e291) >= 1f);
                phi_1784_ = _e304;
                if _e304 {
                    let _e306 = j.i2_;
                    let _e307 = (_e302 < _e306);
                    phi_1782_ = _e307;
                    if !(_e307) {
                        phi_1782_ = (_e302 >= (_e306 | 262144u));
                    }
                    let _e312 = phi_1782_;
                    phi_1784_ = _e312;
                }
                let _e314 = phi_1784_;
                if _e314 {
                    phi_2122_ = 1f;
                    break;
                }
                let _e316 = j.i2_;
                phi_2116_ = 0f;
                phi_2113_ = _e299;
                phi_2110_ = _e291;
                if (_e302 < _e316) {
                    let _e319 = (_e316 | (262144u + _e299));
                    let _e320 = atomicMax((&V0_.j2_[_e265]), _e319);
                    if (_e320 <= _e316) {
                        phi_2119_ = _e291;
                        phi_2114_ = _e299;
                        phi_2111_ = 0f;
                    } else {
                        phi_2120_ = 0f;
                        phi_2115_ = _e299;
                        phi_2112_ = _e291;
                        if (_e320 < _e319) {
                            let _e324 = ((_e320 & 524287u) - 262144u);
                            let _e326 = (f32(_e324) * 0.0009765625f);
                            phi_2120_ = ((_e291 - _e326) / max((1f - (_e326 * _e142.w)), 0.000062f));
                            phi_2115_ = _e324;
                            phi_2112_ = _e326;
                        }
                        let _e333 = phi_2120_;
                        let _e335 = phi_2115_;
                        let _e337 = phi_2112_;
                        phi_2119_ = _e333;
                        phi_2114_ = _e335;
                        phi_2111_ = _e337;
                    }
                    let _e339 = phi_2119_;
                    let _e341 = phi_2114_;
                    let _e343 = phi_2111_;
                    phi_2116_ = _e339;
                    phi_2113_ = _e341;
                    phi_2110_ = _e343;
                }
                let _e345 = phi_2116_;
                let _e347 = phi_2113_;
                let _e349 = phi_2110_;
                phi_2121_ = _e345;
                if (_e349 > 0f) {
                    let _e351 = atomicAdd((&V0_.j2_[_e265]), _e347);
                    let _e356 = (f32(bitcast<i32>(((_e351 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e358 = clamp(_e356, 0f, 1f);
                    phi_2121_ = (_e345 + ((1f - (_e345 * _e142.w)) * ((clamp((_e356 + _e349), 0f, 1f) - _e358) / max((1f - (_e358 * _e142.w)), 0.000062f))));
                }
                let _e370 = phi_2121_;
                phi_2122_ = _e370;
                break;
            }
        }
        let _e372 = phi_2122_;
        phi_2127_ = _e372;
    }
    let _e402 = phi_2127_;
    phi_2141_ = f32();
    if ii {
        let _e403 = gl_FragCoord_1;
        let _e405 = j.M3_;
        let _e407 = j.N3_;
        if ii {
            phi_2125_ = ((fract((52.982918f * fract(((0.06711056f * _e403.x) + (0.00583715f * _e403.y))))) * _e405) + _e407);
        } else {
            phi_2125_ = 0f;
        }
        let _e419 = phi_2125_;
        phi_2141_ = _e419;
    }
    let _e421 = phi_2141_;
    let _e422 = (_e142 * _e402);
    let _e423 = _e422.xyz;
    if (ii && (_e422.w != 0f)) {
        phi_2171_ = (vec3(_e421) + _e423);
    } else {
        phi_2171_ = _e423;
    }
    let _e430 = phi_2171_;
    let _e436 = vec4<f32>(_e430.x, _e422.y, _e422.z, _e422.w);
    let _e442 = vec4<f32>(_e436.x, _e430.y, _e436.z, _e436.w);
    J1_ = vec4<f32>(_e442.x, _e442.y, _e430.z, _e442.w);
    return;
}

@fragment
fn main(@location(9) F1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) F4_: vec2<f32>, @location(7) @interpolate(flat, either) q3_: vec2<u32>, @location(5) R0_: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) F0_: f32) -> @location(0) vec4<f32> {
    F1_1 = F1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    S_1 = S;
    F4_1 = F4_;
    q3_1 = q3_;
    R0_1 = R0_;
    l1_1 = l1_;
    gl_FragCoord_1 = gl_FragCoord;
    F0_1 = F0_;
    main_1();
    let _e21 = J1_;
    return _e21;
}
