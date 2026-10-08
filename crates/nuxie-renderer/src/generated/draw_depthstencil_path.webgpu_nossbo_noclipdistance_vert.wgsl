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
var CD: texture_2d<u32>;
@group(0) @binding(2)
var KB: texture_2d<u32>;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
@group(0) @binding(3)
var WC: texture_2d<u32>;
var<private> Q0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Va: sampler;

fn main_1() {
    var phi_1265_: bool;
    var phi_1266_: i32;
    var phi_1322_: f32;
    var phi_1275_: f32;
    var phi_1272_: bool;
    var phi_1270_: i32;
    var phi_1269_: i32;
    var phi_1267_: i32;
    var phi_1314_: f32;
    var phi_1313_: f32;
    var phi_1278_: bool;
    var phi_1312_: f32;
    var phi_1277_: bool;
    var phi_1283_: i32;
    var phi_1288_: bool;
    var phi_1290_: vec4<u32>;
    var phi_1289_: vec4<u32>;
    var phi_1330_: u32;
    var phi_1307_: vec4<u32>;
    var phi_1296_: bool;
    var phi_1332_: f32;
    var phi_1344_: f32;
    var phi_1336_: f32;
    var phi_646_: bool;
    var phi_1339_: f32;
    var phi_1354_: vec2<f32>;
    var phi_1353_: vec2<f32>;
    var phi_1364_: f32;
    var phi_1372_: vec2<f32>;
    var phi_1361_: f32;
    var phi_1348_: vec2<f32>;
    var phi_1371_: vec2<f32>;
    var phi_1360_: f32;
    var phi_1357_: f32;
    var phi_1347_: vec2<f32>;
    var phi_1383_: vec2<f32>;
    var phi_1308_: vec2<f32>;
    var phi_1382_: vec2<f32>;
    var phi_1429_: bool;
    var phi_1427_: vec4<f32>;
    var phi_1428_: vec4<f32>;
    var phi_901_: bool;
    var phi_1438_: bool;
    var phi_1448_: u32;
    var phi_1447_: u32;

    let _e76 = gl_VertexIndex_1;
    let _e81 = ((_e76 & 536870912i) != 0i);
    let _e82 = (_e76 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1265_ = true;
                break;
            }
            if fe {
                phi_1265_ = true;
                break;
            }
            phi_1265_ = false;
            break;
        }
    }
    let _e85 = phi_1265_;
    if _e85 {
        let _e86 = select(5i, 6i, _e81);
        let _e92 = (_e82 & ((1i << bitcast<u32>(_e86)) - 1i));
        let _e93 = select(1i, 2i, _e81);
        let _e99 = (_e92 & ((1i << bitcast<u32>(_e93)) - 1i));
        phi_1266_ = _e99;
        if (I7_ && !(_e81)) {
            phi_1266_ = (_e99 + 1i);
        }
        let _e104 = phi_1266_;
        phi_1322_ = select(1f, 0f, ((_e104 == 0i) || (_e104 == 3i)));
        phi_1275_ = select(-1f, 1f, (_e104 < 2i));
        phi_1272_ = false;
        phi_1270_ = (_e82 >> bitcast<u32>(_e86));
        phi_1269_ = 8i;
        phi_1267_ = (_e92 >> bitcast<u32>(_e93));
    } else {
        let _e111 = select(4i, 5i, _e81);
        let _e117 = (_e82 & ((1i << bitcast<u32>(_e111)) - 1i));
        let _e121 = (!(_e81) && (_e117 == 9i));
        phi_1322_ = 1f;
        phi_1275_ = 0f;
        phi_1272_ = _e121;
        phi_1270_ = (_e82 >> bitcast<u32>(_e111));
        phi_1269_ = select(8i, 17i, _e81);
        phi_1267_ = select(_e117, 0i, _e121);
    }
    let _e124 = phi_1322_;
    let _e126 = phi_1275_;
    let _e128 = phi_1272_;
    let _e130 = phi_1270_;
    let _e132 = phi_1269_;
    let _e134 = phi_1267_;
    let _e136 = min(_e134, (_e132 - 1i));
    let _e138 = ((_e130 * _e132) + _e136);
    let _e143 = textureLoad(UB, vec2<i32>((_e138 & 2047i), (_e138 >> bitcast<u32>(11i))), 0i);
    let _e147 = (max((_e143.w & 65535u), 1u) - 1u);
    let _e154 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e147 & 255u)), bitcast<i32>((_e147 >> bitcast<u32>(8i)))), 0i);
    let _e156 = bitcast<vec2<f32>>(_e154.xy);
    let _e158 = (_e154.z & 65535u);
    let _e160 = (_e158 * 4u);
    let _e167 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e160 & 255u)), bitcast<i32>((_e160 >> bitcast<u32>(8i)))), 0i);
    let _e168 = bitcast<vec4<f32>>(_e167);
    let _e175 = mat2x2<f32>(vec2<f32>(_e168.x, _e168.y), vec2<f32>(_e168.z, _e168.w));
    let _e176 = (_e160 + 1u);
    let _e183 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e176 & 255u)), bitcast<i32>((_e176 >> bitcast<u32>(8i)))), 0i);
    let _e185 = bitcast<vec2<f32>>(_e183.xy);
    let _e188 = (_e160 + 2u);
    let _e195 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e188 & 255u)), bitcast<i32>((_e188 >> bitcast<u32>(8i)))), 0i);
    let _e197 = (_e143.w & 8388608u);
    if I7_ {
        phi_1312_ = _e126;
        phi_1277_ = false;
    } else {
        if fe {
            let _e198 = (_e197 != 0u);
            phi_1314_ = _e126;
            if _e198 {
                phi_1314_ = -(_e126);
            }
            let _e201 = phi_1314_;
            phi_1313_ = _e201;
            phi_1278_ = _e198;
        } else {
            phi_1313_ = _e126;
            phi_1278_ = (((_e197 != 0u) && !(_e81)) && !(_e128));
        }
        let _e208 = phi_1313_;
        let _e210 = phi_1278_;
        phi_1312_ = _e208;
        phi_1277_ = _e210;
    }
    let _e212 = phi_1312_;
    let _e214 = phi_1277_;
    phi_1283_ = _e134;
    if _e214 {
        phi_1283_ = (_e134 - 1i);
    }
    let _e217 = phi_1283_;
    phi_1330_ = _e143.w;
    phi_1307_ = _e143;
    if (_e217 != _e136) {
        let _e220 = ((_e138 + _e217) - _e136);
        let _e225 = textureLoad(UB, vec2<i32>((_e220 & 2047i), (_e220 >> bitcast<u32>(11i))), 0i);
        if ((_e225.w & 8454143u) != (_e143.w & 8454143u)) {
            if I7_ {
                phi_1288_ = (_e156.x != 0f);
            } else {
                phi_1288_ = true;
            }
            let _e233 = phi_1288_;
            phi_1290_ = _e143;
            if _e233 {
                let _e234 = bitcast<i32>(_e154.w);
                let _e239 = textureLoad(UB, vec2<i32>((_e234 & 2047i), (_e234 >> bitcast<u32>(11i))), 0i);
                phi_1290_ = _e239;
            }
            let _e241 = phi_1290_;
            phi_1289_ = _e241;
        } else {
            phi_1289_ = _e225;
        }
        let _e243 = phi_1289_;
        phi_1330_ = ((_e243.w & 4286578687u) | _e197);
        phi_1307_ = _e243;
    }
    let _e248 = phi_1330_;
    let _e250 = phi_1307_;
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1296_ = true;
                break;
            }
            if fe {
                phi_1296_ = true;
                break;
            }
            phi_1296_ = false;
            break;
        }
    }
    let _e253 = phi_1296_;
    if _e253 {
        let _e256 = (_e212 * sign(determinant(_e175)));
        let _e259 = (f32(_e250.z) * 0.0000000014629181f);
        let _e263 = vec2<f32>(sin(_e259), -(cos(_e259)));
        let _e267 = select(0f, _e256, (_e124 == 0f));
        phi_1371_ = _e263;
        phi_1360_ = _e267;
        phi_1357_ = _e256;
        phi_1347_ = _e263;
        if I7_ {
            let _e269 = ((_e248 & 1048576u) != 0u);
            phi_1332_ = _e256;
            if _e269 {
                phi_1332_ = min(_e256, 0f);
            }
            let _e272 = phi_1332_;
            phi_1344_ = _e272;
            if ((_e248 & 524288u) != 0u) {
                phi_1344_ = max(_e272, 0f);
            }
            let _e277 = phi_1344_;
            let _e278 = (_e248 & 469762048u);
            phi_1372_ = _e263;
            phi_1361_ = _e267;
            phi_1348_ = _e263;
            if (_e278 > 134217728u) {
                let _e283 = f32((_e250.z & 65535u));
                let _e284 = (_e283 * 0.000015259022f);
                let _e288 = sqrt(max((1f - (_e284 * _e284)), 0f));
                phi_1336_ = _e288;
                if (((_e248 & 4194304u) != 0u) == _e269) {
                    phi_1336_ = -(_e288);
                }
                let _e292 = phi_1336_;
                let _e297 = (mat2x2<f32>(vec2<f32>(_e284, _e292), vec2<f32>(-(_e292), _e284)) * _e263);
                let _e298 = (_e278 == 201326592u);
                phi_646_ = _e298;
                if !(_e298) {
                    phi_646_ = ((_e278 != 335544320u) && (_e284 < 0.25f));
                }
                let _e304 = phi_646_;
                let _e306 = ((_e248 & 2097152u) != 0u);
                if (_e278 == 335544320u) {
                    phi_1353_ = (_e263 + _e297);
                } else {
                    phi_1354_ = _e263;
                    if (_e306 || !(_e304)) {
                        if _e304 {
                            phi_1339_ = _e284;
                        } else {
                            phi_1339_ = (65535f / _e283);
                        }
                        let _e313 = phi_1339_;
                        phi_1354_ = (_e297 * _e313);
                    }
                    let _e316 = phi_1354_;
                    phi_1353_ = _e316;
                }
                let _e318 = phi_1353_;
                phi_1364_ = _e267;
                if (!(_e81) && _e304) {
                    phi_1364_ = (0.5f * _e277);
                }
                let _e326 = phi_1364_;
                phi_1372_ = select(_e263, _e297, vec2((_e304 || _e306)));
                phi_1361_ = _e326;
                phi_1348_ = _e318;
            }
            let _e328 = phi_1372_;
            let _e330 = phi_1361_;
            let _e332 = phi_1348_;
            phi_1371_ = _e328;
            phi_1360_ = _e330;
            phi_1357_ = _e277;
            phi_1347_ = _e332;
        }
        let _e334 = phi_1371_;
        let _e336 = phi_1360_;
        let _e338 = phi_1357_;
        let _e340 = phi_1347_;
        let _e345 = ((_e175 * (bitcast<vec2<f32>>(_e250.xy) + (_e340 * (_e338 * bitcast<f32>(_e183.z))))) + _e185);
        phi_1383_ = _e345;
        if (_e336 != 0f) {
            phi_1383_ = (_e345 + (sign((_e334 * _naga_inverse_2x2_f32(_e175))) * _e336));
        }
        let _e353 = phi_1383_;
        phi_1382_ = _e353;
    } else {
        if _e128 {
            phi_1308_ = _e156;
        } else {
            phi_1308_ = bitcast<vec2<f32>>(_e250.xy);
        }
        let _e357 = phi_1308_;
        phi_1382_ = ((_e175 * _e357) + _e185);
    }
    let _e361 = phi_1382_;
    if ((_e76 & 268435456i) != 0i) {
        P0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e368 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e154.z & 255u)), bitcast<i32>((_e158 >> bitcast<u32>(8i)))), 0i);
        let _e370 = (_e368.x & 15u);
        phi_1429_ = false;
        if aj {
            let _e373 = ((_e368.x >> bitcast<u32>(4i)) & 15u);
            Q0_ = f32(_e373);
            phi_1429_ = (_e373 != 0u);
        }
        let _e377 = phi_1429_;
        if (_e370 == 1u) {
            P0_ = unpack4x8unorm(_e368.y);
            if _e377 {
                let _e382 = P0_[3u];
                P0_[3u] = (_e382 * _e124);
            } else {
                let _e384 = P0_;
                P0_ = (_e384 * _e124);
            }
        } else {
            let _e386 = (_e158 * 8u);
            let _e393 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e386 & 255u)), bitcast<i32>((_e386 >> bitcast<u32>(8i)))), 0i);
            let _e401 = (_e386 + 1u);
            let _e408 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e401 & 255u)), bitcast<i32>((_e401 >> bitcast<u32>(8i)))), 0i);
            let _e410 = bitcast<f32>(_e368.y);
            let _e413 = ((mat2x2<f32>(vec2<f32>(_e393.x, _e393.y), vec2<f32>(_e393.z, _e393.w)) * _e361) + _e408.xy);
            let _e419 = vec4<f32>(_e413.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e425 = vec4<f32>(_e419.x, _e413.y, _e419.z, _e419.w);
            let _e430 = vec4<f32>(_e425.x, _e425.y, _e124, _e425.w);
            phi_1427_ = _e430;
            if (_e370 != 2u) {
                phi_1427_ = vec4<f32>(_e430.x, _e430.y, (_e124 + 2f), _e430.w);
            }
            let _e439 = phi_1427_;
            phi_1428_ = _e439;
            if (_e408.z > 0.9f) {
                phi_1428_ = vec4<f32>(_e439.x, _e439.y, -(_e439.z), _e439.w);
            }
            let _e450 = phi_1428_;
            P0_ = vec4<f32>(_e450.x, _e450.y, _e450.z, -(bitcast<f32>(((((_e370 << bitcast<u32>(28i)) | ((u32(_e410) - 1u) << bitcast<u32>(17i))) | (u32((_e408.w * 512f)) << bitcast<u32>(8i))) | u32((fract(_e410) * 256f))))));
        }
        phi_901_ = gj;
        if gj {
            phi_901_ = ((_e368.x & 2048u) != 0u);
        }
        let _e478 = phi_901_;
        if _e478 {
            let _e479 = (_e158 * 8u);
            let _e480 = (_e479 + 4u);
            let _e487 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e480 & 255u)), bitcast<i32>((_e480 >> bitcast<u32>(8i)))), 0i);
            let _e495 = (_e479 + 5u);
            let _e502 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e495 & 255u)), bitcast<i32>((_e495 >> bitcast<u32>(8i)))), 0i);
            let _e505 = ((mat2x2<f32>(vec2<f32>(_e487.x, _e487.y), vec2<f32>(_e487.z, _e487.w)) * _e361) + _e502.xy);
            V0_ = vec3<f32>(_e505.x, _e505.y, (1f + _e502.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e512 = j.Gg;
    let _e514 = j.Hg;
    let _e522 = vec4<f32>(((_e361.x * _e512) - 1f), ((_e361.y * _e514) - sign(_e514)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1438_ = true;
                break;
            }
            if fe {
                phi_1438_ = true;
                break;
            }
            phi_1438_ = false;
            break;
        }
    }
    let _e525 = phi_1438_;
    if _e525 {
        let _e527 = u32((_e124 * 254f));
        phi_1448_ = _e527;
        if ((_e76 & 1073741824i) == 0i) {
            phi_1448_ = (_e527 + bitcast<u32>(1i));
        }
        let _e532 = phi_1448_;
        phi_1447_ = _e532;
    } else {
        phi_1447_ = 255u;
    }
    let _e534 = phi_1447_;
    unnamed.gl_Position = vec4<f32>(_e522.x, _e522.y, ((f32(((_e195.x << bitcast<u32>(8u)) | _e534)) * 0.000000059604645f) + 0.000000029802322f), _e522.w);
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
