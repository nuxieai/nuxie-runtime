struct Ae {
    e2_: array<u32>,
}

struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

struct Ae_1 {
    e2_: array<atomic<u32>>,
}

@id(7) override Kh: bool = true;
@id(2) override Fh: bool = true;
@id(8) override Lh: bool = true;
@id(3) override Gh: bool = true;
@id(1) override Eh: bool = true;
@id(0) override Dh: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var fa: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var O9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;
@group(0) @binding(6)
var<storage, read_write> Q0_: Ae_1;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> V1_1: vec4<f32>;
var<private> B2_1: vec3<f32>;
var<private> M_1: vec4<f32>;
var<private> p4_1: vec2<f32>;
var<private> g3_1: vec2<u32>;
var<private> M0_1: vec4<f32>;
var<private> W1_1: vec2<f32>;
@group(2) @binding(1)
var h0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C1_: vec4<f32>;
var<private> C0_1: f32;
var<private> g2_1: f32;

fn main_1() {
    var phi_1992_: f32;
    var phi_1993_: f32;
    var phi_2009_: vec4<f32>;
    var phi_2008_: vec4<f32>;
    var phi_2007_: vec4<f32>;
    var phi_1173_: bool;
    var phi_1994_: f32;
    var phi_2004_: vec4<f32>;
    var phi_2011_: vec4<f32>;
    var phi_1373_: bool;
    var phi_2012_: f32;
    var phi_2035_: f32;
    var phi_2036_: f32;
    var phi_1310_: bool;
    var phi_2037_: f32;
    var phi_2038_: f32;
    var phi_2040_: f32;
    var phi_976_: bool;
    var phi_2041_: f32;
    var local: bool;
    var phi_1728_: bool;
    var phi_1730_: bool;
    var phi_2069_: f32;
    var phi_2064_: u32;
    var phi_2061_: f32;
    var phi_2068_: f32;
    var phi_2063_: u32;
    var phi_2060_: f32;
    var phi_2065_: f32;
    var phi_2062_: u32;
    var phi_2059_: f32;
    var phi_2070_: f32;
    var phi_2071_: f32;
    var phi_2072_: f32;
    var phi_2073_: f32;
    var phi_2076_: f32;
    var phi_2074_: f32;
    var phi_2090_: f32;
    var phi_2121_: vec3<f32>;

    let _e67 = V1_1;
    let _e68 = B2_1;
    if (_e67.w >= 0f) {
        if Fh {
            phi_2008_ = vec4<f32>(_e67.x, _e67.y, _e67.z, _e67.w);
        } else {
            phi_2008_ = (_e67 * 1f);
        }
        let _e112 = phi_2008_;
        phi_2007_ = _e112;
    } else {
        if (_e67.z > 0f) {
            phi_1992_ = _e67.x;
        } else {
            phi_1992_ = length(_e67.xy);
        }
        let _e78 = phi_1992_;
        let _e79 = clamp(_e78, 0f, 1f);
        let _e80 = abs(_e67.z);
        if (_e80 > 1f) {
            phi_1993_ = ((0.9980469f * _e79) + 0.0009765625f);
        } else {
            phi_1993_ = ((0.001953125f * _e79) + _e80);
        }
        let _e87 = phi_1993_;
        let _e89 = textureSampleLevel(ED, O9_, vec2<f32>(_e87, -(_e67.w)), 0f);
        let _e95 = vec4<f32>(_e89.x, _e89.y, _e89.z, _e89.w);
        if Fh {
            phi_2009_ = _e95;
        } else {
            let _e97 = (_e95.xyz * _e89.w);
            phi_2009_ = vec4<f32>(_e97.x, _e97.y, _e97.z, _e89.w);
        }
        let _e103 = phi_2009_;
        phi_2007_ = _e103;
    }
    let _e114 = phi_2007_;
    phi_1173_ = Lh;
    if Lh {
        phi_1173_ = (_e68.z > 0f);
    }
    let _e118 = phi_1173_;
    phi_2011_ = _e114;
    if _e118 {
        let _e122 = textureSampleLevel(HC, V5_, _e68.xy, (_e68.z - 1f));
        phi_2004_ = _e122;
        if Fh {
            if (_e122.w != 0f) {
                phi_1994_ = (1f / _e122.w);
            } else {
                phi_1994_ = 0f;
            }
            let _e128 = phi_1994_;
            let _e129 = (_e122.xyz * _e128);
            phi_2004_ = vec4<f32>(_e129.x, _e129.y, _e129.z, _e122.w);
        }
        let _e135 = phi_2004_;
        phi_2011_ = (_e114 * _e135);
    }
    let _e138 = phi_2011_;
    let _e139 = M_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e142 = (_e139.y >= 0f);
            local = _e142;
            if _e142 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1310_ = Gh;
                        if Gh {
                            phi_1310_ = (_e139.x < -1.5f);
                        }
                        let _e210 = phi_1310_;
                        if _e210 {
                            let _e216 = textureSampleLevel(YC, fa, vec2<f32>((3f + _e139.x), 0f), 0f);
                            let _e221 = textureSampleLevel(YC, fa, vec2<f32>((1f - _e139.y), 0f), 0f);
                            phi_2037_ = ((1f - _e216.x) - _e221.x);
                            break;
                        } else {
                            phi_2037_ = min(_e139.x, _e139.y);
                            break;
                        }
                    }
                }
                let _e225 = phi_2037_;
                phi_2038_ = _e225;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1373_ = Gh;
                        if Gh {
                            phi_1373_ = (_e139.y < -1.5f);
                        }
                        let _e146 = phi_1373_;
                        if _e146 {
                            let _e150 = max(_e139.w, 0f);
                            if (_e139.z >= 0f) {
                                let _e153 = textureSampleLevel(YC, fa, vec2<f32>(_e150, 0f), 0f);
                                phi_2012_ = _e153.x;
                            } else {
                                phi_2012_ = 0f;
                            }
                            let _e156 = phi_2012_;
                            phi_2035_ = _e156;
                            if (abs(_e139.z) < 1000f) {
                                let _e162 = (-2f - _e139.y);
                                let _e164 = ((_e162 - _e150) * 0.5984134f);
                                let _e167 = (vec4(_e150) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e164));
                                let _e173 = ((_e167 * -(_e139.z)) + vec4(((_e162 * _e139.z) + (abs(_e139.x) - 0.25f))));
                                let _e176 = textureSampleLevel(YC, fa, vec2<f32>(_e173.x, 0f), 0f);
                                let _e179 = textureSampleLevel(YC, fa, vec2<f32>(_e173.y, 0f), 0f);
                                let _e182 = textureSampleLevel(YC, fa, vec2<f32>(_e173.z, 0f), 0f);
                                let _e185 = textureSampleLevel(YC, fa, vec2<f32>(_e173.w, 0f), 0f);
                                let _e191 = (_e167 * 5.0959306f);
                                phi_2035_ = (_e156 + (dot(vec4<f32>(_e176.x, _e179.x, _e182.x, _e185.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e191) * (_e191 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e164));
                            }
                            let _e200 = phi_2035_;
                            phi_2036_ = (_e200 * sign(_e139.x));
                            break;
                        } else {
                            phi_2036_ = _e139.x;
                            break;
                        }
                    }
                }
                let _e205 = phi_2036_;
                phi_2038_ = _e205;
                break;
            }
        }
    }
    let _e227 = phi_2038_;
    let _e228 = p4_1;
    let _e231 = g3_1[1u];
    let _e233 = g3_1[0u];
    let _e234 = vec2<u32>(floor(_e228));
    let _e261 = (_e233 + (((((_e234.y >> bitcast<u32>(5u)) * (_e231 << bitcast<u32>(5u))) + ((_e234.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e234.x & 28u) << bitcast<u32>(5u)) + ((_e234.y & 28u) << bitcast<u32>(2i)))) + (((_e234.y & 3u) << bitcast<u32>(2i)) + (_e234.x & 3u))));
    phi_2040_ = 1f;
    if Eh {
        let _e262 = M0_1;
        let _e265 = min(_e262.xy, _e262.zw);
        phi_2040_ = min(min(_e265.x, _e265.y), 1f);
    }
    let _e271 = phi_2040_;
    phi_976_ = Dh;
    if Dh {
        let _e273 = W1_1[0u];
        phi_976_ = (_e273 != 0f);
    }
    let _e276 = phi_976_;
    phi_2041_ = _e271;
    if _e276 {
        let _e277 = gl_FragCoord_1;
        let _e281 = textureLoad(h0_, vec2<i32>(floor(_e277.xy)), 0i);
        phi_2041_ = min(_e281.x, _e271);
    }
    let _e285 = phi_2041_;
    let _e287 = clamp(_e227, 0f, max(_e285, 0f));
    let _e289 = local;
    if _e289 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e138.w, _e287) >= 1f) {
                    phi_2073_ = 1f;
                    break;
                }
                let _e380 = l.d2_;
                let _e382 = atomicMax((&Q0_.e2_[_e261]), (_e380 | u32(((abs(_e287) * 1024f) + 0.5f))));
                if (_e382 < _e380) {
                    phi_2072_ = _e287;
                } else {
                    let _e386 = (f32((_e382 & 524287u)) * 0.0009765625f);
                    phi_2072_ = ((max(_e386, _e287) - _e386) / max((1f - (_e386 * _e138.w)), 0.000062f));
                }
                let _e394 = phi_2072_;
                phi_2073_ = _e394;
                break;
            }
        }
        let _e396 = phi_2073_;
        phi_2076_ = _e396;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e295 = u32(((abs(_e287) * 1024f) + 0.5f));
                let _e298 = atomicLoad((&Q0_.e2_[_e261]));
                let _e300 = (min(_e138.w, _e287) >= 1f);
                phi_1730_ = _e300;
                if _e300 {
                    let _e302 = l.d2_;
                    let _e303 = (_e298 < _e302);
                    phi_1728_ = _e303;
                    if !(_e303) {
                        phi_1728_ = (_e298 >= (_e302 | 262144u));
                    }
                    let _e308 = phi_1728_;
                    phi_1730_ = _e308;
                }
                let _e310 = phi_1730_;
                if _e310 {
                    phi_2071_ = 1f;
                    break;
                }
                let _e312 = l.d2_;
                phi_2065_ = 0f;
                phi_2062_ = _e295;
                phi_2059_ = _e287;
                if (_e298 < _e312) {
                    let _e315 = (_e312 | (262144u + _e295));
                    let _e316 = atomicMax((&Q0_.e2_[_e261]), _e315);
                    if (_e316 <= _e312) {
                        phi_2068_ = _e287;
                        phi_2063_ = _e295;
                        phi_2060_ = 0f;
                    } else {
                        phi_2069_ = 0f;
                        phi_2064_ = _e295;
                        phi_2061_ = _e287;
                        if (_e316 < _e315) {
                            let _e320 = ((_e316 & 524287u) - 262144u);
                            let _e322 = (f32(_e320) * 0.0009765625f);
                            phi_2069_ = ((_e287 - _e322) / max((1f - (_e322 * _e138.w)), 0.000062f));
                            phi_2064_ = _e320;
                            phi_2061_ = _e322;
                        }
                        let _e329 = phi_2069_;
                        let _e331 = phi_2064_;
                        let _e333 = phi_2061_;
                        phi_2068_ = _e329;
                        phi_2063_ = _e331;
                        phi_2060_ = _e333;
                    }
                    let _e335 = phi_2068_;
                    let _e337 = phi_2063_;
                    let _e339 = phi_2060_;
                    phi_2065_ = _e335;
                    phi_2062_ = _e337;
                    phi_2059_ = _e339;
                }
                let _e341 = phi_2065_;
                let _e343 = phi_2062_;
                let _e345 = phi_2059_;
                phi_2070_ = _e341;
                if (_e345 > 0f) {
                    let _e347 = atomicAdd((&Q0_.e2_[_e261]), _e343);
                    let _e352 = (f32(bitcast<i32>(((_e347 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e354 = clamp(_e352, 0f, 1f);
                    phi_2070_ = (_e341 + ((1f - (_e341 * _e138.w)) * ((clamp((_e352 + _e345), 0f, 1f) - _e354) / max((1f - (_e354 * _e138.w)), 0.000062f))));
                }
                let _e366 = phi_2070_;
                phi_2071_ = _e366;
                break;
            }
        }
        let _e368 = phi_2071_;
        phi_2076_ = _e368;
    }
    let _e398 = phi_2076_;
    phi_2090_ = f32();
    if Kh {
        let _e399 = gl_FragCoord_1;
        let _e401 = l.C3_;
        let _e403 = l.D3_;
        if Kh {
            phi_2074_ = ((fract((52.982918f * fract(((0.06711056f * _e399.x) + (0.00583715f * _e399.y))))) * _e401) + _e403);
        } else {
            phi_2074_ = 0f;
        }
        let _e415 = phi_2074_;
        phi_2090_ = _e415;
    }
    let _e417 = phi_2090_;
    let _e418 = (_e138 * _e398);
    let _e419 = _e418.xyz;
    if (Kh && (_e418.w != 0f)) {
        phi_2121_ = (vec3(_e417) + _e419);
    } else {
        phi_2121_ = _e419;
    }
    let _e426 = phi_2121_;
    let _e432 = vec4<f32>(_e426.x, _e418.y, _e418.z, _e418.w);
    let _e438 = vec4<f32>(_e432.x, _e426.y, _e432.z, _e432.w);
    C1_ = vec4<f32>(_e438.x, _e438.y, _e426.z, _e438.w);
    return;
}

@fragment
fn main(@location(0) V1_: vec4<f32>, @location(9) B2_: vec3<f32>, @location(2) M: vec4<f32>, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    V1_1 = V1_;
    B2_1 = B2_;
    M_1 = M;
    p4_1 = p4_;
    g3_1 = g3_;
    M0_1 = M0_;
    W1_1 = W1_;
    gl_FragCoord_1 = gl_FragCoord;
    C0_1 = C0_;
    g2_1 = g2_;
    main_1();
    let _e21 = C1_;
    return _e21;
}
