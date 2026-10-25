enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct qi {
    v2_: array<vec4<u32>>,
}

struct pi {
    v2_: array<vec4<u32>>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

struct ug {
    v2_: array<vec2<u32>>,
}

struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(16) override N7_: bool = false;
@id(17) override qe: bool = false;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> DD: qi;
@group(0) @binding(2)
var<storage> KB: pi;
@group(0) @binding(4)
var<storage> JB: vg;
var<private> N0_: vec4<f32>;
var<private> O0_: vec3<f32>;
@group(0) @binding(3)
var<storage> XC: ug;
var<private> S0_: f32;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var eb: sampler;

fn main_1() {
    var phi_1291_: bool;
    var phi_1292_: i32;
    var phi_1348_: f32;
    var phi_1301_: f32;
    var phi_1298_: bool;
    var phi_1296_: i32;
    var phi_1295_: i32;
    var phi_1293_: i32;
    var phi_1340_: f32;
    var phi_1339_: f32;
    var phi_1304_: bool;
    var phi_1338_: f32;
    var phi_1303_: bool;
    var phi_1309_: i32;
    var phi_1314_: bool;
    var phi_1316_: vec4<u32>;
    var phi_1315_: vec4<u32>;
    var phi_1356_: u32;
    var phi_1333_: vec4<u32>;
    var phi_1322_: bool;
    var phi_1358_: f32;
    var phi_1370_: f32;
    var phi_1362_: f32;
    var phi_661_: bool;
    var phi_1365_: f32;
    var phi_1380_: vec2<f32>;
    var phi_1379_: vec2<f32>;
    var phi_1390_: f32;
    var phi_1398_: vec2<f32>;
    var phi_1387_: f32;
    var phi_1374_: vec2<f32>;
    var phi_1397_: vec2<f32>;
    var phi_1386_: f32;
    var phi_1383_: f32;
    var phi_1373_: vec2<f32>;
    var phi_1409_: vec2<f32>;
    var phi_1334_: vec2<f32>;
    var phi_1408_: vec2<f32>;
    var phi_1459_: bool;
    var phi_1457_: f32;
    var phi_883_: bool;
    var phi_1460_: f32;
    var phi_1471_: bool;
    var phi_1481_: u32;
    var phi_1480_: u32;

    let _e78 = gl_VertexIndex_1;
    let _e83 = ((_e78 & 536870912i) != 0i);
    let _e84 = (_e78 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if N7_ {
                phi_1291_ = true;
                break;
            }
            if qe {
                phi_1291_ = true;
                break;
            }
            phi_1291_ = false;
            break;
        }
    }
    let _e87 = phi_1291_;
    if _e87 {
        let _e88 = select(5i, 6i, _e83);
        let _e94 = (_e84 & ((1i << bitcast<u32>(_e88)) - 1i));
        let _e95 = select(1i, 2i, _e83);
        let _e101 = (_e94 & ((1i << bitcast<u32>(_e95)) - 1i));
        phi_1292_ = _e101;
        if (N7_ && !(_e83)) {
            phi_1292_ = (_e101 + 1i);
        }
        let _e106 = phi_1292_;
        phi_1348_ = select(1f, 0f, ((_e106 == 0i) || (_e106 == 3i)));
        phi_1301_ = select(-1f, 1f, (_e106 < 2i));
        phi_1298_ = false;
        phi_1296_ = (_e84 >> bitcast<u32>(_e88));
        phi_1295_ = 8i;
        phi_1293_ = (_e94 >> bitcast<u32>(_e95));
    } else {
        let _e113 = select(4i, 5i, _e83);
        let _e119 = (_e84 & ((1i << bitcast<u32>(_e113)) - 1i));
        let _e123 = (!(_e83) && (_e119 == 9i));
        phi_1348_ = 1f;
        phi_1301_ = 0f;
        phi_1298_ = _e123;
        phi_1296_ = (_e84 >> bitcast<u32>(_e113));
        phi_1295_ = select(8i, 17i, _e83);
        phi_1293_ = select(_e119, 0i, _e123);
    }
    let _e126 = phi_1348_;
    let _e128 = phi_1301_;
    let _e130 = phi_1298_;
    let _e132 = phi_1296_;
    let _e134 = phi_1295_;
    let _e136 = phi_1293_;
    let _e138 = min(_e136, (_e134 - 1i));
    let _e140 = ((_e132 * _e134) + _e138);
    let _e145 = textureLoad(VB, vec2<i32>((_e140 & 2047i), (_e140 >> bitcast<u32>(11i))), 0i);
    let _e152 = DD.v2_[(max((_e145.w & 65535u), 1u) - 1u)];
    let _e154 = bitcast<vec2<f32>>(_e152.xy);
    let _e156 = (_e152.z & 65535u);
    let _e158 = (_e156 * 4u);
    let _e161 = KB.v2_[_e158];
    let _e162 = bitcast<vec4<f32>>(_e161);
    let _e169 = mat2x2<f32>(vec2<f32>(_e162.x, _e162.y), vec2<f32>(_e162.z, _e162.w));
    let _e173 = KB.v2_[(_e158 + 1u)];
    let _e175 = bitcast<vec2<f32>>(_e173.xy);
    let _e181 = KB.v2_[(_e158 + 2u)];
    let _e183 = (_e145.w & 8388608u);
    if N7_ {
        phi_1338_ = _e128;
        phi_1303_ = false;
    } else {
        if qe {
            let _e184 = (_e183 != 0u);
            phi_1340_ = _e128;
            if _e184 {
                phi_1340_ = -(_e128);
            }
            let _e187 = phi_1340_;
            phi_1339_ = _e187;
            phi_1304_ = _e184;
        } else {
            phi_1339_ = _e128;
            phi_1304_ = (((_e183 != 0u) && !(_e83)) && !(_e130));
        }
        let _e194 = phi_1339_;
        let _e196 = phi_1304_;
        phi_1338_ = _e194;
        phi_1303_ = _e196;
    }
    let _e198 = phi_1338_;
    let _e200 = phi_1303_;
    phi_1309_ = _e136;
    if _e200 {
        phi_1309_ = (_e136 - 1i);
    }
    let _e203 = phi_1309_;
    phi_1356_ = _e145.w;
    phi_1333_ = _e145;
    if (_e203 != _e138) {
        let _e206 = ((_e140 + _e203) - _e138);
        let _e211 = textureLoad(VB, vec2<i32>((_e206 & 2047i), (_e206 >> bitcast<u32>(11i))), 0i);
        if ((_e211.w & 8454143u) != (_e145.w & 8454143u)) {
            if N7_ {
                phi_1314_ = (_e154.x != 0f);
            } else {
                phi_1314_ = true;
            }
            let _e219 = phi_1314_;
            phi_1316_ = _e145;
            if _e219 {
                let _e220 = bitcast<i32>(_e152.w);
                let _e225 = textureLoad(VB, vec2<i32>((_e220 & 2047i), (_e220 >> bitcast<u32>(11i))), 0i);
                phi_1316_ = _e225;
            }
            let _e227 = phi_1316_;
            phi_1315_ = _e227;
        } else {
            phi_1315_ = _e211;
        }
        let _e229 = phi_1315_;
        phi_1356_ = ((_e229.w & 4286578687u) | _e183);
        phi_1333_ = _e229;
    }
    let _e234 = phi_1356_;
    let _e236 = phi_1333_;
    switch bitcast<i32>(0u) {
        default: {
            if N7_ {
                phi_1322_ = true;
                break;
            }
            if qe {
                phi_1322_ = true;
                break;
            }
            phi_1322_ = false;
            break;
        }
    }
    let _e239 = phi_1322_;
    if _e239 {
        let _e242 = (_e198 * sign(determinant(_e169)));
        let _e245 = (f32(_e236.z) * 0.0000000014629181f);
        let _e249 = vec2<f32>(sin(_e245), -(cos(_e245)));
        let _e253 = select(0f, _e242, (_e126 == 0f));
        phi_1397_ = _e249;
        phi_1386_ = _e253;
        phi_1383_ = _e242;
        phi_1373_ = _e249;
        if N7_ {
            let _e255 = ((_e234 & 1048576u) != 0u);
            phi_1358_ = _e242;
            if _e255 {
                phi_1358_ = min(_e242, 0f);
            }
            let _e258 = phi_1358_;
            phi_1370_ = _e258;
            if ((_e234 & 524288u) != 0u) {
                phi_1370_ = max(_e258, 0f);
            }
            let _e263 = phi_1370_;
            let _e264 = (_e234 & 469762048u);
            phi_1398_ = _e249;
            phi_1387_ = _e253;
            phi_1374_ = _e249;
            if (_e264 > 201326592u) {
                let _e269 = f32((_e236.z & 65535u));
                let _e270 = (_e269 * 0.000015259022f);
                let _e274 = sqrt(max((1f - (_e270 * _e270)), 0f));
                phi_1362_ = _e274;
                if (((_e234 & 4194304u) != 0u) == _e255) {
                    phi_1362_ = -(_e274);
                }
                let _e278 = phi_1362_;
                let _e283 = (mat2x2<f32>(vec2<f32>(_e270, _e278), vec2<f32>(-(_e278), _e270)) * _e249);
                let _e284 = (_e264 == 268435456u);
                phi_661_ = _e284;
                if !(_e284) {
                    phi_661_ = ((_e264 != 402653184u) && (_e270 < 0.25f));
                }
                let _e290 = phi_661_;
                let _e292 = ((_e234 & 2097152u) != 0u);
                if (_e264 == 402653184u) {
                    phi_1379_ = (_e249 + _e283);
                } else {
                    phi_1380_ = _e249;
                    if (_e292 || !(_e290)) {
                        if _e290 {
                            phi_1365_ = _e270;
                        } else {
                            phi_1365_ = (65535f / _e269);
                        }
                        let _e299 = phi_1365_;
                        phi_1380_ = (_e283 * _e299);
                    }
                    let _e302 = phi_1380_;
                    phi_1379_ = _e302;
                }
                let _e304 = phi_1379_;
                phi_1390_ = _e253;
                if (!(_e83) && _e290) {
                    phi_1390_ = (0.5f * _e263);
                }
                let _e312 = phi_1390_;
                phi_1398_ = select(_e249, _e283, vec2((_e290 || _e292)));
                phi_1387_ = _e312;
                phi_1374_ = _e304;
            }
            let _e314 = phi_1398_;
            let _e316 = phi_1387_;
            let _e318 = phi_1374_;
            phi_1397_ = _e314;
            phi_1386_ = _e316;
            phi_1383_ = _e263;
            phi_1373_ = _e318;
        }
        let _e320 = phi_1397_;
        let _e322 = phi_1386_;
        let _e324 = phi_1383_;
        let _e326 = phi_1373_;
        let _e331 = ((_e169 * (bitcast<vec2<f32>>(_e236.xy) + (_e326 * (_e324 * bitcast<f32>(_e173.z))))) + _e175);
        phi_1409_ = _e331;
        if (_e322 != 0f) {
            phi_1409_ = (_e331 + (sign((_e320 * _naga_inverse_2x2_f32(_e169))) * _e322));
        }
        let _e339 = phi_1409_;
        phi_1408_ = _e339;
    } else {
        if _e130 {
            phi_1334_ = _e154;
        } else {
            phi_1334_ = bitcast<vec2<f32>>(_e236.xy);
        }
        let _e343 = phi_1334_;
        phi_1408_ = ((_e169 * _e343) + _e175);
    }
    let _e347 = phi_1408_;
    if kj {
        let _e348 = (_e156 * 8u);
        let _e352 = JB.v2_[(_e348 + 2u)];
        let _e363 = JB.v2_[(_e348 + 3u)];
        if any((_e352 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e368 = ((mat2x2<f32>(vec2<f32>(_e352.x, _e352.y), vec2<f32>(_e352.z, _e352.w)) * _e347) + _e363.xy);
            unnamed.gl_ClipDistance[0i] = (_e368.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e368.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e368.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e368.y);
        } else {
            let _e384 = (_e363.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e384;
            unnamed.gl_ClipDistance[2i] = _e384;
            unnamed.gl_ClipDistance[1i] = _e384;
            unnamed.gl_ClipDistance[0i] = _e384;
        }
    }
    if ((_e78 & 268435456i) != 0i) {
        N0_ = vec4<f32>(0f, 0f, 0f, 0f);
        O0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e395 = XC.v2_[_e156];
        let _e397 = (_e395.x & 15u);
        phi_1459_ = false;
        if lj {
            let _e400 = ((_e395.x >> bitcast<u32>(4i)) & 15u);
            S0_ = f32(_e400);
            phi_1459_ = (_e400 != 0u);
        }
        let _e404 = phi_1459_;
        if (_e397 == 1u) {
            N0_ = unpack4x8unorm(_e395.y);
            if _e404 {
                let _e409 = N0_[3u];
                N0_[3u] = (_e409 * _e126);
            } else {
                let _e411 = N0_;
                N0_ = (_e411 * _e126);
            }
        } else {
            let _e413 = (_e156 * 8u);
            let _e416 = JB.v2_[_e413];
            let _e427 = JB.v2_[(_e413 + 1u)];
            let _e432 = ((mat2x2<f32>(vec2<f32>(_e416.x, _e416.y), vec2<f32>(_e416.z, _e416.w)) * _e347) + _e427.xy);
            let _e443 = ((_e427.w + (f32(_e397) * 0.125f)) + (max(_e427.z, 0f) * 0.00024414063f));
            if (_e427.z < 0f) {
                phi_1457_ = -(_e443);
            } else {
                phi_1457_ = _e443;
            }
            let _e446 = phi_1457_;
            N0_ = vec4<f32>(_e432.x, _e432.y, _e446, (((_e126 * -0.5f) - 0.25f) - round((bitcast<f32>(_e395.y) * 255f))));
        }
        phi_883_ = nj;
        if nj {
            phi_883_ = ((_e395.x & 2048u) != 0u);
        }
        let _e456 = phi_883_;
        if _e456 {
            let _e457 = (_e156 * 8u);
            let _e461 = JB.v2_[(_e457 + 4u)];
            let _e472 = JB.v2_[(_e457 + 5u)];
            let _e475 = ((mat2x2<f32>(vec2<f32>(_e461.x, _e461.y), vec2<f32>(_e461.z, _e461.w)) * _e347) + _e472.xy);
            phi_1460_ = (1f + _e472.z);
            if ((_e395.x & 4096u) != 0u) {
                phi_1460_ = (-1f - f32(((_e395.x & 24576u) >> bitcast<u32>(13u))));
            }
            let _e486 = phi_1460_;
            O0_ = vec3<f32>(_e475.x, _e475.y, _e486);
        } else {
            O0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e491 = j.Sg;
    let _e493 = j.Tg;
    let _e501 = vec4<f32>(((_e347.x * _e491) - 1f), ((_e347.y * _e493) - sign(_e493)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if N7_ {
                phi_1471_ = true;
                break;
            }
            if qe {
                phi_1471_ = true;
                break;
            }
            phi_1471_ = false;
            break;
        }
    }
    let _e504 = phi_1471_;
    if _e504 {
        let _e506 = u32((_e126 * 254f));
        phi_1481_ = _e506;
        if ((_e78 & 1073741824i) == 0i) {
            phi_1481_ = (_e506 + bitcast<u32>(1i));
        }
        let _e511 = phi_1481_;
        phi_1480_ = _e511;
    } else {
        phi_1480_ = 255u;
    }
    let _e513 = phi_1480_;
    unnamed.gl_Position = vec4<f32>(_e501.x, _e501.y, ((f32(((_e181.x << bitcast<u32>(8u)) | _e513)) * 0.000000059604645f) + 0.000000029802322f), _e501.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = unnamed.gl_Position;
    let _e10 = unnamed.gl_ClipDistance;
    let _e11 = N0_;
    let _e12 = O0_;
    let _e13 = S0_;
    return VertexOutput(_e9, _e10, _e11, _e12, _e13);
}

fn _naga_inverse_2x2_f32(m: mat2x2<f32>) -> mat2x2<f32> {
    var adj: mat2x2<f32>;
    adj[0][0] = m[1][1];
    adj[0][1] = -m[0][1];
    adj[1][0] = -m[1][0];
    adj[1][1] = m[0][0];

    let det: f32 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
    return adj * (1 / det);
}
