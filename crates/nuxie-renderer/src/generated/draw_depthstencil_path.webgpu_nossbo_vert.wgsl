enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(15) override K7_: bool = false;
@id(16) override fe: bool = false;
@id(1) override Xi: bool = true;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(5)
var BD: texture_2d<u32>;
@group(0) @binding(2)
var KB: texture_2d<u32>;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> O0_: vec4<f32>;
var<private> U0_: vec3<f32>;
@group(0) @binding(3)
var VC: texture_2d<u32>;
var<private> P0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;

fn main_1() {
    var phi_1337_: bool;
    var phi_1338_: i32;
    var phi_1394_: f32;
    var phi_1347_: f32;
    var phi_1344_: bool;
    var phi_1342_: i32;
    var phi_1341_: i32;
    var phi_1339_: i32;
    var phi_1386_: f32;
    var phi_1385_: f32;
    var phi_1350_: bool;
    var phi_1384_: f32;
    var phi_1349_: bool;
    var phi_1355_: i32;
    var phi_1360_: bool;
    var phi_1362_: vec4<u32>;
    var phi_1361_: vec4<u32>;
    var phi_1402_: u32;
    var phi_1379_: vec4<u32>;
    var phi_1368_: bool;
    var phi_1404_: f32;
    var phi_1416_: f32;
    var phi_1408_: f32;
    var phi_685_: bool;
    var phi_1411_: f32;
    var phi_1426_: vec2<f32>;
    var phi_1425_: vec2<f32>;
    var phi_1436_: f32;
    var phi_1444_: vec2<f32>;
    var phi_1433_: f32;
    var phi_1420_: vec2<f32>;
    var phi_1443_: vec2<f32>;
    var phi_1432_: f32;
    var phi_1429_: f32;
    var phi_1419_: vec2<f32>;
    var phi_1455_: vec2<f32>;
    var phi_1380_: vec2<f32>;
    var phi_1454_: vec2<f32>;
    var phi_1505_: bool;
    var phi_1503_: f32;
    var phi_939_: bool;
    var phi_1515_: bool;
    var phi_1524_: u32;
    var phi_1523_: u32;

    let _e75 = gl_VertexIndex_1;
    let _e80 = ((_e75 & 536870912i) != 0i);
    let _e81 = (_e75 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1337_ = true;
                break;
            }
            if fe {
                phi_1337_ = true;
                break;
            }
            phi_1337_ = false;
            break;
        }
    }
    let _e84 = phi_1337_;
    if _e84 {
        let _e85 = select(5i, 6i, _e80);
        let _e91 = (_e81 & ((1i << bitcast<u32>(_e85)) - 1i));
        let _e92 = select(1i, 2i, _e80);
        let _e98 = (_e91 & ((1i << bitcast<u32>(_e92)) - 1i));
        phi_1338_ = _e98;
        if (K7_ && !(_e80)) {
            phi_1338_ = (_e98 + 1i);
        }
        let _e103 = phi_1338_;
        phi_1394_ = select(1f, 0f, ((_e103 == 0i) || (_e103 == 3i)));
        phi_1347_ = select(-1f, 1f, (_e103 < 2i));
        phi_1344_ = false;
        phi_1342_ = (_e81 >> bitcast<u32>(_e85));
        phi_1341_ = 8i;
        phi_1339_ = (_e91 >> bitcast<u32>(_e92));
    } else {
        let _e110 = select(4i, 5i, _e80);
        let _e116 = (_e81 & ((1i << bitcast<u32>(_e110)) - 1i));
        let _e120 = (!(_e80) && (_e116 == 9i));
        phi_1394_ = 1f;
        phi_1347_ = 0f;
        phi_1344_ = _e120;
        phi_1342_ = (_e81 >> bitcast<u32>(_e110));
        phi_1341_ = select(8i, 17i, _e80);
        phi_1339_ = select(_e116, 0i, _e120);
    }
    let _e123 = phi_1394_;
    let _e125 = phi_1347_;
    let _e127 = phi_1344_;
    let _e129 = phi_1342_;
    let _e131 = phi_1341_;
    let _e133 = phi_1339_;
    let _e135 = min(_e133, (_e131 - 1i));
    let _e137 = ((_e129 * _e131) + _e135);
    let _e142 = textureLoad(UB, vec2<i32>((_e137 & 2047i), (_e137 >> bitcast<u32>(11i))), 0i);
    let _e146 = (max((_e142.w & 65535u), 1u) - 1u);
    let _e153 = textureLoad(BD, vec2<i32>(bitcast<i32>((_e146 & 255u)), bitcast<i32>((_e146 >> bitcast<u32>(8i)))), 0i);
    let _e155 = bitcast<vec2<f32>>(_e153.xy);
    let _e157 = (_e153.z & 65535u);
    let _e159 = (_e157 * 4u);
    let _e166 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e159 & 255u)), bitcast<i32>((_e159 >> bitcast<u32>(8i)))), 0i);
    let _e167 = bitcast<vec4<f32>>(_e166);
    let _e174 = mat2x2<f32>(vec2<f32>(_e167.x, _e167.y), vec2<f32>(_e167.z, _e167.w));
    let _e175 = (_e159 + 1u);
    let _e182 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e175 & 255u)), bitcast<i32>((_e175 >> bitcast<u32>(8i)))), 0i);
    let _e184 = bitcast<vec2<f32>>(_e182.xy);
    let _e187 = (_e159 + 2u);
    let _e194 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e187 & 255u)), bitcast<i32>((_e187 >> bitcast<u32>(8i)))), 0i);
    let _e196 = (_e142.w & 8388608u);
    if K7_ {
        phi_1384_ = _e125;
        phi_1349_ = false;
    } else {
        if fe {
            let _e197 = (_e196 != 0u);
            phi_1386_ = _e125;
            if _e197 {
                phi_1386_ = -(_e125);
            }
            let _e200 = phi_1386_;
            phi_1385_ = _e200;
            phi_1350_ = _e197;
        } else {
            phi_1385_ = _e125;
            phi_1350_ = (((_e196 != 0u) && !(_e80)) && !(_e127));
        }
        let _e207 = phi_1385_;
        let _e209 = phi_1350_;
        phi_1384_ = _e207;
        phi_1349_ = _e209;
    }
    let _e211 = phi_1384_;
    let _e213 = phi_1349_;
    phi_1355_ = _e133;
    if _e213 {
        phi_1355_ = (_e133 - 1i);
    }
    let _e216 = phi_1355_;
    phi_1402_ = _e142.w;
    phi_1379_ = _e142;
    if (_e216 != _e135) {
        let _e219 = ((_e137 + _e216) - _e135);
        let _e224 = textureLoad(UB, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
        if ((_e224.w & 8454143u) != (_e142.w & 8454143u)) {
            if K7_ {
                phi_1360_ = (_e155.x != 0f);
            } else {
                phi_1360_ = true;
            }
            let _e232 = phi_1360_;
            phi_1362_ = _e142;
            if _e232 {
                let _e233 = bitcast<i32>(_e153.w);
                let _e238 = textureLoad(UB, vec2<i32>((_e233 & 2047i), (_e233 >> bitcast<u32>(11i))), 0i);
                phi_1362_ = _e238;
            }
            let _e240 = phi_1362_;
            phi_1361_ = _e240;
        } else {
            phi_1361_ = _e224;
        }
        let _e242 = phi_1361_;
        phi_1402_ = ((_e242.w & 4286578687u) | _e196);
        phi_1379_ = _e242;
    }
    let _e247 = phi_1402_;
    let _e249 = phi_1379_;
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1368_ = true;
                break;
            }
            if fe {
                phi_1368_ = true;
                break;
            }
            phi_1368_ = false;
            break;
        }
    }
    let _e252 = phi_1368_;
    if _e252 {
        let _e255 = (_e211 * sign(determinant(_e174)));
        let _e258 = (f32(_e249.z) * 0.0000000014629181f);
        let _e262 = vec2<f32>(sin(_e258), -(cos(_e258)));
        let _e266 = select(0f, _e255, (_e123 == 0f));
        phi_1443_ = _e262;
        phi_1432_ = _e266;
        phi_1429_ = _e255;
        phi_1419_ = _e262;
        if K7_ {
            let _e268 = ((_e247 & 1048576u) != 0u);
            phi_1404_ = _e255;
            if _e268 {
                phi_1404_ = min(_e255, 0f);
            }
            let _e271 = phi_1404_;
            phi_1416_ = _e271;
            if ((_e247 & 524288u) != 0u) {
                phi_1416_ = max(_e271, 0f);
            }
            let _e276 = phi_1416_;
            let _e277 = (_e247 & 469762048u);
            phi_1444_ = _e262;
            phi_1433_ = _e266;
            phi_1420_ = _e262;
            if (_e277 > 134217728u) {
                let _e282 = f32((_e249.z & 65535u));
                let _e283 = (_e282 * 0.000015259022f);
                let _e287 = sqrt(max((1f - (_e283 * _e283)), 0f));
                phi_1408_ = _e287;
                if (((_e247 & 4194304u) != 0u) == _e268) {
                    phi_1408_ = -(_e287);
                }
                let _e291 = phi_1408_;
                let _e296 = (mat2x2<f32>(vec2<f32>(_e283, _e291), vec2<f32>(-(_e291), _e283)) * _e262);
                let _e297 = (_e277 == 201326592u);
                phi_685_ = _e297;
                if !(_e297) {
                    phi_685_ = ((_e277 != 335544320u) && (_e283 < 0.25f));
                }
                let _e303 = phi_685_;
                let _e305 = ((_e247 & 2097152u) != 0u);
                if (_e277 == 335544320u) {
                    phi_1425_ = (_e262 + _e296);
                } else {
                    phi_1426_ = _e262;
                    if (_e305 || !(_e303)) {
                        if _e303 {
                            phi_1411_ = _e283;
                        } else {
                            phi_1411_ = (65535f / _e282);
                        }
                        let _e312 = phi_1411_;
                        phi_1426_ = (_e296 * _e312);
                    }
                    let _e315 = phi_1426_;
                    phi_1425_ = _e315;
                }
                let _e317 = phi_1425_;
                phi_1436_ = _e266;
                if (!(_e80) && _e303) {
                    phi_1436_ = (0.5f * _e276);
                }
                let _e325 = phi_1436_;
                phi_1444_ = select(_e262, _e296, vec2((_e303 || _e305)));
                phi_1433_ = _e325;
                phi_1420_ = _e317;
            }
            let _e327 = phi_1444_;
            let _e329 = phi_1433_;
            let _e331 = phi_1420_;
            phi_1443_ = _e327;
            phi_1432_ = _e329;
            phi_1429_ = _e276;
            phi_1419_ = _e331;
        }
        let _e333 = phi_1443_;
        let _e335 = phi_1432_;
        let _e337 = phi_1429_;
        let _e339 = phi_1419_;
        let _e344 = ((_e174 * (bitcast<vec2<f32>>(_e249.xy) + (_e339 * (_e337 * bitcast<f32>(_e182.z))))) + _e184);
        phi_1455_ = _e344;
        if (_e335 != 0f) {
            phi_1455_ = (_e344 + (sign((_e333 * _naga_inverse_2x2_f32(_e174))) * _e335));
        }
        let _e352 = phi_1455_;
        phi_1454_ = _e352;
    } else {
        if _e127 {
            phi_1380_ = _e155;
        } else {
            phi_1380_ = bitcast<vec2<f32>>(_e249.xy);
        }
        let _e356 = phi_1380_;
        phi_1454_ = ((_e174 * _e356) + _e184);
    }
    let _e360 = phi_1454_;
    if Xi {
        let _e361 = (_e157 * 8u);
        let _e362 = (_e361 + 2u);
        let _e369 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e362 & 255u)), bitcast<i32>((_e362 >> bitcast<u32>(8i)))), 0i);
        let _e377 = (_e361 + 3u);
        let _e384 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e377 & 255u)), bitcast<i32>((_e377 >> bitcast<u32>(8i)))), 0i);
        if any((_e369 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e389 = ((mat2x2<f32>(vec2<f32>(_e369.x, _e369.y), vec2<f32>(_e369.z, _e369.w)) * _e360) + _e384.xy);
            unnamed.gl_ClipDistance[0i] = (_e389.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e389.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e389.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e389.y);
        } else {
            let _e405 = (_e384.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e405;
            unnamed.gl_ClipDistance[2i] = _e405;
            unnamed.gl_ClipDistance[1i] = _e405;
            unnamed.gl_ClipDistance[0i] = _e405;
        }
    }
    if ((_e75 & 268435456i) != 0i) {
        O0_ = vec4<f32>(0f, 0f, 0f, 0f);
        U0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e420 = textureLoad(VC, vec2<i32>(bitcast<i32>((_e153.z & 255u)), bitcast<i32>((_e157 >> bitcast<u32>(8i)))), 0i);
        let _e422 = (_e420.x & 15u);
        phi_1505_ = false;
        if Yi {
            let _e425 = ((_e420.x >> bitcast<u32>(4i)) & 15u);
            P0_ = f32(_e425);
            phi_1505_ = (_e425 != 0u);
        }
        let _e429 = phi_1505_;
        if (_e422 == 1u) {
            O0_ = unpack4x8unorm(_e420.y);
            if _e429 {
                let _e434 = O0_[3u];
                O0_[3u] = (_e434 * _e123);
            } else {
                let _e436 = O0_;
                O0_ = (_e436 * _e123);
            }
        } else {
            let _e438 = (_e157 * 8u);
            let _e445 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e438 & 255u)), bitcast<i32>((_e438 >> bitcast<u32>(8i)))), 0i);
            let _e453 = (_e438 + 1u);
            let _e460 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e453 & 255u)), bitcast<i32>((_e453 >> bitcast<u32>(8i)))), 0i);
            let _e465 = ((mat2x2<f32>(vec2<f32>(_e445.x, _e445.y), vec2<f32>(_e445.z, _e445.w)) * _e360) + _e460.xy);
            let _e476 = ((_e460.w + (f32(_e422) * 0.125f)) + (max(_e460.z, 0f) * 0.00024414063f));
            if (_e460.z < 0f) {
                phi_1503_ = -(_e476);
            } else {
                phi_1503_ = _e476;
            }
            let _e479 = phi_1503_;
            O0_ = vec4<f32>(_e465.x, _e465.y, _e479, (((_e123 * -0.5f) - 0.25f) - round((bitcast<f32>(_e420.y) * 255f))));
        }
        phi_939_ = ej;
        if ej {
            phi_939_ = ((_e420.x & 2048u) != 0u);
        }
        let _e489 = phi_939_;
        if _e489 {
            let _e490 = (_e157 * 8u);
            let _e491 = (_e490 + 4u);
            let _e498 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e491 & 255u)), bitcast<i32>((_e491 >> bitcast<u32>(8i)))), 0i);
            let _e506 = (_e490 + 5u);
            let _e513 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e506 & 255u)), bitcast<i32>((_e506 >> bitcast<u32>(8i)))), 0i);
            let _e516 = ((mat2x2<f32>(vec2<f32>(_e498.x, _e498.y), vec2<f32>(_e498.z, _e498.w)) * _e360) + _e513.xy);
            U0_ = vec3<f32>(_e516.x, _e516.y, (1f + _e513.z));
        } else {
            U0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e523 = j.Hg;
    let _e525 = j.Ig;
    let _e533 = vec4<f32>(((_e360.x * _e523) - 1f), ((_e360.y * _e525) - sign(_e525)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1515_ = true;
                break;
            }
            if fe {
                phi_1515_ = true;
                break;
            }
            phi_1515_ = false;
            break;
        }
    }
    let _e536 = phi_1515_;
    if _e536 {
        let _e538 = u32((_e123 * 254f));
        phi_1524_ = _e538;
        if ((_e75 & 1073741824i) == 0i) {
            phi_1524_ = (_e538 + bitcast<u32>(1i));
        }
        let _e543 = phi_1524_;
        phi_1523_ = _e543;
    } else {
        phi_1523_ = 255u;
    }
    let _e545 = phi_1523_;
    unnamed.gl_Position = vec4<f32>(_e533.x, _e533.y, ((f32(((_e194.x << bitcast<u32>(8u)) | _e545)) * 0.000000059604645f) + 0.000000029802322f), _e533.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = unnamed.gl_Position;
    let _e10 = unnamed.gl_ClipDistance;
    let _e11 = O0_;
    let _e12 = U0_;
    let _e13 = P0_;
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
