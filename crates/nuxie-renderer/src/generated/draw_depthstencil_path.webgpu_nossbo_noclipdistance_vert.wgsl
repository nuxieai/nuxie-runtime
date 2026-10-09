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

@id(15) override K7_: bool = false;
@id(16) override fe: bool = false;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;

fn main_1() {
    var phi_1250_: bool;
    var phi_1251_: i32;
    var phi_1307_: f32;
    var phi_1260_: f32;
    var phi_1257_: bool;
    var phi_1255_: i32;
    var phi_1254_: i32;
    var phi_1252_: i32;
    var phi_1299_: f32;
    var phi_1298_: f32;
    var phi_1263_: bool;
    var phi_1297_: f32;
    var phi_1262_: bool;
    var phi_1268_: i32;
    var phi_1273_: bool;
    var phi_1275_: vec4<u32>;
    var phi_1274_: vec4<u32>;
    var phi_1315_: u32;
    var phi_1292_: vec4<u32>;
    var phi_1281_: bool;
    var phi_1317_: f32;
    var phi_1329_: f32;
    var phi_1321_: f32;
    var phi_636_: bool;
    var phi_1324_: f32;
    var phi_1339_: vec2<f32>;
    var phi_1338_: vec2<f32>;
    var phi_1349_: f32;
    var phi_1357_: vec2<f32>;
    var phi_1346_: f32;
    var phi_1333_: vec2<f32>;
    var phi_1356_: vec2<f32>;
    var phi_1345_: f32;
    var phi_1342_: f32;
    var phi_1332_: vec2<f32>;
    var phi_1368_: vec2<f32>;
    var phi_1293_: vec2<f32>;
    var phi_1367_: vec2<f32>;
    var phi_1414_: bool;
    var phi_1412_: f32;
    var phi_893_: bool;
    var phi_1422_: bool;
    var phi_1431_: u32;
    var phi_1430_: u32;

    let _e74 = gl_VertexIndex_1;
    let _e79 = ((_e74 & 536870912i) != 0i);
    let _e80 = (_e74 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1250_ = true;
                break;
            }
            if fe {
                phi_1250_ = true;
                break;
            }
            phi_1250_ = false;
            break;
        }
    }
    let _e83 = phi_1250_;
    if _e83 {
        let _e84 = select(5i, 6i, _e79);
        let _e90 = (_e80 & ((1i << bitcast<u32>(_e84)) - 1i));
        let _e91 = select(1i, 2i, _e79);
        let _e97 = (_e90 & ((1i << bitcast<u32>(_e91)) - 1i));
        phi_1251_ = _e97;
        if (K7_ && !(_e79)) {
            phi_1251_ = (_e97 + 1i);
        }
        let _e102 = phi_1251_;
        phi_1307_ = select(1f, 0f, ((_e102 == 0i) || (_e102 == 3i)));
        phi_1260_ = select(-1f, 1f, (_e102 < 2i));
        phi_1257_ = false;
        phi_1255_ = (_e80 >> bitcast<u32>(_e84));
        phi_1254_ = 8i;
        phi_1252_ = (_e90 >> bitcast<u32>(_e91));
    } else {
        let _e109 = select(4i, 5i, _e79);
        let _e115 = (_e80 & ((1i << bitcast<u32>(_e109)) - 1i));
        let _e119 = (!(_e79) && (_e115 == 9i));
        phi_1307_ = 1f;
        phi_1260_ = 0f;
        phi_1257_ = _e119;
        phi_1255_ = (_e80 >> bitcast<u32>(_e109));
        phi_1254_ = select(8i, 17i, _e79);
        phi_1252_ = select(_e115, 0i, _e119);
    }
    let _e122 = phi_1307_;
    let _e124 = phi_1260_;
    let _e126 = phi_1257_;
    let _e128 = phi_1255_;
    let _e130 = phi_1254_;
    let _e132 = phi_1252_;
    let _e134 = min(_e132, (_e130 - 1i));
    let _e136 = ((_e128 * _e130) + _e134);
    let _e141 = textureLoad(UB, vec2<i32>((_e136 & 2047i), (_e136 >> bitcast<u32>(11i))), 0i);
    let _e145 = (max((_e141.w & 65535u), 1u) - 1u);
    let _e152 = textureLoad(BD, vec2<i32>(bitcast<i32>((_e145 & 255u)), bitcast<i32>((_e145 >> bitcast<u32>(8i)))), 0i);
    let _e154 = bitcast<vec2<f32>>(_e152.xy);
    let _e156 = (_e152.z & 65535u);
    let _e158 = (_e156 * 4u);
    let _e165 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e158 & 255u)), bitcast<i32>((_e158 >> bitcast<u32>(8i)))), 0i);
    let _e166 = bitcast<vec4<f32>>(_e165);
    let _e173 = mat2x2<f32>(vec2<f32>(_e166.x, _e166.y), vec2<f32>(_e166.z, _e166.w));
    let _e174 = (_e158 + 1u);
    let _e181 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e174 & 255u)), bitcast<i32>((_e174 >> bitcast<u32>(8i)))), 0i);
    let _e183 = bitcast<vec2<f32>>(_e181.xy);
    let _e186 = (_e158 + 2u);
    let _e193 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e186 & 255u)), bitcast<i32>((_e186 >> bitcast<u32>(8i)))), 0i);
    let _e195 = (_e141.w & 8388608u);
    if K7_ {
        phi_1297_ = _e124;
        phi_1262_ = false;
    } else {
        if fe {
            let _e196 = (_e195 != 0u);
            phi_1299_ = _e124;
            if _e196 {
                phi_1299_ = -(_e124);
            }
            let _e199 = phi_1299_;
            phi_1298_ = _e199;
            phi_1263_ = _e196;
        } else {
            phi_1298_ = _e124;
            phi_1263_ = (((_e195 != 0u) && !(_e79)) && !(_e126));
        }
        let _e206 = phi_1298_;
        let _e208 = phi_1263_;
        phi_1297_ = _e206;
        phi_1262_ = _e208;
    }
    let _e210 = phi_1297_;
    let _e212 = phi_1262_;
    phi_1268_ = _e132;
    if _e212 {
        phi_1268_ = (_e132 - 1i);
    }
    let _e215 = phi_1268_;
    phi_1315_ = _e141.w;
    phi_1292_ = _e141;
    if (_e215 != _e134) {
        let _e218 = ((_e136 + _e215) - _e134);
        let _e223 = textureLoad(UB, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
        if ((_e223.w & 8454143u) != (_e141.w & 8454143u)) {
            if K7_ {
                phi_1273_ = (_e154.x != 0f);
            } else {
                phi_1273_ = true;
            }
            let _e231 = phi_1273_;
            phi_1275_ = _e141;
            if _e231 {
                let _e232 = bitcast<i32>(_e152.w);
                let _e237 = textureLoad(UB, vec2<i32>((_e232 & 2047i), (_e232 >> bitcast<u32>(11i))), 0i);
                phi_1275_ = _e237;
            }
            let _e239 = phi_1275_;
            phi_1274_ = _e239;
        } else {
            phi_1274_ = _e223;
        }
        let _e241 = phi_1274_;
        phi_1315_ = ((_e241.w & 4286578687u) | _e195);
        phi_1292_ = _e241;
    }
    let _e246 = phi_1315_;
    let _e248 = phi_1292_;
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1281_ = true;
                break;
            }
            if fe {
                phi_1281_ = true;
                break;
            }
            phi_1281_ = false;
            break;
        }
    }
    let _e251 = phi_1281_;
    if _e251 {
        let _e254 = (_e210 * sign(determinant(_e173)));
        let _e257 = (f32(_e248.z) * 0.0000000014629181f);
        let _e261 = vec2<f32>(sin(_e257), -(cos(_e257)));
        let _e265 = select(0f, _e254, (_e122 == 0f));
        phi_1356_ = _e261;
        phi_1345_ = _e265;
        phi_1342_ = _e254;
        phi_1332_ = _e261;
        if K7_ {
            let _e267 = ((_e246 & 1048576u) != 0u);
            phi_1317_ = _e254;
            if _e267 {
                phi_1317_ = min(_e254, 0f);
            }
            let _e270 = phi_1317_;
            phi_1329_ = _e270;
            if ((_e246 & 524288u) != 0u) {
                phi_1329_ = max(_e270, 0f);
            }
            let _e275 = phi_1329_;
            let _e276 = (_e246 & 469762048u);
            phi_1357_ = _e261;
            phi_1346_ = _e265;
            phi_1333_ = _e261;
            if (_e276 > 134217728u) {
                let _e281 = f32((_e248.z & 65535u));
                let _e282 = (_e281 * 0.000015259022f);
                let _e286 = sqrt(max((1f - (_e282 * _e282)), 0f));
                phi_1321_ = _e286;
                if (((_e246 & 4194304u) != 0u) == _e267) {
                    phi_1321_ = -(_e286);
                }
                let _e290 = phi_1321_;
                let _e295 = (mat2x2<f32>(vec2<f32>(_e282, _e290), vec2<f32>(-(_e290), _e282)) * _e261);
                let _e296 = (_e276 == 201326592u);
                phi_636_ = _e296;
                if !(_e296) {
                    phi_636_ = ((_e276 != 335544320u) && (_e282 < 0.25f));
                }
                let _e302 = phi_636_;
                let _e304 = ((_e246 & 2097152u) != 0u);
                if (_e276 == 335544320u) {
                    phi_1338_ = (_e261 + _e295);
                } else {
                    phi_1339_ = _e261;
                    if (_e304 || !(_e302)) {
                        if _e302 {
                            phi_1324_ = _e282;
                        } else {
                            phi_1324_ = (65535f / _e281);
                        }
                        let _e311 = phi_1324_;
                        phi_1339_ = (_e295 * _e311);
                    }
                    let _e314 = phi_1339_;
                    phi_1338_ = _e314;
                }
                let _e316 = phi_1338_;
                phi_1349_ = _e265;
                if (!(_e79) && _e302) {
                    phi_1349_ = (0.5f * _e275);
                }
                let _e324 = phi_1349_;
                phi_1357_ = select(_e261, _e295, vec2((_e302 || _e304)));
                phi_1346_ = _e324;
                phi_1333_ = _e316;
            }
            let _e326 = phi_1357_;
            let _e328 = phi_1346_;
            let _e330 = phi_1333_;
            phi_1356_ = _e326;
            phi_1345_ = _e328;
            phi_1342_ = _e275;
            phi_1332_ = _e330;
        }
        let _e332 = phi_1356_;
        let _e334 = phi_1345_;
        let _e336 = phi_1342_;
        let _e338 = phi_1332_;
        let _e343 = ((_e173 * (bitcast<vec2<f32>>(_e248.xy) + (_e338 * (_e336 * bitcast<f32>(_e181.z))))) + _e183);
        phi_1368_ = _e343;
        if (_e334 != 0f) {
            phi_1368_ = (_e343 + (sign((_e332 * _naga_inverse_2x2_f32(_e173))) * _e334));
        }
        let _e351 = phi_1368_;
        phi_1367_ = _e351;
    } else {
        if _e126 {
            phi_1293_ = _e154;
        } else {
            phi_1293_ = bitcast<vec2<f32>>(_e248.xy);
        }
        let _e355 = phi_1293_;
        phi_1367_ = ((_e173 * _e355) + _e183);
    }
    let _e359 = phi_1367_;
    if ((_e74 & 268435456i) != 0i) {
        O0_ = vec4<f32>(0f, 0f, 0f, 0f);
        U0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e366 = textureLoad(VC, vec2<i32>(bitcast<i32>((_e152.z & 255u)), bitcast<i32>((_e156 >> bitcast<u32>(8i)))), 0i);
        let _e368 = (_e366.x & 15u);
        phi_1414_ = false;
        if Yi {
            let _e371 = ((_e366.x >> bitcast<u32>(4i)) & 15u);
            P0_ = f32(_e371);
            phi_1414_ = (_e371 != 0u);
        }
        let _e375 = phi_1414_;
        if (_e368 == 1u) {
            O0_ = unpack4x8unorm(_e366.y);
            if _e375 {
                let _e380 = O0_[3u];
                O0_[3u] = (_e380 * _e122);
            } else {
                let _e382 = O0_;
                O0_ = (_e382 * _e122);
            }
        } else {
            let _e384 = (_e156 * 8u);
            let _e391 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e384 & 255u)), bitcast<i32>((_e384 >> bitcast<u32>(8i)))), 0i);
            let _e399 = (_e384 + 1u);
            let _e406 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e399 & 255u)), bitcast<i32>((_e399 >> bitcast<u32>(8i)))), 0i);
            let _e411 = ((mat2x2<f32>(vec2<f32>(_e391.x, _e391.y), vec2<f32>(_e391.z, _e391.w)) * _e359) + _e406.xy);
            let _e422 = ((_e406.w + (f32(_e368) * 0.125f)) + (max(_e406.z, 0f) * 0.00024414063f));
            if (_e406.z < 0f) {
                phi_1412_ = -(_e422);
            } else {
                phi_1412_ = _e422;
            }
            let _e425 = phi_1412_;
            O0_ = vec4<f32>(_e411.x, _e411.y, _e425, (((_e122 * -0.5f) - 0.25f) - round((bitcast<f32>(_e366.y) * 255f))));
        }
        phi_893_ = ej;
        if ej {
            phi_893_ = ((_e366.x & 2048u) != 0u);
        }
        let _e435 = phi_893_;
        if _e435 {
            let _e436 = (_e156 * 8u);
            let _e437 = (_e436 + 4u);
            let _e444 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e437 & 255u)), bitcast<i32>((_e437 >> bitcast<u32>(8i)))), 0i);
            let _e452 = (_e436 + 5u);
            let _e459 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e452 & 255u)), bitcast<i32>((_e452 >> bitcast<u32>(8i)))), 0i);
            let _e462 = ((mat2x2<f32>(vec2<f32>(_e444.x, _e444.y), vec2<f32>(_e444.z, _e444.w)) * _e359) + _e459.xy);
            U0_ = vec3<f32>(_e462.x, _e462.y, (1f + _e459.z));
        } else {
            U0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e469 = j.Hg;
    let _e471 = j.Ig;
    let _e479 = vec4<f32>(((_e359.x * _e469) - 1f), ((_e359.y * _e471) - sign(_e471)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1422_ = true;
                break;
            }
            if fe {
                phi_1422_ = true;
                break;
            }
            phi_1422_ = false;
            break;
        }
    }
    let _e482 = phi_1422_;
    if _e482 {
        let _e484 = u32((_e122 * 254f));
        phi_1431_ = _e484;
        if ((_e74 & 1073741824i) == 0i) {
            phi_1431_ = (_e484 + bitcast<u32>(1i));
        }
        let _e489 = phi_1431_;
        phi_1430_ = _e489;
    } else {
        phi_1430_ = 255u;
    }
    let _e491 = phi_1430_;
    unnamed.gl_Position = vec4<f32>(_e479.x, _e479.y, ((f32(((_e193.x << bitcast<u32>(8u)) | _e491)) * 0.000000059604645f) + 0.000000029802322f), _e479.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e8 = O0_;
    let _e9 = U0_;
    let _e10 = P0_;
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
