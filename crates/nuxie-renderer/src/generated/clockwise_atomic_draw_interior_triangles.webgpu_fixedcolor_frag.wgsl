struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct Ae {
    g2_: array<u32>,
}

struct i0Sd {
    g2_: array<u32>,
}

struct Ae_1 {
    g2_: array<atomic<u32>>,
}

@id(7) override Lh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;
@id(1) override Fh: bool = true;
@id(0) override Eh: bool = true;

@group(0) @binding(0)
var<uniform> j: SB;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var M9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(6)
var<storage, read_write> S0_: Ae_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> j1_1: f32;
var<private> v4_1: vec2<f32>;
var<private> k3_1: vec2<u32>;
var<private> O0_1: vec4<f32>;
var<private> Y1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Sd;
var<private> F1_: vec4<f32>;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> D0_1: f32;

fn main_1() {
    var phi_1390_: f32;
    var phi_1391_: f32;
    var phi_1405_: vec4<f32>;
    var phi_1404_: vec4<f32>;
    var phi_920_: bool;
    var phi_1392_: f32;
    var phi_1401_: vec4<f32>;
    var phi_1407_: vec4<f32>;
    var phi_1409_: f32;
    var phi_706_: bool;
    var phi_1410_: f32;
    var phi_1130_: bool;
    var phi_1132_: bool;
    var phi_1431_: f32;
    var phi_1426_: u32;
    var phi_1423_: f32;
    var phi_1430_: f32;
    var phi_1425_: u32;
    var phi_1422_: f32;
    var phi_1427_: f32;
    var phi_1424_: u32;
    var phi_1421_: f32;
    var phi_1432_: f32;
    var phi_1433_: f32;
    var phi_1434_: f32;
    var phi_1444_: f32;
    var phi_1464_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e57.xy)));
    let _e63 = j.n6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e93 = g1_1;
    let _e95 = C2_1;
    let _e96 = X1_1;
    let _e98 = (Gh && (u32(_e93) != 0u));
    if (_e96.w >= 0f) {
        phi_1404_ = _e96;
    } else {
        let _e101 = -(_e96.w);
        let _e106 = j.Zb;
        let _e109 = j.ac;
        if (_e96.z > 0f) {
            phi_1390_ = _e96.x;
        } else {
            phi_1390_ = length(_e96.xy);
        }
        let _e117 = phi_1390_;
        let _e118 = clamp(_e117, 0f, 1f);
        let _e119 = abs(_e96.z);
        if (_e119 > 1f) {
            phi_1391_ = ((0.9980469f * _e118) + 0.0009765625f);
        } else {
            phi_1391_ = ((0.001953125f * _e118) + _e119);
        }
        let _e126 = phi_1391_;
        let _e128 = textureSampleLevel(DD, M9_, vec2<f32>(_e126, ((floor(_e101) * _e106) + _e109)), 0f);
        phi_1405_ = _e128;
        if !(_e98) {
            let _e132 = (_e128.xyz * _e128.w);
            phi_1405_ = vec4<f32>(_e132.x, _e132.y, _e132.z, (_e128.w * (fract(_e101) * 1.0039216f)));
        }
        let _e139 = phi_1405_;
        phi_1404_ = _e139;
    }
    let _e141 = phi_1404_;
    phi_920_ = Mh;
    if Mh {
        phi_920_ = (_e95.z > 0f);
    }
    let _e145 = phi_920_;
    phi_1407_ = _e141;
    if _e145 {
        let _e149 = textureSampleLevel(GC, W5_, _e95.xy, (_e95.z - 1f));
        phi_1401_ = _e149;
        if _e98 {
            if (_e149.w != 0f) {
                phi_1392_ = (1f / _e149.w);
            } else {
                phi_1392_ = 0f;
            }
            let _e155 = phi_1392_;
            let _e156 = (_e149.xyz * _e155);
            phi_1401_ = vec4<f32>(_e156.x, _e156.y, _e156.z, _e149.w);
        }
        let _e162 = phi_1401_;
        phi_1407_ = (_e141 * _e162);
    }
    let _e165 = phi_1407_;
    let _e166 = j1_1;
    let _e167 = v4_1;
    let _e170 = k3_1[1u];
    let _e172 = k3_1[0u];
    let _e173 = vec2<u32>(floor(_e167));
    phi_1409_ = 1f;
    if Fh {
        let _e201 = O0_1;
        let _e204 = min(_e201.xy, _e201.zw);
        phi_1409_ = min(min(_e204.x, _e204.y), 1f);
    }
    let _e210 = phi_1409_;
    phi_706_ = Eh;
    if Eh {
        let _e212 = Y1_1[0u];
        phi_706_ = (_e212 != 0f);
    }
    let _e215 = phi_706_;
    phi_1410_ = _e210;
    if _e215 {
        let _e218 = i0_.g2_[_e92];
        phi_1410_ = min(unpack4x8unorm(_e218).x, _e210);
    }
    let _e223 = phi_1410_;
    let _e225 = clamp(_e166, 0f, max(_e223, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e231 = u32(((abs(_e225) * 1024f) + 0.5f));
            let _e234 = atomicLoad((&S0_.g2_[(_e172 + (((((_e173.y >> bitcast<u32>(5u)) * (_e170 << bitcast<u32>(5u))) + ((_e173.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e173.x & 28u) << bitcast<u32>(5u)) + ((_e173.y & 28u) << bitcast<u32>(2i)))) + (((_e173.y & 3u) << bitcast<u32>(2i)) + (_e173.x & 3u))))]));
            let _e236 = (min(_e165.w, _e225) >= 1f);
            phi_1132_ = _e236;
            if _e236 {
                let _e238 = j.f2_;
                let _e239 = (_e234 < _e238);
                phi_1130_ = _e239;
                if !(_e239) {
                    phi_1130_ = (_e234 >= (_e238 | 262144u));
                }
                let _e244 = phi_1130_;
                phi_1132_ = _e244;
            }
            let _e246 = phi_1132_;
            if _e246 {
                phi_1433_ = 1f;
                break;
            }
            let _e248 = j.f2_;
            phi_1427_ = 0f;
            phi_1424_ = _e231;
            phi_1421_ = _e225;
            if (_e234 < _e248) {
                let _e251 = (_e248 | (262144u + _e231));
                let _e252 = atomicMax((&S0_.g2_[(_e172 + (((((_e173.y >> bitcast<u32>(5u)) * (_e170 << bitcast<u32>(5u))) + ((_e173.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e173.x & 28u) << bitcast<u32>(5u)) + ((_e173.y & 28u) << bitcast<u32>(2i)))) + (((_e173.y & 3u) << bitcast<u32>(2i)) + (_e173.x & 3u))))]), _e251);
                if (_e252 <= _e248) {
                    phi_1430_ = min(_e225, 1f);
                    phi_1425_ = _e231;
                    phi_1422_ = 0f;
                } else {
                    phi_1431_ = 0f;
                    phi_1426_ = _e231;
                    phi_1423_ = _e225;
                    if (_e252 < _e251) {
                        let _e257 = ((_e252 & 524287u) - 262144u);
                        let _e259 = (f32(_e257) * 0.0009765625f);
                        phi_1431_ = ((min(_e225, 1f) - _e259) / max((1f - (_e259 * _e165.w)), 0.000062f));
                        phi_1426_ = _e257;
                        phi_1423_ = _e259;
                    }
                    let _e267 = phi_1431_;
                    let _e269 = phi_1426_;
                    let _e271 = phi_1423_;
                    phi_1430_ = _e267;
                    phi_1425_ = _e269;
                    phi_1422_ = _e271;
                }
                let _e273 = phi_1430_;
                let _e275 = phi_1425_;
                let _e277 = phi_1422_;
                phi_1427_ = _e273;
                phi_1424_ = _e275;
                phi_1421_ = _e277;
            }
            let _e279 = phi_1427_;
            let _e281 = phi_1424_;
            let _e283 = phi_1421_;
            phi_1432_ = _e279;
            if (_e283 > 0f) {
                let _e285 = atomicAdd((&S0_.g2_[(_e172 + (((((_e173.y >> bitcast<u32>(5u)) * (_e170 << bitcast<u32>(5u))) + ((_e173.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e173.x & 28u) << bitcast<u32>(5u)) + ((_e173.y & 28u) << bitcast<u32>(2i)))) + (((_e173.y & 3u) << bitcast<u32>(2i)) + (_e173.x & 3u))))]), _e281);
                let _e290 = (f32(bitcast<i32>(((_e285 & 524287u) - 262144u))) * 0.0009765625f);
                let _e292 = clamp(_e290, 0f, 1f);
                phi_1432_ = (_e279 + ((1f - (_e279 * _e165.w)) * ((clamp((_e290 + _e283), 0f, 1f) - _e292) / max((1f - (_e292 * _e165.w)), 0.000062f))));
            }
            let _e304 = phi_1432_;
            phi_1433_ = _e304;
            break;
        }
    }
    let _e306 = phi_1433_;
    phi_1444_ = f32();
    if Lh {
        let _e308 = j.F3_;
        let _e310 = j.G3_;
        if Lh {
            phi_1434_ = ((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e308) + _e310);
        } else {
            phi_1434_ = 0f;
        }
        let _e322 = phi_1434_;
        phi_1444_ = _e322;
    }
    let _e324 = phi_1444_;
    let _e325 = (_e165 * _e306);
    let _e326 = _e325.xyz;
    if (Lh && (_e325.w != 0f)) {
        phi_1464_ = (vec3(_e324) + _e326);
    } else {
        phi_1464_ = _e326;
    }
    let _e333 = phi_1464_;
    let _e339 = vec4<f32>(_e333.x, _e325.y, _e325.z, _e325.w);
    let _e345 = vec4<f32>(_e339.x, _e333.y, _e339.z, _e339.w);
    i0_.g2_[_e92] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    F1_ = vec4<f32>(_e345.x, _e345.y, _e333.z, _e345.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @location(1) @interpolate(flat, either) j1_: f32, @location(8) v4_: vec2<f32>, @location(7) @interpolate(flat, either) k3_: vec2<u32>, @location(5) O0_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @location(3) @interpolate(flat, either) D0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    j1_1 = j1_;
    v4_1 = v4_;
    k3_1 = k3_;
    O0_1 = O0_;
    Y1_1 = Y1_;
    D0_1 = D0_;
    main_1();
    let _e21 = F1_;
    return _e21;
}
