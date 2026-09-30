struct Be {
    g2_: array<u32>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct i0Td {
    g2_: array<u32>,
}

struct Be_1 {
    g2_: array<atomic<u32>>,
}

@id(7) override Lh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;
@id(3) override Hh: bool = true;
@id(1) override Fh: bool = true;
@id(0) override Eh: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;
@group(0) @binding(6)
var<storage, read_write> R0_: Be_1;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> M_1: vec4<f32>;
var<private> r4_1: vec2<f32>;
var<private> i3_1: vec2<u32>;
var<private> N0_1: vec4<f32>;
var<private> Y1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Td;
var<private> E1_: vec4<f32>;
var<private> D0_1: f32;

fn main_1() {
    var phi_2084_: f32;
    var phi_2085_: f32;
    var phi_2099_: vec4<f32>;
    var phi_2098_: vec4<f32>;
    var phi_1244_: bool;
    var phi_2086_: f32;
    var phi_2095_: vec4<f32>;
    var phi_2101_: vec4<f32>;
    var phi_1445_: bool;
    var phi_2102_: f32;
    var phi_2123_: f32;
    var phi_2124_: f32;
    var phi_1382_: bool;
    var phi_2125_: f32;
    var phi_2126_: f32;
    var phi_2128_: f32;
    var phi_1016_: bool;
    var phi_2129_: f32;
    var local: bool;
    var phi_1800_: bool;
    var phi_1802_: bool;
    var phi_2157_: f32;
    var phi_2152_: u32;
    var phi_2149_: f32;
    var phi_2156_: f32;
    var phi_2151_: u32;
    var phi_2148_: f32;
    var phi_2153_: f32;
    var phi_2150_: u32;
    var phi_2147_: f32;
    var phi_2158_: f32;
    var phi_2159_: f32;
    var phi_2160_: f32;
    var phi_2161_: f32;
    var phi_2164_: f32;
    var phi_2162_: f32;
    var phi_2178_: f32;
    var phi_2208_: vec3<f32>;

    let _e71 = gl_FragCoord_1;
    let _e75 = bitcast<vec2<u32>>(vec2<i32>(floor(_e71.xy)));
    let _e77 = j.q6_;
    let _e106 = bitcast<i32>((((((_e75.y >> bitcast<u32>(5u)) * (((_e77 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e75.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e75.x & 28u) << bitcast<u32>(5u)) + ((_e75.y & 28u) << bitcast<u32>(2i)))) + (((_e75.y & 3u) << bitcast<u32>(2i)) + (_e75.x & 3u))));
    let _e107 = g1_1;
    let _e109 = C2_1;
    let _e110 = X1_1;
    let _e112 = (Gh && (u32(_e107) != 0u));
    if (_e110.w >= 0f) {
        phi_2098_ = _e110;
    } else {
        if (_e110.z > 0f) {
            phi_2084_ = _e110.x;
        } else {
            phi_2084_ = length(_e110.xy);
        }
        let _e122 = phi_2084_;
        let _e123 = clamp(_e122, 0f, 1f);
        let _e124 = abs(_e110.z);
        if (_e124 > 1f) {
            phi_2085_ = ((0.9980469f * _e123) + 0.0009765625f);
        } else {
            phi_2085_ = ((0.001953125f * _e123) + _e124);
        }
        let _e131 = phi_2085_;
        let _e133 = textureSampleLevel(DD, P9_, vec2<f32>(_e131, -(_e110.w)), 0f);
        phi_2099_ = _e133;
        if !(_e112) {
            let _e137 = (_e133.xyz * _e133.w);
            let _e143 = vec4<f32>(_e137.x, _e133.y, _e133.z, _e133.w);
            let _e149 = vec4<f32>(_e143.x, _e137.y, _e143.z, _e143.w);
            phi_2099_ = vec4<f32>(_e149.x, _e149.y, _e137.z, _e149.w);
        }
        let _e157 = phi_2099_;
        phi_2098_ = _e157;
    }
    let _e159 = phi_2098_;
    phi_1244_ = Mh;
    if Mh {
        phi_1244_ = (_e109.z > 0f);
    }
    let _e163 = phi_1244_;
    phi_2101_ = _e159;
    if _e163 {
        let _e167 = textureSampleLevel(GC, Y5_, _e109.xy, (_e109.z - 1f));
        phi_2095_ = _e167;
        if _e112 {
            if (_e167.w != 0f) {
                phi_2086_ = (1f / _e167.w);
            } else {
                phi_2086_ = 0f;
            }
            let _e173 = phi_2086_;
            let _e174 = (_e167.xyz * _e173);
            phi_2095_ = vec4<f32>(_e174.x, _e174.y, _e174.z, _e167.w);
        }
        let _e180 = phi_2095_;
        phi_2101_ = (_e159 * _e180);
    }
    let _e183 = phi_2101_;
    let _e184 = M_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e187 = (_e184.y >= 0f);
            local = _e187;
            if _e187 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1382_ = Hh;
                        if Hh {
                            phi_1382_ = (_e184.x < -1.5f);
                        }
                        let _e255 = phi_1382_;
                        if _e255 {
                            let _e261 = textureSampleLevel(XC, ga, vec2<f32>((3f + _e184.x), 0f), 0f);
                            let _e266 = textureSampleLevel(XC, ga, vec2<f32>((1f - _e184.y), 0f), 0f);
                            phi_2125_ = ((1f - _e261.x) - _e266.x);
                            break;
                        } else {
                            phi_2125_ = min(_e184.x, _e184.y);
                            break;
                        }
                    }
                }
                let _e270 = phi_2125_;
                phi_2126_ = _e270;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1445_ = Hh;
                        if Hh {
                            phi_1445_ = (_e184.y < -1.5f);
                        }
                        let _e191 = phi_1445_;
                        if _e191 {
                            let _e195 = max(_e184.w, 0f);
                            if (_e184.z >= 0f) {
                                let _e198 = textureSampleLevel(XC, ga, vec2<f32>(_e195, 0f), 0f);
                                phi_2102_ = _e198.x;
                            } else {
                                phi_2102_ = 0f;
                            }
                            let _e201 = phi_2102_;
                            phi_2123_ = _e201;
                            if (abs(_e184.z) < 1000f) {
                                let _e207 = (-2f - _e184.y);
                                let _e209 = ((_e207 - _e195) * 0.5984134f);
                                let _e212 = (vec4(_e195) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e209));
                                let _e218 = ((_e212 * -(_e184.z)) + vec4(((_e207 * _e184.z) + (abs(_e184.x) - 0.25f))));
                                let _e221 = textureSampleLevel(XC, ga, vec2<f32>(_e218.x, 0f), 0f);
                                let _e224 = textureSampleLevel(XC, ga, vec2<f32>(_e218.y, 0f), 0f);
                                let _e227 = textureSampleLevel(XC, ga, vec2<f32>(_e218.z, 0f), 0f);
                                let _e230 = textureSampleLevel(XC, ga, vec2<f32>(_e218.w, 0f), 0f);
                                let _e236 = (_e212 * 5.0959306f);
                                phi_2123_ = (_e201 + (dot(vec4<f32>(_e221.x, _e224.x, _e227.x, _e230.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e236) * (_e236 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e209));
                            }
                            let _e245 = phi_2123_;
                            phi_2124_ = (_e245 * sign(_e184.x));
                            break;
                        } else {
                            phi_2124_ = _e184.x;
                            break;
                        }
                    }
                }
                let _e250 = phi_2124_;
                phi_2126_ = _e250;
                break;
            }
        }
    }
    let _e272 = phi_2126_;
    let _e273 = r4_1;
    let _e276 = i3_1[1u];
    let _e278 = i3_1[0u];
    let _e279 = vec2<u32>(floor(_e273));
    let _e306 = (_e278 + (((((_e279.y >> bitcast<u32>(5u)) * (_e276 << bitcast<u32>(5u))) + ((_e279.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e279.x & 28u) << bitcast<u32>(5u)) + ((_e279.y & 28u) << bitcast<u32>(2i)))) + (((_e279.y & 3u) << bitcast<u32>(2i)) + (_e279.x & 3u))));
    phi_2128_ = 1f;
    if Fh {
        let _e307 = N0_1;
        let _e310 = min(_e307.xy, _e307.zw);
        phi_2128_ = min(min(_e310.x, _e310.y), 1f);
    }
    let _e316 = phi_2128_;
    phi_1016_ = Eh;
    if Eh {
        let _e318 = Y1_1[0u];
        phi_1016_ = (_e318 != 0f);
    }
    let _e321 = phi_1016_;
    phi_2129_ = _e316;
    if _e321 {
        let _e324 = i0_.g2_[_e106];
        phi_2129_ = min(unpack4x8unorm(_e324).x, _e316);
    }
    let _e329 = phi_2129_;
    let _e331 = clamp(_e272, 0f, max(_e329, 0f));
    let _e333 = local;
    if _e333 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e183.w, _e331) >= 1f) {
                    phi_2161_ = 1f;
                    break;
                }
                let _e424 = j.f2_;
                let _e426 = atomicMax((&R0_.g2_[_e306]), (_e424 | u32(((abs(_e331) * 1024f) + 0.5f))));
                if (_e426 < _e424) {
                    phi_2160_ = _e331;
                } else {
                    let _e430 = (f32((_e426 & 524287u)) * 0.0009765625f);
                    phi_2160_ = ((max(_e430, _e331) - _e430) / max((1f - (_e430 * _e183.w)), 0.000062f));
                }
                let _e438 = phi_2160_;
                phi_2161_ = _e438;
                break;
            }
        }
        let _e440 = phi_2161_;
        phi_2164_ = _e440;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e339 = u32(((abs(_e331) * 1024f) + 0.5f));
                let _e342 = atomicLoad((&R0_.g2_[_e306]));
                let _e344 = (min(_e183.w, _e331) >= 1f);
                phi_1802_ = _e344;
                if _e344 {
                    let _e346 = j.f2_;
                    let _e347 = (_e342 < _e346);
                    phi_1800_ = _e347;
                    if !(_e347) {
                        phi_1800_ = (_e342 >= (_e346 | 262144u));
                    }
                    let _e352 = phi_1800_;
                    phi_1802_ = _e352;
                }
                let _e354 = phi_1802_;
                if _e354 {
                    phi_2159_ = 1f;
                    break;
                }
                let _e356 = j.f2_;
                phi_2153_ = 0f;
                phi_2150_ = _e339;
                phi_2147_ = _e331;
                if (_e342 < _e356) {
                    let _e359 = (_e356 | (262144u + _e339));
                    let _e360 = atomicMax((&R0_.g2_[_e306]), _e359);
                    if (_e360 <= _e356) {
                        phi_2156_ = _e331;
                        phi_2151_ = _e339;
                        phi_2148_ = 0f;
                    } else {
                        phi_2157_ = 0f;
                        phi_2152_ = _e339;
                        phi_2149_ = _e331;
                        if (_e360 < _e359) {
                            let _e364 = ((_e360 & 524287u) - 262144u);
                            let _e366 = (f32(_e364) * 0.0009765625f);
                            phi_2157_ = ((_e331 - _e366) / max((1f - (_e366 * _e183.w)), 0.000062f));
                            phi_2152_ = _e364;
                            phi_2149_ = _e366;
                        }
                        let _e373 = phi_2157_;
                        let _e375 = phi_2152_;
                        let _e377 = phi_2149_;
                        phi_2156_ = _e373;
                        phi_2151_ = _e375;
                        phi_2148_ = _e377;
                    }
                    let _e379 = phi_2156_;
                    let _e381 = phi_2151_;
                    let _e383 = phi_2148_;
                    phi_2153_ = _e379;
                    phi_2150_ = _e381;
                    phi_2147_ = _e383;
                }
                let _e385 = phi_2153_;
                let _e387 = phi_2150_;
                let _e389 = phi_2147_;
                phi_2158_ = _e385;
                if (_e389 > 0f) {
                    let _e391 = atomicAdd((&R0_.g2_[_e306]), _e387);
                    let _e396 = (f32(bitcast<i32>(((_e391 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e398 = clamp(_e396, 0f, 1f);
                    phi_2158_ = (_e385 + ((1f - (_e385 * _e183.w)) * ((clamp((_e396 + _e389), 0f, 1f) - _e398) / max((1f - (_e398 * _e183.w)), 0.000062f))));
                }
                let _e410 = phi_2158_;
                phi_2159_ = _e410;
                break;
            }
        }
        let _e412 = phi_2159_;
        phi_2164_ = _e412;
    }
    let _e442 = phi_2164_;
    phi_2178_ = f32();
    if Lh {
        let _e444 = j.F3_;
        let _e446 = j.G3_;
        if Lh {
            phi_2162_ = ((fract((52.982918f * fract(((0.06711056f * _e71.x) + (0.00583715f * _e71.y))))) * _e444) + _e446);
        } else {
            phi_2162_ = 0f;
        }
        let _e458 = phi_2162_;
        phi_2178_ = _e458;
    }
    let _e460 = phi_2178_;
    let _e461 = (_e183 * _e442);
    let _e462 = _e461.xyz;
    if (Lh && (_e461.w != 0f)) {
        phi_2208_ = (vec3(_e460) + _e462);
    } else {
        phi_2208_ = _e462;
    }
    let _e469 = phi_2208_;
    let _e475 = vec4<f32>(_e469.x, _e461.y, _e461.z, _e461.w);
    let _e481 = vec4<f32>(_e475.x, _e469.y, _e475.z, _e475.w);
    i0_.g2_[_e106] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    E1_ = vec4<f32>(_e481.x, _e481.y, _e469.z, _e481.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @location(2) M: vec4<f32>, @location(8) r4_: vec2<f32>, @location(7) @interpolate(flat, either) i3_: vec2<u32>, @location(5) N0_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @location(3) @interpolate(flat, either) D0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    M_1 = M;
    r4_1 = r4_;
    i3_1 = i3_;
    N0_1 = N0_;
    Y1_1 = Y1_;
    D0_1 = D0_;
    main_1();
    let _e21 = E1_;
    return _e21;
}
