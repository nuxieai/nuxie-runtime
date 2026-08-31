struct ge {
    c2_: array<u32>,
}

struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Gg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Cg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Hg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    zg: u32,
}

struct ge_1 {
    c2_: array<atomic<u32>>,
}

@id(7) override gh: bool = true;
@id(3) override ch: bool = true;
@id(1) override ah: bool = true;
@id(0) override Zg: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var LD: texture_2d<f32>;
@group(3) @binding(8)
var Mb: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
@group(0) @binding(6)
var<storage, read_write> P0_: ge_1;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> X0_1: vec4<f32>;
var<private> O_1: vec4<f32>;
var<private> l4_1: vec2<f32>;
var<private> d3_1: vec2<u32>;
var<private> L0_1: vec4<f32>;
var<private> U1_1: vec2<f32>;
@group(2) @binding(1)
var h0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C1_: vec4<f32>;
var<private> B0_1: f32;
var<private> e2_1: f32;

fn main_1() {
    var phi_1963_: f32;
    var phi_1967_: f32;
    var phi_1968_: f32;
    var phi_1970_: vec4<f32>;
    var phi_1969_: vec4<f32>;
    var phi_1281_: bool;
    var phi_1971_: f32;
    var phi_1986_: f32;
    var phi_1987_: f32;
    var phi_1463_: bool;
    var phi_1988_: f32;
    var phi_1989_: f32;
    var phi_1991_: f32;
    var phi_964_: bool;
    var phi_1992_: f32;
    var local: bool;
    var phi_1620_: bool;
    var phi_1622_: bool;
    var phi_2022_: f32;
    var phi_2017_: u32;
    var phi_2014_: f32;
    var phi_2021_: f32;
    var phi_2016_: u32;
    var phi_2013_: f32;
    var phi_2018_: f32;
    var phi_2015_: u32;
    var phi_2012_: f32;
    var phi_2026_: f32;
    var phi_2028_: f32;
    var phi_2036_: f32;
    var phi_2038_: f32;
    var phi_2043_: vec4<f32>;
    var phi_2041_: f32;
    var phi_2045_: f32;
    var phi_2072_: vec3<f32>;

    let _e65 = X0_1;
    if (_e65.w >= 0f) {
        phi_1969_ = vec4<f32>(_e65.x, _e65.y, _e65.z, _e65.w);
    } else {
        if (_e65.w > -1f) {
            if (_e65.z > 0f) {
                phi_1967_ = _e65.x;
            } else {
                phi_1967_ = length(_e65.xy);
            }
            let _e91 = phi_1967_;
            let _e92 = clamp(_e91, 0f, 1f);
            let _e93 = abs(_e65.z);
            if (_e93 > 1f) {
                phi_1968_ = ((0.9980469f * _e92) + 0.0009765625f);
            } else {
                phi_1968_ = ((0.001953125f * _e92) + _e93);
            }
            let _e100 = phi_1968_;
            let _e103 = textureSampleLevel(LD, Mb, vec2<f32>(_e100, -(_e65.w)), 0f);
            phi_1970_ = vec4<f32>(_e103.x, _e103.y, _e103.z, _e103.w);
        } else {
            let _e71 = textureSampleLevel(IC, S5_, _e65.xy, (-2f - _e65.w));
            if (_e71.w != 0f) {
                phi_1963_ = (1f / _e71.w);
            } else {
                phi_1963_ = 0f;
            }
            let _e78 = phi_1963_;
            let _e79 = (_e71.xyz * _e78);
            phi_1970_ = vec4<f32>(_e79.x, _e79.y, _e79.z, (_e71.w * _e65.z));
        }
        let _e111 = phi_1970_;
        phi_1969_ = _e111;
    }
    let _e119 = phi_1969_;
    let _e120 = O_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e123 = (_e120.y >= 0f);
            local = _e123;
            if _e123 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1463_ = ch;
                        if ch {
                            phi_1463_ = (_e120.x < -1.5f);
                        }
                        let _e191 = phi_1463_;
                        if _e191 {
                            let _e197 = textureSampleLevel(XC, aa, vec2<f32>((3f + _e120.x), 0f), 0f);
                            let _e202 = textureSampleLevel(XC, aa, vec2<f32>((1f - _e120.y), 0f), 0f);
                            phi_1988_ = ((1f - _e197.x) - _e202.x);
                            break;
                        } else {
                            phi_1988_ = min(_e120.x, _e120.y);
                            break;
                        }
                    }
                }
                let _e206 = phi_1988_;
                phi_1989_ = _e206;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1281_ = ch;
                        if ch {
                            phi_1281_ = (_e120.y < -1.5f);
                        }
                        let _e127 = phi_1281_;
                        if _e127 {
                            let _e131 = max(_e120.w, 0f);
                            if (_e120.z >= 0f) {
                                let _e134 = textureSampleLevel(XC, aa, vec2<f32>(_e131, 0f), 0f);
                                phi_1971_ = _e134.x;
                            } else {
                                phi_1971_ = 0f;
                            }
                            let _e137 = phi_1971_;
                            phi_1986_ = _e137;
                            if (abs(_e120.z) < 1000f) {
                                let _e143 = (-2f - _e120.y);
                                let _e145 = ((_e143 - _e131) * 0.5984134f);
                                let _e148 = (vec4(_e131) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e145));
                                let _e154 = ((_e148 * -(_e120.z)) + vec4(((_e143 * _e120.z) + (abs(_e120.x) - 0.25f))));
                                let _e157 = textureSampleLevel(XC, aa, vec2<f32>(_e154.x, 0f), 0f);
                                let _e160 = textureSampleLevel(XC, aa, vec2<f32>(_e154.y, 0f), 0f);
                                let _e163 = textureSampleLevel(XC, aa, vec2<f32>(_e154.z, 0f), 0f);
                                let _e166 = textureSampleLevel(XC, aa, vec2<f32>(_e154.w, 0f), 0f);
                                let _e172 = (_e148 * 5.0959306f);
                                phi_1986_ = (_e137 + (dot(vec4<f32>(_e157.x, _e160.x, _e163.x, _e166.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e172) * (_e172 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e145));
                            }
                            let _e181 = phi_1986_;
                            phi_1987_ = (_e181 * sign(_e120.x));
                            break;
                        } else {
                            phi_1987_ = _e120.x;
                            break;
                        }
                    }
                }
                let _e186 = phi_1987_;
                phi_1989_ = _e186;
                break;
            }
        }
    }
    let _e208 = phi_1989_;
    let _e209 = l4_1;
    let _e212 = d3_1[1u];
    let _e214 = d3_1[0u];
    let _e215 = vec2<u32>(floor(_e209));
    let _e242 = (_e214 + (((((_e215.y >> bitcast<u32>(5u)) * (_e212 << bitcast<u32>(5u))) + ((_e215.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e215.x & 28u) << bitcast<u32>(5u)) + ((_e215.y & 28u) << bitcast<u32>(2i)))) + (((_e215.y & 3u) << bitcast<u32>(2i)) + (_e215.x & 3u))));
    phi_1991_ = 1f;
    if ah {
        let _e243 = L0_1;
        let _e246 = min(_e243.xy, _e243.zw);
        phi_1991_ = min(min(_e246.x, _e246.y), 1f);
    }
    let _e252 = phi_1991_;
    phi_964_ = Zg;
    if Zg {
        let _e254 = U1_1[0u];
        phi_964_ = (_e254 != 0f);
    }
    let _e257 = phi_964_;
    phi_1992_ = _e252;
    if _e257 {
        let _e258 = gl_FragCoord_1;
        let _e262 = textureLoad(h0_, vec2<i32>(floor(_e258.xy)), 0i);
        phi_1992_ = min(_e262.x, _e252);
    }
    let _e266 = phi_1992_;
    let _e268 = clamp(_e208, 0f, max(_e266, 0f));
    let _e270 = local;
    if _e270 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e119.w, _e268) >= 1f) {
                    phi_2038_ = _e119.w;
                    break;
                }
                let _e367 = n.a2_;
                let _e369 = atomicMax((&P0_.c2_[_e242]), (_e367 | u32(((abs(_e268) * 1024f) + 0.5f))));
                if (_e369 < _e367) {
                    phi_2036_ = _e268;
                } else {
                    let _e373 = (f32((_e369 & 524287u)) * 0.0009765625f);
                    phi_2036_ = ((max(_e373, _e268) - _e373) / max((1f - (_e373 * _e119.w)), 0.000062f));
                }
                let _e381 = phi_2036_;
                phi_2038_ = (_e119.w * _e381);
                break;
            }
        }
        let _e384 = phi_2038_;
        phi_2043_ = vec4<f32>(_e119.x, _e119.y, _e119.z, _e384);
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e276 = u32(((abs(_e268) * 1024f) + 0.5f));
                let _e279 = atomicLoad((&P0_.c2_[_e242]));
                let _e281 = (min(_e119.w, _e268) >= 1f);
                phi_1622_ = _e281;
                if _e281 {
                    let _e283 = n.a2_;
                    let _e284 = (_e279 < _e283);
                    phi_1620_ = _e284;
                    if !(_e284) {
                        phi_1620_ = (_e279 >= (_e283 | 262144u));
                    }
                    let _e289 = phi_1620_;
                    phi_1622_ = _e289;
                }
                let _e291 = phi_1622_;
                if _e291 {
                    phi_2028_ = _e119.w;
                    break;
                }
                let _e293 = n.a2_;
                phi_2018_ = 0f;
                phi_2015_ = _e276;
                phi_2012_ = _e268;
                if (_e279 < _e293) {
                    let _e296 = (_e293 | (262144u + _e276));
                    let _e297 = atomicMax((&P0_.c2_[_e242]), _e296);
                    if (_e297 <= _e293) {
                        phi_2021_ = _e268;
                        phi_2016_ = _e276;
                        phi_2013_ = 0f;
                    } else {
                        phi_2022_ = 0f;
                        phi_2017_ = _e276;
                        phi_2014_ = _e268;
                        if (_e297 < _e296) {
                            let _e301 = ((_e297 & 524287u) - 262144u);
                            let _e303 = (f32(_e301) * 0.0009765625f);
                            phi_2022_ = ((_e268 - _e303) / max((1f - (_e303 * _e119.w)), 0.000062f));
                            phi_2017_ = _e301;
                            phi_2014_ = _e303;
                        }
                        let _e310 = phi_2022_;
                        let _e312 = phi_2017_;
                        let _e314 = phi_2014_;
                        phi_2021_ = _e310;
                        phi_2016_ = _e312;
                        phi_2013_ = _e314;
                    }
                    let _e316 = phi_2021_;
                    let _e318 = phi_2016_;
                    let _e320 = phi_2013_;
                    phi_2018_ = _e316;
                    phi_2015_ = _e318;
                    phi_2012_ = _e320;
                }
                let _e322 = phi_2018_;
                let _e324 = phi_2015_;
                let _e326 = phi_2012_;
                phi_2026_ = _e322;
                if (_e326 > 0f) {
                    let _e328 = atomicAdd((&P0_.c2_[_e242]), _e324);
                    let _e333 = (f32(bitcast<i32>(((_e328 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e335 = clamp(_e333, 0f, 1f);
                    phi_2026_ = (_e322 + ((1f - (_e322 * _e119.w)) * ((clamp((_e333 + _e326), 0f, 1f) - _e335) / max((1f - (_e335 * _e119.w)), 0.000062f))));
                }
                let _e347 = phi_2026_;
                phi_2028_ = (_e119.w * _e347);
                break;
            }
        }
        let _e350 = phi_2028_;
        phi_2043_ = vec4<f32>(_e119.x, _e119.y, _e119.z, _e350);
    }
    let _e391 = phi_2043_;
    phi_2045_ = f32();
    if gh {
        let _e392 = gl_FragCoord_1;
        let _e394 = n.z3_;
        let _e396 = n.A3_;
        if gh {
            phi_2041_ = ((fract((52.982918f * fract(((0.06711056f * _e392.x) + (0.00583715f * _e392.y))))) * _e394) + _e396);
        } else {
            phi_2041_ = 0f;
        }
        let _e408 = phi_2041_;
        phi_2045_ = _e408;
    }
    let _e410 = phi_2045_;
    let _e413 = (_e391.xyz * _e391.w);
    let _e419 = vec4<f32>(_e413.x, _e391.y, _e391.z, _e391.w);
    let _e425 = vec4<f32>(_e419.x, _e413.y, _e419.z, _e419.w);
    let _e431 = vec4<f32>(_e425.x, _e425.y, _e413.z, _e425.w);
    let _e432 = _e431.xyz;
    if (gh && (_e391.w != 0f)) {
        phi_2072_ = (vec3(_e410) + _e432);
    } else {
        phi_2072_ = _e432;
    }
    let _e438 = phi_2072_;
    let _e444 = vec4<f32>(_e438.x, _e431.y, _e431.z, _e431.w);
    let _e450 = vec4<f32>(_e444.x, _e438.y, _e444.z, _e444.w);
    C1_ = vec4<f32>(_e450.x, _e450.y, _e438.z, _e450.w);
    return;
}

@fragment
fn main(@location(0) X0_: vec4<f32>, @location(2) O: vec4<f32>, @location(8) l4_: vec2<f32>, @location(7) @interpolate(flat, either) d3_: vec2<u32>, @location(5) L0_: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    X0_1 = X0_;
    O_1 = O;
    l4_1 = l4_;
    d3_1 = d3_;
    L0_1 = L0_;
    U1_1 = U1_;
    gl_FragCoord_1 = gl_FragCoord;
    B0_1 = B0_;
    e2_1 = e2_;
    main_1();
    let _e19 = C1_;
    return _e19;
}
