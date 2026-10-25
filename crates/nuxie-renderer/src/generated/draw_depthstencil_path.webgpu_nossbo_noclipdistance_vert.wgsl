struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
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

@id(16) override O7_: bool = false;
@id(17) override ke: bool = false;
@id(2) override hj: bool = true;
@id(4) override jj: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(5)
var DD: texture_2d<u32>;
@group(0) @binding(2)
var KB: texture_2d<u32>;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
@group(0) @binding(3)
var XC: texture_2d<u32>;
var<private> Q0_: f32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    var phi_1251_: bool;
    var phi_1252_: i32;
    var phi_1308_: f32;
    var phi_1261_: f32;
    var phi_1258_: bool;
    var phi_1256_: i32;
    var phi_1255_: i32;
    var phi_1253_: i32;
    var phi_1300_: f32;
    var phi_1299_: f32;
    var phi_1264_: bool;
    var phi_1298_: f32;
    var phi_1263_: bool;
    var phi_1269_: i32;
    var phi_1274_: bool;
    var phi_1276_: vec4<u32>;
    var phi_1275_: vec4<u32>;
    var phi_1316_: u32;
    var phi_1293_: vec4<u32>;
    var phi_1282_: bool;
    var phi_1318_: f32;
    var phi_1330_: f32;
    var phi_1322_: f32;
    var phi_636_: bool;
    var phi_1325_: f32;
    var phi_1340_: vec2<f32>;
    var phi_1339_: vec2<f32>;
    var phi_1350_: f32;
    var phi_1358_: vec2<f32>;
    var phi_1347_: f32;
    var phi_1334_: vec2<f32>;
    var phi_1357_: vec2<f32>;
    var phi_1346_: f32;
    var phi_1343_: f32;
    var phi_1333_: vec2<f32>;
    var phi_1369_: vec2<f32>;
    var phi_1294_: vec2<f32>;
    var phi_1368_: vec2<f32>;
    var phi_1415_: bool;
    var phi_1413_: f32;
    var phi_893_: bool;
    var phi_1423_: bool;
    var phi_1432_: u32;
    var phi_1431_: u32;

    let _e74 = gl_VertexIndex_1;
    let _e79 = ((_e74 & 536870912i) != 0i);
    let _e80 = (_e74 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1251_ = true;
                break;
            }
            if ke {
                phi_1251_ = true;
                break;
            }
            phi_1251_ = false;
            break;
        }
    }
    let _e83 = phi_1251_;
    if _e83 {
        let _e84 = select(5i, 6i, _e79);
        let _e90 = (_e80 & ((1i << bitcast<u32>(_e84)) - 1i));
        let _e91 = select(1i, 2i, _e79);
        let _e97 = (_e90 & ((1i << bitcast<u32>(_e91)) - 1i));
        phi_1252_ = _e97;
        if (O7_ && !(_e79)) {
            phi_1252_ = (_e97 + 1i);
        }
        let _e102 = phi_1252_;
        phi_1308_ = select(1f, 0f, ((_e102 == 0i) || (_e102 == 3i)));
        phi_1261_ = select(-1f, 1f, (_e102 < 2i));
        phi_1258_ = false;
        phi_1256_ = (_e80 >> bitcast<u32>(_e84));
        phi_1255_ = 8i;
        phi_1253_ = (_e90 >> bitcast<u32>(_e91));
    } else {
        let _e109 = select(4i, 5i, _e79);
        let _e115 = (_e80 & ((1i << bitcast<u32>(_e109)) - 1i));
        let _e119 = (!(_e79) && (_e115 == 9i));
        phi_1308_ = 1f;
        phi_1261_ = 0f;
        phi_1258_ = _e119;
        phi_1256_ = (_e80 >> bitcast<u32>(_e109));
        phi_1255_ = select(8i, 17i, _e79);
        phi_1253_ = select(_e115, 0i, _e119);
    }
    let _e122 = phi_1308_;
    let _e124 = phi_1261_;
    let _e126 = phi_1258_;
    let _e128 = phi_1256_;
    let _e130 = phi_1255_;
    let _e132 = phi_1253_;
    let _e134 = min(_e132, (_e130 - 1i));
    let _e136 = ((_e128 * _e130) + _e134);
    let _e141 = textureLoad(VB, vec2<i32>((_e136 & 2047i), (_e136 >> bitcast<u32>(11i))), 0i);
    let _e145 = (max((_e141.w & 65535u), 1u) - 1u);
    let _e152 = textureLoad(DD, vec2<i32>(bitcast<i32>((_e145 & 255u)), bitcast<i32>((_e145 >> bitcast<u32>(8i)))), 0i);
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
    if O7_ {
        phi_1298_ = _e124;
        phi_1263_ = false;
    } else {
        if ke {
            let _e196 = (_e195 != 0u);
            phi_1300_ = _e124;
            if _e196 {
                phi_1300_ = -(_e124);
            }
            let _e199 = phi_1300_;
            phi_1299_ = _e199;
            phi_1264_ = _e196;
        } else {
            phi_1299_ = _e124;
            phi_1264_ = (((_e195 != 0u) && !(_e79)) && !(_e126));
        }
        let _e206 = phi_1299_;
        let _e208 = phi_1264_;
        phi_1298_ = _e206;
        phi_1263_ = _e208;
    }
    let _e210 = phi_1298_;
    let _e212 = phi_1263_;
    phi_1269_ = _e132;
    if _e212 {
        phi_1269_ = (_e132 - 1i);
    }
    let _e215 = phi_1269_;
    phi_1316_ = _e141.w;
    phi_1293_ = _e141;
    if (_e215 != _e134) {
        let _e218 = ((_e136 + _e215) - _e134);
        let _e223 = textureLoad(VB, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
        if ((_e223.w & 8454143u) != (_e141.w & 8454143u)) {
            if O7_ {
                phi_1274_ = (_e154.x != 0f);
            } else {
                phi_1274_ = true;
            }
            let _e231 = phi_1274_;
            phi_1276_ = _e141;
            if _e231 {
                let _e232 = bitcast<i32>(_e152.w);
                let _e237 = textureLoad(VB, vec2<i32>((_e232 & 2047i), (_e232 >> bitcast<u32>(11i))), 0i);
                phi_1276_ = _e237;
            }
            let _e239 = phi_1276_;
            phi_1275_ = _e239;
        } else {
            phi_1275_ = _e223;
        }
        let _e241 = phi_1275_;
        phi_1316_ = ((_e241.w & 4286578687u) | _e195);
        phi_1293_ = _e241;
    }
    let _e246 = phi_1316_;
    let _e248 = phi_1293_;
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1282_ = true;
                break;
            }
            if ke {
                phi_1282_ = true;
                break;
            }
            phi_1282_ = false;
            break;
        }
    }
    let _e251 = phi_1282_;
    if _e251 {
        let _e254 = (_e210 * sign(determinant(_e173)));
        let _e257 = (f32(_e248.z) * 0.0000000014629181f);
        let _e261 = vec2<f32>(sin(_e257), -(cos(_e257)));
        let _e265 = select(0f, _e254, (_e122 == 0f));
        phi_1357_ = _e261;
        phi_1346_ = _e265;
        phi_1343_ = _e254;
        phi_1333_ = _e261;
        if O7_ {
            let _e267 = ((_e246 & 1048576u) != 0u);
            phi_1318_ = _e254;
            if _e267 {
                phi_1318_ = min(_e254, 0f);
            }
            let _e270 = phi_1318_;
            phi_1330_ = _e270;
            if ((_e246 & 524288u) != 0u) {
                phi_1330_ = max(_e270, 0f);
            }
            let _e275 = phi_1330_;
            let _e276 = (_e246 & 469762048u);
            phi_1358_ = _e261;
            phi_1347_ = _e265;
            phi_1334_ = _e261;
            if (_e276 > 201326592u) {
                let _e281 = f32((_e248.z & 65535u));
                let _e282 = (_e281 * 0.000015259022f);
                let _e286 = sqrt(max((1f - (_e282 * _e282)), 0f));
                phi_1322_ = _e286;
                if (((_e246 & 4194304u) != 0u) == _e267) {
                    phi_1322_ = -(_e286);
                }
                let _e290 = phi_1322_;
                let _e295 = (mat2x2<f32>(vec2<f32>(_e282, _e290), vec2<f32>(-(_e290), _e282)) * _e261);
                let _e296 = (_e276 == 268435456u);
                phi_636_ = _e296;
                if !(_e296) {
                    phi_636_ = ((_e276 != 402653184u) && (_e282 < 0.25f));
                }
                let _e302 = phi_636_;
                let _e304 = ((_e246 & 2097152u) != 0u);
                if (_e276 == 402653184u) {
                    phi_1339_ = (_e261 + _e295);
                } else {
                    phi_1340_ = _e261;
                    if (_e304 || !(_e302)) {
                        if _e302 {
                            phi_1325_ = _e282;
                        } else {
                            phi_1325_ = (65535f / _e281);
                        }
                        let _e311 = phi_1325_;
                        phi_1340_ = (_e295 * _e311);
                    }
                    let _e314 = phi_1340_;
                    phi_1339_ = _e314;
                }
                let _e316 = phi_1339_;
                phi_1350_ = _e265;
                if (!(_e79) && _e302) {
                    phi_1350_ = (0.5f * _e275);
                }
                let _e324 = phi_1350_;
                phi_1358_ = select(_e261, _e295, vec2((_e302 || _e304)));
                phi_1347_ = _e324;
                phi_1334_ = _e316;
            }
            let _e326 = phi_1358_;
            let _e328 = phi_1347_;
            let _e330 = phi_1334_;
            phi_1357_ = _e326;
            phi_1346_ = _e328;
            phi_1343_ = _e275;
            phi_1333_ = _e330;
        }
        let _e332 = phi_1357_;
        let _e334 = phi_1346_;
        let _e336 = phi_1343_;
        let _e338 = phi_1333_;
        let _e343 = ((_e173 * (bitcast<vec2<f32>>(_e248.xy) + (_e338 * (_e336 * bitcast<f32>(_e181.z))))) + _e183);
        phi_1369_ = _e343;
        if (_e334 != 0f) {
            phi_1369_ = (_e343 + (sign((_e332 * _naga_inverse_2x2_f32(_e173))) * _e334));
        }
        let _e351 = phi_1369_;
        phi_1368_ = _e351;
    } else {
        if _e126 {
            phi_1294_ = _e154;
        } else {
            phi_1294_ = bitcast<vec2<f32>>(_e248.xy);
        }
        let _e355 = phi_1294_;
        phi_1368_ = ((_e173 * _e355) + _e183);
    }
    let _e359 = phi_1368_;
    if ((_e74 & 268435456i) != 0i) {
        P0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e366 = textureLoad(XC, vec2<i32>(bitcast<i32>((_e152.z & 255u)), bitcast<i32>((_e156 >> bitcast<u32>(8i)))), 0i);
        let _e368 = (_e366.x & 15u);
        phi_1415_ = false;
        if hj {
            let _e371 = ((_e366.x >> bitcast<u32>(4i)) & 15u);
            Q0_ = f32(_e371);
            phi_1415_ = (_e371 != 0u);
        }
        let _e375 = phi_1415_;
        if (_e368 == 1u) {
            P0_ = unpack4x8unorm(_e366.y);
            if _e375 {
                let _e380 = P0_[3u];
                P0_[3u] = (_e380 * _e122);
            } else {
                let _e382 = P0_;
                P0_ = (_e382 * _e122);
            }
        } else {
            let _e384 = (_e156 * 8u);
            let _e391 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e384 & 255u)), bitcast<i32>((_e384 >> bitcast<u32>(8i)))), 0i);
            let _e399 = (_e384 + 1u);
            let _e406 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e399 & 255u)), bitcast<i32>((_e399 >> bitcast<u32>(8i)))), 0i);
            let _e411 = ((mat2x2<f32>(vec2<f32>(_e391.x, _e391.y), vec2<f32>(_e391.z, _e391.w)) * _e359) + _e406.xy);
            let _e422 = ((_e406.w + (f32(_e368) * 0.125f)) + (max(_e406.z, 0f) * 0.00024414063f));
            if (_e406.z < 0f) {
                phi_1413_ = -(_e422);
            } else {
                phi_1413_ = _e422;
            }
            let _e425 = phi_1413_;
            P0_ = vec4<f32>(_e411.x, _e411.y, _e425, (((_e122 * -0.5f) - 0.25f) - round((bitcast<f32>(_e366.y) * 255f))));
        }
        phi_893_ = jj;
        if jj {
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
            V0_ = vec3<f32>(_e462.x, _e462.y, (1f + _e459.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e469 = j.Mg;
    let _e471 = j.Ng;
    let _e479 = vec4<f32>(((_e359.x * _e469) - 1f), ((_e359.y * _e471) - sign(_e471)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1423_ = true;
                break;
            }
            if ke {
                phi_1423_ = true;
                break;
            }
            phi_1423_ = false;
            break;
        }
    }
    let _e482 = phi_1423_;
    if _e482 {
        let _e484 = u32((_e122 * 254f));
        phi_1432_ = _e484;
        if ((_e74 & 1073741824i) == 0i) {
            phi_1432_ = (_e484 + bitcast<u32>(1i));
        }
        let _e489 = phi_1432_;
        phi_1431_ = _e489;
    } else {
        phi_1431_ = 255u;
    }
    let _e491 = phi_1431_;
    unnamed.gl_Position = vec4<f32>(_e479.x, _e479.y, ((f32(((_e193.x << bitcast<u32>(8u)) | _e491)) * 0.000000059604645f) + 0.000000029802322f), _e479.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e8 = P0_;
    let _e9 = V0_;
    let _e10 = Q0_;
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
