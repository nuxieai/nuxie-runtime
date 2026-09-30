struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct Ae {
    g2_: array<u32>,
}

struct Ae_1 {
    g2_: array<atomic<u32>>,
}

@id(7) override Oh: bool = true;
@id(2) override Jh: bool = true;
@id(8) override Ph: bool = true;
@id(3) override Kh: bool = true;
@id(1) override Ih: bool = true;
@id(0) override Hh: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(0)
var<uniform> j: TB;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var M9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(6)
var<storage, read_write> S0_: Ae_1;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> O_1: vec4<f32>;
var<private> v4_1: vec2<f32>;
var<private> k3_1: vec2<u32>;
var<private> O0_1: vec4<f32>;
var<private> Y1_1: vec2<f32>;
@group(2) @binding(1)
var i0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> F1_: vec4<f32>;
var<private> D0_1: f32;

fn main_1() {
    var phi_2048_: f32;
    var phi_2049_: f32;
    var phi_2063_: vec4<f32>;
    var phi_2062_: vec4<f32>;
    var phi_1222_: bool;
    var phi_2050_: f32;
    var phi_2059_: vec4<f32>;
    var phi_2065_: vec4<f32>;
    var phi_1428_: bool;
    var phi_2066_: f32;
    var phi_2087_: f32;
    var phi_2088_: f32;
    var phi_1365_: bool;
    var phi_2089_: f32;
    var phi_2090_: f32;
    var phi_2092_: f32;
    var phi_1013_: bool;
    var phi_2093_: f32;
    var local: bool;
    var phi_1783_: bool;
    var phi_1785_: bool;
    var phi_2121_: f32;
    var phi_2116_: u32;
    var phi_2113_: f32;
    var phi_2120_: f32;
    var phi_2115_: u32;
    var phi_2112_: f32;
    var phi_2117_: f32;
    var phi_2114_: u32;
    var phi_2111_: f32;
    var phi_2122_: f32;
    var phi_2123_: f32;
    var phi_2124_: f32;
    var phi_2125_: f32;
    var phi_2128_: f32;
    var phi_2126_: f32;
    var phi_2142_: f32;
    var phi_2172_: vec3<f32>;

    let _e70 = g1_1;
    let _e72 = C2_1;
    let _e73 = X1_1;
    let _e75 = (Jh && (u32(_e70) != 0u));
    if (_e73.w >= 0f) {
        phi_2062_ = _e73;
    } else {
        let _e78 = -(_e73.w);
        let _e83 = j.Zb;
        let _e86 = j.ac;
        if (_e73.z > 0f) {
            phi_2048_ = _e73.x;
        } else {
            phi_2048_ = length(_e73.xy);
        }
        let _e94 = phi_2048_;
        let _e95 = clamp(_e94, 0f, 1f);
        let _e96 = abs(_e73.z);
        if (_e96 > 1f) {
            phi_2049_ = ((0.9980469f * _e95) + 0.0009765625f);
        } else {
            phi_2049_ = ((0.001953125f * _e95) + _e96);
        }
        let _e103 = phi_2049_;
        let _e105 = textureSampleLevel(DD, M9_, vec2<f32>(_e103, ((floor(_e78) * _e83) + _e86)), 0f);
        phi_2063_ = _e105;
        if !(_e75) {
            let _e109 = (_e105.xyz * _e105.w);
            phi_2063_ = vec4<f32>(_e109.x, _e109.y, _e109.z, (_e105.w * (fract(_e78) * 1.0039216f)));
        }
        let _e116 = phi_2063_;
        phi_2062_ = _e116;
    }
    let _e118 = phi_2062_;
    phi_1222_ = Ph;
    if Ph {
        phi_1222_ = (_e72.z > 0f);
    }
    let _e122 = phi_1222_;
    phi_2065_ = _e118;
    if _e122 {
        let _e126 = textureSampleLevel(GC, W5_, _e72.xy, (_e72.z - 1f));
        phi_2059_ = _e126;
        if _e75 {
            if (_e126.w != 0f) {
                phi_2050_ = (1f / _e126.w);
            } else {
                phi_2050_ = 0f;
            }
            let _e132 = phi_2050_;
            let _e133 = (_e126.xyz * _e132);
            phi_2059_ = vec4<f32>(_e133.x, _e133.y, _e133.z, _e126.w);
        }
        let _e139 = phi_2059_;
        phi_2065_ = (_e118 * _e139);
    }
    let _e142 = phi_2065_;
    let _e143 = O_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e146 = (_e143.y >= 0f);
            local = _e146;
            if _e146 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1365_ = Kh;
                        if Kh {
                            phi_1365_ = (_e143.x < -1.5f);
                        }
                        let _e214 = phi_1365_;
                        if _e214 {
                            let _e220 = textureSampleLevel(XC, ca, vec2<f32>((3f + _e143.x), 0f), 0f);
                            let _e225 = textureSampleLevel(XC, ca, vec2<f32>((1f - _e143.y), 0f), 0f);
                            phi_2089_ = ((1f - _e220.x) - _e225.x);
                            break;
                        } else {
                            phi_2089_ = min(_e143.x, _e143.y);
                            break;
                        }
                    }
                }
                let _e229 = phi_2089_;
                phi_2090_ = _e229;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1428_ = Kh;
                        if Kh {
                            phi_1428_ = (_e143.y < -1.5f);
                        }
                        let _e150 = phi_1428_;
                        if _e150 {
                            let _e154 = max(_e143.w, 0f);
                            if (_e143.z >= 0f) {
                                let _e157 = textureSampleLevel(XC, ca, vec2<f32>(_e154, 0f), 0f);
                                phi_2066_ = _e157.x;
                            } else {
                                phi_2066_ = 0f;
                            }
                            let _e160 = phi_2066_;
                            phi_2087_ = _e160;
                            if (abs(_e143.z) < 1000f) {
                                let _e166 = (-2f - _e143.y);
                                let _e168 = ((_e166 - _e154) * 0.5984134f);
                                let _e171 = (vec4(_e154) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e168));
                                let _e177 = ((_e171 * -(_e143.z)) + vec4(((_e166 * _e143.z) + (abs(_e143.x) - 0.25f))));
                                let _e180 = textureSampleLevel(XC, ca, vec2<f32>(_e177.x, 0f), 0f);
                                let _e183 = textureSampleLevel(XC, ca, vec2<f32>(_e177.y, 0f), 0f);
                                let _e186 = textureSampleLevel(XC, ca, vec2<f32>(_e177.z, 0f), 0f);
                                let _e189 = textureSampleLevel(XC, ca, vec2<f32>(_e177.w, 0f), 0f);
                                let _e195 = (_e171 * 5.0959306f);
                                phi_2087_ = (_e160 + (dot(vec4<f32>(_e180.x, _e183.x, _e186.x, _e189.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e195) * (_e195 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e168));
                            }
                            let _e204 = phi_2087_;
                            phi_2088_ = (_e204 * sign(_e143.x));
                            break;
                        } else {
                            phi_2088_ = _e143.x;
                            break;
                        }
                    }
                }
                let _e209 = phi_2088_;
                phi_2090_ = _e209;
                break;
            }
        }
    }
    let _e231 = phi_2090_;
    let _e232 = v4_1;
    let _e235 = k3_1[1u];
    let _e237 = k3_1[0u];
    let _e238 = vec2<u32>(floor(_e232));
    let _e265 = (_e237 + (((((_e238.y >> bitcast<u32>(5u)) * (_e235 << bitcast<u32>(5u))) + ((_e238.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e238.x & 28u) << bitcast<u32>(5u)) + ((_e238.y & 28u) << bitcast<u32>(2i)))) + (((_e238.y & 3u) << bitcast<u32>(2i)) + (_e238.x & 3u))));
    phi_2092_ = 1f;
    if Ih {
        let _e266 = O0_1;
        let _e269 = min(_e266.xy, _e266.zw);
        phi_2092_ = min(min(_e269.x, _e269.y), 1f);
    }
    let _e275 = phi_2092_;
    phi_1013_ = Hh;
    if Hh {
        let _e277 = Y1_1[0u];
        phi_1013_ = (_e277 != 0f);
    }
    let _e280 = phi_1013_;
    phi_2093_ = _e275;
    if _e280 {
        let _e281 = gl_FragCoord_1;
        let _e285 = textureLoad(i0_, vec2<i32>(floor(_e281.xy)), 0i);
        phi_2093_ = min(_e285.x, _e275);
    }
    let _e289 = phi_2093_;
    let _e291 = clamp(_e231, 0f, max(_e289, 0f));
    let _e293 = local;
    if _e293 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e142.w, _e291) >= 1f) {
                    phi_2125_ = 1f;
                    break;
                }
                let _e384 = j.f2_;
                let _e386 = atomicMax((&S0_.g2_[_e265]), (_e384 | u32(((abs(_e291) * 1024f) + 0.5f))));
                if (_e386 < _e384) {
                    phi_2124_ = _e291;
                } else {
                    let _e390 = (f32((_e386 & 524287u)) * 0.0009765625f);
                    phi_2124_ = ((max(_e390, _e291) - _e390) / max((1f - (_e390 * _e142.w)), 0.000062f));
                }
                let _e398 = phi_2124_;
                phi_2125_ = _e398;
                break;
            }
        }
        let _e400 = phi_2125_;
        phi_2128_ = _e400;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e299 = u32(((abs(_e291) * 1024f) + 0.5f));
                let _e302 = atomicLoad((&S0_.g2_[_e265]));
                let _e304 = (min(_e142.w, _e291) >= 1f);
                phi_1785_ = _e304;
                if _e304 {
                    let _e306 = j.f2_;
                    let _e307 = (_e302 < _e306);
                    phi_1783_ = _e307;
                    if !(_e307) {
                        phi_1783_ = (_e302 >= (_e306 | 262144u));
                    }
                    let _e312 = phi_1783_;
                    phi_1785_ = _e312;
                }
                let _e314 = phi_1785_;
                if _e314 {
                    phi_2123_ = 1f;
                    break;
                }
                let _e316 = j.f2_;
                phi_2117_ = 0f;
                phi_2114_ = _e299;
                phi_2111_ = _e291;
                if (_e302 < _e316) {
                    let _e319 = (_e316 | (262144u + _e299));
                    let _e320 = atomicMax((&S0_.g2_[_e265]), _e319);
                    if (_e320 <= _e316) {
                        phi_2120_ = _e291;
                        phi_2115_ = _e299;
                        phi_2112_ = 0f;
                    } else {
                        phi_2121_ = 0f;
                        phi_2116_ = _e299;
                        phi_2113_ = _e291;
                        if (_e320 < _e319) {
                            let _e324 = ((_e320 & 524287u) - 262144u);
                            let _e326 = (f32(_e324) * 0.0009765625f);
                            phi_2121_ = ((_e291 - _e326) / max((1f - (_e326 * _e142.w)), 0.000062f));
                            phi_2116_ = _e324;
                            phi_2113_ = _e326;
                        }
                        let _e333 = phi_2121_;
                        let _e335 = phi_2116_;
                        let _e337 = phi_2113_;
                        phi_2120_ = _e333;
                        phi_2115_ = _e335;
                        phi_2112_ = _e337;
                    }
                    let _e339 = phi_2120_;
                    let _e341 = phi_2115_;
                    let _e343 = phi_2112_;
                    phi_2117_ = _e339;
                    phi_2114_ = _e341;
                    phi_2111_ = _e343;
                }
                let _e345 = phi_2117_;
                let _e347 = phi_2114_;
                let _e349 = phi_2111_;
                phi_2122_ = _e345;
                if (_e349 > 0f) {
                    let _e351 = atomicAdd((&S0_.g2_[_e265]), _e347);
                    let _e356 = (f32(bitcast<i32>(((_e351 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e358 = clamp(_e356, 0f, 1f);
                    phi_2122_ = (_e345 + ((1f - (_e345 * _e142.w)) * ((clamp((_e356 + _e349), 0f, 1f) - _e358) / max((1f - (_e358 * _e142.w)), 0.000062f))));
                }
                let _e370 = phi_2122_;
                phi_2123_ = _e370;
                break;
            }
        }
        let _e372 = phi_2123_;
        phi_2128_ = _e372;
    }
    let _e402 = phi_2128_;
    phi_2142_ = f32();
    if Oh {
        let _e403 = gl_FragCoord_1;
        let _e405 = j.F3_;
        let _e407 = j.G3_;
        if Oh {
            phi_2126_ = ((fract((52.982918f * fract(((0.06711056f * _e403.x) + (0.00583715f * _e403.y))))) * _e405) + _e407);
        } else {
            phi_2126_ = 0f;
        }
        let _e419 = phi_2126_;
        phi_2142_ = _e419;
    }
    let _e421 = phi_2142_;
    let _e422 = (_e142 * _e402);
    let _e423 = _e422.xyz;
    if (Oh && (_e422.w != 0f)) {
        phi_2172_ = (vec3(_e421) + _e423);
    } else {
        phi_2172_ = _e423;
    }
    let _e430 = phi_2172_;
    let _e436 = vec4<f32>(_e430.x, _e422.y, _e422.z, _e422.w);
    let _e442 = vec4<f32>(_e436.x, _e430.y, _e436.z, _e436.w);
    F1_ = vec4<f32>(_e442.x, _e442.y, _e430.z, _e442.w);
    return;
}

@fragment
fn main(@location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @location(2) O: vec4<f32>, @location(8) v4_: vec2<f32>, @location(7) @interpolate(flat, either) k3_: vec2<u32>, @location(5) O0_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) D0_: f32) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    O_1 = O;
    v4_1 = v4_;
    k3_1 = k3_;
    O0_1 = O0_;
    Y1_1 = Y1_;
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    main_1();
    let _e21 = F1_;
    return _e21;
}
