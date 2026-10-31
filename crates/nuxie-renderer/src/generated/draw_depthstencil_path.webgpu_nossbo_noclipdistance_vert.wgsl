struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(16) override E6_: bool = false;
@id(17) override qe: bool = false;
@id(2) override mj: bool = true;
@id(4) override oj: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(4)
var CD: texture_2d<u32>;
@group(0) @binding(2)
var JB: texture_2d<u32>;
@group(0) @binding(3)
var KB: texture_2d<f32>;
var<private> M0_: vec4<f32>;
var<private> N0_: vec3<f32>;
var<private> R0_: f32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_1261_: bool;
    var phi_1262_: i32;
    var phi_1324_: f32;
    var phi_1274_: f32;
    var phi_1270_: bool;
    var phi_1266_: i32;
    var phi_1265_: i32;
    var phi_1263_: i32;
    var phi_1367_: f32;
    var phi_1315_: f32;
    var phi_1314_: f32;
    var phi_1277_: bool;
    var phi_1313_: f32;
    var phi_1276_: bool;
    var phi_1283_: i32;
    var phi_1288_: bool;
    var phi_1290_: vec4<u32>;
    var phi_1289_: vec4<u32>;
    var phi_1332_: u32;
    var phi_1308_: vec4<u32>;
    var phi_1297_: bool;
    var phi_1334_: f32;
    var phi_1346_: f32;
    var phi_1338_: f32;
    var phi_638_: bool;
    var phi_1341_: f32;
    var phi_1356_: vec2<f32>;
    var phi_1355_: vec2<f32>;
    var phi_1387_: f32;
    var phi_1395_: vec2<f32>;
    var phi_1384_: f32;
    var phi_1350_: vec2<f32>;
    var phi_1394_: vec2<f32>;
    var phi_1383_: f32;
    var phi_1359_: f32;
    var phi_1349_: vec2<f32>;
    var phi_1406_: vec2<f32>;
    var phi_1309_: vec2<f32>;
    var phi_1405_: vec2<f32>;
    var phi_1453_: bool;
    var phi_1451_: f32;
    var phi_882_: bool;
    var phi_1454_: f32;
    var phi_1463_: bool;
    var phi_1473_: u32;
    var phi_1472_: u32;

    let _e77 = gl_VertexIndex_1;
    let _e82 = ((_e77 & 536870912i) != 0i);
    let _e83 = (_e77 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if E6_ {
                phi_1261_ = true;
                break;
            }
            if qe {
                phi_1261_ = true;
                break;
            }
            phi_1261_ = false;
            break;
        }
    }
    let _e86 = phi_1261_;
    if _e86 {
        let _e87 = select(5i, 6i, _e82);
        let _e93 = (_e83 & ((1i << bitcast<u32>(_e87)) - 1i));
        let _e94 = select(1i, 2i, _e82);
        let _e100 = (_e93 & ((1i << bitcast<u32>(_e94)) - 1i));
        phi_1262_ = _e100;
        if (E6_ && !(_e82)) {
            phi_1262_ = (_e100 + 1i);
        }
        let _e105 = phi_1262_;
        phi_1324_ = select(1f, 0f, ((_e105 == 0i) || (_e105 == 3i)));
        phi_1274_ = select(-1f, 1f, (_e105 < 2i));
        phi_1270_ = false;
        phi_1266_ = (_e83 >> bitcast<u32>(_e87));
        phi_1265_ = 8i;
        phi_1263_ = (_e93 >> bitcast<u32>(_e94));
    } else {
        let _e112 = select(4i, 5i, _e82);
        let _e118 = (_e83 & ((1i << bitcast<u32>(_e112)) - 1i));
        let _e122 = (!(_e82) && (_e118 == 9i));
        phi_1324_ = 1f;
        phi_1274_ = 0f;
        phi_1270_ = _e122;
        phi_1266_ = (_e83 >> bitcast<u32>(_e112));
        phi_1265_ = select(8i, 17i, _e82);
        phi_1263_ = select(_e118, 0i, _e122);
    }
    let _e125 = phi_1324_;
    let _e127 = phi_1274_;
    let _e129 = phi_1270_;
    let _e131 = phi_1266_;
    let _e133 = phi_1265_;
    let _e135 = phi_1263_;
    let _e137 = min(_e135, (_e133 - 1i));
    let _e139 = ((_e131 * _e133) + _e137);
    let _e144 = textureLoad(VB, vec2<i32>((_e139 & 2047i), (_e139 >> bitcast<u32>(11i))), 0i);
    let _e148 = (max((_e144.w & 65535u), 1u) - 1u);
    let _e155 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e148 & 255u)), bitcast<i32>((_e148 >> bitcast<u32>(8i)))), 0i);
    let _e157 = bitcast<vec2<f32>>(_e155.xy);
    let _e159 = (_e155.z & 65535u);
    let _e161 = (_e159 * 4u);
    let _e168 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e161 & 255u)), bitcast<i32>((_e161 >> bitcast<u32>(8i)))), 0i);
    let _e169 = bitcast<vec4<f32>>(_e168);
    let _e176 = mat2x2<f32>(vec2<f32>(_e169.x, _e169.y), vec2<f32>(_e169.z, _e169.w));
    let _e177 = (_e161 + 1u);
    let _e184 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e177 & 255u)), bitcast<i32>((_e177 >> bitcast<u32>(8i)))), 0i);
    let _e186 = bitcast<vec2<f32>>(_e184.xy);
    phi_1367_ = 0f;
    if E6_ {
        let _e190 = (_e161 + 2u);
        let _e197 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e190 & 255u)), bitcast<i32>((_e190 >> bitcast<u32>(8i)))), 0i);
        phi_1367_ = bitcast<f32>(_e197.x);
    }
    let _e201 = phi_1367_;
    let _e202 = (_e144.w & 8388608u);
    if E6_ {
        phi_1313_ = _e127;
        phi_1276_ = false;
    } else {
        if qe {
            let _e203 = (_e202 != 0u);
            phi_1315_ = _e127;
            if _e203 {
                phi_1315_ = -(_e127);
            }
            let _e206 = phi_1315_;
            phi_1314_ = _e206;
            phi_1277_ = _e203;
        } else {
            phi_1314_ = _e127;
            phi_1277_ = (((_e202 != 0u) && !(_e82)) && !(_e129));
        }
        let _e213 = phi_1314_;
        let _e215 = phi_1277_;
        phi_1313_ = _e213;
        phi_1276_ = _e215;
    }
    let _e217 = phi_1313_;
    let _e219 = phi_1276_;
    phi_1283_ = _e135;
    if _e219 {
        phi_1283_ = (_e135 - 1i);
    }
    let _e222 = phi_1283_;
    phi_1332_ = _e144.w;
    phi_1308_ = _e144;
    if (_e222 != _e137) {
        let _e225 = ((_e139 + _e222) - _e137);
        let _e230 = textureLoad(VB, vec2<i32>((_e225 & 2047i), (_e225 >> bitcast<u32>(11i))), 0i);
        if ((_e230.w & 8454143u) != (_e144.w & 8454143u)) {
            if E6_ {
                phi_1288_ = (_e157.x != 0f);
            } else {
                phi_1288_ = true;
            }
            let _e238 = phi_1288_;
            phi_1290_ = _e144;
            if _e238 {
                let _e239 = bitcast<i32>(_e155.w);
                let _e244 = textureLoad(VB, vec2<i32>((_e239 & 2047i), (_e239 >> bitcast<u32>(11i))), 0i);
                phi_1290_ = _e244;
            }
            let _e246 = phi_1290_;
            phi_1289_ = _e246;
        } else {
            phi_1289_ = _e230;
        }
        let _e248 = phi_1289_;
        phi_1332_ = ((_e248.w & 4286578687u) | _e202);
        phi_1308_ = _e248;
    }
    let _e253 = phi_1332_;
    let _e255 = phi_1308_;
    switch bitcast<i32>(0u) {
        default: {
            if E6_ {
                phi_1297_ = true;
                break;
            }
            if qe {
                phi_1297_ = true;
                break;
            }
            phi_1297_ = false;
            break;
        }
    }
    let _e258 = phi_1297_;
    if _e258 {
        let _e261 = (_e217 * sign(determinant(_e176)));
        let _e264 = (f32(_e255.z) * 0.0000000014629181f);
        let _e268 = vec2<f32>(sin(_e264), -(cos(_e264)));
        let _e272 = select(0f, _e261, (_e125 == 0f));
        phi_1394_ = _e268;
        phi_1383_ = _e272;
        phi_1359_ = _e261;
        phi_1349_ = _e268;
        if E6_ {
            let _e274 = ((_e253 & 1048576u) != 0u);
            phi_1334_ = _e261;
            if _e274 {
                phi_1334_ = min(_e261, 0f);
            }
            let _e277 = phi_1334_;
            phi_1346_ = _e277;
            if ((_e253 & 524288u) != 0u) {
                phi_1346_ = max(_e277, 0f);
            }
            let _e282 = phi_1346_;
            let _e283 = (_e253 & 469762048u);
            phi_1395_ = _e268;
            phi_1384_ = _e272;
            phi_1350_ = _e268;
            if (_e283 > 201326592u) {
                let _e288 = f32((_e255.z & 65535u));
                let _e289 = (_e288 * 0.000015259022f);
                let _e293 = sqrt(max((1f - (_e289 * _e289)), 0f));
                phi_1338_ = _e293;
                if (((_e253 & 4194304u) != 0u) == _e274) {
                    phi_1338_ = -(_e293);
                }
                let _e297 = phi_1338_;
                let _e302 = (mat2x2<f32>(vec2<f32>(_e289, _e297), vec2<f32>(-(_e297), _e289)) * _e268);
                let _e303 = (_e283 == 268435456u);
                phi_638_ = _e303;
                if !(_e303) {
                    phi_638_ = ((_e283 != 402653184u) && (_e289 < 0.25f));
                }
                let _e309 = phi_638_;
                let _e311 = ((_e253 & 2097152u) != 0u);
                if (_e283 == 402653184u) {
                    phi_1355_ = (_e268 + _e302);
                } else {
                    phi_1356_ = _e268;
                    if (_e311 || !(_e309)) {
                        if _e309 {
                            phi_1341_ = _e289;
                        } else {
                            phi_1341_ = (65535f / _e288);
                        }
                        let _e318 = phi_1341_;
                        phi_1356_ = (_e302 * _e318);
                    }
                    let _e321 = phi_1356_;
                    phi_1355_ = _e321;
                }
                let _e323 = phi_1355_;
                phi_1387_ = _e272;
                if (!(_e82) && _e309) {
                    phi_1387_ = (0.5f * _e282);
                }
                let _e331 = phi_1387_;
                phi_1395_ = select(_e268, _e302, vec2((_e309 || _e311)));
                phi_1384_ = _e331;
                phi_1350_ = _e323;
            }
            let _e333 = phi_1395_;
            let _e335 = phi_1384_;
            let _e337 = phi_1350_;
            phi_1394_ = _e333;
            phi_1383_ = _e335;
            phi_1359_ = _e282;
            phi_1349_ = _e337;
        }
        let _e339 = phi_1394_;
        let _e341 = phi_1383_;
        let _e343 = phi_1359_;
        let _e345 = phi_1349_;
        let _e350 = ((_e176 * (bitcast<vec2<f32>>(_e255.xy) + (_e345 * (_e343 * _e201)))) + _e186);
        phi_1406_ = _e350;
        if (_e341 != 0f) {
            phi_1406_ = (_e350 + (sign((_e339 * _naga_inverse_2x2_f32(_e176))) * _e341));
        }
        let _e358 = phi_1406_;
        phi_1405_ = _e358;
    } else {
        if _e129 {
            phi_1309_ = _e157;
        } else {
            phi_1309_ = bitcast<vec2<f32>>(_e255.xy);
        }
        let _e362 = phi_1309_;
        phi_1405_ = ((_e176 * _e362) + _e186);
    }
    let _e366 = phi_1405_;
    if ((_e77 & 268435456i) != 0i) {
        M0_ = vec4<f32>(0f, 0f, 0f, 0f);
        N0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e367 = (_e184.z & 15u);
        phi_1453_ = false;
        if mj {
            let _e370 = ((_e184.z >> bitcast<u32>(4i)) & 15u);
            R0_ = f32(_e370);
            phi_1453_ = (_e370 != 0u);
        }
        let _e374 = phi_1453_;
        if (_e367 == 1u) {
            M0_ = unpack4x8unorm(_e184.w);
            if _e374 {
                let _e379 = M0_[3u];
                M0_[3u] = (_e379 * _e125);
            } else {
                let _e381 = M0_;
                M0_ = (_e381 * _e125);
            }
        } else {
            let _e383 = (_e159 * 8u);
            let _e390 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e383 & 255u)), bitcast<i32>((_e383 >> bitcast<u32>(8i)))), 0i);
            let _e398 = (_e383 + 1u);
            let _e405 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e398 & 255u)), bitcast<i32>((_e398 >> bitcast<u32>(8i)))), 0i);
            let _e410 = ((mat2x2<f32>(vec2<f32>(_e390.x, _e390.y), vec2<f32>(_e390.z, _e390.w)) * _e366) + _e405.xy);
            let _e421 = ((_e405.w + (f32(_e367) * 0.125f)) + (max(_e405.z, 0f) * 0.00024414063f));
            if (_e405.z < 0f) {
                phi_1451_ = -(_e421);
            } else {
                phi_1451_ = _e421;
            }
            let _e424 = phi_1451_;
            M0_ = vec4<f32>(_e410.x, _e410.y, _e424, (((_e125 * -0.5f) - 0.25f) - round((bitcast<f32>(_e184.w) * 255f))));
        }
        phi_882_ = oj;
        if oj {
            phi_882_ = ((_e184.z & 2048u) != 0u);
        }
        let _e434 = phi_882_;
        if _e434 {
            let _e435 = (_e159 * 8u);
            let _e436 = (_e435 + 4u);
            let _e443 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e436 & 255u)), bitcast<i32>((_e436 >> bitcast<u32>(8i)))), 0i);
            let _e451 = (_e435 + 5u);
            let _e458 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e451 & 255u)), bitcast<i32>((_e451 >> bitcast<u32>(8i)))), 0i);
            let _e461 = ((mat2x2<f32>(vec2<f32>(_e443.x, _e443.y), vec2<f32>(_e443.z, _e443.w)) * _e366) + _e458.xy);
            phi_1454_ = (1f + _e458.z);
            if ((_e184.z & 4096u) != 0u) {
                phi_1454_ = (-1f - f32(((_e184.z & 24576u) >> bitcast<u32>(13u))));
            }
            let _e472 = phi_1454_;
            N0_ = vec3<f32>(_e461.x, _e461.y, _e472);
        } else {
            N0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e477 = j.Sg;
    let _e479 = j.Tg;
    let _e487 = vec4<f32>(((_e366.x * _e477) - 1f), ((_e366.y * _e479) - sign(_e479)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if E6_ {
                phi_1463_ = true;
                break;
            }
            if qe {
                phi_1463_ = true;
                break;
            }
            phi_1463_ = false;
            break;
        }
    }
    let _e490 = phi_1463_;
    if _e490 {
        let _e492 = u32((_e125 * 254f));
        phi_1473_ = _e492;
        if ((_e77 & 1073741824i) == 0i) {
            phi_1473_ = (_e492 + bitcast<u32>(1i));
        }
        let _e497 = phi_1473_;
        phi_1472_ = _e497;
    } else {
        phi_1472_ = 255u;
    }
    let _e499 = phi_1472_;
    unnamed.gl_Position = vec4<f32>(_e487.x, _e487.y, ((f32((((_e184.z >> bitcast<u32>(16u)) << bitcast<u32>(8u)) | _e499)) * 0.000000059604645f) + 0.000000029802322f), _e487.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e8 = M0_;
    let _e9 = N0_;
    let _e10 = R0_;
    let _e11 = unnamed.gl_Position;
    return VertexOutput(_e8, _e9, _e10, _e11);
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
