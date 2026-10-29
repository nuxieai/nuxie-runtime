struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct Kf {
    x2_: array<u32>,
}

struct Kf_1 {
    x2_: array<atomic<u32>>,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;
@id(3) override mj: bool = true;
@id(1) override kj: bool = true;
@id(0) override jj: bool = true;

@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var Q8_: sampler;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;
@group(0) @binding(5)
var<storage, read_write> X0_: Kf_1;
var<private> N0_1: vec3<f32>;
var<private> R0_1: f32;
var<private> M0_1: vec4<f32>;
var<private> S_1: vec4<f32>;
var<private> P4_1: vec2<f32>;
var<private> B3_1: vec2<u32>;
var<private> V0_1: vec4<f32>;
var<private> l2_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
var<private> I0_1: f32;

fn main_1() {
    var phi_2261_: f32;
    var phi_2262_: f32;
    var phi_2263_: vec2<f32>;
    var phi_2280_: vec4<f32>;
    var phi_2279_: vec4<f32>;
    var phi_1327_: bool;
    var phi_1342_: bool;
    var phi_2264_: f32;
    var phi_2275_: vec4<f32>;
    var phi_2282_: vec4<f32>;
    var phi_2283_: vec4<f32>;
    var phi_1628_: bool;
    var phi_2284_: f32;
    var phi_2311_: f32;
    var phi_2312_: f32;
    var phi_1565_: bool;
    var phi_2313_: f32;
    var phi_2314_: f32;
    var phi_2316_: f32;
    var phi_1114_: bool;
    var phi_2317_: f32;
    var local: bool;
    var phi_1983_: bool;
    var phi_1985_: bool;
    var phi_2345_: f32;
    var phi_2340_: u32;
    var phi_2337_: f32;
    var phi_2344_: f32;
    var phi_2339_: u32;
    var phi_2336_: f32;
    var phi_2341_: f32;
    var phi_2338_: u32;
    var phi_2335_: f32;
    var phi_2346_: f32;
    var phi_2347_: f32;
    var phi_2348_: f32;
    var phi_2349_: f32;
    var phi_2352_: f32;
    var phi_2350_: f32;
    var phi_2366_: f32;
    var phi_2399_: vec3<f32>;

    let _e76 = R0_1;
    let _e78 = N0_1;
    let _e79 = M0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e82 = (lj && (u32(_e76) != 0u));
            if (_e79.w >= 0f) {
                phi_2279_ = _e79;
            } else {
                let _e86 = j.O8_;
                let _e88 = j.P8_;
                let _e91 = abs(_e79.z);
                let _e92 = floor(_e91);
                let _e95 = (((_e91 - _e92) * 8f) + 0.0009765625f);
                let _e97 = ((_e92 * _e86) + _e88);
                let _e98 = floor(_e95);
                let _e99 = _e79.xy;
                switch bitcast<i32>(0u) {
                    default: {
                        if (sj && (_e98 == 4f)) {
                            let _e113 = normalize(_e99);
                            let _e116 = acos(clamp(_e113.x, -1f, 1f));
                            if (_e113.y >= 0f) {
                                phi_2262_ = _e116;
                            } else {
                                phi_2262_ = -(_e116);
                            }
                            let _e121 = phi_2262_;
                            phi_2263_ = vec2<f32>((_e121 * 0.15915494f), _e97);
                            break;
                        } else {
                            if (_e98 == 2f) {
                                phi_2261_ = _e79.x;
                            } else {
                                phi_2261_ = length(_e99);
                            }
                            let _e107 = phi_2261_;
                            phi_2263_ = vec2<f32>(((clamp(_e107, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e79.z < 0f))) + _e95), _e97);
                            break;
                        }
                    }
                }
                let _e125 = phi_2263_;
                let _e126 = textureSampleLevel(YC, Q8_, _e125, 0f);
                phi_2280_ = _e126;
                if !(_e82) {
                    let _e132 = (_e126.xyz * _e126.w);
                    phi_2280_ = vec4<f32>(_e132.x, _e132.y, _e132.z, (_e126.w * (ceil(_e79.w) * -0.003921569f)));
                }
                let _e139 = phi_2280_;
                phi_2279_ = _e139;
            }
            let _e141 = phi_2279_;
            phi_1327_ = nj;
            if nj {
                phi_1327_ = (_e78.z < 0f);
            }
            let _e145 = phi_1327_;
            if _e145 {
                let _e147 = textureSampleLevel(RB, v4_, _e78.xy, 0f);
                phi_2283_ = _e147;
                break;
            }
            phi_1342_ = nj;
            if nj {
                phi_1342_ = (_e78.z > 0f);
            }
            let _e151 = phi_1342_;
            phi_2282_ = _e141;
            if _e151 {
                let _e155 = textureSampleLevel(RB, v4_, _e78.xy, (_e78.z - 1f));
                phi_2275_ = _e155;
                if _e82 {
                    if (_e155.w != 0f) {
                        phi_2264_ = (1f / _e155.w);
                    } else {
                        phi_2264_ = 0f;
                    }
                    let _e161 = phi_2264_;
                    let _e162 = (_e155.xyz * _e161);
                    phi_2275_ = vec4<f32>(_e162.x, _e162.y, _e162.z, _e155.w);
                }
                let _e168 = phi_2275_;
                phi_2282_ = (_e141 * _e168);
            }
            let _e171 = phi_2282_;
            phi_2283_ = _e171;
            break;
        }
    }
    let _e173 = phi_2283_;
    let _e174 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e177 = (_e174.y >= 0f);
            local = _e177;
            if _e177 {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1565_ = mj;
                        if mj {
                            phi_1565_ = (_e174.x < -1.5f);
                        }
                        let _e245 = phi_1565_;
                        if _e245 {
                            let _e251 = textureSampleLevel(ZC, fb, vec2<f32>((3f + _e174.x), 0f), 0f);
                            let _e256 = textureSampleLevel(ZC, fb, vec2<f32>((1f - _e174.y), 0f), 0f);
                            phi_2313_ = ((1f - _e251.x) - _e256.x);
                            break;
                        } else {
                            phi_2313_ = min(_e174.x, _e174.y);
                            break;
                        }
                    }
                }
                let _e260 = phi_2313_;
                phi_2314_ = _e260;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_1628_ = mj;
                        if mj {
                            phi_1628_ = (_e174.y < -1.5f);
                        }
                        let _e181 = phi_1628_;
                        if _e181 {
                            let _e185 = max(_e174.w, 0f);
                            if (_e174.z >= 0f) {
                                let _e188 = textureSampleLevel(ZC, fb, vec2<f32>(_e185, 0f), 0f);
                                phi_2284_ = _e188.x;
                            } else {
                                phi_2284_ = 0f;
                            }
                            let _e191 = phi_2284_;
                            phi_2311_ = _e191;
                            if (abs(_e174.z) < 1000f) {
                                let _e197 = (-2f - _e174.y);
                                let _e199 = ((_e197 - _e185) * 0.5984134f);
                                let _e202 = (vec4(_e185) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e199));
                                let _e208 = ((_e202 * -(_e174.z)) + vec4(((_e197 * _e174.z) + (abs(_e174.x) - 0.25f))));
                                let _e211 = textureSampleLevel(ZC, fb, vec2<f32>(_e208.x, 0f), 0f);
                                let _e214 = textureSampleLevel(ZC, fb, vec2<f32>(_e208.y, 0f), 0f);
                                let _e217 = textureSampleLevel(ZC, fb, vec2<f32>(_e208.z, 0f), 0f);
                                let _e220 = textureSampleLevel(ZC, fb, vec2<f32>(_e208.w, 0f), 0f);
                                let _e226 = (_e202 * 5.0959306f);
                                phi_2311_ = (_e191 + (dot(vec4<f32>(_e211.x, _e214.x, _e217.x, _e220.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e226) * (_e226 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e199));
                            }
                            let _e235 = phi_2311_;
                            phi_2312_ = (_e235 * sign(_e174.x));
                            break;
                        } else {
                            phi_2312_ = _e174.x;
                            break;
                        }
                    }
                }
                let _e240 = phi_2312_;
                phi_2314_ = _e240;
                break;
            }
        }
    }
    let _e262 = phi_2314_;
    let _e263 = P4_1;
    let _e266 = B3_1[1u];
    let _e268 = B3_1[0u];
    let _e269 = vec2<u32>(floor(_e263));
    let _e296 = (_e268 + (((((_e269.y >> bitcast<u32>(5u)) * (_e266 << bitcast<u32>(5u))) + ((_e269.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e269.x & 28u) << bitcast<u32>(5u)) + ((_e269.y & 28u) << bitcast<u32>(2i)))) + (((_e269.y & 3u) << bitcast<u32>(2i)) + (_e269.x & 3u))));
    phi_2316_ = 1f;
    if kj {
        let _e297 = V0_1;
        let _e300 = min(_e297.xy, _e297.zw);
        phi_2316_ = min(min(_e300.x, _e300.y), 1f);
    }
    let _e306 = phi_2316_;
    phi_1114_ = jj;
    if jj {
        let _e308 = l2_1[0u];
        phi_1114_ = (_e308 != 0f);
    }
    let _e311 = phi_1114_;
    phi_2317_ = _e306;
    if _e311 {
        let _e312 = gl_FragCoord_1;
        let _e316 = textureLoad(m0_, vec2<i32>(floor(_e312.xy)), 0i);
        phi_2317_ = min(_e316.x, _e306);
    }
    let _e320 = phi_2317_;
    let _e322 = clamp(_e262, 0f, max(_e320, 0f));
    let _e324 = local;
    if _e324 {
        switch bitcast<i32>(0u) {
            default: {
                if (min(_e173.w, _e322) >= 1f) {
                    phi_2349_ = 1f;
                    break;
                }
                let _e415 = j.w2_;
                let _e417 = atomicMax((&X0_.x2_[_e296]), (_e415 | u32(((abs(_e322) * 1024f) + 0.5f))));
                if (_e417 < _e415) {
                    phi_2348_ = _e322;
                } else {
                    let _e421 = (f32((_e417 & 524287u)) * 0.0009765625f);
                    phi_2348_ = ((max(_e421, _e322) - _e421) / max((1f - (_e421 * _e173.w)), 0.000062f));
                }
                let _e429 = phi_2348_;
                phi_2349_ = _e429;
                break;
            }
        }
        let _e431 = phi_2349_;
        phi_2352_ = _e431;
    } else {
        switch bitcast<i32>(0u) {
            default: {
                let _e330 = u32(((abs(_e322) * 1024f) + 0.5f));
                let _e333 = atomicLoad((&X0_.x2_[_e296]));
                let _e335 = (min(_e173.w, _e322) >= 1f);
                phi_1985_ = _e335;
                if _e335 {
                    let _e337 = j.w2_;
                    let _e338 = (_e333 < _e337);
                    phi_1983_ = _e338;
                    if !(_e338) {
                        phi_1983_ = (_e333 >= (_e337 | 262144u));
                    }
                    let _e343 = phi_1983_;
                    phi_1985_ = _e343;
                }
                let _e345 = phi_1985_;
                if _e345 {
                    phi_2347_ = 1f;
                    break;
                }
                let _e347 = j.w2_;
                phi_2341_ = 0f;
                phi_2338_ = _e330;
                phi_2335_ = _e322;
                if (_e333 < _e347) {
                    let _e350 = (_e347 | (262144u + _e330));
                    let _e351 = atomicMax((&X0_.x2_[_e296]), _e350);
                    if (_e351 <= _e347) {
                        phi_2344_ = _e322;
                        phi_2339_ = _e330;
                        phi_2336_ = 0f;
                    } else {
                        phi_2345_ = 0f;
                        phi_2340_ = _e330;
                        phi_2337_ = _e322;
                        if (_e351 < _e350) {
                            let _e355 = ((_e351 & 524287u) - 262144u);
                            let _e357 = (f32(_e355) * 0.0009765625f);
                            phi_2345_ = ((_e322 - _e357) / max((1f - (_e357 * _e173.w)), 0.000062f));
                            phi_2340_ = _e355;
                            phi_2337_ = _e357;
                        }
                        let _e364 = phi_2345_;
                        let _e366 = phi_2340_;
                        let _e368 = phi_2337_;
                        phi_2344_ = _e364;
                        phi_2339_ = _e366;
                        phi_2336_ = _e368;
                    }
                    let _e370 = phi_2344_;
                    let _e372 = phi_2339_;
                    let _e374 = phi_2336_;
                    phi_2341_ = _e370;
                    phi_2338_ = _e372;
                    phi_2335_ = _e374;
                }
                let _e376 = phi_2341_;
                let _e378 = phi_2338_;
                let _e380 = phi_2335_;
                phi_2346_ = _e376;
                if (_e380 > 0f) {
                    let _e382 = atomicAdd((&X0_.x2_[_e296]), _e378);
                    let _e387 = (f32(bitcast<i32>(((_e382 & 524287u) - 262144u))) * 0.0009765625f);
                    let _e389 = clamp(_e387, 0f, 1f);
                    phi_2346_ = (_e376 + ((1f - (_e376 * _e173.w)) * ((clamp((_e387 + _e380), 0f, 1f) - _e389) / max((1f - (_e389 * _e173.w)), 0.000062f))));
                }
                let _e401 = phi_2346_;
                phi_2347_ = _e401;
                break;
            }
        }
        let _e403 = phi_2347_;
        phi_2352_ = _e403;
    }
    let _e433 = phi_2352_;
    phi_2366_ = f32();
    if rj {
        let _e434 = gl_FragCoord_1;
        let _e436 = j.I3_;
        let _e438 = j.J3_;
        if rj {
            phi_2350_ = ((fract((52.982918f * fract(((0.06711056f * _e434.x) + (0.00583715f * _e434.y))))) * _e436) + _e438);
        } else {
            phi_2350_ = 0f;
        }
        let _e450 = phi_2350_;
        phi_2366_ = _e450;
    }
    let _e452 = phi_2366_;
    let _e453 = (_e173 * _e433);
    let _e454 = _e453.xyz;
    if (rj && (_e453.w != 0f)) {
        phi_2399_ = (vec3(_e452) + _e454);
    } else {
        phi_2399_ = _e454;
    }
    let _e461 = phi_2399_;
    let _e467 = vec4<f32>(_e461.x, _e453.y, _e453.z, _e453.w);
    let _e473 = vec4<f32>(_e467.x, _e461.y, _e467.z, _e467.w);
    Q1_ = vec4<f32>(_e473.x, _e473.y, _e461.z, _e473.w);
    return;
}

@fragment
fn main(@location(9) N0_: vec3<f32>, @location(6) @interpolate(flat, either) R0_: f32, @location(0) M0_: vec4<f32>, @location(2) S: vec4<f32>, @location(8) P4_: vec2<f32>, @location(7) @interpolate(flat, either) B3_: vec2<u32>, @location(5) V0_: vec4<f32>, @location(4) @interpolate(flat, either) l2_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) I0_: f32) -> @location(0) vec4<f32> {
    N0_1 = N0_;
    R0_1 = R0_;
    M0_1 = M0_;
    S_1 = S;
    P4_1 = P4_;
    B3_1 = B3_;
    V0_1 = V0_;
    l2_1 = l2_;
    gl_FragCoord_1 = gl_FragCoord;
    I0_1 = I0_;
    main_1();
    let _e21 = Q1_;
    return _e21;
}
