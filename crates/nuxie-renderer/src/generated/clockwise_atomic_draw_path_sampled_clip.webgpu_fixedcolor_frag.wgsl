struct je {
    c2_: array<u32>,
}

struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct je_1 {
    c2_: array<atomic<u32>>,
}

@id(7) override lh: bool = true;
@id(8) override mh: bool = true;
@id(3) override hh: bool = true;
@id(1) override fh: bool = true;
@id(0) override eh: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Ob: sampler;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var U5_: sampler;
@group(0) @binding(6)
var<storage, read_write> P0_: je_1;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> f1_1: vec4<f32>;
var<private> A2_1: vec3<f32>;
var<private> O_1: vec4<f32>;
var<private> n4_1: vec2<f32>;
var<private> f3_1: vec2<u32>;
var<private> M0_1: vec4<f32>;
var<private> U1_1: vec2<f32>;
@group(2) @binding(1)
var h0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C1_: vec4<f32>;
var<private> B0_1: f32;
var<private> e2_1: f32;

fn main_1() {
    var phi_1958_: f32;
    var phi_1959_: f32;
    var phi_1970_: vec4<f32>;
    var phi_1183_: bool;
    var phi_1960_: f32;
    var phi_1971_: vec4<f32>;
    var phi_1277_: bool;
    var phi_1972_: f32;
    var phi_1989_: f32;
    var phi_1990_: f32;
    var phi_1459_: bool;
    var phi_1991_: f32;
    var phi_1992_: f32;
    var phi_1994_: f32;
    var phi_962_: bool;
    var phi_1995_: f32;
    var local: bool;
    var phi_1616_: bool;
    var phi_1618_: bool;
    var phi_2025_: f32;
    var phi_2020_: u32;
    var phi_2017_: f32;
    var phi_2024_: f32;
    var phi_2019_: u32;
    var phi_2016_: f32;
    var phi_2021_: f32;
    var phi_2018_: u32;
    var phi_2015_: f32;
    var phi_2029_: f32;
    var phi_2031_: f32;
    var phi_2039_: f32;
    var phi_2041_: f32;
    var phi_2046_: vec4<f32>;
    var phi_2044_: f32;
    var phi_2048_: f32;
    var phi_2076_: vec3<f32>;

    let _e66 = f1_1;
    let _e67 = A2_1;
    if (_e66.w >= 0f) {
        phi_1970_ = vec4<f32>(_e66.x, _e66.y, _e66.z, _e66.w);
    } else {
        if (_e66.z > 0f) {
            phi_1958_ = _e66.x;
        } else {
            phi_1958_ = length(_e66.xy);
        }
        let _e76 = phi_1958_;
        let _e77 = clamp(_e76, 0f, 1f);
        let _e78 = abs(_e66.z);
        if (_e78 > 1f) {
            phi_1959_ = ((0.9980469f * _e77) + 0.0009765625f);
        } else {
            phi_1959_ = ((0.001953125f * _e77) + _e78);
        }
        let _e85 = phi_1959_;
        let _e88 = textureSampleLevel(MD, Ob, vec2<f32>(_e85, -(_e66.w)), 0f);
        phi_1970_ = vec4<f32>(_e88.x, _e88.y, _e88.z, _e88.w);
    }
    let _e102 = phi_1970_;
    phi_1183_ = mh;
    if mh {
        phi_1183_ = (_e67.z > 0f);
    }
    let _e106 = phi_1183_;
    phi_1971_ = _e102;
    if _e106 {
        let _e110 = textureSampleLevel(JC, U5_, _e67.xy, (_e67.z - 1f));
        if (_e110.w != 0f) {
            phi_1960_ = (1f / _e110.w);
        } else {
            phi_1960_ = 0f;
        }
        let _e116 = phi_1960_;
        let _e117 = (_e110.xyz * _e116);
        phi_1971_ = (_e102 * vec4<f32>(_e117.x, _e117.y, _e117.z, _e110.w));
    }
    let _e124 = phi_1971_;
    let _e125 = O_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e128 = (_e125.y >= 0f);
            local = _e128;
            if _e128 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1459_ = hh;
                        if hh {
                            phi_1459_ = (_e125.x < -1.5f);
                        }
                        let _e196 = phi_1459_;
                        if _e196 {
                            let _e202 = textureSampleLevel(YC, aa, vec2<f32>((3f + _e125.x), 0f), 0f);
                            let _e207 = textureSampleLevel(YC, aa, vec2<f32>((1f - _e125.y), 0f), 0f);
                            phi_1991_ = ((1f - _e202.x) - _e207.x);
                            break;
                        } else {
                            phi_1991_ = min(_e125.x, _e125.y);
                            break;
                        }
                    }
                }
                let _e211 = phi_1991_;
                phi_1992_ = _e211;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1277_ = hh;
                        if hh {
                            phi_1277_ = (_e125.y < -1.5f);
                        }
                        let _e132 = phi_1277_;
                        if _e132 {
                            let _e136 = max(_e125.w, 0f);
                            if (_e125.z >= 0f) {
                                let _e139 = textureSampleLevel(YC, aa, vec2<f32>(_e136, 0f), 0f);
                                phi_1972_ = _e139.x;
                            } else {
                                phi_1972_ = 0f;
                            }
                            let _e142 = phi_1972_;
                            phi_1989_ = _e142;
                            if (abs(_e125.z) < 1000f) {
                                let _e148 = (-2f - _e125.y);
                                let _e150 = ((_e148 - _e136) * 0.5984134f);
                                let _e153 = (vec4(_e136) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e150));
                                let _e159 = ((_e153 * -(_e125.z)) + vec4(((_e148 * _e125.z) + (abs(_e125.x) - 0.25f))));
                                let _e162 = textureSampleLevel(YC, aa, vec2<f32>(_e159.x, 0f), 0f);
                                let _e165 = textureSampleLevel(YC, aa, vec2<f32>(_e159.y, 0f), 0f);
                                let _e168 = textureSampleLevel(YC, aa, vec2<f32>(_e159.z, 0f), 0f);
                                let _e171 = textureSampleLevel(YC, aa, vec2<f32>(_e159.w, 0f), 0f);
                                let _e177 = (_e153 * 5.0959306f);
                                phi_1989_ = (_e142 + (dot(vec4<f32>(_e162.x, _e165.x, _e168.x, _e171.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e177) * (_e177 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e150));
                            }
                            let _e186 = phi_1989_;
                            phi_1990_ = (_e186 * sign(_e125.x));
                            break;
                        } else {
                            phi_1990_ = _e125.x;
                            break;
                        }
                    }
                }
                let _e191 = phi_1990_;
                phi_1992_ = _e191;
                break;
            }
        }
    }
    let _e213 = phi_1992_;
    let _e214 = n4_1;
    let _e217 = f3_1[1u];
    let _e219 = f3_1[0u];
    let _e220 = vec2<u32>(floor(_e214));
    let _e247 = (_e219 + (((((_e220.y >> bitcast<u32>(5u)) * (_e217 << bitcast<u32>(5u))) + ((_e220.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e220.x & 28u) << bitcast<u32>(5u)) + ((_e220.y & 28u) << bitcast<u32>(2i)))) + (((_e220.y & 3u) << bitcast<u32>(2i)) + (_e220.x & 3u))));
    phi_1994_ = 1f;
    if fh {
        let _e248 = M0_1;
        let _e251 = min(_e248.xy, _e248.zw);
        phi_1994_ = min(min(_e251.x, _e251.y), 1f);
    }
    let _e257 = phi_1994_;
    phi_962_ = eh;
    if eh {
        let _e259 = U1_1[0u];
        phi_962_ = (_e259 != 0f);
    }
    let _e262 = phi_962_;
    phi_1995_ = _e257;
    if _e262 {
        let _e263 = gl_FragCoord_1;
        let _e267 = textureLoad(h0_, vec2<i32>(floor(_e263.xy)), 0i);
        phi_1995_ = min(_e267.x, _e257);
    }
    let _e271 = phi_1995_;
    let _e273 = clamp(_e213, 0f, max(_e271, 0f));
    let _e275 = local;
    if _e275 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e124.w, _e273) >= 1f) {
                    phi_2041_ = _e124.w;
                    break;
                }
                let _e372 = m.a2_;
                let _e374 = atomicMax((&P0_.c2_[_e247]), (_e372 | u32(((abs(_e273) * 1024f) + 0.5f))));
                if (_e374 < _e372) {
                    phi_2039_ = _e273;
                } else {
                    let _e378 = (f32((_e374 & 524287u)) * 0.0009765625f);
                    phi_2039_ = ((max(_e378, _e273) - _e378) / max((1f - (_e378 * _e124.w)), 0.000062f));
                }
                let _e386 = phi_2039_;
                phi_2041_ = (_e124.w * _e386);
                break;
            }
        }
        let _e389 = phi_2041_;
        phi_2046_ = vec4<f32>(_e124.x, _e124.y, _e124.z, _e389);
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e281 = u32(((abs(_e273) * 1024f) + 0.5f));
                let _e284 = atomicLoad((&P0_.c2_[_e247]));
                let _e286 = (min(_e124.w, _e273) >= 1f);
                phi_1618_ = _e286;
                if _e286 {
                    let _e288 = m.a2_;
                    let _e289 = (_e284 < _e288);
                    phi_1616_ = _e289;
                    if !(_e289) {
                        phi_1616_ = (_e284 >= (_e288 | 262144u));
                    }
                    let _e294 = phi_1616_;
                    phi_1618_ = _e294;
                }
                let _e296 = phi_1618_;
                if _e296 {
                    phi_2031_ = _e124.w;
                    break;
                }
                let _e298 = m.a2_;
                phi_2021_ = 0f;
                phi_2018_ = _e281;
                phi_2015_ = _e273;
                if (_e284 < _e298) {
                    let _e301 = (_e298 | (262144u + _e281));
                    let _e302 = atomicMax((&P0_.c2_[_e247]), _e301);
                    if (_e302 <= _e298) {
                        phi_2024_ = _e273;
                        phi_2019_ = _e281;
                        phi_2016_ = 0f;
                    } else {
                        phi_2025_ = 0f;
                        phi_2020_ = _e281;
                        phi_2017_ = _e273;
                        if (_e302 < _e301) {
                            let _e306 = ((_e302 & 524287u) - 262144u);
                            let _e308 = (f32(_e306) * 0.0009765625f);
                            phi_2025_ = ((_e273 - _e308) / max((1f - (_e308 * _e124.w)), 0.000062f));
                            phi_2020_ = _e306;
                            phi_2017_ = _e308;
                        }
                        let _e315 = phi_2025_;
                        let _e317 = phi_2020_;
                        let _e319 = phi_2017_;
                        phi_2024_ = _e315;
                        phi_2019_ = _e317;
                        phi_2016_ = _e319;
                    }
                    let _e321 = phi_2024_;
                    let _e323 = phi_2019_;
                    let _e325 = phi_2016_;
                    phi_2021_ = _e321;
                    phi_2018_ = _e323;
                    phi_2015_ = _e325;
                }
                let _e327 = phi_2021_;
                let _e329 = phi_2018_;
                let _e331 = phi_2015_;
                phi_2029_ = _e327;
                if (_e331 > 0f) {
                    let _e333 = atomicAdd((&P0_.c2_[_e247]), _e329);
                    let _e338 = (f32(bitcast<i32>(((_e333 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e340 = clamp(_e338, 0f, 1f);
                    phi_2029_ = (_e327 + ((1f - (_e327 * _e124.w)) * ((clamp((_e338 + _e331), 0f, 1f) - _e340) / max((1f - (_e340 * _e124.w)), 0.000062f))));
                }
                let _e352 = phi_2029_;
                phi_2031_ = (_e124.w * _e352);
                break;
            }
        }
        let _e355 = phi_2031_;
        phi_2046_ = vec4<f32>(_e124.x, _e124.y, _e124.z, _e355);
    }
    let _e396 = phi_2046_;
    phi_2048_ = f32();
    if lh {
        let _e397 = gl_FragCoord_1;
        let _e399 = m.B3_;
        let _e401 = m.C3_;
        if lh {
            phi_2044_ = ((fract((52.982918f * fract(((0.06711056f * _e397.x) + (0.00583715f * _e397.y))))) * _e399) + _e401);
        } else {
            phi_2044_ = 0f;
        }
        let _e413 = phi_2044_;
        phi_2048_ = _e413;
    }
    let _e415 = phi_2048_;
    let _e418 = (_e396.xyz * _e396.w);
    let _e424 = vec4<f32>(_e418.x, _e396.y, _e396.z, _e396.w);
    let _e430 = vec4<f32>(_e424.x, _e418.y, _e424.z, _e424.w);
    let _e436 = vec4<f32>(_e430.x, _e430.y, _e418.z, _e430.w);
    let _e437 = _e436.xyz;
    if (lh && (_e396.w != 0f)) {
        phi_2076_ = (vec3(_e415) + _e437);
    } else {
        phi_2076_ = _e437;
    }
    let _e443 = phi_2076_;
    let _e449 = vec4<f32>(_e443.x, _e436.y, _e436.z, _e436.w);
    let _e455 = vec4<f32>(_e449.x, _e443.y, _e449.z, _e449.w);
    C1_ = vec4<f32>(_e455.x, _e455.y, _e443.z, _e455.w);
    return;
}

@fragment
fn main(@location(0) f1_: vec4<f32>, @location(9) A2_: vec3<f32>, @location(2) O: vec4<f32>, @location(8) n4_: vec2<f32>, @location(7) @interpolate(flat, either) f3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    f1_1 = f1_;
    A2_1 = A2_;
    O_1 = O;
    n4_1 = n4_;
    f3_1 = f3_;
    M0_1 = M0_;
    U1_1 = U1_;
    gl_FragCoord_1 = gl_FragCoord;
    B0_1 = B0_;
    e2_1 = e2_;
    main_1();
    let _e21 = C1_;
    return _e21;
}
