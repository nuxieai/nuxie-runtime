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
var<private> f1_1: vec4<f32>;
var<private> O_1: vec4<f32>;
var<private> l4_1: vec2<f32>;
var<private> a3_1: vec2<u32>;
var<private> L0_1: vec4<f32>;
var<private> U1_1: vec2<f32>;
@group(2) @binding(1)
var h0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C1_: vec4<f32>;
var<private> A0_1: f32;
var<private> e2_1: f32;

fn main_1() {
    var phi_1923_: f32;
    var phi_1927_: f32;
    var phi_1928_: f32;
    var phi_1930_: vec4<f32>;
    var phi_1929_: vec4<f32>;
    var phi_1257_: bool;
    var phi_1931_: f32;
    var phi_1946_: f32;
    var phi_1947_: f32;
    var phi_1439_: bool;
    var phi_1948_: f32;
    var phi_1949_: f32;
    var phi_1951_: f32;
    var phi_943_: bool;
    var phi_1952_: f32;
    var local: bool;
    var phi_1596_: bool;
    var phi_1598_: bool;
    var phi_1982_: f32;
    var phi_1977_: u32;
    var phi_1974_: f32;
    var phi_1981_: f32;
    var phi_1976_: u32;
    var phi_1973_: f32;
    var phi_1978_: f32;
    var phi_1975_: u32;
    var phi_1972_: f32;
    var phi_1986_: f32;
    var phi_1988_: f32;
    var phi_1996_: f32;
    var phi_1998_: f32;
    var phi_2003_: vec4<f32>;
    var phi_2001_: f32;
    var phi_2005_: f32;
    var phi_2032_: vec4<f32>;

    let _e65 = f1_1;
    if (_e65.w >= 0f) {
        phi_1929_ = vec4<f32>(_e65.x, _e65.y, _e65.z, _e65.w);
    } else {
        if (_e65.w > -1f) {
            if (_e65.z > 0f) {
                phi_1927_ = _e65.x;
            } else {
                phi_1927_ = length(_e65.xy);
            }
            let _e91 = phi_1927_;
            let _e92 = clamp(_e91, 0f, 1f);
            let _e93 = abs(_e65.z);
            if (_e93 > 1f) {
                phi_1928_ = ((0.9980469f * _e92) + 0.0009765625f);
            } else {
                phi_1928_ = ((0.001953125f * _e92) + _e93);
            }
            let _e100 = phi_1928_;
            let _e103 = textureSampleLevel(KD, Jb, vec2<f32>(_e100, -(_e65.w)), 0f);
            phi_1930_ = vec4<f32>(_e103.x, _e103.y, _e103.z, _e103.w);
        } else {
            let _e71 = textureSampleLevel(IC, S5_, _e65.xy, (-2f - _e65.w));
            if (_e71.w != 0f) {
                phi_1923_ = (1f / _e71.w);
            } else {
                phi_1923_ = 0f;
            }
            let _e78 = phi_1923_;
            let _e79 = (_e71.xyz * _e78);
            phi_1930_ = vec4<f32>(_e79.x, _e79.y, _e79.z, (_e71.w * _e65.z));
        }
        let _e111 = phi_1930_;
        phi_1929_ = _e111;
    }
    let _e119 = phi_1929_;
    let _e120 = O_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e123 = (_e120.y >= 0f);
            local = _e123;
            if _e123 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1439_ = Xg;
                        if Xg {
                            phi_1439_ = (_e120.x < -1.5f);
                        }
                        let _e191 = phi_1439_;
                        if _e191 {
                            let _e197 = textureSampleLevel(XC, Z9_, vec2<f32>((3f + _e120.x), 0f), 0f);
                            let _e202 = textureSampleLevel(XC, Z9_, vec2<f32>((1f - _e120.y), 0f), 0f);
                            phi_1948_ = ((1f - _e197.x) - _e202.x);
                            break;
                        } else {
                            phi_1948_ = min(_e120.x, _e120.y);
                            break;
                        }
                    }
                }
                let _e206 = phi_1948_;
                phi_1949_ = _e206;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1257_ = Xg;
                        if Xg {
                            phi_1257_ = (_e120.y < -1.5f);
                        }
                        let _e127 = phi_1257_;
                        if _e127 {
                            let _e131 = max(_e120.w, 0f);
                            if (_e120.z >= 0f) {
                                let _e134 = textureSampleLevel(XC, Z9_, vec2<f32>(_e131, 0f), 0f);
                                phi_1931_ = _e134.x;
                            } else {
                                phi_1931_ = 0f;
                            }
                            let _e137 = phi_1931_;
                            phi_1946_ = _e137;
                            if (abs(_e120.z) < 1000f) {
                                let _e143 = (-2f - _e120.y);
                                let _e145 = ((_e143 - _e131) * 0.5984134f);
                                let _e148 = (vec4(_e131) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e145));
                                let _e154 = ((_e148 * -(_e120.z)) + vec4(((_e143 * _e120.z) + (abs(_e120.x) - 0.25f))));
                                let _e157 = textureSampleLevel(XC, Z9_, vec2<f32>(_e154.x, 0f), 0f);
                                let _e160 = textureSampleLevel(XC, Z9_, vec2<f32>(_e154.y, 0f), 0f);
                                let _e163 = textureSampleLevel(XC, Z9_, vec2<f32>(_e154.z, 0f), 0f);
                                let _e166 = textureSampleLevel(XC, Z9_, vec2<f32>(_e154.w, 0f), 0f);
                                let _e172 = (_e148 * 5.0959306f);
                                phi_1946_ = (_e137 + (dot(vec4<f32>(_e157.x, _e160.x, _e163.x, _e166.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e172) * (_e172 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e145));
                            }
                            let _e181 = phi_1946_;
                            phi_1947_ = (_e181 * sign(_e120.x));
                            break;
                        } else {
                            phi_1947_ = _e120.x;
                            break;
                        }
                    }
                }
                let _e186 = phi_1947_;
                phi_1949_ = _e186;
                break;
            }
        }
    }
    let _e208 = phi_1949_;
    let _e209 = l4_1;
    let _e212 = a3_1[1u];
    let _e214 = a3_1[0u];
    let _e215 = vec2<u32>(floor(_e209));
    let _e242 = (_e214 + (((((_e215.y >> bitcast<u32>(5u)) * (_e212 << bitcast<u32>(5u))) + ((_e215.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e215.x & 28u) << bitcast<u32>(5u)) + ((_e215.y & 28u) << bitcast<u32>(2i)))) + (((_e215.y & 3u) << bitcast<u32>(2i)) + (_e215.x & 3u))));
    phi_1951_ = 1f;
    if Vg {
        let _e243 = L0_1;
        let _e246 = min(_e243.xy, _e243.zw);
        phi_1951_ = min(min(_e246.x, _e246.y), 1f);
    }
    let _e252 = phi_1951_;
    phi_943_ = Ug;
    if Ug {
        let _e254 = U1_1[0u];
        phi_943_ = (_e254 != 0f);
    }
    let _e257 = phi_943_;
    phi_1952_ = _e252;
    if _e257 {
        let _e258 = gl_FragCoord_1;
        let _e262 = textureLoad(h0_, vec2<i32>(floor(_e258.xy)), 0i);
        phi_1952_ = min(_e262.x, _e252);
    }
    let _e266 = phi_1952_;
    let _e268 = clamp(_e208, 0f, max(_e266, 0f));
    let _e270 = local;
    if _e270 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e119.w, _e268) >= 1f) {
                    phi_1998_ = _e119.w;
                    break;
                }
                let _e367 = m.a2_;
                let _e369 = atomicMax((&P0_.c2_[_e242]), (_e367 | u32(((abs(_e268) * 1024f) + 0.5f))));
                if (_e369 < _e367) {
                    phi_1996_ = _e268;
                } else {
                    let _e373 = (f32((_e369 & 524287u)) * 0.0009765625f);
                    phi_1996_ = ((max(_e373, _e268) - _e373) / max((1f - (_e373 * _e119.w)), 0.000062f));
                }
                let _e381 = phi_1996_;
                phi_1998_ = (_e119.w * _e381);
                break;
            }
        }
        let _e384 = phi_1998_;
        phi_2003_ = vec4<f32>(_e119.x, _e119.y, _e119.z, _e384);
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e276 = u32(((abs(_e268) * 1024f) + 0.5f));
                let _e279 = atomicLoad((&P0_.c2_[_e242]));
                let _e281 = (min(_e119.w, _e268) >= 1f);
                phi_1598_ = _e281;
                if _e281 {
                    let _e283 = m.a2_;
                    let _e284 = (_e279 < _e283);
                    phi_1596_ = _e284;
                    if !(_e284) {
                        phi_1596_ = (_e279 >= (_e283 | 262144u));
                    }
                    let _e289 = phi_1596_;
                    phi_1598_ = _e289;
                }
                let _e291 = phi_1598_;
                if _e291 {
                    phi_1988_ = _e119.w;
                    break;
                }
                let _e293 = m.a2_;
                phi_1978_ = 0f;
                phi_1975_ = _e276;
                phi_1972_ = _e268;
                if (_e279 < _e293) {
                    let _e296 = (_e293 | (262144u + _e276));
                    let _e297 = atomicMax((&P0_.c2_[_e242]), _e296);
                    if (_e297 <= _e293) {
                        phi_1981_ = _e268;
                        phi_1976_ = _e276;
                        phi_1973_ = 0f;
                    } else {
                        phi_1982_ = 0f;
                        phi_1977_ = _e276;
                        phi_1974_ = _e268;
                        if (_e297 < _e296) {
                            let _e301 = ((_e297 & 524287u) - 262144u);
                            let _e303 = (f32(_e301) * 0.0009765625f);
                            phi_1982_ = ((_e268 - _e303) / max((1f - (_e303 * _e119.w)), 0.000062f));
                            phi_1977_ = _e301;
                            phi_1974_ = _e303;
                        }
                        let _e310 = phi_1982_;
                        let _e312 = phi_1977_;
                        let _e314 = phi_1974_;
                        phi_1981_ = _e310;
                        phi_1976_ = _e312;
                        phi_1973_ = _e314;
                    }
                    let _e316 = phi_1981_;
                    let _e318 = phi_1976_;
                    let _e320 = phi_1973_;
                    phi_1978_ = _e316;
                    phi_1975_ = _e318;
                    phi_1972_ = _e320;
                }
                let _e322 = phi_1978_;
                let _e324 = phi_1975_;
                let _e326 = phi_1972_;
                phi_1986_ = _e322;
                if (_e326 > 0f) {
                    let _e328 = atomicAdd((&P0_.c2_[_e242]), _e324);
                    let _e333 = (f32(bitcast<i32>(((_e328 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e335 = clamp(_e333, 0f, 1f);
                    phi_1986_ = (_e322 + ((1f - (_e322 * _e119.w)) * ((clamp((_e333 + _e326), 0f, 1f) - _e335) / max((1f - (_e335 * _e119.w)), 0.000062f))));
                }
                let _e347 = phi_1986_;
                phi_1988_ = (_e119.w * _e347);
                break;
            }
        }
        let _e350 = phi_1988_;
        phi_2003_ = vec4<f32>(_e119.x, _e119.y, _e119.z, _e350);
    }
    let _e391 = phi_2003_;
    phi_2005_ = f32();
    if bh {
        let _e392 = gl_FragCoord_1;
        let _e394 = m.y3_;
        let _e396 = m.z3_;
        if bh {
            phi_2001_ = ((fract((52.982918f * fract(((0.06711056f * _e392.x) + (0.00583715f * _e392.y))))) * _e394) + _e396);
        } else {
            phi_2001_ = 0f;
        }
        let _e408 = phi_2001_;
        phi_2005_ = _e408;
    }
    let _e410 = phi_2005_;
    let _e413 = (_e391.xyz * _e391.w);
    let _e419 = vec4<f32>(_e413.x, _e391.y, _e391.z, _e391.w);
    let _e425 = vec4<f32>(_e419.x, _e413.y, _e419.z, _e419.w);
    let _e431 = vec4<f32>(_e425.x, _e425.y, _e413.z, _e425.w);
    phi_2032_ = _e431;
    if bh {
        let _e434 = (_e431.xyz + vec3(_e410));
        let _e440 = vec4<f32>(_e434.x, _e431.y, _e431.z, _e431.w);
        let _e446 = vec4<f32>(_e440.x, _e434.y, _e440.z, _e440.w);
        phi_2032_ = vec4<f32>(_e446.x, _e446.y, _e434.z, _e446.w);
    }
    let _e454 = phi_2032_;
    C1_ = _e454;
    return;
}

@fragment
fn main(@location(0) f1_: vec4<f32>, @location(2) O: vec4<f32>, @location(8) l4_: vec2<f32>, @location(7) @interpolate(flat, either) a3_: vec2<u32>, @location(5) L0_: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) A0_: f32, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    f1_1 = f1_;
    O_1 = O;
    l4_1 = l4_;
    a3_1 = a3_;
    L0_1 = L0_;
    U1_1 = U1_;
    gl_FragCoord_1 = gl_FragCoord;
    A0_1 = A0_;
    e2_1 = e2_;
    main_1();
    let _e19 = C1_;
    return _e19;
}
