struct ce {
    c2_: array<u32>,
}

struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct h0xd {
    c2_: array<u32>,
}

struct ce_1 {
    c2_: array<atomic<u32>>,
}

@id(7) override bh: bool = true;
@id(3) override Xg: bool = true;
@id(1) override Vg: bool = true;
@id(0) override Ug: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
@group(0) @binding(6)
var<storage, read_write> P0_: ce_1;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> f1_1: vec4<f32>;
var<private> O_1: vec4<f32>;
var<private> l4_1: vec2<f32>;
var<private> a3_1: vec2<u32>;
var<private> L0_1: vec4<f32>;
var<private> U1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> h0_: h0xd;
var<private> C1_: vec4<f32>;
var<private> A0_1: f32;
var<private> e2_1: f32;

fn main_1() {
    var phi_2027_: f32;
    var phi_2031_: f32;
    var phi_2032_: f32;
    var phi_2034_: vec4<f32>;
    var phi_2033_: vec4<f32>;
    var phi_1335_: bool;
    var phi_2035_: f32;
    var phi_2050_: f32;
    var phi_2051_: f32;
    var phi_1517_: bool;
    var phi_2052_: f32;
    var phi_2053_: f32;
    var phi_2055_: f32;
    var phi_981_: bool;
    var phi_2056_: f32;
    var local: bool;
    var phi_1674_: bool;
    var phi_1676_: bool;
    var phi_2086_: f32;
    var phi_2081_: u32;
    var phi_2078_: f32;
    var phi_2085_: f32;
    var phi_2080_: u32;
    var phi_2077_: f32;
    var phi_2082_: f32;
    var phi_2079_: u32;
    var phi_2076_: f32;
    var phi_2090_: f32;
    var phi_2092_: f32;
    var phi_2100_: f32;
    var phi_2102_: f32;
    var phi_2107_: vec4<f32>;
    var phi_2105_: f32;
    var phi_2109_: f32;
    var phi_2164_: vec4<f32>;

    let _e69 = gl_FragCoord_1;
    let _e73 = bitcast<vec2<u32>>(vec2<i32>(floor(_e69.xy)));
    let _e75 = m.m6_;
    let _e104 = bitcast<i32>((((((_e73.y >> bitcast<u32>(5u)) * (((_e75 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e73.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e73.x & 28u) << bitcast<u32>(5u)) + ((_e73.y & 28u) << bitcast<u32>(2i)))) + (((_e73.y & 3u) << bitcast<u32>(2i)) + (_e73.x & 3u))));
    let _e105 = f1_1;
    if (_e105.w >= 0f) {
        phi_2033_ = vec4<f32>(_e105.x, _e105.y, _e105.z, _e105.w);
    } else {
        if (_e105.w > -1f) {
            if (_e105.z > 0f) {
                phi_2031_ = _e105.x;
            } else {
                phi_2031_ = length(_e105.xy);
            }
            let _e131 = phi_2031_;
            let _e132 = clamp(_e131, 0f, 1f);
            let _e133 = abs(_e105.z);
            if (_e133 > 1f) {
                phi_2032_ = ((0.9980469f * _e132) + 0.0009765625f);
            } else {
                phi_2032_ = ((0.001953125f * _e132) + _e133);
            }
            let _e140 = phi_2032_;
            let _e143 = textureSampleLevel(KD, Jb, vec2<f32>(_e140, -(_e105.w)), 0f);
            phi_2034_ = vec4<f32>(_e143.x, _e143.y, _e143.z, _e143.w);
        } else {
            let _e111 = textureSampleLevel(IC, S5_, _e105.xy, (-2f - _e105.w));
            if (_e111.w != 0f) {
                phi_2027_ = (1f / _e111.w);
            } else {
                phi_2027_ = 0f;
            }
            let _e118 = phi_2027_;
            let _e119 = (_e111.xyz * _e118);
            phi_2034_ = vec4<f32>(_e119.x, _e119.y, _e119.z, (_e111.w * _e105.z));
        }
        let _e151 = phi_2034_;
        phi_2033_ = _e151;
    }
    let _e159 = phi_2033_;
    let _e160 = O_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e163 = (_e160.y >= 0f);
            local = _e163;
            if _e163 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1517_ = Xg;
                        if Xg {
                            phi_1517_ = (_e160.x < -1.5f);
                        }
                        let _e231 = phi_1517_;
                        if _e231 {
                            let _e237 = textureSampleLevel(XC, Z9_, vec2<f32>((3f + _e160.x), 0f), 0f);
                            let _e242 = textureSampleLevel(XC, Z9_, vec2<f32>((1f - _e160.y), 0f), 0f);
                            phi_2052_ = ((1f - _e237.x) - _e242.x);
                            break;
                        } else {
                            phi_2052_ = min(_e160.x, _e160.y);
                            break;
                        }
                    }
                }
                let _e246 = phi_2052_;
                phi_2053_ = _e246;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1335_ = Xg;
                        if Xg {
                            phi_1335_ = (_e160.y < -1.5f);
                        }
                        let _e167 = phi_1335_;
                        if _e167 {
                            let _e171 = max(_e160.w, 0f);
                            if (_e160.z >= 0f) {
                                let _e174 = textureSampleLevel(XC, Z9_, vec2<f32>(_e171, 0f), 0f);
                                phi_2035_ = _e174.x;
                            } else {
                                phi_2035_ = 0f;
                            }
                            let _e177 = phi_2035_;
                            phi_2050_ = _e177;
                            if (abs(_e160.z) < 1000f) {
                                let _e183 = (-2f - _e160.y);
                                let _e185 = ((_e183 - _e171) * 0.5984134f);
                                let _e188 = (vec4(_e171) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e185));
                                let _e194 = ((_e188 * -(_e160.z)) + vec4(((_e183 * _e160.z) + (abs(_e160.x) - 0.25f))));
                                let _e197 = textureSampleLevel(XC, Z9_, vec2<f32>(_e194.x, 0f), 0f);
                                let _e200 = textureSampleLevel(XC, Z9_, vec2<f32>(_e194.y, 0f), 0f);
                                let _e203 = textureSampleLevel(XC, Z9_, vec2<f32>(_e194.z, 0f), 0f);
                                let _e206 = textureSampleLevel(XC, Z9_, vec2<f32>(_e194.w, 0f), 0f);
                                let _e212 = (_e188 * 5.0959306f);
                                phi_2050_ = (_e177 + (dot(vec4<f32>(_e197.x, _e200.x, _e203.x, _e206.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e212) * (_e212 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e185));
                            }
                            let _e221 = phi_2050_;
                            phi_2051_ = (_e221 * sign(_e160.x));
                            break;
                        } else {
                            phi_2051_ = _e160.x;
                            break;
                        }
                    }
                }
                let _e226 = phi_2051_;
                phi_2053_ = _e226;
                break;
            }
        }
    }
    let _e248 = phi_2053_;
    let _e249 = l4_1;
    let _e252 = a3_1[1u];
    let _e254 = a3_1[0u];
    let _e255 = vec2<u32>(floor(_e249));
    let _e282 = (_e254 + (((((_e255.y >> bitcast<u32>(5u)) * (_e252 << bitcast<u32>(5u))) + ((_e255.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e255.x & 28u) << bitcast<u32>(5u)) + ((_e255.y & 28u) << bitcast<u32>(2i)))) + (((_e255.y & 3u) << bitcast<u32>(2i)) + (_e255.x & 3u))));
    phi_2055_ = 1f;
    if Vg {
        let _e283 = L0_1;
        let _e286 = min(_e283.xy, _e283.zw);
        phi_2055_ = min(min(_e286.x, _e286.y), 1f);
    }
    let _e292 = phi_2055_;
    phi_981_ = Ug;
    if Ug {
        let _e294 = U1_1[0u];
        phi_981_ = (_e294 != 0f);
    }
    let _e297 = phi_981_;
    phi_2056_ = _e292;
    if _e297 {
        let _e300 = h0_.c2_[_e104];
        phi_2056_ = min(unpack4x8unorm(_e300).x, _e292);
    }
    let _e305 = phi_2056_;
    let _e307 = clamp(_e248, 0f, max(_e305, 0f));
    let _e309 = local;
    if _e309 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e159.w, _e307) >= 1f) {
                    phi_2102_ = _e159.w;
                    break;
                }
                let _e406 = m.a2_;
                let _e408 = atomicMax((&P0_.c2_[_e282]), (_e406 | u32(((abs(_e307) * 1024f) + 0.5f))));
                if (_e408 < _e406) {
                    phi_2100_ = _e307;
                } else {
                    let _e412 = (f32((_e408 & 524287u)) * 0.0009765625f);
                    phi_2100_ = ((max(_e412, _e307) - _e412) / max((1f - (_e412 * _e159.w)), 0.000062f));
                }
                let _e420 = phi_2100_;
                phi_2102_ = (_e159.w * _e420);
                break;
            }
        }
        let _e423 = phi_2102_;
        phi_2107_ = vec4<f32>(_e159.x, _e159.y, _e159.z, _e423);
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e315 = u32(((abs(_e307) * 1024f) + 0.5f));
                let _e318 = atomicLoad((&P0_.c2_[_e282]));
                let _e320 = (min(_e159.w, _e307) >= 1f);
                phi_1676_ = _e320;
                if _e320 {
                    let _e322 = m.a2_;
                    let _e323 = (_e318 < _e322);
                    phi_1674_ = _e323;
                    if !(_e323) {
                        phi_1674_ = (_e318 >= (_e322 | 262144u));
                    }
                    let _e328 = phi_1674_;
                    phi_1676_ = _e328;
                }
                let _e330 = phi_1676_;
                if _e330 {
                    phi_2092_ = _e159.w;
                    break;
                }
                let _e332 = m.a2_;
                phi_2082_ = 0f;
                phi_2079_ = _e315;
                phi_2076_ = _e307;
                if (_e318 < _e332) {
                    let _e335 = (_e332 | (262144u + _e315));
                    let _e336 = atomicMax((&P0_.c2_[_e282]), _e335);
                    if (_e336 <= _e332) {
                        phi_2085_ = _e307;
                        phi_2080_ = _e315;
                        phi_2077_ = 0f;
                    } else {
                        phi_2086_ = 0f;
                        phi_2081_ = _e315;
                        phi_2078_ = _e307;
                        if (_e336 < _e335) {
                            let _e340 = ((_e336 & 524287u) - 262144u);
                            let _e342 = (f32(_e340) * 0.0009765625f);
                            phi_2086_ = ((_e307 - _e342) / max((1f - (_e342 * _e159.w)), 0.000062f));
                            phi_2081_ = _e340;
                            phi_2078_ = _e342;
                        }
                        let _e349 = phi_2086_;
                        let _e351 = phi_2081_;
                        let _e353 = phi_2078_;
                        phi_2085_ = _e349;
                        phi_2080_ = _e351;
                        phi_2077_ = _e353;
                    }
                    let _e355 = phi_2085_;
                    let _e357 = phi_2080_;
                    let _e359 = phi_2077_;
                    phi_2082_ = _e355;
                    phi_2079_ = _e357;
                    phi_2076_ = _e359;
                }
                let _e361 = phi_2082_;
                let _e363 = phi_2079_;
                let _e365 = phi_2076_;
                phi_2090_ = _e361;
                if (_e365 > 0f) {
                    let _e367 = atomicAdd((&P0_.c2_[_e282]), _e363);
                    let _e372 = (f32(bitcast<i32>(((_e367 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e374 = clamp(_e372, 0f, 1f);
                    phi_2090_ = (_e361 + ((1f - (_e361 * _e159.w)) * ((clamp((_e372 + _e365), 0f, 1f) - _e374) / max((1f - (_e374 * _e159.w)), 0.000062f))));
                }
                let _e386 = phi_2090_;
                phi_2092_ = (_e159.w * _e386);
                break;
            }
        }
        let _e389 = phi_2092_;
        phi_2107_ = vec4<f32>(_e159.x, _e159.y, _e159.z, _e389);
    }
    let _e430 = phi_2107_;
    phi_2109_ = f32();
    if bh {
        let _e432 = m.y3_;
        let _e434 = m.z3_;
        if bh {
            phi_2105_ = ((fract((52.982918f * fract(((0.06711056f * _e69.x) + (0.00583715f * _e69.y))))) * _e432) + _e434);
        } else {
            phi_2105_ = 0f;
        }
        let _e446 = phi_2105_;
        phi_2109_ = _e446;
    }
    let _e448 = phi_2109_;
    let _e451 = (_e430.xyz * _e430.w);
    let _e457 = vec4<f32>(_e451.x, _e430.y, _e430.z, _e430.w);
    let _e463 = vec4<f32>(_e457.x, _e451.y, _e457.z, _e457.w);
    let _e469 = vec4<f32>(_e463.x, _e463.y, _e451.z, _e463.w);
    phi_2164_ = _e469;
    if bh {
        let _e472 = (_e469.xyz + vec3(_e448));
        let _e478 = vec4<f32>(_e472.x, _e469.y, _e469.z, _e469.w);
        let _e484 = vec4<f32>(_e478.x, _e472.y, _e478.z, _e478.w);
        phi_2164_ = vec4<f32>(_e484.x, _e484.y, _e472.z, _e484.w);
    }
    let _e492 = phi_2164_;
    h0_.c2_[_e104] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    C1_ = _e492;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) f1_: vec4<f32>, @location(2) O: vec4<f32>, @location(8) l4_: vec2<f32>, @location(7) @interpolate(flat) a3_: vec2<u32>, @location(5) L0_: vec4<f32>, @location(4) @interpolate(flat) U1_: vec2<f32>, @location(3) @interpolate(flat) A0_: f32, @location(6) @interpolate(flat) e2_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    f1_1 = f1_;
    O_1 = O;
    l4_1 = l4_;
    a3_1 = a3_;
    L0_1 = L0_;
    U1_1 = U1_;
    A0_1 = A0_;
    e2_1 = e2_;
    main_1();
    let _e19 = C1_;
    return _e19;
}
