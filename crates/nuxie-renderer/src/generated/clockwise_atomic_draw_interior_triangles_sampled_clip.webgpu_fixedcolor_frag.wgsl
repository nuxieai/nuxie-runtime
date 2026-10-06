struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct Ve {
    k2_: array<u32>,
}

struct Ve_1 {
    k2_: array<atomic<u32>>,
}

@id(7) override ti: bool = true;
@id(2) override oi: bool = true;
@id(8) override ui: bool = true;
@id(1) override ni: bool = true;
@id(0) override mi: bool = true;

@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;
@group(0) @binding(6)
var<storage, read_write> V0_: Ve_1;
var<private> v1_1: vec3<f32>;
var<private> Q0_1: f32;
var<private> a1_1: vec4<f32>;
var<private> m1_1: f32;
var<private> G4_1: vec2<f32>;
var<private> r3_1: vec2<u32>;
var<private> R0_1: vec4<f32>;
var<private> l1_1: vec2<f32>;
@group(2) @binding(1)
var m0_: texture_2d<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> L1_: vec4<f32>;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> F0_1: f32;

fn main_1() {
    var phi_1339_: f32;
    var phi_1340_: f32;
    var phi_1356_: vec4<f32>;
    var phi_1355_: vec4<f32>;
    var phi_873_: bool;
    var phi_888_: bool;
    var phi_1341_: f32;
    var phi_1351_: vec4<f32>;
    var phi_1358_: vec4<f32>;
    var phi_1359_: vec4<f32>;
    var phi_1361_: f32;
    var phi_691_: bool;
    var phi_1362_: f32;
    var phi_1104_: bool;
    var phi_1106_: bool;
    var phi_1383_: f32;
    var phi_1378_: u32;
    var phi_1375_: f32;
    var phi_1382_: f32;
    var phi_1377_: u32;
    var phi_1374_: f32;
    var phi_1379_: f32;
    var phi_1376_: u32;
    var phi_1373_: f32;
    var phi_1384_: f32;
    var phi_1385_: f32;
    var phi_1386_: f32;
    var phi_1396_: f32;
    var phi_1418_: vec3<f32>;

    let _e53 = Q0_1;
    let _e55 = v1_1;
    let _e56 = a1_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e59 = (oi && (u32(_e53) != 0u));
            if (_e56.w >= 0f) {
                phi_1355_ = _e56;
            } else {
                let _e62 = -(_e56.w);
                let _e67 = j.xc;
                let _e70 = j.yc;
                if (_e56.z > 0f) {
                    phi_1339_ = _e56.x;
                } else {
                    phi_1339_ = length(_e56.xy);
                }
                let _e78 = phi_1339_;
                let _e79 = clamp(_e78, 0f, 1f);
                let _e80 = abs(_e56.z);
                if (_e80 > 1f) {
                    phi_1340_ = ((0.9980469f * _e79) + 0.0009765625f);
                } else {
                    phi_1340_ = ((0.001953125f * _e79) + _e80);
                }
                let _e87 = phi_1340_;
                let _e89 = textureSampleLevel(ED, ia, vec2<f32>(_e87, ((floor(_e62) * _e67) + _e70)), 0f);
                phi_1356_ = _e89;
                if !(_e59) {
                    let _e93 = (_e89.xyz * _e89.w);
                    phi_1356_ = vec4<f32>(_e93.x, _e93.y, _e93.z, (_e89.w * (fract(_e62) * 1.0039216f)));
                }
                let _e100 = phi_1356_;
                phi_1355_ = _e100;
            }
            let _e102 = phi_1355_;
            phi_873_ = ui;
            if ui {
                phi_873_ = (_e55.z < 0f);
            }
            let _e106 = phi_873_;
            if _e106 {
                let _e108 = textureSampleLevel(CC, r5_, _e55.xy, 0f);
                phi_1359_ = _e108;
                break;
            }
            phi_888_ = ui;
            if ui {
                phi_888_ = (_e55.z > 0f);
            }
            let _e112 = phi_888_;
            phi_1358_ = _e102;
            if _e112 {
                let _e116 = textureSampleLevel(CC, r5_, _e55.xy, (_e55.z - 1f));
                phi_1351_ = _e116;
                if _e59 {
                    if (_e116.w != 0f) {
                        phi_1341_ = (1f / _e116.w);
                    } else {
                        phi_1341_ = 0f;
                    }
                    let _e122 = phi_1341_;
                    let _e123 = (_e116.xyz * _e122);
                    phi_1351_ = vec4<f32>(_e123.x, _e123.y, _e123.z, _e116.w);
                }
                let _e129 = phi_1351_;
                phi_1358_ = (_e102 * _e129);
            }
            let _e132 = phi_1358_;
            phi_1359_ = _e132;
            break;
        }
    }
    let _e134 = phi_1359_;
    let _e135 = m1_1;
    let _e136 = G4_1;
    let _e139 = r3_1[1u];
    let _e141 = r3_1[0u];
    let _e142 = vec2<u32>(floor(_e136));
    phi_1361_ = 1f;
    if ni {
        let _e170 = R0_1;
        let _e173 = min(_e170.xy, _e170.zw);
        phi_1361_ = min(min(_e173.x, _e173.y), 1f);
    }
    let _e179 = phi_1361_;
    phi_691_ = mi;
    if mi {
        let _e181 = l1_1[0u];
        phi_691_ = (_e181 != 0f);
    }
    let _e184 = phi_691_;
    phi_1362_ = _e179;
    if _e184 {
        let _e185 = gl_FragCoord_1;
        let _e189 = textureLoad(m0_, vec2<i32>(floor(_e185.xy)), 0i);
        phi_1362_ = min(_e189.x, _e179);
    }
    let _e193 = phi_1362_;
    let _e195 = clamp(_e135, 0f, max(_e193, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e201 = u32(((abs(_e195) * 1024f) + 0.5f));
            let _e204 = atomicLoad((&V0_.k2_[(_e141 + (((((_e142.y >> bitcast<u32>(5u)) * (_e139 << bitcast<u32>(5u))) + ((_e142.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e142.x & 28u) << bitcast<u32>(5u)) + ((_e142.y & 28u) << bitcast<u32>(2i)))) + (((_e142.y & 3u) << bitcast<u32>(2i)) + (_e142.x & 3u))))]));
            let _e206 = (min(_e134.w, _e195) >= 1f);
            phi_1106_ = _e206;
            if _e206 {
                let _e208 = j.j2_;
                let _e209 = (_e204 < _e208);
                phi_1104_ = _e209;
                if !(_e209) {
                    phi_1104_ = (_e204 >= (_e208 | 262144u));
                }
                let _e214 = phi_1104_;
                phi_1106_ = _e214;
            }
            let _e216 = phi_1106_;
            if _e216 {
                phi_1385_ = 1f;
                break;
            }
            let _e218 = j.j2_;
            phi_1379_ = 0f;
            phi_1376_ = _e201;
            phi_1373_ = _e195;
            if (_e204 < _e218) {
                let _e221 = (_e218 | (262144u + _e201));
                let _e222 = atomicMax((&V0_.k2_[(_e141 + (((((_e142.y >> bitcast<u32>(5u)) * (_e139 << bitcast<u32>(5u))) + ((_e142.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e142.x & 28u) << bitcast<u32>(5u)) + ((_e142.y & 28u) << bitcast<u32>(2i)))) + (((_e142.y & 3u) << bitcast<u32>(2i)) + (_e142.x & 3u))))]), _e221);
                if (_e222 <= _e218) {
                    phi_1382_ = min(_e195, 1f);
                    phi_1377_ = _e201;
                    phi_1374_ = 0f;
                } else {
                    phi_1383_ = 0f;
                    phi_1378_ = _e201;
                    phi_1375_ = _e195;
                    if (_e222 < _e221) {
                        let _e227 = ((_e222 & 524287u) - 262144u);
                        let _e229 = (f32(_e227) * 0.0009765625f);
                        phi_1383_ = ((min(_e195, 1f) - _e229) / max((1f - (_e229 * _e134.w)), 0.000062f));
                        phi_1378_ = _e227;
                        phi_1375_ = _e229;
                    }
                    let _e237 = phi_1383_;
                    let _e239 = phi_1378_;
                    let _e241 = phi_1375_;
                    phi_1382_ = _e237;
                    phi_1377_ = _e239;
                    phi_1374_ = _e241;
                }
                let _e243 = phi_1382_;
                let _e245 = phi_1377_;
                let _e247 = phi_1374_;
                phi_1379_ = _e243;
                phi_1376_ = _e245;
                phi_1373_ = _e247;
            }
            let _e249 = phi_1379_;
            let _e251 = phi_1376_;
            let _e253 = phi_1373_;
            phi_1384_ = _e249;
            if (_e253 > 0f) {
                let _e255 = atomicAdd((&V0_.k2_[(_e141 + (((((_e142.y >> bitcast<u32>(5u)) * (_e139 << bitcast<u32>(5u))) + ((_e142.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e142.x & 28u) << bitcast<u32>(5u)) + ((_e142.y & 28u) << bitcast<u32>(2i)))) + (((_e142.y & 3u) << bitcast<u32>(2i)) + (_e142.x & 3u))))]), _e251);
                let _e260 = (f32(bitcast<i32>(((_e255 & 524287u) - 262144u))) * 0.0009765625f);
                let _e262 = clamp(_e260, 0f, 1f);
                phi_1384_ = (_e249 + ((1f - (_e249 * _e134.w)) * ((clamp((_e260 + _e253), 0f, 1f) - _e262) / max((1f - (_e262 * _e134.w)), 0.000062f))));
            }
            let _e274 = phi_1384_;
            phi_1385_ = _e274;
            break;
        }
    }
    let _e276 = phi_1385_;
    phi_1396_ = f32();
    if ti {
        let _e277 = gl_FragCoord_1;
        let _e279 = j.M3_;
        let _e281 = j.N3_;
        if ti {
            phi_1386_ = ((fract((52.982918f * fract(((0.06711056f * _e277.x) + (0.00583715f * _e277.y))))) * _e279) + _e281);
        } else {
            phi_1386_ = 0f;
        }
        let _e293 = phi_1386_;
        phi_1396_ = _e293;
    }
    let _e295 = phi_1396_;
    let _e296 = (_e134 * _e276);
    let _e297 = _e296.xyz;
    if (ti && (_e296.w != 0f)) {
        phi_1418_ = (vec3(_e295) + _e297);
    } else {
        phi_1418_ = _e297;
    }
    let _e304 = phi_1418_;
    let _e310 = vec4<f32>(_e304.x, _e296.y, _e296.z, _e296.w);
    let _e316 = vec4<f32>(_e310.x, _e304.y, _e310.z, _e310.w);
    L1_ = vec4<f32>(_e316.x, _e316.y, _e304.z, _e316.w);
    return;
}

@fragment
fn main(@location(9) v1_: vec3<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(0) a1_: vec4<f32>, @location(1) @interpolate(flat, either) m1_: f32, @location(8) G4_: vec2<f32>, @location(7) @interpolate(flat, either) r3_: vec2<u32>, @location(5) R0_: vec4<f32>, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(3) @interpolate(flat, either) F0_: f32) -> @location(0) vec4<f32> {
    v1_1 = v1_;
    Q0_1 = Q0_;
    a1_1 = a1_;
    m1_1 = m1_;
    G4_1 = G4_;
    r3_1 = r3_;
    R0_1 = R0_;
    l1_1 = l1_;
    gl_FragCoord_1 = gl_FragCoord;
    F0_1 = F0_;
    main_1();
    let _e21 = L1_;
    return _e21;
}
