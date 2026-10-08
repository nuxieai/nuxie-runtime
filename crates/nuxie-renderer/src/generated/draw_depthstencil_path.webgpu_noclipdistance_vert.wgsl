struct hi {
    v2_: array<vec4<u32>>,
}

struct gi {
    v2_: array<vec4<u32>>,
}

struct kg {
    v2_: array<vec4<f32>>,
}

struct jg {
    v2_: array<vec2<u32>>,
}

struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
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

@id(15) override I7_: bool = false;
@id(16) override fe: bool = false;
@id(2) override aj: bool = true;
@id(8) override gj: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> CD: hi;
@group(0) @binding(2)
var<storage> KB: gi;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
@group(0) @binding(3)
var<storage> WC: jg;
var<private> Q0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Va: sampler;

fn main_1() {
    var phi_1196_: bool;
    var phi_1197_: i32;
    var phi_1253_: f32;
    var phi_1206_: f32;
    var phi_1203_: bool;
    var phi_1201_: i32;
    var phi_1200_: i32;
    var phi_1198_: i32;
    var phi_1245_: f32;
    var phi_1244_: f32;
    var phi_1209_: bool;
    var phi_1243_: f32;
    var phi_1208_: bool;
    var phi_1214_: i32;
    var phi_1219_: bool;
    var phi_1221_: vec4<u32>;
    var phi_1220_: vec4<u32>;
    var phi_1261_: u32;
    var phi_1238_: vec4<u32>;
    var phi_1227_: bool;
    var phi_1263_: f32;
    var phi_1275_: f32;
    var phi_1267_: f32;
    var phi_622_: bool;
    var phi_1270_: f32;
    var phi_1285_: vec2<f32>;
    var phi_1284_: vec2<f32>;
    var phi_1295_: f32;
    var phi_1303_: vec2<f32>;
    var phi_1292_: f32;
    var phi_1279_: vec2<f32>;
    var phi_1302_: vec2<f32>;
    var phi_1291_: f32;
    var phi_1288_: f32;
    var phi_1278_: vec2<f32>;
    var phi_1314_: vec2<f32>;
    var phi_1239_: vec2<f32>;
    var phi_1313_: vec2<f32>;
    var phi_1360_: bool;
    var phi_1358_: vec4<f32>;
    var phi_1359_: vec4<f32>;
    var phi_845_: bool;
    var phi_1369_: bool;
    var phi_1379_: u32;
    var phi_1378_: u32;

    let _e76 = gl_VertexIndex_1;
    let _e81 = ((_e76 & 536870912i) != 0i);
    let _e82 = (_e76 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1196_ = true;
                break;
            }
            if fe {
                phi_1196_ = true;
                break;
            }
            phi_1196_ = false;
            break;
        }
    }
    let _e85 = phi_1196_;
    if _e85 {
        let _e86 = select(5i, 6i, _e81);
        let _e92 = (_e82 & ((1i << bitcast<u32>(_e86)) - 1i));
        let _e93 = select(1i, 2i, _e81);
        let _e99 = (_e92 & ((1i << bitcast<u32>(_e93)) - 1i));
        phi_1197_ = _e99;
        if (I7_ && !(_e81)) {
            phi_1197_ = (_e99 + 1i);
        }
        let _e104 = phi_1197_;
        phi_1253_ = select(1f, 0f, ((_e104 == 0i) || (_e104 == 3i)));
        phi_1206_ = select(-1f, 1f, (_e104 < 2i));
        phi_1203_ = false;
        phi_1201_ = (_e82 >> bitcast<u32>(_e86));
        phi_1200_ = 8i;
        phi_1198_ = (_e92 >> bitcast<u32>(_e93));
    } else {
        let _e111 = select(4i, 5i, _e81);
        let _e117 = (_e82 & ((1i << bitcast<u32>(_e111)) - 1i));
        let _e121 = (!(_e81) && (_e117 == 9i));
        phi_1253_ = 1f;
        phi_1206_ = 0f;
        phi_1203_ = _e121;
        phi_1201_ = (_e82 >> bitcast<u32>(_e111));
        phi_1200_ = select(8i, 17i, _e81);
        phi_1198_ = select(_e117, 0i, _e121);
    }
    let _e124 = phi_1253_;
    let _e126 = phi_1206_;
    let _e128 = phi_1203_;
    let _e130 = phi_1201_;
    let _e132 = phi_1200_;
    let _e134 = phi_1198_;
    let _e136 = min(_e134, (_e132 - 1i));
    let _e138 = ((_e130 * _e132) + _e136);
    let _e143 = textureLoad(UB, vec2<i32>((_e138 & 2047i), (_e138 >> bitcast<u32>(11i))), 0i);
    let _e150 = CD.v2_[(max((_e143.w & 65535u), 1u) - 1u)];
    let _e152 = bitcast<vec2<f32>>(_e150.xy);
    let _e154 = (_e150.z & 65535u);
    let _e156 = (_e154 * 4u);
    let _e159 = KB.v2_[_e156];
    let _e160 = bitcast<vec4<f32>>(_e159);
    let _e167 = mat2x2<f32>(vec2<f32>(_e160.x, _e160.y), vec2<f32>(_e160.z, _e160.w));
    let _e171 = KB.v2_[(_e156 + 1u)];
    let _e173 = bitcast<vec2<f32>>(_e171.xy);
    let _e179 = KB.v2_[(_e156 + 2u)];
    let _e181 = (_e143.w & 8388608u);
    if I7_ {
        phi_1243_ = _e126;
        phi_1208_ = false;
    } else {
        if fe {
            let _e182 = (_e181 != 0u);
            phi_1245_ = _e126;
            if _e182 {
                phi_1245_ = -(_e126);
            }
            let _e185 = phi_1245_;
            phi_1244_ = _e185;
            phi_1209_ = _e182;
        } else {
            phi_1244_ = _e126;
            phi_1209_ = (((_e181 != 0u) && !(_e81)) && !(_e128));
        }
        let _e192 = phi_1244_;
        let _e194 = phi_1209_;
        phi_1243_ = _e192;
        phi_1208_ = _e194;
    }
    let _e196 = phi_1243_;
    let _e198 = phi_1208_;
    phi_1214_ = _e134;
    if _e198 {
        phi_1214_ = (_e134 - 1i);
    }
    let _e201 = phi_1214_;
    phi_1261_ = _e143.w;
    phi_1238_ = _e143;
    if (_e201 != _e136) {
        let _e204 = ((_e138 + _e201) - _e136);
        let _e209 = textureLoad(UB, vec2<i32>((_e204 & 2047i), (_e204 >> bitcast<u32>(11i))), 0i);
        if ((_e209.w & 8454143u) != (_e143.w & 8454143u)) {
            if I7_ {
                phi_1219_ = (_e152.x != 0f);
            } else {
                phi_1219_ = true;
            }
            let _e217 = phi_1219_;
            phi_1221_ = _e143;
            if _e217 {
                let _e218 = bitcast<i32>(_e150.w);
                let _e223 = textureLoad(UB, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
                phi_1221_ = _e223;
            }
            let _e225 = phi_1221_;
            phi_1220_ = _e225;
        } else {
            phi_1220_ = _e209;
        }
        let _e227 = phi_1220_;
        phi_1261_ = ((_e227.w & 4286578687u) | _e181);
        phi_1238_ = _e227;
    }
    let _e232 = phi_1261_;
    let _e234 = phi_1238_;
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1227_ = true;
                break;
            }
            if fe {
                phi_1227_ = true;
                break;
            }
            phi_1227_ = false;
            break;
        }
    }
    let _e237 = phi_1227_;
    if _e237 {
        let _e240 = (_e196 * sign(determinant(_e167)));
        let _e243 = (f32(_e234.z) * 0.0000000014629181f);
        let _e247 = vec2<f32>(sin(_e243), -(cos(_e243)));
        let _e251 = select(0f, _e240, (_e124 == 0f));
        phi_1302_ = _e247;
        phi_1291_ = _e251;
        phi_1288_ = _e240;
        phi_1278_ = _e247;
        if I7_ {
            let _e253 = ((_e232 & 1048576u) != 0u);
            phi_1263_ = _e240;
            if _e253 {
                phi_1263_ = min(_e240, 0f);
            }
            let _e256 = phi_1263_;
            phi_1275_ = _e256;
            if ((_e232 & 524288u) != 0u) {
                phi_1275_ = max(_e256, 0f);
            }
            let _e261 = phi_1275_;
            let _e262 = (_e232 & 469762048u);
            phi_1303_ = _e247;
            phi_1292_ = _e251;
            phi_1279_ = _e247;
            if (_e262 > 134217728u) {
                let _e267 = f32((_e234.z & 65535u));
                let _e268 = (_e267 * 0.000015259022f);
                let _e272 = sqrt(max((1f - (_e268 * _e268)), 0f));
                phi_1267_ = _e272;
                if (((_e232 & 4194304u) != 0u) == _e253) {
                    phi_1267_ = -(_e272);
                }
                let _e276 = phi_1267_;
                let _e281 = (mat2x2<f32>(vec2<f32>(_e268, _e276), vec2<f32>(-(_e276), _e268)) * _e247);
                let _e282 = (_e262 == 201326592u);
                phi_622_ = _e282;
                if !(_e282) {
                    phi_622_ = ((_e262 != 335544320u) && (_e268 < 0.25f));
                }
                let _e288 = phi_622_;
                let _e290 = ((_e232 & 2097152u) != 0u);
                if (_e262 == 335544320u) {
                    phi_1284_ = (_e247 + _e281);
                } else {
                    phi_1285_ = _e247;
                    if (_e290 || !(_e288)) {
                        if _e288 {
                            phi_1270_ = _e268;
                        } else {
                            phi_1270_ = (65535f / _e267);
                        }
                        let _e297 = phi_1270_;
                        phi_1285_ = (_e281 * _e297);
                    }
                    let _e300 = phi_1285_;
                    phi_1284_ = _e300;
                }
                let _e302 = phi_1284_;
                phi_1295_ = _e251;
                if (!(_e81) && _e288) {
                    phi_1295_ = (0.5f * _e261);
                }
                let _e310 = phi_1295_;
                phi_1303_ = select(_e247, _e281, vec2((_e288 || _e290)));
                phi_1292_ = _e310;
                phi_1279_ = _e302;
            }
            let _e312 = phi_1303_;
            let _e314 = phi_1292_;
            let _e316 = phi_1279_;
            phi_1302_ = _e312;
            phi_1291_ = _e314;
            phi_1288_ = _e261;
            phi_1278_ = _e316;
        }
        let _e318 = phi_1302_;
        let _e320 = phi_1291_;
        let _e322 = phi_1288_;
        let _e324 = phi_1278_;
        let _e329 = ((_e167 * (bitcast<vec2<f32>>(_e234.xy) + (_e324 * (_e322 * bitcast<f32>(_e171.z))))) + _e173);
        phi_1314_ = _e329;
        if (_e320 != 0f) {
            phi_1314_ = (_e329 + (sign((_e318 * _naga_inverse_2x2_f32(_e167))) * _e320));
        }
        let _e337 = phi_1314_;
        phi_1313_ = _e337;
    } else {
        if _e128 {
            phi_1239_ = _e152;
        } else {
            phi_1239_ = bitcast<vec2<f32>>(_e234.xy);
        }
        let _e341 = phi_1239_;
        phi_1313_ = ((_e167 * _e341) + _e173);
    }
    let _e345 = phi_1313_;
    if ((_e76 & 268435456i) != 0i) {
        P0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e348 = WC.v2_[_e154];
        let _e350 = (_e348.x & 15u);
        phi_1360_ = false;
        if aj {
            let _e353 = ((_e348.x >> bitcast<u32>(4i)) & 15u);
            Q0_ = f32(_e353);
            phi_1360_ = (_e353 != 0u);
        }
        let _e357 = phi_1360_;
        if (_e350 == 1u) {
            P0_ = unpack4x8unorm(_e348.y);
            if _e357 {
                let _e362 = P0_[3u];
                P0_[3u] = (_e362 * _e124);
            } else {
                let _e364 = P0_;
                P0_ = (_e364 * _e124);
            }
        } else {
            let _e366 = (_e154 * 8u);
            let _e369 = JB.v2_[_e366];
            let _e380 = JB.v2_[(_e366 + 1u)];
            let _e382 = bitcast<f32>(_e348.y);
            let _e385 = ((mat2x2<f32>(vec2<f32>(_e369.x, _e369.y), vec2<f32>(_e369.z, _e369.w)) * _e345) + _e380.xy);
            let _e391 = vec4<f32>(_e385.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e397 = vec4<f32>(_e391.x, _e385.y, _e391.z, _e391.w);
            let _e402 = vec4<f32>(_e397.x, _e397.y, _e124, _e397.w);
            phi_1358_ = _e402;
            if (_e350 != 2u) {
                phi_1358_ = vec4<f32>(_e402.x, _e402.y, (_e124 + 2f), _e402.w);
            }
            let _e411 = phi_1358_;
            phi_1359_ = _e411;
            if (_e380.z > 0.9f) {
                phi_1359_ = vec4<f32>(_e411.x, _e411.y, -(_e411.z), _e411.w);
            }
            let _e422 = phi_1359_;
            P0_ = vec4<f32>(_e422.x, _e422.y, _e422.z, -(bitcast<f32>(((((_e350 << bitcast<u32>(28i)) | ((u32(_e382) - 1u) << bitcast<u32>(17i))) | (u32((_e380.w * 512f)) << bitcast<u32>(8i))) | u32((fract(_e382) * 256f))))));
        }
        phi_845_ = gj;
        if gj {
            phi_845_ = ((_e348.x & 2048u) != 0u);
        }
        let _e450 = phi_845_;
        if _e450 {
            let _e451 = (_e154 * 8u);
            let _e455 = JB.v2_[(_e451 + 4u)];
            let _e466 = JB.v2_[(_e451 + 5u)];
            let _e469 = ((mat2x2<f32>(vec2<f32>(_e455.x, _e455.y), vec2<f32>(_e455.z, _e455.w)) * _e345) + _e466.xy);
            V0_ = vec3<f32>(_e469.x, _e469.y, (1f + _e466.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e476 = j.Gg;
    let _e478 = j.Hg;
    let _e486 = vec4<f32>(((_e345.x * _e476) - 1f), ((_e345.y * _e478) - sign(_e478)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1369_ = true;
                break;
            }
            if fe {
                phi_1369_ = true;
                break;
            }
            phi_1369_ = false;
            break;
        }
    }
    let _e489 = phi_1369_;
    if _e489 {
        let _e491 = u32((_e124 * 254f));
        phi_1379_ = _e491;
        if ((_e76 & 1073741824i) == 0i) {
            phi_1379_ = (_e491 + bitcast<u32>(1i));
        }
        let _e496 = phi_1379_;
        phi_1378_ = _e496;
    } else {
        phi_1378_ = 255u;
    }
    let _e498 = phi_1378_;
    unnamed.gl_Position = vec4<f32>(_e486.x, _e486.y, ((f32(((_e179.x << bitcast<u32>(8u)) | _e498)) * 0.000000059604645f) + 0.000000029802322f), _e486.w);
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
