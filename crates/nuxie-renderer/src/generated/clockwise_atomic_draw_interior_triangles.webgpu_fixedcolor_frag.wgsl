struct Be {
    g2_: array<u32>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct i0Td {
    g2_: array<u32>,
}

struct Be_1 {
    g2_: array<atomic<u32>>,
}

@id(7) override Lh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;
@id(1) override Fh: bool = true;
@id(0) override Eh: bool = true;

@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;
@group(0) @binding(6)
var<storage, read_write> R0_: Be_1;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> j1_1: f32;
var<private> r4_1: vec2<f32>;
var<private> i3_1: vec2<u32>;
var<private> N0_1: vec4<f32>;
var<private> Y1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Td;
var<private> E1_: vec4<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> D0_1: f32;

fn main_1() {
    var phi_1322_: f32;
    var phi_1323_: f32;
    var phi_1337_: vec4<f32>;
    var phi_1336_: vec4<f32>;
    var phi_864_: bool;
    var phi_1324_: f32;
    var phi_1333_: vec4<f32>;
    var phi_1339_: vec4<f32>;
    var phi_1341_: f32;
    var phi_671_: bool;
    var phi_1342_: f32;
    var phi_1069_: bool;
    var phi_1071_: bool;
    var phi_1363_: f32;
    var phi_1358_: u32;
    var phi_1355_: f32;
    var phi_1362_: f32;
    var phi_1357_: u32;
    var phi_1354_: f32;
    var phi_1359_: f32;
    var phi_1356_: u32;
    var phi_1353_: f32;
    var phi_1364_: f32;
    var phi_1365_: f32;
    var phi_1366_: f32;
    var phi_1376_: f32;
    var phi_1396_: vec3<f32>;

    let _e54 = gl_FragCoord_1;
    let _e58 = bitcast<vec2<u32>>(vec2<i32>(floor(_e54.xy)));
    let _e60 = j.q6_;
    let _e89 = bitcast<i32>((((((_e58.y >> bitcast<u32>(5u)) * (((_e60 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e58.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e58.x & 28u) << bitcast<u32>(5u)) + ((_e58.y & 28u) << bitcast<u32>(2i)))) + (((_e58.y & 3u) << bitcast<u32>(2i)) + (_e58.x & 3u))));
    let _e90 = g1_1;
    let _e92 = C2_1;
    let _e93 = X1_1;
    let _e95 = (Gh && (u32(_e90) != 0u));
    if (_e93.w >= 0f) {
        phi_1336_ = _e93;
    } else {
        if (_e93.z > 0f) {
            phi_1322_ = _e93.x;
        } else {
            phi_1322_ = length(_e93.xy);
        }
        let _e105 = phi_1322_;
        let _e106 = clamp(_e105, 0f, 1f);
        let _e107 = abs(_e93.z);
        if (_e107 > 1f) {
            phi_1323_ = ((0.9980469f * _e106) + 0.0009765625f);
        } else {
            phi_1323_ = ((0.001953125f * _e106) + _e107);
        }
        let _e114 = phi_1323_;
        let _e116 = textureSampleLevel(DD, P9_, vec2<f32>(_e114, -(_e93.w)), 0f);
        phi_1337_ = _e116;
        if !(_e95) {
            let _e120 = (_e116.xyz * _e116.w);
            let _e126 = vec4<f32>(_e120.x, _e116.y, _e116.z, _e116.w);
            let _e132 = vec4<f32>(_e126.x, _e120.y, _e126.z, _e126.w);
            phi_1337_ = vec4<f32>(_e132.x, _e132.y, _e120.z, _e132.w);
        }
        let _e140 = phi_1337_;
        phi_1336_ = _e140;
    }
    let _e142 = phi_1336_;
    phi_864_ = Mh;
    if Mh {
        phi_864_ = (_e92.z > 0f);
    }
    let _e146 = phi_864_;
    phi_1339_ = _e142;
    if _e146 {
        let _e150 = textureSampleLevel(GC, Y5_, _e92.xy, (_e92.z - 1f));
        phi_1333_ = _e150;
        if _e95 {
            if (_e150.w != 0f) {
                phi_1324_ = (1f / _e150.w);
            } else {
                phi_1324_ = 0f;
            }
            let _e156 = phi_1324_;
            let _e157 = (_e150.xyz * _e156);
            phi_1333_ = vec4<f32>(_e157.x, _e157.y, _e157.z, _e150.w);
        }
        let _e163 = phi_1333_;
        phi_1339_ = (_e142 * _e163);
    }
    let _e166 = phi_1339_;
    let _e167 = j1_1;
    let _e168 = r4_1;
    let _e171 = i3_1[1u];
    let _e173 = i3_1[0u];
    let _e174 = vec2<u32>(floor(_e168));
    phi_1341_ = 1f;
    if Fh {
        let _e202 = N0_1;
        let _e205 = min(_e202.xy, _e202.zw);
        phi_1341_ = min(min(_e205.x, _e205.y), 1f);
    }
    let _e211 = phi_1341_;
    phi_671_ = Eh;
    if Eh {
        let _e213 = Y1_1[0u];
        phi_671_ = (_e213 != 0f);
    }
    let _e216 = phi_671_;
    phi_1342_ = _e211;
    if _e216 {
        let _e219 = i0_.g2_[_e89];
        phi_1342_ = min(unpack4x8unorm(_e219).x, _e211);
    }
    let _e224 = phi_1342_;
    let _e226 = clamp(_e167, 0f, max(_e224, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e232 = u32(((abs(_e226) * 1024f) + 0.5f));
            let _e235 = atomicLoad((&R0_.g2_[(_e173 + (((((_e174.y >> bitcast<u32>(5u)) * (_e171 << bitcast<u32>(5u))) + ((_e174.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e174.x & 28u) << bitcast<u32>(5u)) + ((_e174.y & 28u) << bitcast<u32>(2i)))) + (((_e174.y & 3u) << bitcast<u32>(2i)) + (_e174.x & 3u))))]));
            let _e237 = (min(_e166.w, _e226) >= 1f);
            phi_1071_ = _e237;
            if _e237 {
                let _e239 = j.f2_;
                let _e240 = (_e235 < _e239);
                phi_1069_ = _e240;
                if !(_e240) {
                    phi_1069_ = (_e235 >= (_e239 | 262144u));
                }
                let _e245 = phi_1069_;
                phi_1071_ = _e245;
            }
            let _e247 = phi_1071_;
            if _e247 {
                phi_1365_ = 1f;
                break;
            }
            let _e249 = j.f2_;
            phi_1359_ = 0f;
            phi_1356_ = _e232;
            phi_1353_ = _e226;
            if (_e235 < _e249) {
                let _e252 = (_e249 | (262144u + _e232));
                let _e253 = atomicMax((&R0_.g2_[(_e173 + (((((_e174.y >> bitcast<u32>(5u)) * (_e171 << bitcast<u32>(5u))) + ((_e174.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e174.x & 28u) << bitcast<u32>(5u)) + ((_e174.y & 28u) << bitcast<u32>(2i)))) + (((_e174.y & 3u) << bitcast<u32>(2i)) + (_e174.x & 3u))))]), _e252);
                if (_e253 <= _e249) {
                    phi_1362_ = min(_e226, 1f);
                    phi_1357_ = _e232;
                    phi_1354_ = 0f;
                } else {
                    phi_1363_ = 0f;
                    phi_1358_ = _e232;
                    phi_1355_ = _e226;
                    if (_e253 < _e252) {
                        let _e258 = ((_e253 & 524287u) - 262144u);
                        let _e260 = (f32(_e258) * 0.0009765625f);
                        phi_1363_ = ((min(_e226, 1f) - _e260) / max((1f - (_e260 * _e166.w)), 0.000062f));
                        phi_1358_ = _e258;
                        phi_1355_ = _e260;
                    }
                    let _e268 = phi_1363_;
                    let _e270 = phi_1358_;
                    let _e272 = phi_1355_;
                    phi_1362_ = _e268;
                    phi_1357_ = _e270;
                    phi_1354_ = _e272;
                }
                let _e274 = phi_1362_;
                let _e276 = phi_1357_;
                let _e278 = phi_1354_;
                phi_1359_ = _e274;
                phi_1356_ = _e276;
                phi_1353_ = _e278;
            }
            let _e280 = phi_1359_;
            let _e282 = phi_1356_;
            let _e284 = phi_1353_;
            phi_1364_ = _e280;
            if (_e284 > 0f) {
                let _e286 = atomicAdd((&R0_.g2_[(_e173 + (((((_e174.y >> bitcast<u32>(5u)) * (_e171 << bitcast<u32>(5u))) + ((_e174.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e174.x & 28u) << bitcast<u32>(5u)) + ((_e174.y & 28u) << bitcast<u32>(2i)))) + (((_e174.y & 3u) << bitcast<u32>(2i)) + (_e174.x & 3u))))]), _e282);
                let _e291 = (f32(bitcast<i32>(((_e286 & 524287u) - 262144u))) * 0.0009765625f);
                let _e293 = clamp(_e291, 0f, 1f);
                phi_1364_ = (_e280 + ((1f - (_e280 * _e166.w)) * ((clamp((_e291 + _e284), 0f, 1f) - _e293) / max((1f - (_e293 * _e166.w)), 0.000062f))));
            }
            let _e305 = phi_1364_;
            phi_1365_ = _e305;
            break;
        }
    }
    let _e307 = phi_1365_;
    phi_1376_ = f32();
    if Lh {
        let _e309 = j.F3_;
        let _e311 = j.G3_;
        if Lh {
            phi_1366_ = ((fract((52.982918f * fract(((0.06711056f * _e54.x) + (0.00583715f * _e54.y))))) * _e309) + _e311);
        } else {
            phi_1366_ = 0f;
        }
        let _e323 = phi_1366_;
        phi_1376_ = _e323;
    }
    let _e325 = phi_1376_;
    let _e326 = (_e166 * _e307);
    let _e327 = _e326.xyz;
    if (Lh && (_e326.w != 0f)) {
        phi_1396_ = (vec3(_e325) + _e327);
    } else {
        phi_1396_ = _e327;
    }
    let _e334 = phi_1396_;
    let _e340 = vec4<f32>(_e334.x, _e326.y, _e326.z, _e326.w);
    let _e346 = vec4<f32>(_e340.x, _e334.y, _e340.z, _e340.w);
    i0_.g2_[_e89] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    E1_ = vec4<f32>(_e346.x, _e346.y, _e334.z, _e346.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @location(1) @interpolate(flat, either) j1_: f32, @location(8) r4_: vec2<f32>, @location(7) @interpolate(flat, either) i3_: vec2<u32>, @location(5) N0_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @location(3) @interpolate(flat, either) D0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    j1_1 = j1_;
    r4_1 = r4_;
    i3_1 = i3_;
    N0_1 = N0_;
    Y1_1 = Y1_;
    D0_1 = D0_;
    main_1();
    let _e21 = E1_;
    return _e21;
}
