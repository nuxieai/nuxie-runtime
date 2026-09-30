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

struct m0ge {
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
var<private> gl_FragCoord_1: vec4<f32>;
var<private> F1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> m1_1: f32;
var<private> F4_1: vec2<f32>;
var<private> q3_1: vec2<u32>;
var<private> R0_1: vec4<f32>;
var<private> l1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> m0_: m0ge;
var<private> J1_: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> F0_1: f32;

fn main_1() {
    var phi_1389_: f32;
    var phi_1390_: f32;
    var phi_1404_: vec4<f32>;
    var phi_1403_: vec4<f32>;
    var phi_919_: bool;
    var phi_1391_: f32;
    var phi_1400_: vec4<f32>;
    var phi_1406_: vec4<f32>;
    var phi_1408_: f32;
    var phi_706_: bool;
    var phi_1409_: f32;
    var phi_1129_: bool;
    var phi_1131_: bool;
    var phi_1430_: f32;
    var phi_1425_: u32;
    var phi_1422_: f32;
    var phi_1429_: f32;
    var phi_1424_: u32;
    var phi_1421_: f32;
    var phi_1426_: f32;
    var phi_1423_: u32;
    var phi_1420_: f32;
    var phi_1431_: f32;
    var phi_1432_: f32;
    var phi_1433_: f32;
    var phi_1443_: f32;
    var phi_1463_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e57.xy)));
    let _e63 = j.z6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e93 = Q0_1;
    let _e95 = F1_1;
    let _e96 = a1_1;
    let _e98 = (di && (u32(_e93) != 0u));
    if (_e96.w >= 0f) {
        phi_1403_ = _e96;
    } else {
        let _e101 = -(_e96.w);
        let _e106 = j.wc;
        let _e109 = j.xc;
        if (_e96.z > 0f) {
            phi_1389_ = _e96.x;
        } else {
            phi_1389_ = length(_e96.xy);
        }
        let _e117 = phi_1389_;
        let _e118 = clamp(_e117, 0f, 1f);
        let _e119 = abs(_e96.z);
        if (_e119 > 1f) {
            phi_1390_ = ((0.9980469f * _e118) + 0.0009765625f);
        } else {
            phi_1390_ = ((0.001953125f * _e118) + _e119);
        }
        let _e126 = phi_1390_;
        let _e128 = textureSampleLevel(FD, ha, vec2<f32>(_e126, ((floor(_e101) * _e106) + _e109)), 0f);
        phi_1404_ = _e128;
        if !(_e98) {
            let _e132 = (_e128.xyz * _e128.w);
            phi_1404_ = vec4<f32>(_e132.x, _e132.y, _e132.z, (_e128.w * (fract(_e101) * 1.0039216f)));
        }
        let _e139 = phi_1404_;
        phi_1403_ = _e139;
    }
    let _e141 = phi_1403_;
    phi_919_ = ji;
    if ji {
        phi_919_ = (_e95.z > 0f);
    }
    let _e145 = phi_919_;
    phi_1406_ = _e141;
    if _e145 {
        let _e149 = textureSampleLevel(IC, f6_, _e95.xy, (_e95.z - 1f));
        phi_1400_ = _e149;
        if _e98 {
            if (_e149.w != 0f) {
                phi_1391_ = (1f / _e149.w);
            } else {
                phi_1391_ = 0f;
            }
            let _e155 = phi_1391_;
            let _e156 = (_e149.xyz * _e155);
            phi_1400_ = vec4<f32>(_e156.x, _e156.y, _e156.z, _e149.w);
        }
        let _e162 = phi_1400_;
        phi_1406_ = (_e141 * _e162);
    }
    let _e165 = phi_1406_;
    let _e166 = m1_1;
    let _e167 = F4_1;
    let _e170 = q3_1[1u];
    let _e172 = q3_1[0u];
    let _e173 = vec2<u32>(floor(_e167));
    phi_1408_ = 1f;
    if ci {
        let _e201 = R0_1;
        let _e204 = min(_e201.xy, _e201.zw);
        phi_1408_ = min(min(_e204.x, _e204.y), 1f);
    }
    let _e210 = phi_1408_;
    phi_706_ = bi;
    if bi {
        let _e212 = l1_1[0u];
        phi_706_ = (_e212 != 0f);
    }
    let _e215 = phi_706_;
    phi_1409_ = _e210;
    if _e215 {
        let _e218 = m0_.j2_[_e92];
        phi_1409_ = min(unpack4x8unorm(_e218).x, _e210);
    }
    let _e223 = phi_1409_;
    let _e225 = clamp(_e166, 0f, max(_e223, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e231 = u32(((abs(_e225) * 1024f) + 0.5f));
            let _e234 = atomicLoad((&V0_.j2_[(_e172 + (((((_e173.y >> bitcast<u32>(5u)) * (_e170 << bitcast<u32>(5u))) + ((_e173.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e173.x & 28u) << bitcast<u32>(5u)) + ((_e173.y & 28u) << bitcast<u32>(2i)))) + (((_e173.y & 3u) << bitcast<u32>(2i)) + (_e173.x & 3u))))]));
            let _e236 = (min(_e165.w, _e225) >= 1f);
            phi_1131_ = _e236;
            if _e236 {
                let _e238 = j.i2_;
                let _e239 = (_e234 < _e238);
                phi_1129_ = _e239;
                if !(_e239) {
                    phi_1129_ = (_e234 >= (_e238 | 262144u));
                }
                let _e244 = phi_1129_;
                phi_1131_ = _e244;
            }
            let _e246 = phi_1131_;
            if _e246 {
                phi_1432_ = 1f;
                break;
            }
            let _e248 = j.i2_;
            phi_1426_ = 0f;
            phi_1423_ = _e231;
            phi_1420_ = _e225;
            if (_e234 < _e248) {
                let _e251 = (_e248 | (262144u + _e231));
                let _e252 = atomicMax((&V0_.j2_[(_e172 + (((((_e173.y >> bitcast<u32>(5u)) * (_e170 << bitcast<u32>(5u))) + ((_e173.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e173.x & 28u) << bitcast<u32>(5u)) + ((_e173.y & 28u) << bitcast<u32>(2i)))) + (((_e173.y & 3u) << bitcast<u32>(2i)) + (_e173.x & 3u))))]), _e251);
                if (_e252 <= _e248) {
                    phi_1429_ = min(_e225, 1f);
                    phi_1424_ = _e231;
                    phi_1421_ = 0f;
                } else {
                    phi_1430_ = 0f;
                    phi_1425_ = _e231;
                    phi_1422_ = _e225;
                    if (_e252 < _e251) {
                        let _e257 = ((_e252 & 524287u) - 262144u);
                        let _e259 = (f32(_e257) * 0.0009765625f);
                        phi_1430_ = ((min(_e225, 1f) - _e259) / max((1f - (_e259 * _e165.w)), 0.000062f));
                        phi_1425_ = _e257;
                        phi_1422_ = _e259;
                    }
                    let _e267 = phi_1430_;
                    let _e269 = phi_1425_;
                    let _e271 = phi_1422_;
                    phi_1429_ = _e267;
                    phi_1424_ = _e269;
                    phi_1421_ = _e271;
                }
                let _e273 = phi_1429_;
                let _e275 = phi_1424_;
                let _e277 = phi_1421_;
                phi_1426_ = _e273;
                phi_1423_ = _e275;
                phi_1420_ = _e277;
            }
            let _e279 = phi_1426_;
            let _e281 = phi_1423_;
            let _e283 = phi_1420_;
            phi_1431_ = _e279;
            if (_e283 > 0f) {
                let _e285 = atomicAdd((&V0_.j2_[(_e172 + (((((_e173.y >> bitcast<u32>(5u)) * (_e170 << bitcast<u32>(5u))) + ((_e173.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e173.x & 28u) << bitcast<u32>(5u)) + ((_e173.y & 28u) << bitcast<u32>(2i)))) + (((_e173.y & 3u) << bitcast<u32>(2i)) + (_e173.x & 3u))))]), _e281);
                let _e290 = (f32(bitcast<i32>(((_e285 & 524287u) - 262144u))) * 0.0009765625f);
                let _e292 = clamp(_e290, 0f, 1f);
                phi_1431_ = (_e279 + ((1f - (_e279 * _e165.w)) * ((clamp((_e290 + _e283), 0f, 1f) - _e292) / max((1f - (_e292 * _e165.w)), 0.000062f))));
            }
            let _e304 = phi_1431_;
            phi_1432_ = _e304;
            break;
        }
    }
    let _e306 = phi_1432_;
    phi_1443_ = f32();
    if ii {
        let _e308 = j.M3_;
        let _e310 = j.N3_;
        if ii {
            phi_1433_ = ((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e308) + _e310);
        } else {
            phi_1433_ = 0f;
        }
        let _e322 = phi_1433_;
        phi_1443_ = _e322;
    }
    let _e324 = phi_1443_;
    let _e325 = (_e165 * _e306);
    let _e326 = _e325.xyz;
    if (ii && (_e325.w != 0f)) {
        phi_1463_ = (vec3(_e324) + _e326);
    } else {
        phi_1463_ = _e326;
    }
    let _e333 = phi_1463_;
    let _e339 = vec4<f32>(_e333.x, _e325.y, _e325.z, _e325.w);
    let _e345 = vec4<f32>(_e339.x, _e333.y, _e339.z, _e339.w);
    m0_.j2_[_e92] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    J1_ = vec4<f32>(_e345.x, _e345.y, _e333.z, _e345.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) F1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @location(1) @interpolate(flat, either) m1_: f32, @location(8) F4_: vec2<f32>, @location(7) @interpolate(flat, either) q3_: vec2<u32>, @location(5) R0_: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @location(3) @interpolate(flat, either) F0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    F1_1 = F1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    m1_1 = m1_;
    F4_1 = F4_;
    q3_1 = q3_;
    R0_1 = R0_;
    l1_1 = l1_;
    F0_1 = F0_;
    main_1();
    let _e21 = J1_;
    return _e21;
}
