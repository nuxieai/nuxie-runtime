struct TB {
    uc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Ob: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    ih: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    mh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    fh: u32,
    Nb: u32,
    ac: f32,
    bc: f32,
}

struct Ae {
    g2_: array<u32>,
}

struct Ae_1 {
    g2_: array<atomic<u32>>,
}

@id(7) override Ph: bool = true;
@id(2) override Kh: bool = true;
@id(8) override Qh: bool = true;
@id(1) override Jh: bool = true;
@id(0) override Ih: bool = true;

@group(0) @binding(0)
var<uniform> j: TB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(6)
var<storage, read_write> S0_: Ae_1;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> j1_1: f32;
var<private> v4_1: vec2<f32>;
var<private> k3_1: vec2<u32>;
var<private> O0_1: vec4<f32>;
var<private> Y1_1: vec2<f32>;
@group(2) @binding(1)
var i0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> F1_: vec4<f32>;
@group(3) @binding(9)
var da: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> D0_1: f32;

fn main_1() {
    var phi_1286_: f32;
    var phi_1287_: f32;
    var phi_1301_: vec4<f32>;
    var phi_1300_: vec4<f32>;
    var phi_842_: bool;
    var phi_1288_: f32;
    var phi_1297_: vec4<f32>;
    var phi_1303_: vec4<f32>;
    var phi_1305_: f32;
    var phi_668_: bool;
    var phi_1306_: f32;
    var phi_1052_: bool;
    var phi_1054_: bool;
    var phi_1327_: f32;
    var phi_1322_: u32;
    var phi_1319_: f32;
    var phi_1326_: f32;
    var phi_1321_: u32;
    var phi_1318_: f32;
    var phi_1323_: f32;
    var phi_1320_: u32;
    var phi_1317_: f32;
    var phi_1328_: f32;
    var phi_1329_: f32;
    var phi_1330_: f32;
    var phi_1340_: f32;
    var phi_1360_: vec3<f32>;

    let _e53 = g1_1;
    let _e55 = C2_1;
    let _e56 = X1_1;
    let _e58 = (Kh && (u32(_e53) != 0u));
    if (_e56.w >= 0f) {
        phi_1300_ = _e56;
    } else {
        let _e61 = -(_e56.w);
        let _e66 = j.ac;
        let _e69 = j.bc;
        if (_e56.z > 0f) {
            phi_1286_ = _e56.x;
        } else {
            phi_1286_ = length(_e56.xy);
        }
        let _e77 = phi_1286_;
        let _e78 = clamp(_e77, 0f, 1f);
        let _e79 = abs(_e56.z);
        if (_e79 > 1f) {
            phi_1287_ = ((0.9980469f * _e78) + 0.0009765625f);
        } else {
            phi_1287_ = ((0.001953125f * _e78) + _e79);
        }
        let _e86 = phi_1287_;
        let _e88 = textureSampleLevel(ED, N9_, vec2<f32>(_e86, ((floor(_e61) * _e66) + _e69)), 0f);
        phi_1301_ = _e88;
        if !(_e58) {
            let _e92 = (_e88.xyz * _e88.w);
            phi_1301_ = vec4<f32>(_e92.x, _e92.y, _e92.z, (_e88.w * (fract(_e61) * 1.0039216f)));
        }
        let _e99 = phi_1301_;
        phi_1300_ = _e99;
    }
    let _e101 = phi_1300_;
    phi_842_ = Qh;
    if Qh {
        phi_842_ = (_e55.z > 0f);
    }
    let _e105 = phi_842_;
    phi_1303_ = _e101;
    if _e105 {
        let _e109 = textureSampleLevel(HC, W5_, _e55.xy, (_e55.z - 1f));
        phi_1297_ = _e109;
        if _e58 {
            if (_e109.w != 0f) {
                phi_1288_ = (1f / _e109.w);
            } else {
                phi_1288_ = 0f;
            }
            let _e115 = phi_1288_;
            let _e116 = (_e109.xyz * _e115);
            phi_1297_ = vec4<f32>(_e116.x, _e116.y, _e116.z, _e109.w);
        }
        let _e122 = phi_1297_;
        phi_1303_ = (_e101 * _e122);
    }
    let _e125 = phi_1303_;
    let _e126 = j1_1;
    let _e127 = v4_1;
    let _e130 = k3_1[1u];
    let _e132 = k3_1[0u];
    let _e133 = vec2<u32>(floor(_e127));
    phi_1305_ = 1f;
    if Jh {
        let _e161 = O0_1;
        let _e164 = min(_e161.xy, _e161.zw);
        phi_1305_ = min(min(_e164.x, _e164.y), 1f);
    }
    let _e170 = phi_1305_;
    phi_668_ = Ih;
    if Ih {
        let _e172 = Y1_1[0u];
        phi_668_ = (_e172 != 0f);
    }
    let _e175 = phi_668_;
    phi_1306_ = _e170;
    if _e175 {
        let _e176 = gl_FragCoord_1;
        let _e180 = textureLoad(i0_, vec2<i32>(floor(_e176.xy)), 0i);
        phi_1306_ = min(_e180.x, _e170);
    }
    let _e184 = phi_1306_;
    let _e186 = clamp(_e126, 0f, max(_e184, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e192 = u32(((abs(_e186) * 1024f) + 0.5f));
            let _e195 = atomicLoad((&S0_.g2_[(_e132 + (((((_e133.y >> bitcast<u32>(5u)) * (_e130 << bitcast<u32>(5u))) + ((_e133.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e133.x & 28u) << bitcast<u32>(5u)) + ((_e133.y & 28u) << bitcast<u32>(2i)))) + (((_e133.y & 3u) << bitcast<u32>(2i)) + (_e133.x & 3u))))]));
            let _e197 = (min(_e125.w, _e186) >= 1f);
            phi_1054_ = _e197;
            if _e197 {
                let _e199 = j.f2_;
                let _e200 = (_e195 < _e199);
                phi_1052_ = _e200;
                if !(_e200) {
                    phi_1052_ = (_e195 >= (_e199 | 262144u));
                }
                let _e205 = phi_1052_;
                phi_1054_ = _e205;
            }
            let _e207 = phi_1054_;
            if _e207 {
                phi_1329_ = 1f;
                break;
            }
            let _e209 = j.f2_;
            phi_1323_ = 0f;
            phi_1320_ = _e192;
            phi_1317_ = _e186;
            if (_e195 < _e209) {
                let _e212 = (_e209 | (262144u + _e192));
                let _e213 = atomicMax((&S0_.g2_[(_e132 + (((((_e133.y >> bitcast<u32>(5u)) * (_e130 << bitcast<u32>(5u))) + ((_e133.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e133.x & 28u) << bitcast<u32>(5u)) + ((_e133.y & 28u) << bitcast<u32>(2i)))) + (((_e133.y & 3u) << bitcast<u32>(2i)) + (_e133.x & 3u))))]), _e212);
                if (_e213 <= _e209) {
                    phi_1326_ = min(_e186, 1f);
                    phi_1321_ = _e192;
                    phi_1318_ = 0f;
                } else {
                    phi_1327_ = 0f;
                    phi_1322_ = _e192;
                    phi_1319_ = _e186;
                    if (_e213 < _e212) {
                        let _e218 = ((_e213 & 524287u) - 262144u);
                        let _e220 = (f32(_e218) * 0.0009765625f);
                        phi_1327_ = ((min(_e186, 1f) - _e220) / max((1f - (_e220 * _e125.w)), 0.000062f));
                        phi_1322_ = _e218;
                        phi_1319_ = _e220;
                    }
                    let _e228 = phi_1327_;
                    let _e230 = phi_1322_;
                    let _e232 = phi_1319_;
                    phi_1326_ = _e228;
                    phi_1321_ = _e230;
                    phi_1318_ = _e232;
                }
                let _e234 = phi_1326_;
                let _e236 = phi_1321_;
                let _e238 = phi_1318_;
                phi_1323_ = _e234;
                phi_1320_ = _e236;
                phi_1317_ = _e238;
            }
            let _e240 = phi_1323_;
            let _e242 = phi_1320_;
            let _e244 = phi_1317_;
            phi_1328_ = _e240;
            if (_e244 > 0f) {
                let _e246 = atomicAdd((&S0_.g2_[(_e132 + (((((_e133.y >> bitcast<u32>(5u)) * (_e130 << bitcast<u32>(5u))) + ((_e133.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e133.x & 28u) << bitcast<u32>(5u)) + ((_e133.y & 28u) << bitcast<u32>(2i)))) + (((_e133.y & 3u) << bitcast<u32>(2i)) + (_e133.x & 3u))))]), _e242);
                let _e251 = (f32(bitcast<i32>(((_e246 & 524287u) - 262144u))) * 0.0009765625f);
                let _e253 = clamp(_e251, 0f, 1f);
                phi_1328_ = (_e240 + ((1f - (_e240 * _e125.w)) * ((clamp((_e251 + _e244), 0f, 1f) - _e253) / max((1f - (_e253 * _e125.w)), 0.000062f))));
            }
            let _e265 = phi_1328_;
            phi_1329_ = _e265;
            break;
        }
    }
    let _e267 = phi_1329_;
    phi_1340_ = f32();
    if Ph {
        let _e268 = gl_FragCoord_1;
        let _e270 = j.F3_;
        let _e272 = j.G3_;
        if Ph {
            phi_1330_ = ((fract((52.982918f * fract(((0.06711056f * _e268.x) + (0.00583715f * _e268.y))))) * _e270) + _e272);
        } else {
            phi_1330_ = 0f;
        }
        let _e284 = phi_1330_;
        phi_1340_ = _e284;
    }
    let _e286 = phi_1340_;
    let _e287 = (_e125 * _e267);
    let _e288 = _e287.xyz;
    if (Ph && (_e287.w != 0f)) {
        phi_1360_ = (vec3(_e286) + _e288);
    } else {
        phi_1360_ = _e288;
    }
    let _e295 = phi_1360_;
    let _e301 = vec4<f32>(_e295.x, _e287.y, _e287.z, _e287.w);
    let _e307 = vec4<f32>(_e301.x, _e295.y, _e301.z, _e301.w);
    F1_ = vec4<f32>(_e307.x, _e307.y, _e295.z, _e307.w);
    return;
}

@fragment
fn main(@location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @location(1) @interpolate(flat, either) j1_: f32, @location(8) v4_: vec2<f32>, @location(7) @interpolate(flat, either) k3_: vec2<u32>, @location(5) O0_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) D0_: f32) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    j1_1 = j1_;
    v4_1 = v4_;
    k3_1 = k3_;
    O0_1 = O0_;
    Y1_1 = Y1_;
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    main_1();
    let _e21 = F1_;
    return _e21;
}
