struct UB {
    Rc: f32,
    Ud: f32,
    cg: f32,
    dg: f32,
    B6_: u32,
    Y9_: u32,
    Of: u32,
    Pf: u32,
    k8_: vec4<i32>,
    Mh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Qh: f32,
    U4_: u32,
    c3_: f32,
    Wd: f32,
    If: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Jh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct Ue {
    k2_: array<u32>,
}

struct Ue_1 {
    k2_: array<atomic<u32>>,
}

@id(7) override si: bool = true;
@id(2) override ni: bool = true;
@id(8) override ti: bool = true;
@id(3) override oi: bool = true;
@id(1) override mi: bool = true;
@id(0) override li: bool = true;

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(11)
var DC: texture_2d<f32>;
@group(1) @binding(13)
var v5_: sampler;
@group(0) @binding(6)
var<storage, read_write> W0_: Ue_1;
var<private> r1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> G4_1: vec2<f32>;
var<private> q3_1: vec2<u32>;
var<private> S0_1: vec4<f32>;
var<private> l1_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> K1_: vec4<f32>;
var<private> F0_1: f32;

fn main_1() {
    var phi_2101_: f32;
    var phi_2102_: f32;
    var phi_2118_: vec4<f32>;
    var phi_2117_: vec4<f32>;
    var phi_1253_: bool;
    var phi_1268_: bool;
    var phi_2103_: f32;
    var phi_2113_: vec4<f32>;
    var phi_2120_: vec4<f32>;
    var phi_2121_: vec4<f32>;
    var phi_1480_: bool;
    var phi_2122_: f32;
    var phi_2147_: f32;
    var phi_2148_: f32;
    var phi_1417_: bool;
    var phi_2149_: f32;
    var phi_2150_: f32;
    var phi_2152_: f32;
    var phi_1036_: bool;
    var phi_2153_: f32;
    var local: bool;
    var phi_1835_: bool;
    var phi_1837_: bool;
    var phi_2181_: f32;
    var phi_2176_: u32;
    var phi_2173_: f32;
    var phi_2180_: f32;
    var phi_2175_: u32;
    var phi_2172_: f32;
    var phi_2177_: f32;
    var phi_2174_: u32;
    var phi_2171_: f32;
    var phi_2182_: f32;
    var phi_2183_: f32;
    var phi_2184_: f32;
    var phi_2185_: f32;
    var phi_2188_: f32;
    var phi_2186_: f32;
    var phi_2202_: f32;
    var phi_2234_: vec3<f32>;

    let _e70 = Q0_1;
    let _e72 = r1_1;
    let _e73 = a1_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e76 = (ni && (u32(_e70) != 0u));
            if (_e73.w >= 0f) {
                phi_2117_ = _e73;
            } else {
                let _e79 = -(_e73.w);
                let _e84 = j.xc;
                let _e87 = j.yc;
                if (_e73.z > 0f) {
                    phi_2101_ = _e73.x;
                } else {
                    phi_2101_ = length(_e73.xy);
                }
                let _e95 = phi_2101_;
                let _e96 = clamp(_e95, 0f, 1f);
                let _e97 = abs(_e73.z);
                if (_e97 > 1f) {
                    phi_2102_ = ((0.9980469f * _e96) + 0.0009765625f);
                } else {
                    phi_2102_ = ((0.001953125f * _e96) + _e97);
                }
                let _e104 = phi_2102_;
                let _e106 = textureSampleLevel(FD, ia, vec2<f32>(_e104, ((floor(_e79) * _e84) + _e87)), 0f);
                phi_2118_ = _e106;
                if !(_e76) {
                    let _e110 = (_e106.xyz * _e106.w);
                    phi_2118_ = vec4<f32>(_e110.x, _e110.y, _e110.z, (_e106.w * (fract(_e79) * 1.0039216f)));
                }
                let _e117 = phi_2118_;
                phi_2117_ = _e117;
            }
            let _e119 = phi_2117_;
            phi_1253_ = ti;
            if ti {
                phi_1253_ = (_e72.z < 0f);
            }
            let _e123 = phi_1253_;
            if _e123 {
                let _e125 = textureSampleLevel(DC, v5_, _e72.xy, 0f);
                phi_2121_ = _e125;
                break;
            }
            phi_1268_ = ti;
            if ti {
                phi_1268_ = (_e72.z > 0f);
            }
            let _e129 = phi_1268_;
            phi_2120_ = _e119;
            if _e129 {
                let _e133 = textureSampleLevel(DC, v5_, _e72.xy, (_e72.z - 1f));
                phi_2113_ = _e133;
                if _e76 {
                    if (_e133.w != 0f) {
                        phi_2103_ = (1f / _e133.w);
                    } else {
                        phi_2103_ = 0f;
                    }
                    let _e139 = phi_2103_;
                    let _e140 = (_e133.xyz * _e139);
                    phi_2113_ = vec4<f32>(_e140.x, _e140.y, _e140.z, _e133.w);
                }
                let _e146 = phi_2113_;
                phi_2120_ = (_e119 * _e146);
            }
            let _e149 = phi_2120_;
            phi_2121_ = _e149;
            break;
        }
    }
    let _e151 = phi_2121_;
    let _e152 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e155 = (_e152.y >= 0f);
            local = _e155;
            if _e155 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1417_ = oi;
                        if oi {
                            phi_1417_ = (_e152.x < -1.5f);
                        }
                        let _e223 = phi_1417_;
                        if _e223 {
                            let _e229 = textureSampleLevel(ZC, xa, vec2<f32>((3f + _e152.x), 0f), 0f);
                            let _e234 = textureSampleLevel(ZC, xa, vec2<f32>((1f - _e152.y), 0f), 0f);
                            phi_2149_ = ((1f - _e229.x) - _e234.x);
                            break;
                        } else {
                            phi_2149_ = min(_e152.x, _e152.y);
                            break;
                        }
                    }
                }
                let _e238 = phi_2149_;
                phi_2150_ = _e238;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1480_ = oi;
                        if oi {
                            phi_1480_ = (_e152.y < -1.5f);
                        }
                        let _e159 = phi_1480_;
                        if _e159 {
                            let _e163 = max(_e152.w, 0f);
                            if (_e152.z >= 0f) {
                                let _e166 = textureSampleLevel(ZC, xa, vec2<f32>(_e163, 0f), 0f);
                                phi_2122_ = _e166.x;
                            } else {
                                phi_2122_ = 0f;
                            }
                            let _e169 = phi_2122_;
                            phi_2147_ = _e169;
                            if (abs(_e152.z) < 1000f) {
                                let _e175 = (-2f - _e152.y);
                                let _e177 = ((_e175 - _e163) * 0.5984134f);
                                let _e180 = (vec4(_e163) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e177));
                                let _e186 = ((_e180 * -(_e152.z)) + vec4(((_e175 * _e152.z) + (abs(_e152.x) - 0.25f))));
                                let _e189 = textureSampleLevel(ZC, xa, vec2<f32>(_e186.x, 0f), 0f);
                                let _e192 = textureSampleLevel(ZC, xa, vec2<f32>(_e186.y, 0f), 0f);
                                let _e195 = textureSampleLevel(ZC, xa, vec2<f32>(_e186.z, 0f), 0f);
                                let _e198 = textureSampleLevel(ZC, xa, vec2<f32>(_e186.w, 0f), 0f);
                                let _e204 = (_e180 * 5.0959306f);
                                phi_2147_ = (_e169 + (dot(vec4<f32>(_e189.x, _e192.x, _e195.x, _e198.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e204) * (_e204 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e177));
                            }
                            let _e213 = phi_2147_;
                            phi_2148_ = (_e213 * sign(_e152.x));
                            break;
                        } else {
                            phi_2148_ = _e152.x;
                            break;
                        }
                    }
                }
                let _e218 = phi_2148_;
                phi_2150_ = _e218;
                break;
            }
        }
    }
    let _e240 = phi_2150_;
    let _e241 = G4_1;
    let _e244 = q3_1[1u];
    let _e246 = q3_1[0u];
    let _e247 = vec2<u32>(floor(_e241));
    let _e274 = (_e246 + (((((_e247.y >> bitcast<u32>(5u)) * (_e244 << bitcast<u32>(5u))) + ((_e247.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e247.x & 28u) << bitcast<u32>(5u)) + ((_e247.y & 28u) << bitcast<u32>(2i)))) + (((_e247.y & 3u) << bitcast<u32>(2i)) + (_e247.x & 3u))));
    phi_2152_ = 1f;
    if mi {
        let _e275 = S0_1;
        let _e278 = min(_e275.xy, _e275.zw);
        phi_2152_ = min(min(_e278.x, _e278.y), 1f);
    }
    let _e284 = phi_2152_;
    phi_1036_ = li;
    if li {
        let _e286 = l1_1[0u];
        phi_1036_ = (_e286 != 0f);
    }
    let _e289 = phi_1036_;
    phi_2153_ = _e284;
    if _e289 {
        let _e290 = gl_FragCoord_1;
        let _e294 = textureLoad(m0_, vec2<i32>(floor(_e290.xy)), 0i);
        phi_2153_ = min(_e294.x, _e284);
    }
    let _e298 = phi_2153_;
    let _e300 = clamp(_e240, 0f, max(_e298, 0f));
    let _e302 = local;
    if _e302 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e151.w, _e300) >= 1f) {
                    phi_2185_ = 1f;
                    break;
                }
                let _e393 = j.j2_;
                let _e395 = atomicMax((&W0_.k2_[_e274]), (_e393 | u32(((abs(_e300) * 1024f) + 0.5f))));
                if (_e395 < _e393) {
                    phi_2184_ = _e300;
                } else {
                    let _e399 = (f32((_e395 & 524287u)) * 0.0009765625f);
                    phi_2184_ = ((max(_e399, _e300) - _e399) / max((1f - (_e399 * _e151.w)), 0.000062f));
                }
                let _e407 = phi_2184_;
                phi_2185_ = _e407;
                break;
            }
        }
        let _e409 = phi_2185_;
        phi_2188_ = _e409;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e308 = u32(((abs(_e300) * 1024f) + 0.5f));
                let _e311 = atomicLoad((&W0_.k2_[_e274]));
                let _e313 = (min(_e151.w, _e300) >= 1f);
                phi_1837_ = _e313;
                if _e313 {
                    let _e315 = j.j2_;
                    let _e316 = (_e311 < _e315);
                    phi_1835_ = _e316;
                    if !(_e316) {
                        phi_1835_ = (_e311 >= (_e315 | 262144u));
                    }
                    let _e321 = phi_1835_;
                    phi_1837_ = _e321;
                }
                let _e323 = phi_1837_;
                if _e323 {
                    phi_2183_ = 1f;
                    break;
                }
                let _e325 = j.j2_;
                phi_2177_ = 0f;
                phi_2174_ = _e308;
                phi_2171_ = _e300;
                if (_e311 < _e325) {
                    let _e328 = (_e325 | (262144u + _e308));
                    let _e329 = atomicMax((&W0_.k2_[_e274]), _e328);
                    if (_e329 <= _e325) {
                        phi_2180_ = _e300;
                        phi_2175_ = _e308;
                        phi_2172_ = 0f;
                    } else {
                        phi_2181_ = 0f;
                        phi_2176_ = _e308;
                        phi_2173_ = _e300;
                        if (_e329 < _e328) {
                            let _e333 = ((_e329 & 524287u) - 262144u);
                            let _e335 = (f32(_e333) * 0.0009765625f);
                            phi_2181_ = ((_e300 - _e335) / max((1f - (_e335 * _e151.w)), 0.000062f));
                            phi_2176_ = _e333;
                            phi_2173_ = _e335;
                        }
                        let _e342 = phi_2181_;
                        let _e344 = phi_2176_;
                        let _e346 = phi_2173_;
                        phi_2180_ = _e342;
                        phi_2175_ = _e344;
                        phi_2172_ = _e346;
                    }
                    let _e348 = phi_2180_;
                    let _e350 = phi_2175_;
                    let _e352 = phi_2172_;
                    phi_2177_ = _e348;
                    phi_2174_ = _e350;
                    phi_2171_ = _e352;
                }
                let _e354 = phi_2177_;
                let _e356 = phi_2174_;
                let _e358 = phi_2171_;
                phi_2182_ = _e354;
                if (_e358 > 0f) {
                    let _e360 = atomicAdd((&W0_.k2_[_e274]), _e356);
                    let _e365 = (f32(bitcast<i32>(((_e360 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e367 = clamp(_e365, 0f, 1f);
                    phi_2182_ = (_e354 + ((1f - (_e354 * _e151.w)) * ((clamp((_e365 + _e358), 0f, 1f) - _e367) / max((1f - (_e367 * _e151.w)), 0.000062f))));
                }
                let _e379 = phi_2182_;
                phi_2183_ = _e379;
                break;
            }
        }
        let _e381 = phi_2183_;
        phi_2188_ = _e381;
    }
    let _e411 = phi_2188_;
    phi_2202_ = f32();
    if si {
        let _e412 = gl_FragCoord_1;
        let _e414 = j.M3_;
        let _e416 = j.N3_;
        if si {
            phi_2186_ = ((fract((52.982918f * fract(((0.06711056f * _e412.x) + (0.00583715f * _e412.y))))) * _e414) + _e416);
        } else {
            phi_2186_ = 0f;
        }
        let _e428 = phi_2186_;
        phi_2202_ = _e428;
    }
    let _e430 = phi_2202_;
    let _e431 = (_e151 * _e411);
    let _e432 = _e431.xyz;
    if (si && (_e431.w != 0f)) {
        phi_2234_ = (vec3(_e430) + _e432);
    } else {
        phi_2234_ = _e432;
    }
    let _e439 = phi_2234_;
    let _e445 = vec4<f32>(_e439.x, _e431.y, _e431.z, _e431.w);
    let _e451 = vec4<f32>(_e445.x, _e439.y, _e445.z, _e445.w);
    K1_ = vec4<f32>(_e451.x, _e451.y, _e439.z, _e451.w);
    return;
}

@fragment
fn main(@location(9) r1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) G4_: vec2<f32>, @location(7) @interpolate(flat, either) q3_: vec2<u32>, @location(5) S0_: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) F0_: f32) -> @location(0) vec4<f32> {
    r1_1 = r1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    S_1 = S;
    G4_1 = G4_;
    q3_1 = q3_;
    S0_1 = S0_;
    l1_1 = l1_;
    gl_FragCoord_1 = gl_FragCoord;
    F0_1 = F0_;
    main_1();
    let _e21 = K1_;
    return _e21;
}
