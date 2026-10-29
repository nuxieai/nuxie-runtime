enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

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

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(16) override E6_: bool = false;
@id(17) override qe: bool = false;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
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
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_1348_: bool;
    var phi_1349_: i32;
    var phi_1411_: f32;
    var phi_1361_: f32;
    var phi_1357_: bool;
    var phi_1353_: i32;
    var phi_1352_: i32;
    var phi_1350_: i32;
    var phi_1454_: f32;
    var phi_1402_: f32;
    var phi_1401_: f32;
    var phi_1364_: bool;
    var phi_1400_: f32;
    var phi_1363_: bool;
    var phi_1370_: i32;
    var phi_1375_: bool;
    var phi_1377_: vec4<u32>;
    var phi_1376_: vec4<u32>;
    var phi_1419_: u32;
    var phi_1395_: vec4<u32>;
    var phi_1384_: bool;
    var phi_1421_: f32;
    var phi_1433_: f32;
    var phi_1425_: f32;
    var phi_687_: bool;
    var phi_1428_: f32;
    var phi_1443_: vec2<f32>;
    var phi_1442_: vec2<f32>;
    var phi_1474_: f32;
    var phi_1482_: vec2<f32>;
    var phi_1471_: f32;
    var phi_1437_: vec2<f32>;
    var phi_1481_: vec2<f32>;
    var phi_1470_: f32;
    var phi_1446_: f32;
    var phi_1436_: vec2<f32>;
    var phi_1493_: vec2<f32>;
    var phi_1396_: vec2<f32>;
    var phi_1492_: vec2<f32>;
    var phi_1544_: bool;
    var phi_1542_: f32;
    var phi_928_: bool;
    var phi_1545_: f32;
    var phi_1556_: bool;
    var phi_1566_: u32;
    var phi_1565_: u32;

    let _e78 = gl_VertexIndex_1;
    let _e83 = ((_e78 & 536870912i) != 0i);
    let _e84 = (_e78 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if E6_ {
                phi_1348_ = true;
                break;
            }
            if qe {
                phi_1348_ = true;
                break;
            }
            phi_1348_ = false;
            break;
        }
    }
    let _e87 = phi_1348_;
    if _e87 {
        let _e88 = select(5i, 6i, _e83);
        let _e94 = (_e84 & ((1i << bitcast<u32>(_e88)) - 1i));
        let _e95 = select(1i, 2i, _e83);
        let _e101 = (_e94 & ((1i << bitcast<u32>(_e95)) - 1i));
        phi_1349_ = _e101;
        if (E6_ && !(_e83)) {
            phi_1349_ = (_e101 + 1i);
        }
        let _e106 = phi_1349_;
        phi_1411_ = select(1f, 0f, ((_e106 == 0i) || (_e106 == 3i)));
        phi_1361_ = select(-1f, 1f, (_e106 < 2i));
        phi_1357_ = false;
        phi_1353_ = (_e84 >> bitcast<u32>(_e88));
        phi_1352_ = 8i;
        phi_1350_ = (_e94 >> bitcast<u32>(_e95));
    } else {
        let _e113 = select(4i, 5i, _e83);
        let _e119 = (_e84 & ((1i << bitcast<u32>(_e113)) - 1i));
        let _e123 = (!(_e83) && (_e119 == 9i));
        phi_1411_ = 1f;
        phi_1361_ = 0f;
        phi_1357_ = _e123;
        phi_1353_ = (_e84 >> bitcast<u32>(_e113));
        phi_1352_ = select(8i, 17i, _e83);
        phi_1350_ = select(_e119, 0i, _e123);
    }
    let _e126 = phi_1411_;
    let _e128 = phi_1361_;
    let _e130 = phi_1357_;
    let _e132 = phi_1353_;
    let _e134 = phi_1352_;
    let _e136 = phi_1350_;
    let _e138 = min(_e136, (_e134 - 1i));
    let _e140 = ((_e132 * _e134) + _e138);
    let _e145 = textureLoad(VB, vec2<i32>((_e140 & 2047i), (_e140 >> bitcast<u32>(11i))), 0i);
    let _e149 = (max((_e145.w & 65535u), 1u) - 1u);
    let _e156 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e149 & 255u)), bitcast<i32>((_e149 >> bitcast<u32>(8i)))), 0i);
    let _e158 = bitcast<vec2<f32>>(_e156.xy);
    let _e160 = (_e156.z & 65535u);
    let _e162 = (_e160 * 4u);
    let _e169 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e162 & 255u)), bitcast<i32>((_e162 >> bitcast<u32>(8i)))), 0i);
    let _e170 = bitcast<vec4<f32>>(_e169);
    let _e177 = mat2x2<f32>(vec2<f32>(_e170.x, _e170.y), vec2<f32>(_e170.z, _e170.w));
    let _e178 = (_e162 + 1u);
    let _e185 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e178 & 255u)), bitcast<i32>((_e178 >> bitcast<u32>(8i)))), 0i);
    let _e187 = bitcast<vec2<f32>>(_e185.xy);
    phi_1454_ = 0f;
    if E6_ {
        let _e191 = (_e162 + 2u);
        let _e198 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e191 & 255u)), bitcast<i32>((_e191 >> bitcast<u32>(8i)))), 0i);
        phi_1454_ = bitcast<f32>(_e198.x);
    }
    let _e202 = phi_1454_;
    let _e203 = (_e145.w & 8388608u);
    if E6_ {
        phi_1400_ = _e128;
        phi_1363_ = false;
    } else {
        if qe {
            let _e204 = (_e203 != 0u);
            phi_1402_ = _e128;
            if _e204 {
                phi_1402_ = -(_e128);
            }
            let _e207 = phi_1402_;
            phi_1401_ = _e207;
            phi_1364_ = _e204;
        } else {
            phi_1401_ = _e128;
            phi_1364_ = (((_e203 != 0u) && !(_e83)) && !(_e130));
        }
        let _e214 = phi_1401_;
        let _e216 = phi_1364_;
        phi_1400_ = _e214;
        phi_1363_ = _e216;
    }
    let _e218 = phi_1400_;
    let _e220 = phi_1363_;
    phi_1370_ = _e136;
    if _e220 {
        phi_1370_ = (_e136 - 1i);
    }
    let _e223 = phi_1370_;
    phi_1419_ = _e145.w;
    phi_1395_ = _e145;
    if (_e223 != _e138) {
        let _e226 = ((_e140 + _e223) - _e138);
        let _e231 = textureLoad(VB, vec2<i32>((_e226 & 2047i), (_e226 >> bitcast<u32>(11i))), 0i);
        if ((_e231.w & 8454143u) != (_e145.w & 8454143u)) {
            if E6_ {
                phi_1375_ = (_e158.x != 0f);
            } else {
                phi_1375_ = true;
            }
            let _e239 = phi_1375_;
            phi_1377_ = _e145;
            if _e239 {
                let _e240 = bitcast<i32>(_e156.w);
                let _e245 = textureLoad(VB, vec2<i32>((_e240 & 2047i), (_e240 >> bitcast<u32>(11i))), 0i);
                phi_1377_ = _e245;
            }
            let _e247 = phi_1377_;
            phi_1376_ = _e247;
        } else {
            phi_1376_ = _e231;
        }
        let _e249 = phi_1376_;
        phi_1419_ = ((_e249.w & 4286578687u) | _e203);
        phi_1395_ = _e249;
    }
    let _e254 = phi_1419_;
    let _e256 = phi_1395_;
    switch bitcast<i32>(0u) {
        default: {
            if E6_ {
                phi_1384_ = true;
                break;
            }
            if qe {
                phi_1384_ = true;
                break;
            }
            phi_1384_ = false;
            break;
        }
    }
    let _e259 = phi_1384_;
    if _e259 {
        let _e262 = (_e218 * sign(determinant(_e177)));
        let _e265 = (f32(_e256.z) * 0.0000000014629181f);
        let _e269 = vec2<f32>(sin(_e265), -(cos(_e265)));
        let _e273 = select(0f, _e262, (_e126 == 0f));
        phi_1481_ = _e269;
        phi_1470_ = _e273;
        phi_1446_ = _e262;
        phi_1436_ = _e269;
        if E6_ {
            let _e275 = ((_e254 & 1048576u) != 0u);
            phi_1421_ = _e262;
            if _e275 {
                phi_1421_ = min(_e262, 0f);
            }
            let _e278 = phi_1421_;
            phi_1433_ = _e278;
            if ((_e254 & 524288u) != 0u) {
                phi_1433_ = max(_e278, 0f);
            }
            let _e283 = phi_1433_;
            let _e284 = (_e254 & 469762048u);
            phi_1482_ = _e269;
            phi_1471_ = _e273;
            phi_1437_ = _e269;
            if (_e284 > 201326592u) {
                let _e289 = f32((_e256.z & 65535u));
                let _e290 = (_e289 * 0.000015259022f);
                let _e294 = sqrt(max((1f - (_e290 * _e290)), 0f));
                phi_1425_ = _e294;
                if (((_e254 & 4194304u) != 0u) == _e275) {
                    phi_1425_ = -(_e294);
                }
                let _e298 = phi_1425_;
                let _e303 = (mat2x2<f32>(vec2<f32>(_e290, _e298), vec2<f32>(-(_e298), _e290)) * _e269);
                let _e304 = (_e284 == 268435456u);
                phi_687_ = _e304;
                if !(_e304) {
                    phi_687_ = ((_e284 != 402653184u) && (_e290 < 0.25f));
                }
                let _e310 = phi_687_;
                let _e312 = ((_e254 & 2097152u) != 0u);
                if (_e284 == 402653184u) {
                    phi_1442_ = (_e269 + _e303);
                } else {
                    phi_1443_ = _e269;
                    if (_e312 || !(_e310)) {
                        if _e310 {
                            phi_1428_ = _e290;
                        } else {
                            phi_1428_ = (65535f / _e289);
                        }
                        let _e319 = phi_1428_;
                        phi_1443_ = (_e303 * _e319);
                    }
                    let _e322 = phi_1443_;
                    phi_1442_ = _e322;
                }
                let _e324 = phi_1442_;
                phi_1474_ = _e273;
                if (!(_e83) && _e310) {
                    phi_1474_ = (0.5f * _e283);
                }
                let _e332 = phi_1474_;
                phi_1482_ = select(_e269, _e303, vec2((_e310 || _e312)));
                phi_1471_ = _e332;
                phi_1437_ = _e324;
            }
            let _e334 = phi_1482_;
            let _e336 = phi_1471_;
            let _e338 = phi_1437_;
            phi_1481_ = _e334;
            phi_1470_ = _e336;
            phi_1446_ = _e283;
            phi_1436_ = _e338;
        }
        let _e340 = phi_1481_;
        let _e342 = phi_1470_;
        let _e344 = phi_1446_;
        let _e346 = phi_1436_;
        let _e351 = ((_e177 * (bitcast<vec2<f32>>(_e256.xy) + (_e346 * (_e344 * _e202)))) + _e187);
        phi_1493_ = _e351;
        if (_e342 != 0f) {
            phi_1493_ = (_e351 + (sign((_e340 * _naga_inverse_2x2_f32(_e177))) * _e342));
        }
        let _e359 = phi_1493_;
        phi_1492_ = _e359;
    } else {
        if _e130 {
            phi_1396_ = _e158;
        } else {
            phi_1396_ = bitcast<vec2<f32>>(_e256.xy);
        }
        let _e363 = phi_1396_;
        phi_1492_ = ((_e177 * _e363) + _e187);
    }
    let _e367 = phi_1492_;
    if kj {
        let _e368 = (_e160 * 8u);
        let _e369 = (_e368 + 2u);
        let _e376 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e369 & 255u)), bitcast<i32>((_e369 >> bitcast<u32>(8i)))), 0i);
        let _e384 = (_e368 + 3u);
        let _e391 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e384 & 255u)), bitcast<i32>((_e384 >> bitcast<u32>(8i)))), 0i);
        if any((_e376 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e396 = ((mat2x2<f32>(vec2<f32>(_e376.x, _e376.y), vec2<f32>(_e376.z, _e376.w)) * _e367) + _e391.xy);
            unnamed.gl_ClipDistance[0i] = (_e396.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e396.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e396.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e396.y);
        } else {
            let _e412 = (_e391.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e412;
            unnamed.gl_ClipDistance[2i] = _e412;
            unnamed.gl_ClipDistance[1i] = _e412;
            unnamed.gl_ClipDistance[0i] = _e412;
        }
    }
    if ((_e78 & 268435456i) != 0i) {
        M0_ = vec4<f32>(0f, 0f, 0f, 0f);
        N0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e421 = (_e185.z & 15u);
        phi_1544_ = false;
        if lj {
            let _e424 = ((_e185.z >> bitcast<u32>(4i)) & 15u);
            R0_ = f32(_e424);
            phi_1544_ = (_e424 != 0u);
        }
        let _e428 = phi_1544_;
        if (_e421 == 1u) {
            M0_ = unpack4x8unorm(_e185.w);
            if _e428 {
                let _e433 = M0_[3u];
                M0_[3u] = (_e433 * _e126);
            } else {
                let _e435 = M0_;
                M0_ = (_e435 * _e126);
            }
        } else {
            let _e437 = (_e160 * 8u);
            let _e444 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e437 & 255u)), bitcast<i32>((_e437 >> bitcast<u32>(8i)))), 0i);
            let _e452 = (_e437 + 1u);
            let _e459 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e452 & 255u)), bitcast<i32>((_e452 >> bitcast<u32>(8i)))), 0i);
            let _e464 = ((mat2x2<f32>(vec2<f32>(_e444.x, _e444.y), vec2<f32>(_e444.z, _e444.w)) * _e367) + _e459.xy);
            let _e475 = ((_e459.w + (f32(_e421) * 0.125f)) + (max(_e459.z, 0f) * 0.00024414063f));
            if (_e459.z < 0f) {
                phi_1542_ = -(_e475);
            } else {
                phi_1542_ = _e475;
            }
            let _e478 = phi_1542_;
            M0_ = vec4<f32>(_e464.x, _e464.y, _e478, (((_e126 * -0.5f) - 0.25f) - round((bitcast<f32>(_e185.w) * 255f))));
        }
        phi_928_ = nj;
        if nj {
            phi_928_ = ((_e185.z & 2048u) != 0u);
        }
        let _e488 = phi_928_;
        if _e488 {
            let _e489 = (_e160 * 8u);
            let _e490 = (_e489 + 4u);
            let _e497 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e490 & 255u)), bitcast<i32>((_e490 >> bitcast<u32>(8i)))), 0i);
            let _e505 = (_e489 + 5u);
            let _e512 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e505 & 255u)), bitcast<i32>((_e505 >> bitcast<u32>(8i)))), 0i);
            let _e515 = ((mat2x2<f32>(vec2<f32>(_e497.x, _e497.y), vec2<f32>(_e497.z, _e497.w)) * _e367) + _e512.xy);
            phi_1545_ = (1f + _e512.z);
            if ((_e185.z & 4096u) != 0u) {
                phi_1545_ = (-1f - f32(((_e185.z & 24576u) >> bitcast<u32>(13u))));
            }
            let _e526 = phi_1545_;
            N0_ = vec3<f32>(_e515.x, _e515.y, _e526);
        } else {
            N0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e531 = j.Rg;
    let _e533 = j.Sg;
    let _e541 = vec4<f32>(((_e367.x * _e531) - 1f), ((_e367.y * _e533) - sign(_e533)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if E6_ {
                phi_1556_ = true;
                break;
            }
            if qe {
                phi_1556_ = true;
                break;
            }
            phi_1556_ = false;
            break;
        }
    }
    let _e544 = phi_1556_;
    if _e544 {
        let _e546 = u32((_e126 * 254f));
        phi_1566_ = _e546;
        if ((_e78 & 1073741824i) == 0i) {
            phi_1566_ = (_e546 + bitcast<u32>(1i));
        }
        let _e551 = phi_1566_;
        phi_1565_ = _e551;
    } else {
        phi_1565_ = 255u;
    }
    let _e553 = phi_1565_;
    unnamed.gl_Position = vec4<f32>(_e541.x, _e541.y, ((f32((((_e185.z >> bitcast<u32>(16u)) << bitcast<u32>(8u)) | _e553)) * 0.000000059604645f) + 0.000000029802322f), _e541.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = unnamed.gl_Position;
    let _e10 = unnamed.gl_ClipDistance;
    let _e11 = M0_;
    let _e12 = N0_;
    let _e13 = R0_;
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
