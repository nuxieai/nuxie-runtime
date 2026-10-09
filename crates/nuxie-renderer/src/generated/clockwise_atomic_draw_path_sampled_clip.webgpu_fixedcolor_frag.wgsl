struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct zf {
    r2_: array<u32>,
}

struct zf_1 {
    r2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(0) member: vec4<f32>,
    @location(1) member_1: vec4<f32>,
}

@id(7) override dj: bool = true;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;
@id(3) override Zi: bool = true;
@id(1) override Xi: bool = true;
@id(0) override Wi: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(6)
var<storage, read_write> Y0_: zf_1;
var<private> U0_1: vec3<f32>;
var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> L4_1: vec2<f32>;
var<private> z3_1: vec2<u32>;
var<private> V0_1: vec4<f32>;
var<private> i2_1: vec2<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> L1_: vec4<f32>;
var<private> G0_1: f32;
var<private> m0_: vec4<f32>;

fn main_1() {
    var phi_2161_: f32;
    var phi_2176_: vec4<f32>;
    var phi_2175_: vec4<f32>;
    var phi_1270_: bool;
    var phi_1285_: bool;
    var phi_2162_: f32;
    var phi_2171_: vec4<f32>;
    var phi_2178_: vec4<f32>;
    var phi_2179_: vec4<f32>;
    var phi_1531_: bool;
    var phi_2180_: f32;
    var phi_2203_: f32;
    var phi_2204_: f32;
    var phi_1468_: bool;
    var phi_2205_: f32;
    var phi_2206_: f32;
    var phi_2208_: f32;
    var phi_1070_: bool;
    var phi_2209_: f32;
    var local: bool;
    var phi_1886_: bool;
    var phi_1888_: bool;
    var phi_2237_: f32;
    var phi_2232_: u32;
    var phi_2229_: f32;
    var phi_2236_: f32;
    var phi_2231_: u32;
    var phi_2228_: f32;
    var phi_2233_: f32;
    var phi_2230_: u32;
    var phi_2227_: f32;
    var phi_2238_: f32;
    var phi_2239_: f32;
    var phi_2240_: f32;
    var phi_2241_: f32;
    var phi_2244_: f32;
    var phi_2242_: f32;
    var phi_2258_: f32;
    var phi_2289_: vec3<f32>;

    let _e72 = P0_1;
    let _e74 = U0_1;
    let _e75 = O0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e78 = (Yi && (u32(_e72) != 0u));
            if (_e75.w >= 0f) {
                phi_2175_ = _e75;
            } else {
                let _e82 = j.L8_;
                let _e84 = j.M8_;
                let _e87 = abs(_e75.z);
                let _e88 = floor(_e87);
                let _e91 = (((_e87 - _e88) * 8f) + 0.0009765625f);
                if (floor(_e91) == 2f) {
                    phi_2161_ = _e75.x;
                } else {
                    phi_2161_ = length(_e75.xy);
                }
                let _e100 = phi_2161_;
                let _e106 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e100, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e75.z < 0f))) + _e91), ((_e88 * _e82) + _e84)), 0f);
                phi_2176_ = _e106;
                if !(_e78) {
                    let _e112 = (_e106.xyz * _e106.w);
                    phi_2176_ = vec4<f32>(_e112.x, _e112.y, _e112.z, (_e106.w * (ceil(_e75.w) * -0.003921569f)));
                }
                let _e119 = phi_2176_;
                phi_2175_ = _e119;
            }
            let _e121 = phi_2175_;
            phi_1270_ = ej;
            if ej {
                phi_1270_ = (_e74.z < 0f);
            }
            let _e125 = phi_1270_;
            if _e125 {
                let _e127 = textureSampleLevel(TB, U4_, _e74.xy, 0f);
                phi_2179_ = _e127;
                break;
            }
            phi_1285_ = ej;
            if ej {
                phi_1285_ = (_e74.z > 0f);
            }
            let _e131 = phi_1285_;
            phi_2178_ = _e121;
            if _e131 {
                let _e135 = textureSampleLevel(TB, U4_, _e74.xy, (_e74.z - 1f));
                phi_2171_ = _e135;
                if _e78 {
                    if (_e135.w != 0f) {
                        phi_2162_ = (1f / _e135.w);
                    } else {
                        phi_2162_ = 0f;
                    }
                    let _e141 = phi_2162_;
                    let _e142 = (_e135.xyz * _e141);
                    phi_2171_ = vec4<f32>(_e142.x, _e142.y, _e142.z, _e135.w);
                }
                let _e148 = phi_2171_;
                phi_2178_ = (_e121 * _e148);
            }
            let _e151 = phi_2178_;
            phi_2179_ = _e151;
            break;
        }
    }
    let _e153 = phi_2179_;
    let _e154 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e157 = (_e154.y >= 0f);
            local = _e157;
            if _e157 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1468_ = Zi;
                        if Zi {
                            phi_1468_ = (_e154.x < -1.5f);
                        }
                        let _e225 = phi_1468_;
                        if _e225 {
                            let _e231 = textureSampleLevel(YC, ab, vec2<f32>((3f + _e154.x), 0f), 0f);
                            let _e236 = textureSampleLevel(YC, ab, vec2<f32>((1f - _e154.y), 0f), 0f);
                            phi_2205_ = ((1f - _e231.x) - _e236.x);
                            break;
                        } else {
                            phi_2205_ = min(_e154.x, _e154.y);
                            break;
                        }
                    }
                }
                let _e240 = phi_2205_;
                phi_2206_ = _e240;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1531_ = Zi;
                        if Zi {
                            phi_1531_ = (_e154.y < -1.5f);
                        }
                        let _e161 = phi_1531_;
                        if _e161 {
                            let _e165 = max(_e154.w, 0f);
                            if (_e154.z >= 0f) {
                                let _e168 = textureSampleLevel(YC, ab, vec2<f32>(_e165, 0f), 0f);
                                phi_2180_ = _e168.x;
                            } else {
                                phi_2180_ = 0f;
                            }
                            let _e171 = phi_2180_;
                            phi_2203_ = _e171;
                            if (abs(_e154.z) < 1000f) {
                                let _e177 = (-2f - _e154.y);
                                let _e179 = ((_e177 - _e165) * 0.5984134f);
                                let _e182 = (vec4(_e165) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e179));
                                let _e188 = ((_e182 * -(_e154.z)) + vec4(((_e177 * _e154.z) + (abs(_e154.x) - 0.25f))));
                                let _e191 = textureSampleLevel(YC, ab, vec2<f32>(_e188.x, 0f), 0f);
                                let _e194 = textureSampleLevel(YC, ab, vec2<f32>(_e188.y, 0f), 0f);
                                let _e197 = textureSampleLevel(YC, ab, vec2<f32>(_e188.z, 0f), 0f);
                                let _e200 = textureSampleLevel(YC, ab, vec2<f32>(_e188.w, 0f), 0f);
                                let _e206 = (_e182 * 5.0959306f);
                                phi_2203_ = (_e171 + (dot(vec4<f32>(_e191.x, _e194.x, _e197.x, _e200.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e206) * (_e206 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e179));
                            }
                            let _e215 = phi_2203_;
                            phi_2204_ = (_e215 * sign(_e154.x));
                            break;
                        } else {
                            phi_2204_ = _e154.x;
                            break;
                        }
                    }
                }
                let _e220 = phi_2204_;
                phi_2206_ = _e220;
                break;
            }
        }
    }
    let _e242 = phi_2206_;
    let _e243 = L4_1;
    let _e246 = z3_1[1u];
    let _e248 = z3_1[0u];
    let _e249 = vec2<u32>(floor(_e243));
    let _e276 = (_e248 + (((((_e249.y >> bitcast<u32>(5u)) * (_e246 << bitcast<u32>(5u))) + ((_e249.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e249.x & 28u) << bitcast<u32>(5u)) + ((_e249.y & 28u) << bitcast<u32>(2i)))) + (((_e249.y & 3u) << bitcast<u32>(2i)) + (_e249.x & 3u))));
    phi_2208_ = 1f;
    if Xi {
        let _e277 = V0_1;
        let _e280 = min(_e277.xy, _e277.zw);
        phi_2208_ = min(min(_e280.x, _e280.y), 1f);
    }
    let _e286 = phi_2208_;
    phi_1070_ = Wi;
    if Wi {
        let _e288 = i2_1[0u];
        phi_1070_ = (_e288 != 0f);
    }
    let _e291 = phi_1070_;
    phi_2209_ = _e286;
    if _e291 {
        phi_2209_ = min(0f, _e286);
    }
    let _e294 = phi_2209_;
    let _e296 = clamp(_e242, 0f, max(_e294, 0f));
    let _e298 = local;
    if _e298 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e153.w, _e296) >= 1f) {
                    phi_2241_ = 1f;
                    break;
                }
                let _e389 = j.q2_;
                let _e391 = atomicMax((&Y0_.r2_[_e276]), (_e389 | u32(((abs(_e296) * 1024f) + 0.5f))));
                if (_e391 < _e389) {
                    phi_2240_ = _e296;
                } else {
                    let _e395 = (f32((_e391 & 524287u)) * 0.0009765625f);
                    phi_2240_ = ((max(_e395, _e296) - _e395) / max((1f - (_e395 * _e153.w)), 0.000062f));
                }
                let _e403 = phi_2240_;
                phi_2241_ = _e403;
                break;
            }
        }
        let _e405 = phi_2241_;
        phi_2244_ = _e405;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e304 = u32(((abs(_e296) * 1024f) + 0.5f));
                let _e307 = atomicLoad((&Y0_.r2_[_e276]));
                let _e309 = (min(_e153.w, _e296) >= 1f);
                phi_1888_ = _e309;
                if _e309 {
                    let _e311 = j.q2_;
                    let _e312 = (_e307 < _e311);
                    phi_1886_ = _e312;
                    if !(_e312) {
                        phi_1886_ = (_e307 >= (_e311 | 262144u));
                    }
                    let _e317 = phi_1886_;
                    phi_1888_ = _e317;
                }
                let _e319 = phi_1888_;
                if _e319 {
                    phi_2239_ = 1f;
                    break;
                }
                let _e321 = j.q2_;
                phi_2233_ = 0f;
                phi_2230_ = _e304;
                phi_2227_ = _e296;
                if (_e307 < _e321) {
                    let _e324 = (_e321 | (262144u + _e304));
                    let _e325 = atomicMax((&Y0_.r2_[_e276]), _e324);
                    if (_e325 <= _e321) {
                        phi_2236_ = _e296;
                        phi_2231_ = _e304;
                        phi_2228_ = 0f;
                    } else {
                        phi_2237_ = 0f;
                        phi_2232_ = _e304;
                        phi_2229_ = _e296;
                        if (_e325 < _e324) {
                            let _e329 = ((_e325 & 524287u) - 262144u);
                            let _e331 = (f32(_e329) * 0.0009765625f);
                            phi_2237_ = ((_e296 - _e331) / max((1f - (_e331 * _e153.w)), 0.000062f));
                            phi_2232_ = _e329;
                            phi_2229_ = _e331;
                        }
                        let _e338 = phi_2237_;
                        let _e340 = phi_2232_;
                        let _e342 = phi_2229_;
                        phi_2236_ = _e338;
                        phi_2231_ = _e340;
                        phi_2228_ = _e342;
                    }
                    let _e344 = phi_2236_;
                    let _e346 = phi_2231_;
                    let _e348 = phi_2228_;
                    phi_2233_ = _e344;
                    phi_2230_ = _e346;
                    phi_2227_ = _e348;
                }
                let _e350 = phi_2233_;
                let _e352 = phi_2230_;
                let _e354 = phi_2227_;
                phi_2238_ = _e350;
                if (_e354 > 0f) {
                    let _e356 = atomicAdd((&Y0_.r2_[_e276]), _e352);
                    let _e361 = (f32(bitcast<i32>(((_e356 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e363 = clamp(_e361, 0f, 1f);
                    phi_2238_ = (_e350 + ((1f - (_e350 * _e153.w)) * ((clamp((_e361 + _e354), 0f, 1f) - _e363) / max((1f - (_e363 * _e153.w)), 0.000062f))));
                }
                let _e375 = phi_2238_;
                phi_2239_ = _e375;
                break;
            }
        }
        let _e377 = phi_2239_;
        phi_2244_ = _e377;
    }
    let _e407 = phi_2244_;
    phi_2258_ = f32();
    if dj {
        let _e408 = gl_FragCoord_1;
        let _e410 = j.F3_;
        let _e412 = j.G3_;
        if dj {
            phi_2242_ = ((fract((52.982918f * fract(((0.06711056f * _e408.x) + (0.00583715f * _e408.y))))) * _e410) + _e412);
        } else {
            phi_2242_ = 0f;
        }
        let _e424 = phi_2242_;
        phi_2258_ = _e424;
    }
    let _e426 = phi_2258_;
    let _e427 = (_e153 * _e407);
    let _e428 = _e427.xyz;
    if (dj && (_e427.w != 0f)) {
        phi_2289_ = (vec3(_e426) + _e428);
    } else {
        phi_2289_ = _e428;
    }
    let _e435 = phi_2289_;
    let _e441 = vec4<f32>(_e435.x, _e427.y, _e427.z, _e427.w);
    let _e447 = vec4<f32>(_e441.x, _e435.y, _e441.z, _e441.w);
    L1_ = vec4<f32>(_e447.x, _e447.y, _e435.z, _e447.w);
    return;
}

@fragment
fn main(@location(9) U0_: vec3<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) L4_: vec2<f32>, @location(7) @interpolate(flat, either) z3_: vec2<u32>, @location(5) V0_: vec4<f32>, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32) -> FragmentOutput {
    U0_1 = U0_;
    P0_1 = P0_;
    O0_1 = O0_;
    S_1 = S;
    L4_1 = L4_;
    z3_1 = z3_;
    V0_1 = V0_;
    i2_1 = i2_;
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    main_1();
    let _e22 = L1_;
    let _e23 = m0_;
    return FragmentOutput(_e22, _e23);
}
