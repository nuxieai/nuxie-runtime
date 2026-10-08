enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(15) override A6_: bool = false;
@id(1) override Vi: bool = true;
@id(2) override Wi: bool = true;
@id(8) override cj: bool = true;

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
var<private> V0_: vec3<f32>;
@group(0) @binding(3)
var WC: texture_2d<u32>;
var<private> P0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Ta: sampler;

fn main_1() {
    var phi_1284_: i32;
    var phi_1324_: f32;
    var phi_1312_: f32;
    var phi_1290_: bool;
    var phi_1288_: i32;
    var phi_1287_: i32;
    var phi_1285_: i32;
    var phi_1293_: i32;
    var phi_1292_: i32;
    var phi_1296_: bool;
    var phi_1298_: vec4<u32>;
    var phi_1297_: vec4<u32>;
    var phi_1317_: u32;
    var phi_1308_: vec4<u32>;
    var phi_1319_: f32;
    var phi_1329_: f32;
    var phi_1333_: f32;
    var phi_666_: bool;
    var phi_1336_: f32;
    var phi_1347_: vec2<f32>;
    var phi_1346_: vec2<f32>;
    var phi_1353_: f32;
    var phi_1360_: vec2<f32>;
    var phi_1352_: f32;
    var phi_1343_: vec2<f32>;
    var phi_1369_: vec2<f32>;
    var phi_1309_: vec2<f32>;
    var phi_1368_: vec2<f32>;
    var phi_1411_: bool;
    var phi_1409_: vec4<f32>;
    var phi_1410_: vec4<f32>;
    var phi_918_: bool;
    var phi_1430_: u32;
    var phi_1429_: u32;

    let _e76 = gl_VertexIndex_1;
    let _e81 = ((_e76 & 536870912i) != 0i);
    let _e82 = (_e76 & 268435455i);
    if A6_ {
        let _e83 = select(5i, 6i, _e81);
        let _e89 = (_e82 & ((1i << bitcast<u32>(_e83)) - 1i));
        let _e90 = select(1i, 2i, _e81);
        let _e96 = (_e89 & ((1i << bitcast<u32>(_e90)) - 1i));
        phi_1284_ = _e96;
        if !(_e81) {
            phi_1284_ = (_e96 + 1i);
        }
        let _e100 = phi_1284_;
        phi_1324_ = select(1f, 0f, ((_e100 == 0i) || (_e100 == 3i)));
        phi_1312_ = select(1f, -1f, (_e100 < 2i));
        phi_1290_ = false;
        phi_1288_ = (_e82 >> bitcast<u32>(_e83));
        phi_1287_ = 8i;
        phi_1285_ = (_e89 >> bitcast<u32>(_e90));
    } else {
        let _e107 = select(4i, 5i, _e81);
        let _e113 = (_e82 & ((1i << bitcast<u32>(_e107)) - 1i));
        let _e117 = (!(_e81) && (_e113 == 9i));
        phi_1324_ = 1f;
        phi_1312_ = 0f;
        phi_1290_ = _e117;
        phi_1288_ = (_e82 >> bitcast<u32>(_e107));
        phi_1287_ = select(8i, 17i, _e81);
        phi_1285_ = select(_e113, 0i, _e117);
    }
    let _e120 = phi_1324_;
    let _e122 = phi_1312_;
    let _e124 = phi_1290_;
    let _e126 = phi_1288_;
    let _e128 = phi_1287_;
    let _e130 = phi_1285_;
    let _e132 = min(_e130, (_e128 - 1i));
    let _e134 = ((_e126 * _e128) + _e132);
    let _e139 = textureLoad(UB, vec2<i32>((_e134 & 2047i), (_e134 >> bitcast<u32>(11i))), 0i);
    let _e143 = (max((_e139.w & 65535u), 1u) - 1u);
    let _e150 = textureLoad(BD, vec2<i32>(bitcast<i32>((_e143 & 255u)), bitcast<i32>((_e143 >> bitcast<u32>(8i)))), 0i);
    let _e152 = bitcast<vec2<f32>>(_e150.xy);
    let _e154 = (_e150.z & 65535u);
    let _e156 = (_e154 * 4u);
    let _e163 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e156 & 255u)), bitcast<i32>((_e156 >> bitcast<u32>(8i)))), 0i);
    let _e164 = bitcast<vec4<f32>>(_e163);
    let _e171 = mat2x2<f32>(vec2<f32>(_e164.x, _e164.y), vec2<f32>(_e164.z, _e164.w));
    let _e172 = (_e156 + 1u);
    let _e179 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e172 & 255u)), bitcast<i32>((_e172 >> bitcast<u32>(8i)))), 0i);
    let _e181 = bitcast<vec2<f32>>(_e179.xy);
    let _e184 = (_e156 + 2u);
    let _e191 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e184 & 255u)), bitcast<i32>((_e184 >> bitcast<u32>(8i)))), 0i);
    let _e193 = (_e139.w & 8388608u);
    if A6_ {
        phi_1292_ = _e130;
    } else {
        phi_1293_ = _e130;
        if (((_e193 != 0u) && !(_e81)) && !(_e124)) {
            phi_1293_ = (_e130 - 1i);
        }
        let _e201 = phi_1293_;
        phi_1292_ = _e201;
    }
    let _e203 = phi_1292_;
    phi_1317_ = _e139.w;
    phi_1308_ = _e139;
    if (_e203 != _e132) {
        let _e206 = ((_e134 + _e203) - _e132);
        let _e211 = textureLoad(UB, vec2<i32>((_e206 & 2047i), (_e206 >> bitcast<u32>(11i))), 0i);
        if ((_e211.w & 8454143u) != (_e139.w & 8454143u)) {
            if A6_ {
                phi_1296_ = (_e152.x != 0f);
            } else {
                phi_1296_ = true;
            }
            let _e219 = phi_1296_;
            phi_1298_ = _e139;
            if _e219 {
                let _e220 = bitcast<i32>(_e150.w);
                let _e225 = textureLoad(UB, vec2<i32>((_e220 & 2047i), (_e220 >> bitcast<u32>(11i))), 0i);
                phi_1298_ = _e225;
            }
            let _e227 = phi_1298_;
            phi_1297_ = _e227;
        } else {
            phi_1297_ = _e211;
        }
        let _e229 = phi_1297_;
        phi_1317_ = ((_e229.w & 4286578687u) | _e193);
        phi_1308_ = _e229;
    }
    let _e234 = phi_1317_;
    let _e236 = phi_1308_;
    if A6_ {
        let _e239 = (f32(_e236.z) * 0.0000000014629181f);
        let _e243 = vec2<f32>(sin(_e239), -(cos(_e239)));
        let _e248 = (_e122 * sign(determinant(_e171)));
        let _e250 = ((_e234 & 1048576u) != 0u);
        phi_1319_ = _e248;
        if _e250 {
            phi_1319_ = min(_e248, 0f);
        }
        let _e253 = phi_1319_;
        phi_1329_ = _e253;
        if ((_e234 & 524288u) != 0u) {
            phi_1329_ = max(_e253, 0f);
        }
        let _e258 = phi_1329_;
        let _e260 = select(0f, _e258, (_e120 == 0f));
        let _e261 = (_e234 & 469762048u);
        phi_1360_ = _e243;
        phi_1352_ = _e260;
        phi_1343_ = _e243;
        if (_e261 > 134217728u) {
            let _e266 = f32((_e236.z & 65535u));
            let _e267 = (_e266 * 0.000015259022f);
            let _e271 = sqrt(max((1f - (_e267 * _e267)), 0f));
            phi_1333_ = _e271;
            if (((_e234 & 4194304u) != 0u) == _e250) {
                phi_1333_ = -(_e271);
            }
            let _e275 = phi_1333_;
            let _e280 = (mat2x2<f32>(vec2<f32>(_e267, _e275), vec2<f32>(-(_e275), _e267)) * _e243);
            let _e281 = (_e261 == 201326592u);
            phi_666_ = _e281;
            if !(_e281) {
                phi_666_ = ((_e261 != 335544320u) && (_e267 < 0.25f));
            }
            let _e287 = phi_666_;
            let _e289 = ((_e234 & 2097152u) != 0u);
            if (_e261 == 335544320u) {
                phi_1346_ = (_e243 + _e280);
            } else {
                phi_1347_ = _e243;
                if (_e289 || !(_e287)) {
                    if _e287 {
                        phi_1336_ = _e267;
                    } else {
                        phi_1336_ = (65535f / _e266);
                    }
                    let _e296 = phi_1336_;
                    phi_1347_ = (_e280 * _e296);
                }
                let _e299 = phi_1347_;
                phi_1346_ = _e299;
            }
            let _e301 = phi_1346_;
            phi_1353_ = _e260;
            if (!(_e81) && _e287) {
                phi_1353_ = (0.5f * _e258);
            }
            let _e309 = phi_1353_;
            phi_1360_ = select(_e243, _e280, vec2((_e287 || _e289)));
            phi_1352_ = _e309;
            phi_1343_ = _e301;
        }
        let _e311 = phi_1360_;
        let _e313 = phi_1352_;
        let _e315 = phi_1343_;
        let _e320 = ((_e171 * (bitcast<vec2<f32>>(_e236.xy) + (_e315 * (_e258 * bitcast<f32>(_e179.z))))) + _e181);
        phi_1369_ = _e320;
        if (_e313 != 0f) {
            phi_1369_ = (_e320 + (sign((_e311 * _naga_inverse_2x2_f32(_e171))) * _e313));
        }
        let _e328 = phi_1369_;
        phi_1368_ = _e328;
    } else {
        if _e124 {
            phi_1309_ = _e152;
        } else {
            phi_1309_ = bitcast<vec2<f32>>(_e236.xy);
        }
        let _e332 = phi_1309_;
        phi_1368_ = ((_e171 * _e332) + _e181);
    }
    let _e336 = phi_1368_;
    if Vi {
        let _e337 = (_e154 * 8u);
        let _e338 = (_e337 + 2u);
        let _e345 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e338 & 255u)), bitcast<i32>((_e338 >> bitcast<u32>(8i)))), 0i);
        let _e353 = (_e337 + 3u);
        let _e360 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e353 & 255u)), bitcast<i32>((_e353 >> bitcast<u32>(8i)))), 0i);
        if any((_e345 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e365 = ((mat2x2<f32>(vec2<f32>(_e345.x, _e345.y), vec2<f32>(_e345.z, _e345.w)) * _e336) + _e360.xy);
            unnamed.gl_ClipDistance[0i] = (_e365.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e365.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e365.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e365.y);
        } else {
            let _e381 = (_e360.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e381;
            unnamed.gl_ClipDistance[2i] = _e381;
            unnamed.gl_ClipDistance[1i] = _e381;
            unnamed.gl_ClipDistance[0i] = _e381;
        }
    }
    if ((_e76 & 268435456i) != 0i) {
        O0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e396 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e150.z & 255u)), bitcast<i32>((_e154 >> bitcast<u32>(8i)))), 0i);
        let _e398 = (_e396.x & 15u);
        phi_1411_ = false;
        if Wi {
            let _e401 = ((_e396.x >> bitcast<u32>(4i)) & 15u);
            P0_ = f32(_e401);
            phi_1411_ = (_e401 != 0u);
        }
        let _e405 = phi_1411_;
        if (_e398 == 1u) {
            O0_ = unpack4x8unorm(_e396.y);
            if _e405 {
                let _e410 = O0_[3u];
                O0_[3u] = (_e410 * _e120);
            } else {
                let _e412 = O0_;
                O0_ = (_e412 * _e120);
            }
        } else {
            let _e414 = (_e154 * 8u);
            let _e421 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e414 & 255u)), bitcast<i32>((_e414 >> bitcast<u32>(8i)))), 0i);
            let _e429 = (_e414 + 1u);
            let _e436 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e429 & 255u)), bitcast<i32>((_e429 >> bitcast<u32>(8i)))), 0i);
            let _e438 = bitcast<f32>(_e396.y);
            let _e441 = ((mat2x2<f32>(vec2<f32>(_e421.x, _e421.y), vec2<f32>(_e421.z, _e421.w)) * _e336) + _e436.xy);
            let _e447 = vec4<f32>(_e441.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e453 = vec4<f32>(_e447.x, _e441.y, _e447.z, _e447.w);
            let _e458 = vec4<f32>(_e453.x, _e453.y, _e120, _e453.w);
            phi_1409_ = _e458;
            if (_e398 != 2u) {
                phi_1409_ = vec4<f32>(_e458.x, _e458.y, (_e120 + 2f), _e458.w);
            }
            let _e467 = phi_1409_;
            phi_1410_ = _e467;
            if (_e436.z > 0.9f) {
                phi_1410_ = vec4<f32>(_e467.x, _e467.y, -(_e467.z), _e467.w);
            }
            let _e478 = phi_1410_;
            O0_ = vec4<f32>(_e478.x, _e478.y, _e478.z, -(bitcast<f32>(((((_e398 << bitcast<u32>(28i)) | ((u32(_e438) - 1u) << bitcast<u32>(17i))) | (u32((_e436.w * 512f)) << bitcast<u32>(8i))) | u32((fract(_e438) * 256f))))));
        }
        phi_918_ = cj;
        if cj {
            phi_918_ = ((_e396.x & 2048u) != 0u);
        }
        let _e506 = phi_918_;
        if _e506 {
            let _e507 = (_e154 * 8u);
            let _e508 = (_e507 + 4u);
            let _e515 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e508 & 255u)), bitcast<i32>((_e508 >> bitcast<u32>(8i)))), 0i);
            let _e523 = (_e507 + 5u);
            let _e530 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e523 & 255u)), bitcast<i32>((_e523 >> bitcast<u32>(8i)))), 0i);
            let _e533 = ((mat2x2<f32>(vec2<f32>(_e515.x, _e515.y), vec2<f32>(_e515.z, _e515.w)) * _e336) + _e530.xy);
            V0_ = vec3<f32>(_e533.x, _e533.y, (1f + _e530.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e540 = j.Dg;
    let _e542 = j.Eg;
    let _e550 = vec4<f32>(((_e336.x * _e540) - 1f), ((_e336.y * _e542) - sign(_e542)), 0f, 1f);
    if A6_ {
        let _e552 = u32((_e120 * 254f));
        phi_1430_ = _e552;
        if ((_e76 & 1073741824i) == 0i) {
            phi_1430_ = (_e552 + bitcast<u32>(1i));
        }
        let _e557 = phi_1430_;
        phi_1429_ = _e557;
    } else {
        phi_1429_ = 255u;
    }
    let _e559 = phi_1429_;
    unnamed.gl_Position = vec4<f32>(_e550.x, _e550.y, ((f32(((_e191.x << bitcast<u32>(8u)) | _e559)) * 0.000000059604645f) + 0.000000029802322f), _e550.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = unnamed.gl_Position;
    let _e10 = unnamed.gl_ClipDistance;
    let _e11 = O0_;
    let _e12 = V0_;
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
