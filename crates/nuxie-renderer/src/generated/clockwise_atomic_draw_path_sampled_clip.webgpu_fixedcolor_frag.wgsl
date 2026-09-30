struct ze {
    e2_: array<u32>,
}

struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct ze_1 {
    e2_: array<atomic<u32>>,
}

@id(7) override Jh: bool = true;
@id(8) override Kh: bool = true;
@id(3) override Fh: bool = true;
@id(1) override Dh: bool = true;
@id(0) override Ch: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var X5_: sampler;
@group(0) @binding(6)
var<storage, read_write> Q0_: ze_1;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> V1_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
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
    var phi_1985_: f32;
    var phi_1986_: f32;
    var phi_1997_: vec4<f32>;
    var phi_1164_: bool;
    var phi_1987_: f32;
    var phi_1998_: vec4<f32>;
    var phi_1301_: bool;
    var phi_1999_: f32;
    var phi_2016_: f32;
    var phi_2017_: f32;
    var phi_1483_: bool;
    var phi_2018_: f32;
    var phi_2019_: f32;
    var phi_2021_: f32;
    var phi_976_: bool;
    var phi_2022_: f32;
    var local: bool;
    var phi_1640_: bool;
    var phi_1642_: bool;
    var phi_2052_: f32;
    var phi_2047_: u32;
    var phi_2044_: f32;
    var phi_2051_: f32;
    var phi_2046_: u32;
    var phi_2043_: f32;
    var phi_2048_: f32;
    var phi_2045_: u32;
    var phi_2042_: f32;
    var phi_2056_: f32;
    var phi_2058_: f32;
    var phi_2066_: f32;
    var phi_2068_: f32;
    var phi_2073_: vec4<f32>;
    var phi_2071_: f32;
    var phi_2075_: f32;
    var phi_2103_: vec3<f32>;

    let _e66 = V1_1;
    let _e67 = C2_1;
    if (_e66.w >= 0f) {
        phi_1997_ = vec4<f32>(_e66.x, _e66.y, _e66.z, _e66.w);
    } else {
        if (_e66.z > 0f) {
            phi_1985_ = _e66.x;
        } else {
            phi_1985_ = length(_e66.xy);
        }
        let _e77 = phi_1985_;
        let _e78 = clamp(_e77, 0f, 1f);
        let _e79 = abs(_e66.z);
        if (_e79 > 1f) {
            phi_1986_ = ((0.9980469f * _e78) + 0.0009765625f);
        } else {
            phi_1986_ = ((0.001953125f * _e78) + _e79);
        }
        let _e86 = phi_1986_;
        let _e88 = textureSampleLevel(ED, N9_, vec2<f32>(_e86, -(_e66.w)), 0f);
        phi_1997_ = vec4<f32>(_e88.x, _e88.y, _e88.z, _e88.w);
    }
    let _e102 = phi_1997_;
    phi_1164_ = Kh;
    if Kh {
        phi_1164_ = (_e67.z > 0f);
    }
    let _e106 = phi_1164_;
    phi_1998_ = _e102;
    if _e106 {
        let _e110 = textureSampleLevel(HC, X5_, _e67.xy, (_e67.z - 1f));
        if (_e110.w != 0f) {
            phi_1987_ = (1f / _e110.w);
        } else {
            phi_1987_ = 0f;
        }
        let _e116 = phi_1987_;
        let _e117 = (_e110.xyz * _e116);
        phi_1998_ = (_e102 * vec4<f32>(_e117.x, _e117.y, _e117.z, _e110.w));
    }
    let _e124 = phi_1998_;
    let _e125 = M_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e128 = (_e125.y >= 0f);
            local = _e128;
            if _e128 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1483_ = Fh;
                        if Fh {
                            phi_1483_ = (_e125.x < -1.5f);
                        }
                        let _e196 = phi_1483_;
                        if _e196 {
                            let _e202 = textureSampleLevel(YC, ea, vec2<f32>((3f + _e125.x), 0f), 0f);
                            let _e207 = textureSampleLevel(YC, ea, vec2<f32>((1f - _e125.y), 0f), 0f);
                            phi_2018_ = ((1f - _e202.x) - _e207.x);
                            break;
                        } else {
                            phi_2018_ = min(_e125.x, _e125.y);
                            break;
                        }
                    }
                }
                let _e211 = phi_2018_;
                phi_2019_ = _e211;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1301_ = Fh;
                        if Fh {
                            phi_1301_ = (_e125.y < -1.5f);
                        }
                        let _e132 = phi_1301_;
                        if _e132 {
                            let _e136 = max(_e125.w, 0f);
                            if (_e125.z >= 0f) {
                                let _e139 = textureSampleLevel(YC, ea, vec2<f32>(_e136, 0f), 0f);
                                phi_1999_ = _e139.x;
                            } else {
                                phi_1999_ = 0f;
                            }
                            let _e142 = phi_1999_;
                            phi_2016_ = _e142;
                            if (abs(_e125.z) < 1000f) {
                                let _e148 = (-2f - _e125.y);
                                let _e150 = ((_e148 - _e136) * 0.5984134f);
                                let _e153 = (vec4(_e136) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e150));
                                let _e159 = ((_e153 * -(_e125.z)) + vec4(((_e148 * _e125.z) + (abs(_e125.x) - 0.25f))));
                                let _e162 = textureSampleLevel(YC, ea, vec2<f32>(_e159.x, 0f), 0f);
                                let _e165 = textureSampleLevel(YC, ea, vec2<f32>(_e159.y, 0f), 0f);
                                let _e168 = textureSampleLevel(YC, ea, vec2<f32>(_e159.z, 0f), 0f);
                                let _e171 = textureSampleLevel(YC, ea, vec2<f32>(_e159.w, 0f), 0f);
                                let _e177 = (_e153 * 5.0959306f);
                                phi_2016_ = (_e142 + (dot(vec4<f32>(_e162.x, _e165.x, _e168.x, _e171.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e177) * (_e177 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e150));
                            }
                            let _e186 = phi_2016_;
                            phi_2017_ = (_e186 * sign(_e125.x));
                            break;
                        } else {
                            phi_2017_ = _e125.x;
                            break;
                        }
                    }
                }
                let _e191 = phi_2017_;
                phi_2019_ = _e191;
                break;
            }
        }
    }
    let _e213 = phi_2019_;
    let _e214 = p4_1;
    let _e217 = g3_1[1u];
    let _e219 = g3_1[0u];
    let _e220 = vec2<u32>(floor(_e214));
    let _e247 = (_e219 + (((((_e220.y >> bitcast<u32>(5u)) * (_e217 << bitcast<u32>(5u))) + ((_e220.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e220.x & 28u) << bitcast<u32>(5u)) + ((_e220.y & 28u) << bitcast<u32>(2i)))) + (((_e220.y & 3u) << bitcast<u32>(2i)) + (_e220.x & 3u))));
    phi_2021_ = 1f;
    if Dh {
        let _e248 = M0_1;
        let _e251 = min(_e248.xy, _e248.zw);
        phi_2021_ = min(min(_e251.x, _e251.y), 1f);
    }
    let _e257 = phi_2021_;
    phi_976_ = Ch;
    if Ch {
        let _e259 = W1_1[0u];
        phi_976_ = (_e259 != 0f);
    }
    let _e262 = phi_976_;
    phi_2022_ = _e257;
    if _e262 {
        let _e263 = gl_FragCoord_1;
        let _e267 = textureLoad(h0_, vec2<i32>(floor(_e263.xy)), 0i);
        phi_2022_ = min(_e267.x, _e257);
    }
    let _e271 = phi_2022_;
    let _e273 = clamp(_e213, 0f, max(_e271, 0f));
    let _e275 = local;
    if _e275 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e124.w, _e273) >= 1f) {
                    phi_2068_ = _e124.w;
                    break;
                }
                let _e372 = l.d2_;
                let _e374 = atomicMax((&Q0_.e2_[_e247]), (_e372 | u32(((abs(_e273) * 1024f) + 0.5f))));
                if (_e374 < _e372) {
                    phi_2066_ = _e273;
                } else {
                    let _e378 = (f32((_e374 & 524287u)) * 0.0009765625f);
                    phi_2066_ = ((max(_e378, _e273) - _e378) / max((1f - (_e378 * _e124.w)), 0.000062f));
                }
                let _e386 = phi_2066_;
                phi_2068_ = (_e124.w * _e386);
                break;
            }
        }
        let _e389 = phi_2068_;
        phi_2073_ = vec4<f32>(_e124.x, _e124.y, _e124.z, _e389);
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e281 = u32(((abs(_e273) * 1024f) + 0.5f));
                let _e284 = atomicLoad((&Q0_.e2_[_e247]));
                let _e286 = (min(_e124.w, _e273) >= 1f);
                phi_1642_ = _e286;
                if _e286 {
                    let _e288 = l.d2_;
                    let _e289 = (_e284 < _e288);
                    phi_1640_ = _e289;
                    if !(_e289) {
                        phi_1640_ = (_e284 >= (_e288 | 262144u));
                    }
                    let _e294 = phi_1640_;
                    phi_1642_ = _e294;
                }
                let _e296 = phi_1642_;
                if _e296 {
                    phi_2058_ = _e124.w;
                    break;
                }
                let _e298 = l.d2_;
                phi_2048_ = 0f;
                phi_2045_ = _e281;
                phi_2042_ = _e273;
                if (_e284 < _e298) {
                    let _e301 = (_e298 | (262144u + _e281));
                    let _e302 = atomicMax((&Q0_.e2_[_e247]), _e301);
                    if (_e302 <= _e298) {
                        phi_2051_ = _e273;
                        phi_2046_ = _e281;
                        phi_2043_ = 0f;
                    } else {
                        phi_2052_ = 0f;
                        phi_2047_ = _e281;
                        phi_2044_ = _e273;
                        if (_e302 < _e301) {
                            let _e306 = ((_e302 & 524287u) - 262144u);
                            let _e308 = (f32(_e306) * 0.0009765625f);
                            phi_2052_ = ((_e273 - _e308) / max((1f - (_e308 * _e124.w)), 0.000062f));
                            phi_2047_ = _e306;
                            phi_2044_ = _e308;
                        }
                        let _e315 = phi_2052_;
                        let _e317 = phi_2047_;
                        let _e319 = phi_2044_;
                        phi_2051_ = _e315;
                        phi_2046_ = _e317;
                        phi_2043_ = _e319;
                    }
                    let _e321 = phi_2051_;
                    let _e323 = phi_2046_;
                    let _e325 = phi_2043_;
                    phi_2048_ = _e321;
                    phi_2045_ = _e323;
                    phi_2042_ = _e325;
                }
                let _e327 = phi_2048_;
                let _e329 = phi_2045_;
                let _e331 = phi_2042_;
                phi_2056_ = _e327;
                if (_e331 > 0f) {
                    let _e333 = atomicAdd((&Q0_.e2_[_e247]), _e329);
                    let _e338 = (f32(bitcast<i32>(((_e333 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e340 = clamp(_e338, 0f, 1f);
                    phi_2056_ = (_e327 + ((1f - (_e327 * _e124.w)) * ((clamp((_e338 + _e331), 0f, 1f) - _e340) / max((1f - (_e340 * _e124.w)), 0.000062f))));
                }
                let _e352 = phi_2056_;
                phi_2058_ = (_e124.w * _e352);
                break;
            }
        }
        let _e355 = phi_2058_;
        phi_2073_ = vec4<f32>(_e124.x, _e124.y, _e124.z, _e355);
    }
    let _e396 = phi_2073_;
    phi_2075_ = f32();
    if Jh {
        let _e397 = gl_FragCoord_1;
        let _e399 = l.C3_;
        let _e401 = l.D3_;
        if Jh {
            phi_2071_ = ((fract((52.982918f * fract(((0.06711056f * _e397.x) + (0.00583715f * _e397.y))))) * _e399) + _e401);
        } else {
            phi_2071_ = 0f;
        }
        let _e413 = phi_2071_;
        phi_2075_ = _e413;
    }
    let _e415 = phi_2075_;
    let _e418 = (_e396.xyz * _e396.w);
    let _e424 = vec4<f32>(_e418.x, _e396.y, _e396.z, _e396.w);
    let _e430 = vec4<f32>(_e424.x, _e418.y, _e424.z, _e424.w);
    let _e436 = vec4<f32>(_e430.x, _e430.y, _e418.z, _e430.w);
    let _e437 = _e436.xyz;
    if (Jh && (_e396.w != 0f)) {
        phi_2103_ = (vec3(_e415) + _e437);
    } else {
        phi_2103_ = _e437;
    }
    let _e443 = phi_2103_;
    let _e449 = vec4<f32>(_e443.x, _e436.y, _e436.z, _e436.w);
    let _e455 = vec4<f32>(_e449.x, _e443.y, _e449.z, _e449.w);
    C1_ = vec4<f32>(_e455.x, _e455.y, _e443.z, _e455.w);
    return;
}

@fragment
fn main(@location(0) V1_: vec4<f32>, @location(9) C2_: vec3<f32>, @location(2) M: vec4<f32>, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    V1_1 = V1_;
    C2_1 = C2_;
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
