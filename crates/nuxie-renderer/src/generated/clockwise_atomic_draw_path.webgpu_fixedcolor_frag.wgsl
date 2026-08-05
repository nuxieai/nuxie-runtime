struct ge {
    c2_: array<u32>,
}

struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Fg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Bg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Gg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    yg: u32,
}

struct h0Bd {
    c2_: array<u32>,
}

struct ge_1 {
    c2_: array<atomic<u32>>,
}

@id(7) override fh: bool = true;
@id(3) override bh: bool = true;
@id(1) override Zg: bool = true;
@id(0) override Yg: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var KD: texture_2d<f32>;
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
var<private> gl_FragCoord_1: vec4<f32>;
var<private> f1_1: vec4<f32>;
var<private> O_1: vec4<f32>;
var<private> l4_1: vec2<f32>;
var<private> d3_1: vec2<u32>;
var<private> L0_1: vec4<f32>;
var<private> U1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Bd;
var<private> C1_: vec4<f32>;
var<private> B0_1: f32;
var<private> e2_1: f32;

fn main_1() {
    var phi_2066_: f32;
    var phi_2070_: f32;
    var phi_2071_: f32;
    var phi_2073_: vec4<f32>;
    var phi_2072_: vec4<f32>;
    var phi_1358_: bool;
    var phi_2074_: f32;
    var phi_2089_: f32;
    var phi_2090_: f32;
    var phi_1540_: bool;
    var phi_2091_: f32;
    var phi_2092_: f32;
    var phi_2094_: f32;
    var phi_1002_: bool;
    var phi_2095_: f32;
    var local: bool;
    var phi_1697_: bool;
    var phi_1699_: bool;
    var phi_2125_: f32;
    var phi_2120_: u32;
    var phi_2117_: f32;
    var phi_2124_: f32;
    var phi_2119_: u32;
    var phi_2116_: f32;
    var phi_2121_: f32;
    var phi_2118_: u32;
    var phi_2115_: f32;
    var phi_2129_: f32;
    var phi_2131_: f32;
    var phi_2139_: f32;
    var phi_2141_: f32;
    var phi_2146_: vec4<f32>;
    var phi_2144_: f32;
    var phi_2148_: f32;
    var phi_2175_: vec3<f32>;

    let _e69 = gl_FragCoord_1;
    let _e73 = bitcast<vec2<u32>>(vec2<i32>(floor(_e69.xy)));
    let _e75 = n.m6_;
    let _e104 = bitcast<i32>((((((_e73.y >> bitcast<u32>(5u)) * (((_e75 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e73.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e73.x & 28u) << bitcast<u32>(5u)) + ((_e73.y & 28u) << bitcast<u32>(2i)))) + (((_e73.y & 3u) << bitcast<u32>(2i)) + (_e73.x & 3u))));
    let _e105 = f1_1;
    if (_e105.w >= 0f) {
        phi_2072_ = vec4<f32>(_e105.x, _e105.y, _e105.z, _e105.w);
    } else {
        if (_e105.w > -1f) {
            if (_e105.z > 0f) {
                phi_2070_ = _e105.x;
            } else {
                phi_2070_ = length(_e105.xy);
            }
            let _e131 = phi_2070_;
            let _e132 = clamp(_e131, 0f, 1f);
            let _e133 = abs(_e105.z);
            if (_e133 > 1f) {
                phi_2071_ = ((0.9980469f * _e132) + 0.0009765625f);
            } else {
                phi_2071_ = ((0.001953125f * _e132) + _e133);
            }
            let _e140 = phi_2071_;
            let _e143 = textureSampleLevel(KD, Mb, vec2<f32>(_e140, -(_e105.w)), 0f);
            phi_2073_ = vec4<f32>(_e143.x, _e143.y, _e143.z, _e143.w);
        } else {
            let _e111 = textureSampleLevel(IC, S5_, _e105.xy, (-2f - _e105.w));
            if (_e111.w != 0f) {
                phi_2066_ = (1f / _e111.w);
            } else {
                phi_2066_ = 0f;
            }
            let _e118 = phi_2066_;
            let _e119 = (_e111.xyz * _e118);
            phi_2073_ = vec4<f32>(_e119.x, _e119.y, _e119.z, (_e111.w * _e105.z));
        }
        let _e151 = phi_2073_;
        phi_2072_ = _e151;
    }
    let _e159 = phi_2072_;
    let _e160 = O_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e163 = (_e160.y >= 0f);
            local = _e163;
            if _e163 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1540_ = bh;
                        if bh {
                            phi_1540_ = (_e160.x < -1.5f);
                        }
                        let _e231 = phi_1540_;
                        if _e231 {
                            let _e237 = textureSampleLevel(XC, aa, vec2<f32>((3f + _e160.x), 0f), 0f);
                            let _e242 = textureSampleLevel(XC, aa, vec2<f32>((1f - _e160.y), 0f), 0f);
                            phi_2091_ = ((1f - _e237.x) - _e242.x);
                            break;
                        } else {
                            phi_2091_ = min(_e160.x, _e160.y);
                            break;
                        }
                    }
                }
                let _e246 = phi_2091_;
                phi_2092_ = _e246;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1358_ = bh;
                        if bh {
                            phi_1358_ = (_e160.y < -1.5f);
                        }
                        let _e167 = phi_1358_;
                        if _e167 {
                            let _e171 = max(_e160.w, 0f);
                            if (_e160.z >= 0f) {
                                let _e174 = textureSampleLevel(XC, aa, vec2<f32>(_e171, 0f), 0f);
                                phi_2074_ = _e174.x;
                            } else {
                                phi_2074_ = 0f;
                            }
                            let _e177 = phi_2074_;
                            phi_2089_ = _e177;
                            if (abs(_e160.z) < 1000f) {
                                let _e183 = (-2f - _e160.y);
                                let _e185 = ((_e183 - _e171) * 0.5984134f);
                                let _e188 = (vec4(_e171) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e185));
                                let _e194 = ((_e188 * -(_e160.z)) + vec4(((_e183 * _e160.z) + (abs(_e160.x) - 0.25f))));
                                let _e197 = textureSampleLevel(XC, aa, vec2<f32>(_e194.x, 0f), 0f);
                                let _e200 = textureSampleLevel(XC, aa, vec2<f32>(_e194.y, 0f), 0f);
                                let _e203 = textureSampleLevel(XC, aa, vec2<f32>(_e194.z, 0f), 0f);
                                let _e206 = textureSampleLevel(XC, aa, vec2<f32>(_e194.w, 0f), 0f);
                                let _e212 = (_e188 * 5.0959306f);
                                phi_2089_ = (_e177 + (dot(vec4<f32>(_e197.x, _e200.x, _e203.x, _e206.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e212) * (_e212 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e185));
                            }
                            let _e221 = phi_2089_;
                            phi_2090_ = (_e221 * sign(_e160.x));
                            break;
                        } else {
                            phi_2090_ = _e160.x;
                            break;
                        }
                    }
                }
                let _e226 = phi_2090_;
                phi_2092_ = _e226;
                break;
            }
        }
    }
    let _e248 = phi_2092_;
    let _e249 = l4_1;
    let _e252 = d3_1[1u];
    let _e254 = d3_1[0u];
    let _e255 = vec2<u32>(floor(_e249));
    let _e282 = (_e254 + (((((_e255.y >> bitcast<u32>(5u)) * (_e252 << bitcast<u32>(5u))) + ((_e255.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e255.x & 28u) << bitcast<u32>(5u)) + ((_e255.y & 28u) << bitcast<u32>(2i)))) + (((_e255.y & 3u) << bitcast<u32>(2i)) + (_e255.x & 3u))));
    phi_2094_ = 1f;
    if Zg {
        let _e283 = L0_1;
        let _e286 = min(_e283.xy, _e283.zw);
        phi_2094_ = min(min(_e286.x, _e286.y), 1f);
    }
    let _e292 = phi_2094_;
    phi_1002_ = Yg;
    if Yg {
        let _e294 = U1_1[0u];
        phi_1002_ = (_e294 != 0f);
    }
    let _e297 = phi_1002_;
    phi_2095_ = _e292;
    if _e297 {
        let _e300 = h0_.c2_[_e104];
        phi_2095_ = min(unpack4x8unorm(_e300).x, _e292);
    }
    let _e305 = phi_2095_;
    let _e307 = clamp(_e248, 0f, max(_e305, 0f));
    let _e309 = local;
    if _e309 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e159.w, _e307) >= 1f) {
                    phi_2141_ = _e159.w;
                    break;
                }
                let _e406 = n.a2_;
                let _e408 = atomicMax((&P0_.c2_[_e282]), (_e406 | u32(((abs(_e307) * 1024f) + 0.5f))));
                if (_e408 < _e406) {
                    phi_2139_ = _e307;
                } else {
                    let _e412 = (f32((_e408 & 524287u)) * 0.0009765625f);
                    phi_2139_ = ((max(_e412, _e307) - _e412) / max((1f - (_e412 * _e159.w)), 0.000062f));
                }
                let _e420 = phi_2139_;
                phi_2141_ = (_e159.w * _e420);
                break;
            }
        }
        let _e423 = phi_2141_;
        phi_2146_ = vec4<f32>(_e159.x, _e159.y, _e159.z, _e423);
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e315 = u32(((abs(_e307) * 1024f) + 0.5f));
                let _e318 = atomicLoad((&P0_.c2_[_e282]));
                let _e320 = (min(_e159.w, _e307) >= 1f);
                phi_1699_ = _e320;
                if _e320 {
                    let _e322 = n.a2_;
                    let _e323 = (_e318 < _e322);
                    phi_1697_ = _e323;
                    if !(_e323) {
                        phi_1697_ = (_e318 >= (_e322 | 262144u));
                    }
                    let _e328 = phi_1697_;
                    phi_1699_ = _e328;
                }
                let _e330 = phi_1699_;
                if _e330 {
                    phi_2131_ = _e159.w;
                    break;
                }
                let _e332 = n.a2_;
                phi_2121_ = 0f;
                phi_2118_ = _e315;
                phi_2115_ = _e307;
                if (_e318 < _e332) {
                    let _e335 = (_e332 | (262144u + _e315));
                    let _e336 = atomicMax((&P0_.c2_[_e282]), _e335);
                    if (_e336 <= _e332) {
                        phi_2124_ = _e307;
                        phi_2119_ = _e315;
                        phi_2116_ = 0f;
                    } else {
                        phi_2125_ = 0f;
                        phi_2120_ = _e315;
                        phi_2117_ = _e307;
                        if (_e336 < _e335) {
                            let _e340 = ((_e336 & 524287u) - 262144u);
                            let _e342 = (f32(_e340) * 0.0009765625f);
                            phi_2125_ = ((_e307 - _e342) / max((1f - (_e342 * _e159.w)), 0.000062f));
                            phi_2120_ = _e340;
                            phi_2117_ = _e342;
                        }
                        let _e349 = phi_2125_;
                        let _e351 = phi_2120_;
                        let _e353 = phi_2117_;
                        phi_2124_ = _e349;
                        phi_2119_ = _e351;
                        phi_2116_ = _e353;
                    }
                    let _e355 = phi_2124_;
                    let _e357 = phi_2119_;
                    let _e359 = phi_2116_;
                    phi_2121_ = _e355;
                    phi_2118_ = _e357;
                    phi_2115_ = _e359;
                }
                let _e361 = phi_2121_;
                let _e363 = phi_2118_;
                let _e365 = phi_2115_;
                phi_2129_ = _e361;
                if (_e365 > 0f) {
                    let _e367 = atomicAdd((&P0_.c2_[_e282]), _e363);
                    let _e372 = (f32(bitcast<i32>(((_e367 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e374 = clamp(_e372, 0f, 1f);
                    phi_2129_ = (_e361 + ((1f - (_e361 * _e159.w)) * ((clamp((_e372 + _e365), 0f, 1f) - _e374) / max((1f - (_e374 * _e159.w)), 0.000062f))));
                }
                let _e386 = phi_2129_;
                phi_2131_ = (_e159.w * _e386);
                break;
            }
        }
        let _e389 = phi_2131_;
        phi_2146_ = vec4<f32>(_e159.x, _e159.y, _e159.z, _e389);
    }
    let _e430 = phi_2146_;
    phi_2148_ = f32();
    if fh {
        let _e432 = n.z3_;
        let _e434 = n.A3_;
        if fh {
            phi_2144_ = ((fract((52.982918f * fract(((0.06711056f * _e69.x) + (0.00583715f * _e69.y))))) * _e432) + _e434);
        } else {
            phi_2144_ = 0f;
        }
        let _e446 = phi_2144_;
        phi_2148_ = _e446;
    }
    let _e448 = phi_2148_;
    let _e451 = (_e430.xyz * _e430.w);
    let _e457 = vec4<f32>(_e451.x, _e430.y, _e430.z, _e430.w);
    let _e463 = vec4<f32>(_e457.x, _e451.y, _e457.z, _e457.w);
    let _e469 = vec4<f32>(_e463.x, _e463.y, _e451.z, _e463.w);
    let _e470 = _e469.xyz;
    if (fh && (_e430.w != 0f)) {
        phi_2175_ = (vec3(_e448) + _e470);
    } else {
        phi_2175_ = _e470;
    }
    let _e476 = phi_2175_;
    let _e482 = vec4<f32>(_e476.x, _e469.y, _e469.z, _e469.w);
    let _e488 = vec4<f32>(_e482.x, _e476.y, _e482.z, _e482.w);
    h0_.c2_[_e104] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    C1_ = vec4<f32>(_e488.x, _e488.y, _e476.z, _e488.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) f1_: vec4<f32>, @location(2) O: vec4<f32>, @location(8) l4_: vec2<f32>, @location(7) @interpolate(flat, either) d3_: vec2<u32>, @location(5) L0_: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    f1_1 = f1_;
    O_1 = O;
    l4_1 = l4_;
    d3_1 = d3_;
    L0_1 = L0_;
    U1_1 = U1_;
    B0_1 = B0_;
    e2_1 = e2_;
    main_1();
    let _e19 = C1_;
    return _e19;
}
