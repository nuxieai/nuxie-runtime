enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
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

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(15) override I7_: bool = false;
@id(16) override fe: bool = false;
@id(1) override Zi: bool = true;
@id(2) override aj: bool = true;
@id(8) override gj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
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
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Va: sampler;

fn main_1() {
    var phi_1352_: bool;
    var phi_1353_: i32;
    var phi_1409_: f32;
    var phi_1362_: f32;
    var phi_1359_: bool;
    var phi_1357_: i32;
    var phi_1356_: i32;
    var phi_1354_: i32;
    var phi_1401_: f32;
    var phi_1400_: f32;
    var phi_1365_: bool;
    var phi_1399_: f32;
    var phi_1364_: bool;
    var phi_1370_: i32;
    var phi_1375_: bool;
    var phi_1377_: vec4<u32>;
    var phi_1376_: vec4<u32>;
    var phi_1417_: u32;
    var phi_1394_: vec4<u32>;
    var phi_1383_: bool;
    var phi_1419_: f32;
    var phi_1431_: f32;
    var phi_1423_: f32;
    var phi_695_: bool;
    var phi_1426_: f32;
    var phi_1441_: vec2<f32>;
    var phi_1440_: vec2<f32>;
    var phi_1451_: f32;
    var phi_1459_: vec2<f32>;
    var phi_1448_: f32;
    var phi_1435_: vec2<f32>;
    var phi_1458_: vec2<f32>;
    var phi_1447_: f32;
    var phi_1444_: f32;
    var phi_1434_: vec2<f32>;
    var phi_1470_: vec2<f32>;
    var phi_1395_: vec2<f32>;
    var phi_1469_: vec2<f32>;
    var phi_1520_: bool;
    var phi_1518_: vec4<f32>;
    var phi_1519_: vec4<f32>;
    var phi_947_: bool;
    var phi_1531_: bool;
    var phi_1541_: u32;
    var phi_1540_: u32;

    let _e77 = gl_VertexIndex_1;
    let _e82 = ((_e77 & 536870912i) != 0i);
    let _e83 = (_e77 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1352_ = true;
                break;
            }
            if fe {
                phi_1352_ = true;
                break;
            }
            phi_1352_ = false;
            break;
        }
    }
    let _e86 = phi_1352_;
    if _e86 {
        let _e87 = select(5i, 6i, _e82);
        let _e93 = (_e83 & ((1i << bitcast<u32>(_e87)) - 1i));
        let _e94 = select(1i, 2i, _e82);
        let _e100 = (_e93 & ((1i << bitcast<u32>(_e94)) - 1i));
        phi_1353_ = _e100;
        if (I7_ && !(_e82)) {
            phi_1353_ = (_e100 + 1i);
        }
        let _e105 = phi_1353_;
        phi_1409_ = select(1f, 0f, ((_e105 == 0i) || (_e105 == 3i)));
        phi_1362_ = select(-1f, 1f, (_e105 < 2i));
        phi_1359_ = false;
        phi_1357_ = (_e83 >> bitcast<u32>(_e87));
        phi_1356_ = 8i;
        phi_1354_ = (_e93 >> bitcast<u32>(_e94));
    } else {
        let _e112 = select(4i, 5i, _e82);
        let _e118 = (_e83 & ((1i << bitcast<u32>(_e112)) - 1i));
        let _e122 = (!(_e82) && (_e118 == 9i));
        phi_1409_ = 1f;
        phi_1362_ = 0f;
        phi_1359_ = _e122;
        phi_1357_ = (_e83 >> bitcast<u32>(_e112));
        phi_1356_ = select(8i, 17i, _e82);
        phi_1354_ = select(_e118, 0i, _e122);
    }
    let _e125 = phi_1409_;
    let _e127 = phi_1362_;
    let _e129 = phi_1359_;
    let _e131 = phi_1357_;
    let _e133 = phi_1356_;
    let _e135 = phi_1354_;
    let _e137 = min(_e135, (_e133 - 1i));
    let _e139 = ((_e131 * _e133) + _e137);
    let _e144 = textureLoad(UB, vec2<i32>((_e139 & 2047i), (_e139 >> bitcast<u32>(11i))), 0i);
    let _e148 = (max((_e144.w & 65535u), 1u) - 1u);
    let _e155 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e148 & 255u)), bitcast<i32>((_e148 >> bitcast<u32>(8i)))), 0i);
    let _e157 = bitcast<vec2<f32>>(_e155.xy);
    let _e159 = (_e155.z & 65535u);
    let _e161 = (_e159 * 4u);
    let _e168 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e161 & 255u)), bitcast<i32>((_e161 >> bitcast<u32>(8i)))), 0i);
    let _e169 = bitcast<vec4<f32>>(_e168);
    let _e176 = mat2x2<f32>(vec2<f32>(_e169.x, _e169.y), vec2<f32>(_e169.z, _e169.w));
    let _e177 = (_e161 + 1u);
    let _e184 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e177 & 255u)), bitcast<i32>((_e177 >> bitcast<u32>(8i)))), 0i);
    let _e186 = bitcast<vec2<f32>>(_e184.xy);
    let _e189 = (_e161 + 2u);
    let _e196 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e189 & 255u)), bitcast<i32>((_e189 >> bitcast<u32>(8i)))), 0i);
    let _e198 = (_e144.w & 8388608u);
    if I7_ {
        phi_1399_ = _e127;
        phi_1364_ = false;
    } else {
        if fe {
            let _e199 = (_e198 != 0u);
            phi_1401_ = _e127;
            if _e199 {
                phi_1401_ = -(_e127);
            }
            let _e202 = phi_1401_;
            phi_1400_ = _e202;
            phi_1365_ = _e199;
        } else {
            phi_1400_ = _e127;
            phi_1365_ = (((_e198 != 0u) && !(_e82)) && !(_e129));
        }
        let _e209 = phi_1400_;
        let _e211 = phi_1365_;
        phi_1399_ = _e209;
        phi_1364_ = _e211;
    }
    let _e213 = phi_1399_;
    let _e215 = phi_1364_;
    phi_1370_ = _e135;
    if _e215 {
        phi_1370_ = (_e135 - 1i);
    }
    let _e218 = phi_1370_;
    phi_1417_ = _e144.w;
    phi_1394_ = _e144;
    if (_e218 != _e137) {
        let _e221 = ((_e139 + _e218) - _e137);
        let _e226 = textureLoad(UB, vec2<i32>((_e221 & 2047i), (_e221 >> bitcast<u32>(11i))), 0i);
        if ((_e226.w & 8454143u) != (_e144.w & 8454143u)) {
            if I7_ {
                phi_1375_ = (_e157.x != 0f);
            } else {
                phi_1375_ = true;
            }
            let _e234 = phi_1375_;
            phi_1377_ = _e144;
            if _e234 {
                let _e235 = bitcast<i32>(_e155.w);
                let _e240 = textureLoad(UB, vec2<i32>((_e235 & 2047i), (_e235 >> bitcast<u32>(11i))), 0i);
                phi_1377_ = _e240;
            }
            let _e242 = phi_1377_;
            phi_1376_ = _e242;
        } else {
            phi_1376_ = _e226;
        }
        let _e244 = phi_1376_;
        phi_1417_ = ((_e244.w & 4286578687u) | _e198);
        phi_1394_ = _e244;
    }
    let _e249 = phi_1417_;
    let _e251 = phi_1394_;
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1383_ = true;
                break;
            }
            if fe {
                phi_1383_ = true;
                break;
            }
            phi_1383_ = false;
            break;
        }
    }
    let _e254 = phi_1383_;
    if _e254 {
        let _e257 = (_e213 * sign(determinant(_e176)));
        let _e260 = (f32(_e251.z) * 0.0000000014629181f);
        let _e264 = vec2<f32>(sin(_e260), -(cos(_e260)));
        let _e268 = select(0f, _e257, (_e125 == 0f));
        phi_1458_ = _e264;
        phi_1447_ = _e268;
        phi_1444_ = _e257;
        phi_1434_ = _e264;
        if I7_ {
            let _e270 = ((_e249 & 1048576u) != 0u);
            phi_1419_ = _e257;
            if _e270 {
                phi_1419_ = min(_e257, 0f);
            }
            let _e273 = phi_1419_;
            phi_1431_ = _e273;
            if ((_e249 & 524288u) != 0u) {
                phi_1431_ = max(_e273, 0f);
            }
            let _e278 = phi_1431_;
            let _e279 = (_e249 & 469762048u);
            phi_1459_ = _e264;
            phi_1448_ = _e268;
            phi_1435_ = _e264;
            if (_e279 > 134217728u) {
                let _e284 = f32((_e251.z & 65535u));
                let _e285 = (_e284 * 0.000015259022f);
                let _e289 = sqrt(max((1f - (_e285 * _e285)), 0f));
                phi_1423_ = _e289;
                if (((_e249 & 4194304u) != 0u) == _e270) {
                    phi_1423_ = -(_e289);
                }
                let _e293 = phi_1423_;
                let _e298 = (mat2x2<f32>(vec2<f32>(_e285, _e293), vec2<f32>(-(_e293), _e285)) * _e264);
                let _e299 = (_e279 == 201326592u);
                phi_695_ = _e299;
                if !(_e299) {
                    phi_695_ = ((_e279 != 335544320u) && (_e285 < 0.25f));
                }
                let _e305 = phi_695_;
                let _e307 = ((_e249 & 2097152u) != 0u);
                if (_e279 == 335544320u) {
                    phi_1440_ = (_e264 + _e298);
                } else {
                    phi_1441_ = _e264;
                    if (_e307 || !(_e305)) {
                        if _e305 {
                            phi_1426_ = _e285;
                        } else {
                            phi_1426_ = (65535f / _e284);
                        }
                        let _e314 = phi_1426_;
                        phi_1441_ = (_e298 * _e314);
                    }
                    let _e317 = phi_1441_;
                    phi_1440_ = _e317;
                }
                let _e319 = phi_1440_;
                phi_1451_ = _e268;
                if (!(_e82) && _e305) {
                    phi_1451_ = (0.5f * _e278);
                }
                let _e327 = phi_1451_;
                phi_1459_ = select(_e264, _e298, vec2((_e305 || _e307)));
                phi_1448_ = _e327;
                phi_1435_ = _e319;
            }
            let _e329 = phi_1459_;
            let _e331 = phi_1448_;
            let _e333 = phi_1435_;
            phi_1458_ = _e329;
            phi_1447_ = _e331;
            phi_1444_ = _e278;
            phi_1434_ = _e333;
        }
        let _e335 = phi_1458_;
        let _e337 = phi_1447_;
        let _e339 = phi_1444_;
        let _e341 = phi_1434_;
        let _e346 = ((_e176 * (bitcast<vec2<f32>>(_e251.xy) + (_e341 * (_e339 * bitcast<f32>(_e184.z))))) + _e186);
        phi_1470_ = _e346;
        if (_e337 != 0f) {
            phi_1470_ = (_e346 + (sign((_e335 * _naga_inverse_2x2_f32(_e176))) * _e337));
        }
        let _e354 = phi_1470_;
        phi_1469_ = _e354;
    } else {
        if _e129 {
            phi_1395_ = _e157;
        } else {
            phi_1395_ = bitcast<vec2<f32>>(_e251.xy);
        }
        let _e358 = phi_1395_;
        phi_1469_ = ((_e176 * _e358) + _e186);
    }
    let _e362 = phi_1469_;
    if Zi {
        let _e363 = (_e159 * 8u);
        let _e364 = (_e363 + 2u);
        let _e371 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e364 & 255u)), bitcast<i32>((_e364 >> bitcast<u32>(8i)))), 0i);
        let _e379 = (_e363 + 3u);
        let _e386 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e379 & 255u)), bitcast<i32>((_e379 >> bitcast<u32>(8i)))), 0i);
        if any((_e371 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e391 = ((mat2x2<f32>(vec2<f32>(_e371.x, _e371.y), vec2<f32>(_e371.z, _e371.w)) * _e362) + _e386.xy);
            unnamed.gl_ClipDistance[0i] = (_e391.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e391.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e391.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e391.y);
        } else {
            let _e407 = (_e386.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e407;
            unnamed.gl_ClipDistance[2i] = _e407;
            unnamed.gl_ClipDistance[1i] = _e407;
            unnamed.gl_ClipDistance[0i] = _e407;
        }
    }
    if ((_e77 & 268435456i) != 0i) {
        P0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e422 = textureLoad(WC, vec2<i32>(bitcast<i32>((_e155.z & 255u)), bitcast<i32>((_e159 >> bitcast<u32>(8i)))), 0i);
        let _e424 = (_e422.x & 15u);
        phi_1520_ = false;
        if aj {
            let _e427 = ((_e422.x >> bitcast<u32>(4i)) & 15u);
            Q0_ = f32(_e427);
            phi_1520_ = (_e427 != 0u);
        }
        let _e431 = phi_1520_;
        if (_e424 == 1u) {
            P0_ = unpack4x8unorm(_e422.y);
            if _e431 {
                let _e436 = P0_[3u];
                P0_[3u] = (_e436 * _e125);
            } else {
                let _e438 = P0_;
                P0_ = (_e438 * _e125);
            }
        } else {
            let _e440 = (_e159 * 8u);
            let _e447 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e440 & 255u)), bitcast<i32>((_e440 >> bitcast<u32>(8i)))), 0i);
            let _e455 = (_e440 + 1u);
            let _e462 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e455 & 255u)), bitcast<i32>((_e455 >> bitcast<u32>(8i)))), 0i);
            let _e464 = bitcast<f32>(_e422.y);
            let _e467 = ((mat2x2<f32>(vec2<f32>(_e447.x, _e447.y), vec2<f32>(_e447.z, _e447.w)) * _e362) + _e462.xy);
            let _e473 = vec4<f32>(_e467.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e479 = vec4<f32>(_e473.x, _e467.y, _e473.z, _e473.w);
            let _e484 = vec4<f32>(_e479.x, _e479.y, _e125, _e479.w);
            phi_1518_ = _e484;
            if (_e424 != 2u) {
                phi_1518_ = vec4<f32>(_e484.x, _e484.y, (_e125 + 2f), _e484.w);
            }
            let _e493 = phi_1518_;
            phi_1519_ = _e493;
            if (_e462.z > 0.9f) {
                phi_1519_ = vec4<f32>(_e493.x, _e493.y, -(_e493.z), _e493.w);
            }
            let _e504 = phi_1519_;
            P0_ = vec4<f32>(_e504.x, _e504.y, _e504.z, -(bitcast<f32>(((((_e424 << bitcast<u32>(28i)) | ((u32(_e464) - 1u) << bitcast<u32>(17i))) | (u32((_e462.w * 512f)) << bitcast<u32>(8i))) | u32((fract(_e464) * 256f))))));
        }
        phi_947_ = gj;
        if gj {
            phi_947_ = ((_e422.x & 2048u) != 0u);
        }
        let _e532 = phi_947_;
        if _e532 {
            let _e533 = (_e159 * 8u);
            let _e534 = (_e533 + 4u);
            let _e541 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e534 & 255u)), bitcast<i32>((_e534 >> bitcast<u32>(8i)))), 0i);
            let _e549 = (_e533 + 5u);
            let _e556 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e549 & 255u)), bitcast<i32>((_e549 >> bitcast<u32>(8i)))), 0i);
            let _e559 = ((mat2x2<f32>(vec2<f32>(_e541.x, _e541.y), vec2<f32>(_e541.z, _e541.w)) * _e362) + _e556.xy);
            V0_ = vec3<f32>(_e559.x, _e559.y, (1f + _e556.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e566 = j.Gg;
    let _e568 = j.Hg;
    let _e576 = vec4<f32>(((_e362.x * _e566) - 1f), ((_e362.y * _e568) - sign(_e568)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1531_ = true;
                break;
            }
            if fe {
                phi_1531_ = true;
                break;
            }
            phi_1531_ = false;
            break;
        }
    }
    let _e579 = phi_1531_;
    if _e579 {
        let _e581 = u32((_e125 * 254f));
        phi_1541_ = _e581;
        if ((_e77 & 1073741824i) == 0i) {
            phi_1541_ = (_e581 + bitcast<u32>(1i));
        }
        let _e586 = phi_1541_;
        phi_1540_ = _e586;
    } else {
        phi_1540_ = 255u;
    }
    let _e588 = phi_1540_;
    unnamed.gl_Position = vec4<f32>(_e576.x, _e576.y, ((f32(((_e196.x << bitcast<u32>(8u)) | _e588)) * 0.000000059604645f) + 0.000000029802322f), _e576.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = unnamed.gl_Position;
    let _e10 = unnamed.gl_ClipDistance;
    let _e11 = P0_;
    let _e12 = V0_;
    let _e13 = Q0_;
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
