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
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> M_1: vec4<f32>;
var<private> r4_1: vec2<f32>;
var<private> i3_1: vec2<u32>;
var<private> N0_1: vec4<f32>;
var<private> Y1_1: vec2<f32>;
@group(2) @binding(1)
var i0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> E1_: vec4<f32>;
var<private> D0_1: f32;

fn main_1() {
    var phi_1980_: f32;
    var phi_1981_: f32;
    var phi_1995_: vec4<f32>;
    var phi_1994_: vec4<f32>;
    var phi_1166_: bool;
    var phi_1982_: f32;
    var phi_1991_: vec4<f32>;
    var phi_1997_: vec4<f32>;
    var phi_1367_: bool;
    var phi_1998_: f32;
    var phi_2019_: f32;
    var phi_2020_: f32;
    var phi_1304_: bool;
    var phi_2021_: f32;
    var phi_2022_: f32;
    var phi_2024_: f32;
    var phi_978_: bool;
    var phi_2025_: f32;
    var local: bool;
    var phi_1722_: bool;
    var phi_1724_: bool;
    var phi_2053_: f32;
    var phi_2048_: u32;
    var phi_2045_: f32;
    var phi_2052_: f32;
    var phi_2047_: u32;
    var phi_2044_: f32;
    var phi_2049_: f32;
    var phi_2046_: u32;
    var phi_2043_: f32;
    var phi_2054_: f32;
    var phi_2055_: f32;
    var phi_2056_: f32;
    var phi_2057_: f32;
    var phi_2060_: f32;
    var phi_2058_: f32;
    var phi_2074_: f32;
    var phi_2104_: vec3<f32>;

    let _e67 = g1_1;
    let _e69 = C2_1;
    let _e70 = X1_1;
    let _e72 = (Gh && (u32(_e67) != 0u));
    if (_e70.w >= 0f) {
        phi_1994_ = _e70;
    } else {
        if (_e70.z > 0f) {
            phi_1980_ = _e70.x;
        } else {
            phi_1980_ = length(_e70.xy);
        }
        let _e82 = phi_1980_;
        let _e83 = clamp(_e82, 0f, 1f);
        let _e84 = abs(_e70.z);
        if (_e84 > 1f) {
            phi_1981_ = ((0.9980469f * _e83) + 0.0009765625f);
        } else {
            phi_1981_ = ((0.001953125f * _e83) + _e84);
        }
        let _e91 = phi_1981_;
        let _e93 = textureSampleLevel(DD, P9_, vec2<f32>(_e91, -(_e70.w)), 0f);
        phi_1995_ = _e93;
        if !(_e72) {
            let _e97 = (_e93.xyz * _e93.w);
            let _e103 = vec4<f32>(_e97.x, _e93.y, _e93.z, _e93.w);
            let _e109 = vec4<f32>(_e103.x, _e97.y, _e103.z, _e103.w);
            phi_1995_ = vec4<f32>(_e109.x, _e109.y, _e97.z, _e109.w);
        }
        let _e117 = phi_1995_;
        phi_1994_ = _e117;
    }
    let _e119 = phi_1994_;
    phi_1166_ = Mh;
    if Mh {
        phi_1166_ = (_e69.z > 0f);
    }
    let _e123 = phi_1166_;
    phi_1997_ = _e119;
    if _e123 {
        let _e127 = textureSampleLevel(GC, Y5_, _e69.xy, (_e69.z - 1f));
        phi_1991_ = _e127;
        if _e72 {
            if (_e127.w != 0f) {
                phi_1982_ = (1f / _e127.w);
            } else {
                phi_1982_ = 0f;
            }
            let _e133 = phi_1982_;
            let _e134 = (_e127.xyz * _e133);
            phi_1991_ = vec4<f32>(_e134.x, _e134.y, _e134.z, _e127.w);
        }
        let _e140 = phi_1991_;
        phi_1997_ = (_e119 * _e140);
    }
    let _e143 = phi_1997_;
    let _e144 = M_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e147 = (_e144.y >= 0f);
            local = _e147;
            if _e147 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1304_ = Hh;
                        if Hh {
                            phi_1304_ = (_e144.x < -1.5f);
                        }
                        let _e215 = phi_1304_;
                        if _e215 {
                            let _e221 = textureSampleLevel(XC, ga, vec2<f32>((3f + _e144.x), 0f), 0f);
                            let _e226 = textureSampleLevel(XC, ga, vec2<f32>((1f - _e144.y), 0f), 0f);
                            phi_2021_ = ((1f - _e221.x) - _e226.x);
                            break;
                        } else {
                            phi_2021_ = min(_e144.x, _e144.y);
                            break;
                        }
                    }
                }
                let _e230 = phi_2021_;
                phi_2022_ = _e230;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1367_ = Hh;
                        if Hh {
                            phi_1367_ = (_e144.y < -1.5f);
                        }
                        let _e151 = phi_1367_;
                        if _e151 {
                            let _e155 = max(_e144.w, 0f);
                            if (_e144.z >= 0f) {
                                let _e158 = textureSampleLevel(XC, ga, vec2<f32>(_e155, 0f), 0f);
                                phi_1998_ = _e158.x;
                            } else {
                                phi_1998_ = 0f;
                            }
                            let _e161 = phi_1998_;
                            phi_2019_ = _e161;
                            if (abs(_e144.z) < 1000f) {
                                let _e167 = (-2f - _e144.y);
                                let _e169 = ((_e167 - _e155) * 0.5984134f);
                                let _e172 = (vec4(_e155) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e169));
                                let _e178 = ((_e172 * -(_e144.z)) + vec4(((_e167 * _e144.z) + (abs(_e144.x) - 0.25f))));
                                let _e181 = textureSampleLevel(XC, ga, vec2<f32>(_e178.x, 0f), 0f);
                                let _e184 = textureSampleLevel(XC, ga, vec2<f32>(_e178.y, 0f), 0f);
                                let _e187 = textureSampleLevel(XC, ga, vec2<f32>(_e178.z, 0f), 0f);
                                let _e190 = textureSampleLevel(XC, ga, vec2<f32>(_e178.w, 0f), 0f);
                                let _e196 = (_e172 * 5.0959306f);
                                phi_2019_ = (_e161 + (dot(vec4<f32>(_e181.x, _e184.x, _e187.x, _e190.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e196) * (_e196 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e169));
                            }
                            let _e205 = phi_2019_;
                            phi_2020_ = (_e205 * sign(_e144.x));
                            break;
                        } else {
                            phi_2020_ = _e144.x;
                            break;
                        }
                    }
                }
                let _e210 = phi_2020_;
                phi_2022_ = _e210;
                break;
            }
        }
    }
    let _e232 = phi_2022_;
    let _e233 = r4_1;
    let _e236 = i3_1[1u];
    let _e238 = i3_1[0u];
    let _e239 = vec2<u32>(floor(_e233));
    let _e266 = (_e238 + (((((_e239.y >> bitcast<u32>(5u)) * (_e236 << bitcast<u32>(5u))) + ((_e239.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e239.x & 28u) << bitcast<u32>(5u)) + ((_e239.y & 28u) << bitcast<u32>(2i)))) + (((_e239.y & 3u) << bitcast<u32>(2i)) + (_e239.x & 3u))));
    phi_2024_ = 1f;
    if Fh {
        let _e267 = N0_1;
        let _e270 = min(_e267.xy, _e267.zw);
        phi_2024_ = min(min(_e270.x, _e270.y), 1f);
    }
    let _e276 = phi_2024_;
    phi_978_ = Eh;
    if Eh {
        let _e278 = Y1_1[0u];
        phi_978_ = (_e278 != 0f);
    }
    let _e281 = phi_978_;
    phi_2025_ = _e276;
    if _e281 {
        let _e282 = gl_FragCoord_1;
        let _e286 = textureLoad(i0_, vec2<i32>(floor(_e282.xy)), 0i);
        phi_2025_ = min(_e286.x, _e276);
    }
    let _e290 = phi_2025_;
    let _e292 = clamp(_e232, 0f, max(_e290, 0f));
    let _e294 = local;
    if _e294 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e143.w, _e292) >= 1f) {
                    phi_2057_ = 1f;
                    break;
                }
                let _e385 = j.f2_;
                let _e387 = atomicMax((&R0_.g2_[_e266]), (_e385 | u32(((abs(_e292) * 1024f) + 0.5f))));
                if (_e387 < _e385) {
                    phi_2056_ = _e292;
                } else {
                    let _e391 = (f32((_e387 & 524287u)) * 0.0009765625f);
                    phi_2056_ = ((max(_e391, _e292) - _e391) / max((1f - (_e391 * _e143.w)), 0.000062f));
                }
                let _e399 = phi_2056_;
                phi_2057_ = _e399;
                break;
            }
        }
        let _e401 = phi_2057_;
        phi_2060_ = _e401;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e300 = u32(((abs(_e292) * 1024f) + 0.5f));
                let _e303 = atomicLoad((&R0_.g2_[_e266]));
                let _e305 = (min(_e143.w, _e292) >= 1f);
                phi_1724_ = _e305;
                if _e305 {
                    let _e307 = j.f2_;
                    let _e308 = (_e303 < _e307);
                    phi_1722_ = _e308;
                    if !(_e308) {
                        phi_1722_ = (_e303 >= (_e307 | 262144u));
                    }
                    let _e313 = phi_1722_;
                    phi_1724_ = _e313;
                }
                let _e315 = phi_1724_;
                if _e315 {
                    phi_2055_ = 1f;
                    break;
                }
                let _e317 = j.f2_;
                phi_2049_ = 0f;
                phi_2046_ = _e300;
                phi_2043_ = _e292;
                if (_e303 < _e317) {
                    let _e320 = (_e317 | (262144u + _e300));
                    let _e321 = atomicMax((&R0_.g2_[_e266]), _e320);
                    if (_e321 <= _e317) {
                        phi_2052_ = _e292;
                        phi_2047_ = _e300;
                        phi_2044_ = 0f;
                    } else {
                        phi_2053_ = 0f;
                        phi_2048_ = _e300;
                        phi_2045_ = _e292;
                        if (_e321 < _e320) {
                            let _e325 = ((_e321 & 524287u) - 262144u);
                            let _e327 = (f32(_e325) * 0.0009765625f);
                            phi_2053_ = ((_e292 - _e327) / max((1f - (_e327 * _e143.w)), 0.000062f));
                            phi_2048_ = _e325;
                            phi_2045_ = _e327;
                        }
                        let _e334 = phi_2053_;
                        let _e336 = phi_2048_;
                        let _e338 = phi_2045_;
                        phi_2052_ = _e334;
                        phi_2047_ = _e336;
                        phi_2044_ = _e338;
                    }
                    let _e340 = phi_2052_;
                    let _e342 = phi_2047_;
                    let _e344 = phi_2044_;
                    phi_2049_ = _e340;
                    phi_2046_ = _e342;
                    phi_2043_ = _e344;
                }
                let _e346 = phi_2049_;
                let _e348 = phi_2046_;
                let _e350 = phi_2043_;
                phi_2054_ = _e346;
                if (_e350 > 0f) {
                    let _e352 = atomicAdd((&R0_.g2_[_e266]), _e348);
                    let _e357 = (f32(bitcast<i32>(((_e352 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e359 = clamp(_e357, 0f, 1f);
                    phi_2054_ = (_e346 + ((1f - (_e346 * _e143.w)) * ((clamp((_e357 + _e350), 0f, 1f) - _e359) / max((1f - (_e359 * _e143.w)), 0.000062f))));
                }
                let _e371 = phi_2054_;
                phi_2055_ = _e371;
                break;
            }
        }
        let _e373 = phi_2055_;
        phi_2060_ = _e373;
    }
    let _e403 = phi_2060_;
    phi_2074_ = f32();
    if Lh {
        let _e404 = gl_FragCoord_1;
        let _e406 = j.F3_;
        let _e408 = j.G3_;
        if Lh {
            phi_2058_ = ((fract((52.982918f * fract(((0.06711056f * _e404.x) + (0.00583715f * _e404.y))))) * _e406) + _e408);
        } else {
            phi_2058_ = 0f;
        }
        let _e420 = phi_2058_;
        phi_2074_ = _e420;
    }
    let _e422 = phi_2074_;
    let _e423 = (_e143 * _e403);
    let _e424 = _e423.xyz;
    if (Lh && (_e423.w != 0f)) {
        phi_2104_ = (vec3(_e422) + _e424);
    } else {
        phi_2104_ = _e424;
    }
    let _e431 = phi_2104_;
    let _e437 = vec4<f32>(_e431.x, _e423.y, _e423.z, _e423.w);
    let _e443 = vec4<f32>(_e437.x, _e431.y, _e437.z, _e437.w);
    E1_ = vec4<f32>(_e443.x, _e443.y, _e431.z, _e443.w);
    return;
}

@fragment
fn main(@location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @location(2) M: vec4<f32>, @location(8) r4_: vec2<f32>, @location(7) @interpolate(flat, either) i3_: vec2<u32>, @location(5) N0_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) D0_: f32) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    M_1 = M;
    r4_1 = r4_;
    i3_1 = i3_;
    N0_1 = N0_;
    Y1_1 = Y1_;
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    main_1();
    let _e21 = E1_;
    return _e21;
}
