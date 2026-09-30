struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Re {
    j2_: array<u32>,
}

struct Re_1 {
    j2_: array<atomic<u32>>,
}

@id(7) override ii: bool = true;
@id(2) override di: bool = true;
@id(8) override ji: bool = true;
@id(1) override ci: bool = true;
@id(0) override bi: bool = true;

@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var f6_: sampler;
@group(0) @binding(6)
var<storage, read_write> V0_: Re_1;
var<private> F1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> m1_1: f32;
var<private> F4_1: vec2<f32>;
var<private> q3_1: vec2<u32>;
var<private> R0_1: vec4<f32>;
var<private> l1_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> J1_: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> F0_1: f32;

fn main_1() {
    var phi_1285_: f32;
    var phi_1286_: f32;
    var phi_1300_: vec4<f32>;
    var phi_1299_: vec4<f32>;
    var phi_841_: bool;
    var phi_1287_: f32;
    var phi_1296_: vec4<f32>;
    var phi_1302_: vec4<f32>;
    var phi_1304_: f32;
    var phi_668_: bool;
    var phi_1305_: f32;
    var phi_1051_: bool;
    var phi_1053_: bool;
    var phi_1326_: f32;
    var phi_1321_: u32;
    var phi_1318_: f32;
    var phi_1325_: f32;
    var phi_1320_: u32;
    var phi_1317_: f32;
    var phi_1322_: f32;
    var phi_1319_: u32;
    var phi_1316_: f32;
    var phi_1327_: f32;
    var phi_1328_: f32;
    var phi_1329_: f32;
    var phi_1339_: f32;
    var phi_1359_: vec3<f32>;

    let _e53 = Q0_1;
    let _e55 = F1_1;
    let _e56 = a1_1;
    let _e58 = (di && (u32(_e53) != 0u));
    if (_e56.w >= 0f) {
        phi_1299_ = _e56;
    } else {
        let _e61 = -(_e56.w);
        let _e66 = j.wc;
        let _e69 = j.xc;
        if (_e56.z > 0f) {
            phi_1285_ = _e56.x;
        } else {
            phi_1285_ = length(_e56.xy);
        }
        let _e77 = phi_1285_;
        let _e78 = clamp(_e77, 0f, 1f);
        let _e79 = abs(_e56.z);
        if (_e79 > 1f) {
            phi_1286_ = ((0.9980469f * _e78) + 0.0009765625f);
        } else {
            phi_1286_ = ((0.001953125f * _e78) + _e79);
        }
        let _e86 = phi_1286_;
        let _e88 = textureSampleLevel(FD, ha, vec2<f32>(_e86, ((floor(_e61) * _e66) + _e69)), 0f);
        phi_1300_ = _e88;
        if !(_e58) {
            let _e92 = (_e88.xyz * _e88.w);
            phi_1300_ = vec4<f32>(_e92.x, _e92.y, _e92.z, (_e88.w * (fract(_e61) * 1.0039216f)));
        }
        let _e99 = phi_1300_;
        phi_1299_ = _e99;
    }
    let _e101 = phi_1299_;
    phi_841_ = ji;
    if ji {
        phi_841_ = (_e55.z > 0f);
    }
    let _e105 = phi_841_;
    phi_1302_ = _e101;
    if _e105 {
        let _e109 = textureSampleLevel(IC, f6_, _e55.xy, (_e55.z - 1f));
        phi_1296_ = _e109;
        if _e58 {
            if (_e109.w != 0f) {
                phi_1287_ = (1f / _e109.w);
            } else {
                phi_1287_ = 0f;
            }
            let _e115 = phi_1287_;
            let _e116 = (_e109.xyz * _e115);
            phi_1296_ = vec4<f32>(_e116.x, _e116.y, _e116.z, _e109.w);
        }
        let _e122 = phi_1296_;
        phi_1302_ = (_e101 * _e122);
    }
    let _e125 = phi_1302_;
    let _e126 = m1_1;
    let _e127 = F4_1;
    let _e130 = q3_1[1u];
    let _e132 = q3_1[0u];
    let _e133 = vec2<u32>(floor(_e127));
    phi_1304_ = 1f;
    if ci {
        let _e161 = R0_1;
        let _e164 = min(_e161.xy, _e161.zw);
        phi_1304_ = min(min(_e164.x, _e164.y), 1f);
    }
    let _e170 = phi_1304_;
    phi_668_ = bi;
    if bi {
        let _e172 = l1_1[0u];
        phi_668_ = (_e172 != 0f);
    }
    let _e175 = phi_668_;
    phi_1305_ = _e170;
    if _e175 {
        let _e176 = gl_FragCoord_1;
        let _e180 = textureLoad(m0_, vec2<i32>(floor(_e176.xy)), 0i);
        phi_1305_ = min(_e180.x, _e170);
    }
    let _e184 = phi_1305_;
    let _e186 = clamp(_e126, 0f, max(_e184, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e192 = u32(((abs(_e186) * 1024f) + 0.5f));
            let _e195 = atomicLoad((&V0_.j2_[(_e132 + (((((_e133.y >> bitcast<u32>(5u)) * (_e130 << bitcast<u32>(5u))) + ((_e133.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e133.x & 28u) << bitcast<u32>(5u)) + ((_e133.y & 28u) << bitcast<u32>(2i)))) + (((_e133.y & 3u) << bitcast<u32>(2i)) + (_e133.x & 3u))))]));
            let _e197 = (min(_e125.w, _e186) >= 1f);
            phi_1053_ = _e197;
            if _e197 {
                let _e199 = j.i2_;
                let _e200 = (_e195 < _e199);
                phi_1051_ = _e200;
                if !(_e200) {
                    phi_1051_ = (_e195 >= (_e199 | 262144u));
                }
                let _e205 = phi_1051_;
                phi_1053_ = _e205;
            }
            let _e207 = phi_1053_;
            if _e207 {
                phi_1328_ = 1f;
                break;
            }
            let _e209 = j.i2_;
            phi_1322_ = 0f;
            phi_1319_ = _e192;
            phi_1316_ = _e186;
            if (_e195 < _e209) {
                let _e212 = (_e209 | (262144u + _e192));
                let _e213 = atomicMax((&V0_.j2_[(_e132 + (((((_e133.y >> bitcast<u32>(5u)) * (_e130 << bitcast<u32>(5u))) + ((_e133.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e133.x & 28u) << bitcast<u32>(5u)) + ((_e133.y & 28u) << bitcast<u32>(2i)))) + (((_e133.y & 3u) << bitcast<u32>(2i)) + (_e133.x & 3u))))]), _e212);
                if (_e213 <= _e209) {
                    phi_1325_ = min(_e186, 1f);
                    phi_1320_ = _e192;
                    phi_1317_ = 0f;
                } else {
                    phi_1326_ = 0f;
                    phi_1321_ = _e192;
                    phi_1318_ = _e186;
                    if (_e213 < _e212) {
                        let _e218 = ((_e213 & 524287u) - 262144u);
                        let _e220 = (f32(_e218) * 0.0009765625f);
                        phi_1326_ = ((min(_e186, 1f) - _e220) / max((1f - (_e220 * _e125.w)), 0.000062f));
                        phi_1321_ = _e218;
                        phi_1318_ = _e220;
                    }
                    let _e228 = phi_1326_;
                    let _e230 = phi_1321_;
                    let _e232 = phi_1318_;
                    phi_1325_ = _e228;
                    phi_1320_ = _e230;
                    phi_1317_ = _e232;
                }
                let _e234 = phi_1325_;
                let _e236 = phi_1320_;
                let _e238 = phi_1317_;
                phi_1322_ = _e234;
                phi_1319_ = _e236;
                phi_1316_ = _e238;
            }
            let _e240 = phi_1322_;
            let _e242 = phi_1319_;
            let _e244 = phi_1316_;
            phi_1327_ = _e240;
            if (_e244 > 0f) {
                let _e246 = atomicAdd((&V0_.j2_[(_e132 + (((((_e133.y >> bitcast<u32>(5u)) * (_e130 << bitcast<u32>(5u))) + ((_e133.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e133.x & 28u) << bitcast<u32>(5u)) + ((_e133.y & 28u) << bitcast<u32>(2i)))) + (((_e133.y & 3u) << bitcast<u32>(2i)) + (_e133.x & 3u))))]), _e242);
                let _e251 = (f32(bitcast<i32>(((_e246 & 524287u) - 262144u))) * 0.0009765625f);
                let _e253 = clamp(_e251, 0f, 1f);
                phi_1327_ = (_e240 + ((1f - (_e240 * _e125.w)) * ((clamp((_e251 + _e244), 0f, 1f) - _e253) / max((1f - (_e253 * _e125.w)), 0.000062f))));
            }
            let _e265 = phi_1327_;
            phi_1328_ = _e265;
            break;
        }
    }
    let _e267 = phi_1328_;
    phi_1339_ = f32();
    if ii {
        let _e268 = gl_FragCoord_1;
        let _e270 = j.M3_;
        let _e272 = j.N3_;
        if ii {
            phi_1329_ = ((fract((52.982918f * fract(((0.06711056f * _e268.x) + (0.00583715f * _e268.y))))) * _e270) + _e272);
        } else {
            phi_1329_ = 0f;
        }
        let _e284 = phi_1329_;
        phi_1339_ = _e284;
    }
    let _e286 = phi_1339_;
    let _e287 = (_e125 * _e267);
    let _e288 = _e287.xyz;
    if (ii && (_e287.w != 0f)) {
        phi_1359_ = (vec3(_e286) + _e288);
    } else {
        phi_1359_ = _e288;
    }
    let _e295 = phi_1359_;
    let _e301 = vec4<f32>(_e295.x, _e287.y, _e287.z, _e287.w);
    let _e307 = vec4<f32>(_e301.x, _e295.y, _e301.z, _e301.w);
    J1_ = vec4<f32>(_e307.x, _e307.y, _e295.z, _e307.w);
    return;
}

@fragment
fn main(@location(9) F1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @location(1) @interpolate(flat, either) m1_: f32, @location(8) F4_: vec2<f32>, @location(7) @interpolate(flat, either) q3_: vec2<u32>, @location(5) R0_: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) F0_: f32) -> @location(0) vec4<f32> {
    F1_1 = F1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    m1_1 = m1_;
    F4_1 = F4_;
    q3_1 = q3_;
    R0_1 = R0_;
    l1_1 = l1_;
    gl_FragCoord_1 = gl_FragCoord;
    F0_1 = F0_;
    main_1();
    let _e21 = J1_;
    return _e21;
}
