struct Ae {
    e2_: array<u32>,
}

struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

struct h0Sd {
    e2_: array<u32>,
}

struct Ae_1 {
    e2_: array<atomic<u32>>,
}

@id(7) override Kh: bool = true;
@id(2) override Fh: bool = true;
@id(8) override Lh: bool = true;
@id(1) override Eh: bool = true;
@id(0) override Dh: bool = true;

@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var O9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;
@group(0) @binding(6)
var<storage, read_write> Q0_: Ae_1;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> V1_1: vec4<f32>;
var<private> B2_1: vec3<f32>;
var<private> h1_1: f32;
var<private> p4_1: vec2<f32>;
var<private> g3_1: vec2<u32>;
var<private> M0_1: vec4<f32>;
var<private> W1_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Sd;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var fa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> C0_1: f32;
var<private> g2_1: f32;

fn main_1() {
    var phi_1350_: vec4<f32>;
    var phi_1334_: f32;
    var phi_1335_: f32;
    var phi_1351_: vec4<f32>;
    var phi_1349_: vec4<f32>;
    var phi_871_: bool;
    var phi_1336_: f32;
    var phi_1346_: vec4<f32>;
    var phi_1353_: vec4<f32>;
    var phi_1355_: f32;
    var phi_669_: bool;
    var phi_1356_: f32;
    var phi_1075_: bool;
    var phi_1077_: bool;
    var phi_1377_: f32;
    var phi_1372_: u32;
    var phi_1369_: f32;
    var phi_1376_: f32;
    var phi_1371_: u32;
    var phi_1368_: f32;
    var phi_1373_: f32;
    var phi_1370_: u32;
    var phi_1367_: f32;
    var phi_1378_: f32;
    var phi_1379_: f32;
    var phi_1380_: f32;
    var phi_1390_: f32;
    var phi_1411_: vec3<f32>;

    let _e54 = gl_FragCoord_1;
    let _e58 = bitcast<vec2<u32>>(vec2<i32>(floor(_e54.xy)));
    let _e60 = l.o6_;
    let _e89 = bitcast<i32>((((((_e58.y >> bitcast<u32>(5u)) * (((_e60 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e58.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e58.x & 28u) << bitcast<u32>(5u)) + ((_e58.y & 28u) << bitcast<u32>(2i)))) + (((_e58.y & 3u) << bitcast<u32>(2i)) + (_e58.x & 3u))));
    let _e90 = V1_1;
    let _e91 = B2_1;
    if (_e90.w >= 0f) {
        if Fh {
            phi_1350_ = vec4<f32>(_e90.x, _e90.y, _e90.z, _e90.w);
        } else {
            phi_1350_ = (_e90 * 1f);
        }
        let _e102 = phi_1350_;
        phi_1349_ = _e102;
    } else {
        if (_e90.z > 0f) {
            phi_1334_ = _e90.x;
        } else {
            phi_1334_ = length(_e90.xy);
        }
        let _e110 = phi_1334_;
        let _e111 = clamp(_e110, 0f, 1f);
        let _e112 = abs(_e90.z);
        if (_e112 > 1f) {
            phi_1335_ = ((0.9980469f * _e111) + 0.0009765625f);
        } else {
            phi_1335_ = ((0.001953125f * _e111) + _e112);
        }
        let _e119 = phi_1335_;
        let _e121 = textureSampleLevel(ED, O9_, vec2<f32>(_e119, -(_e90.w)), 0f);
        let _e127 = vec4<f32>(_e121.x, _e121.y, _e121.z, _e121.w);
        if Fh {
            phi_1351_ = _e127;
        } else {
            let _e129 = (_e127.xyz * _e121.w);
            phi_1351_ = vec4<f32>(_e129.x, _e129.y, _e129.z, _e121.w);
        }
        let _e135 = phi_1351_;
        phi_1349_ = _e135;
    }
    let _e137 = phi_1349_;
    phi_871_ = Lh;
    if Lh {
        phi_871_ = (_e91.z > 0f);
    }
    let _e141 = phi_871_;
    phi_1353_ = _e137;
    if _e141 {
        let _e145 = textureSampleLevel(HC, V5_, _e91.xy, (_e91.z - 1f));
        phi_1346_ = _e145;
        if Fh {
            if (_e145.w != 0f) {
                phi_1336_ = (1f / _e145.w);
            } else {
                phi_1336_ = 0f;
            }
            let _e151 = phi_1336_;
            let _e152 = (_e145.xyz * _e151);
            phi_1346_ = vec4<f32>(_e152.x, _e152.y, _e152.z, _e145.w);
        }
        let _e158 = phi_1346_;
        phi_1353_ = (_e137 * _e158);
    }
    let _e161 = phi_1353_;
    let _e162 = h1_1;
    let _e163 = p4_1;
    let _e166 = g3_1[1u];
    let _e168 = g3_1[0u];
    let _e169 = vec2<u32>(floor(_e163));
    phi_1355_ = 1f;
    if Eh {
        let _e197 = M0_1;
        let _e200 = min(_e197.xy, _e197.zw);
        phi_1355_ = min(min(_e200.x, _e200.y), 1f);
    }
    let _e206 = phi_1355_;
    phi_669_ = Dh;
    if Dh {
        let _e208 = W1_1[0u];
        phi_669_ = (_e208 != 0f);
    }
    let _e211 = phi_669_;
    phi_1356_ = _e206;
    if _e211 {
        let _e214 = h0_.e2_[_e89];
        phi_1356_ = min(unpack4x8unorm(_e214).x, _e206);
    }
    let _e219 = phi_1356_;
    let _e221 = clamp(_e162, 0f, max(_e219, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e227 = u32(((abs(_e221) * 1024f) + 0.5f));
            let _e230 = atomicLoad((&Q0_.e2_[(_e168 + (((((_e169.y >> bitcast<u32>(5u)) * (_e166 << bitcast<u32>(5u))) + ((_e169.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e169.x & 28u) << bitcast<u32>(5u)) + ((_e169.y & 28u) << bitcast<u32>(2i)))) + (((_e169.y & 3u) << bitcast<u32>(2i)) + (_e169.x & 3u))))]));
            let _e232 = (min(_e161.w, _e221) >= 1f);
            phi_1077_ = _e232;
            if _e232 {
                let _e234 = l.d2_;
                let _e235 = (_e230 < _e234);
                phi_1075_ = _e235;
                if !(_e235) {
                    phi_1075_ = (_e230 >= (_e234 | 262144u));
                }
                let _e240 = phi_1075_;
                phi_1077_ = _e240;
            }
            let _e242 = phi_1077_;
            if _e242 {
                phi_1379_ = 1f;
                break;
            }
            let _e244 = l.d2_;
            phi_1373_ = 0f;
            phi_1370_ = _e227;
            phi_1367_ = _e221;
            if (_e230 < _e244) {
                let _e247 = (_e244 | (262144u + _e227));
                let _e248 = atomicMax((&Q0_.e2_[(_e168 + (((((_e169.y >> bitcast<u32>(5u)) * (_e166 << bitcast<u32>(5u))) + ((_e169.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e169.x & 28u) << bitcast<u32>(5u)) + ((_e169.y & 28u) << bitcast<u32>(2i)))) + (((_e169.y & 3u) << bitcast<u32>(2i)) + (_e169.x & 3u))))]), _e247);
                if (_e248 <= _e244) {
                    phi_1376_ = min(_e221, 1f);
                    phi_1371_ = _e227;
                    phi_1368_ = 0f;
                } else {
                    phi_1377_ = 0f;
                    phi_1372_ = _e227;
                    phi_1369_ = _e221;
                    if (_e248 < _e247) {
                        let _e253 = ((_e248 & 524287u) - 262144u);
                        let _e255 = (f32(_e253) * 0.0009765625f);
                        phi_1377_ = ((min(_e221, 1f) - _e255) / max((1f - (_e255 * _e161.w)), 0.000062f));
                        phi_1372_ = _e253;
                        phi_1369_ = _e255;
                    }
                    let _e263 = phi_1377_;
                    let _e265 = phi_1372_;
                    let _e267 = phi_1369_;
                    phi_1376_ = _e263;
                    phi_1371_ = _e265;
                    phi_1368_ = _e267;
                }
                let _e269 = phi_1376_;
                let _e271 = phi_1371_;
                let _e273 = phi_1368_;
                phi_1373_ = _e269;
                phi_1370_ = _e271;
                phi_1367_ = _e273;
            }
            let _e275 = phi_1373_;
            let _e277 = phi_1370_;
            let _e279 = phi_1367_;
            phi_1378_ = _e275;
            if (_e279 > 0f) {
                let _e281 = atomicAdd((&Q0_.e2_[(_e168 + (((((_e169.y >> bitcast<u32>(5u)) * (_e166 << bitcast<u32>(5u))) + ((_e169.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e169.x & 28u) << bitcast<u32>(5u)) + ((_e169.y & 28u) << bitcast<u32>(2i)))) + (((_e169.y & 3u) << bitcast<u32>(2i)) + (_e169.x & 3u))))]), _e277);
                let _e286 = (f32(bitcast<i32>(((_e281 & 524287u) - 262144u))) * 0.0009765625f);
                let _e288 = clamp(_e286, 0f, 1f);
                phi_1378_ = (_e275 + ((1f - (_e275 * _e161.w)) * ((clamp((_e286 + _e279), 0f, 1f) - _e288) / max((1f - (_e288 * _e161.w)), 0.000062f))));
            }
            let _e300 = phi_1378_;
            phi_1379_ = _e300;
            break;
        }
    }
    let _e302 = phi_1379_;
    phi_1390_ = f32();
    if Kh {
        let _e304 = l.C3_;
        let _e306 = l.D3_;
        if Kh {
            phi_1380_ = ((fract((52.982918f * fract(((0.06711056f * _e54.x) + (0.00583715f * _e54.y))))) * _e304) + _e306);
        } else {
            phi_1380_ = 0f;
        }
        let _e318 = phi_1380_;
        phi_1390_ = _e318;
    }
    let _e320 = phi_1390_;
    let _e321 = (_e161 * _e302);
    let _e322 = _e321.xyz;
    if (Kh && (_e321.w != 0f)) {
        phi_1411_ = (vec3(_e320) + _e322);
    } else {
        phi_1411_ = _e322;
    }
    let _e329 = phi_1411_;
    let _e335 = vec4<f32>(_e329.x, _e321.y, _e321.z, _e321.w);
    let _e341 = vec4<f32>(_e335.x, _e329.y, _e335.z, _e335.w);
    h0_.e2_[_e89] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    C1_ = vec4<f32>(_e341.x, _e341.y, _e329.z, _e341.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) V1_: vec4<f32>, @location(9) B2_: vec3<f32>, @location(1) @interpolate(flat, either) h1_: f32, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(5) M0_: vec4<f32>, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    V1_1 = V1_;
    B2_1 = B2_;
    h1_1 = h1_;
    p4_1 = p4_;
    g3_1 = g3_;
    M0_1 = M0_;
    W1_1 = W1_;
    C0_1 = C0_;
    g2_1 = g2_;
    main_1();
    let _e21 = C1_;
    return _e21;
}
