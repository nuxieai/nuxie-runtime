enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
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

@id(16) override O7_: bool = false;
@id(17) override qe: bool = false;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(5)
var DD: texture_2d<u32>;
@group(0) @binding(2)
var KB: texture_2d<u32>;
@group(0) @binding(4)
var JB: texture_2d<f32>;
var<private> N0_: vec4<f32>;
var<private> O0_: vec3<f32>;
@group(0) @binding(3)
var XC: texture_2d<u32>;
var<private> S0_: f32;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var eb: sampler;

fn main_1() {
    var phi_1360_: bool;
    var phi_1361_: i32;
    var phi_1417_: f32;
    var phi_1370_: f32;
    var phi_1367_: bool;
    var phi_1365_: i32;
    var phi_1364_: i32;
    var phi_1362_: i32;
    var phi_1409_: f32;
    var phi_1408_: f32;
    var phi_1373_: bool;
    var phi_1407_: f32;
    var phi_1372_: bool;
    var phi_1378_: i32;
    var phi_1383_: bool;
    var phi_1385_: vec4<u32>;
    var phi_1384_: vec4<u32>;
    var phi_1425_: u32;
    var phi_1402_: vec4<u32>;
    var phi_1391_: bool;
    var phi_1427_: f32;
    var phi_1439_: f32;
    var phi_1431_: f32;
    var phi_685_: bool;
    var phi_1434_: f32;
    var phi_1449_: vec2<f32>;
    var phi_1448_: vec2<f32>;
    var phi_1459_: f32;
    var phi_1467_: vec2<f32>;
    var phi_1456_: f32;
    var phi_1443_: vec2<f32>;
    var phi_1466_: vec2<f32>;
    var phi_1455_: f32;
    var phi_1452_: f32;
    var phi_1442_: vec2<f32>;
    var phi_1478_: vec2<f32>;
    var phi_1403_: vec2<f32>;
    var phi_1477_: vec2<f32>;
    var phi_1528_: bool;
    var phi_1526_: f32;
    var phi_939_: bool;
    var phi_1529_: f32;
    var phi_1540_: bool;
    var phi_1550_: u32;
    var phi_1549_: u32;

    let _e78 = gl_VertexIndex_1;
    let _e83 = ((_e78 & 536870912i) != 0i);
    let _e84 = (_e78 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1360_ = true;
                break;
            }
            if qe {
                phi_1360_ = true;
                break;
            }
            phi_1360_ = false;
            break;
        }
    }
    let _e87 = phi_1360_;
    if _e87 {
        let _e88 = select(5i, 6i, _e83);
        let _e94 = (_e84 & ((1i << bitcast<u32>(_e88)) - 1i));
        let _e95 = select(1i, 2i, _e83);
        let _e101 = (_e94 & ((1i << bitcast<u32>(_e95)) - 1i));
        phi_1361_ = _e101;
        if (O7_ && !(_e83)) {
            phi_1361_ = (_e101 + 1i);
        }
        let _e106 = phi_1361_;
        phi_1417_ = select(1f, 0f, ((_e106 == 0i) || (_e106 == 3i)));
        phi_1370_ = select(-1f, 1f, (_e106 < 2i));
        phi_1367_ = false;
        phi_1365_ = (_e84 >> bitcast<u32>(_e88));
        phi_1364_ = 8i;
        phi_1362_ = (_e94 >> bitcast<u32>(_e95));
    } else {
        let _e113 = select(4i, 5i, _e83);
        let _e119 = (_e84 & ((1i << bitcast<u32>(_e113)) - 1i));
        let _e123 = (!(_e83) && (_e119 == 9i));
        phi_1417_ = 1f;
        phi_1370_ = 0f;
        phi_1367_ = _e123;
        phi_1365_ = (_e84 >> bitcast<u32>(_e113));
        phi_1364_ = select(8i, 17i, _e83);
        phi_1362_ = select(_e119, 0i, _e123);
    }
    let _e126 = phi_1417_;
    let _e128 = phi_1370_;
    let _e130 = phi_1367_;
    let _e132 = phi_1365_;
    let _e134 = phi_1364_;
    let _e136 = phi_1362_;
    let _e138 = min(_e136, (_e134 - 1i));
    let _e140 = ((_e132 * _e134) + _e138);
    let _e145 = textureLoad(VB, vec2<i32>((_e140 & 2047i), (_e140 >> bitcast<u32>(11i))), 0i);
    let _e149 = (max((_e145.w & 65535u), 1u) - 1u);
    let _e156 = textureLoad(DD, vec2<i32>(bitcast<i32>((_e149 & 255u)), bitcast<i32>((_e149 >> bitcast<u32>(8i)))), 0i);
    let _e158 = bitcast<vec2<f32>>(_e156.xy);
    let _e160 = (_e156.z & 65535u);
    let _e162 = (_e160 * 4u);
    let _e169 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e162 & 255u)), bitcast<i32>((_e162 >> bitcast<u32>(8i)))), 0i);
    let _e170 = bitcast<vec4<f32>>(_e169);
    let _e177 = mat2x2<f32>(vec2<f32>(_e170.x, _e170.y), vec2<f32>(_e170.z, _e170.w));
    let _e178 = (_e162 + 1u);
    let _e185 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e178 & 255u)), bitcast<i32>((_e178 >> bitcast<u32>(8i)))), 0i);
    let _e187 = bitcast<vec2<f32>>(_e185.xy);
    let _e190 = (_e162 + 2u);
    let _e197 = textureLoad(KB, vec2<i32>(bitcast<i32>((_e190 & 255u)), bitcast<i32>((_e190 >> bitcast<u32>(8i)))), 0i);
    let _e199 = (_e145.w & 8388608u);
    if O7_ {
        phi_1407_ = _e128;
        phi_1372_ = false;
    } else {
        if qe {
            let _e200 = (_e199 != 0u);
            phi_1409_ = _e128;
            if _e200 {
                phi_1409_ = -(_e128);
            }
            let _e203 = phi_1409_;
            phi_1408_ = _e203;
            phi_1373_ = _e200;
        } else {
            phi_1408_ = _e128;
            phi_1373_ = (((_e199 != 0u) && !(_e83)) && !(_e130));
        }
        let _e210 = phi_1408_;
        let _e212 = phi_1373_;
        phi_1407_ = _e210;
        phi_1372_ = _e212;
    }
    let _e214 = phi_1407_;
    let _e216 = phi_1372_;
    phi_1378_ = _e136;
    if _e216 {
        phi_1378_ = (_e136 - 1i);
    }
    let _e219 = phi_1378_;
    phi_1425_ = _e145.w;
    phi_1402_ = _e145;
    if (_e219 != _e138) {
        let _e222 = ((_e140 + _e219) - _e138);
        let _e227 = textureLoad(VB, vec2<i32>((_e222 & 2047i), (_e222 >> bitcast<u32>(11i))), 0i);
        if ((_e227.w & 8454143u) != (_e145.w & 8454143u)) {
            if O7_ {
                phi_1383_ = (_e158.x != 0f);
            } else {
                phi_1383_ = true;
            }
            let _e235 = phi_1383_;
            phi_1385_ = _e145;
            if _e235 {
                let _e236 = bitcast<i32>(_e156.w);
                let _e241 = textureLoad(VB, vec2<i32>((_e236 & 2047i), (_e236 >> bitcast<u32>(11i))), 0i);
                phi_1385_ = _e241;
            }
            let _e243 = phi_1385_;
            phi_1384_ = _e243;
        } else {
            phi_1384_ = _e227;
        }
        let _e245 = phi_1384_;
        phi_1425_ = ((_e245.w & 4286578687u) | _e199);
        phi_1402_ = _e245;
    }
    let _e250 = phi_1425_;
    let _e252 = phi_1402_;
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1391_ = true;
                break;
            }
            if qe {
                phi_1391_ = true;
                break;
            }
            phi_1391_ = false;
            break;
        }
    }
    let _e255 = phi_1391_;
    if _e255 {
        let _e258 = (_e214 * sign(determinant(_e177)));
        let _e261 = (f32(_e252.z) * 0.0000000014629181f);
        let _e265 = vec2<f32>(sin(_e261), -(cos(_e261)));
        let _e269 = select(0f, _e258, (_e126 == 0f));
        phi_1466_ = _e265;
        phi_1455_ = _e269;
        phi_1452_ = _e258;
        phi_1442_ = _e265;
        if O7_ {
            let _e271 = ((_e250 & 1048576u) != 0u);
            phi_1427_ = _e258;
            if _e271 {
                phi_1427_ = min(_e258, 0f);
            }
            let _e274 = phi_1427_;
            phi_1439_ = _e274;
            if ((_e250 & 524288u) != 0u) {
                phi_1439_ = max(_e274, 0f);
            }
            let _e279 = phi_1439_;
            let _e280 = (_e250 & 469762048u);
            phi_1467_ = _e265;
            phi_1456_ = _e269;
            phi_1443_ = _e265;
            if (_e280 > 201326592u) {
                let _e285 = f32((_e252.z & 65535u));
                let _e286 = (_e285 * 0.000015259022f);
                let _e290 = sqrt(max((1f - (_e286 * _e286)), 0f));
                phi_1431_ = _e290;
                if (((_e250 & 4194304u) != 0u) == _e271) {
                    phi_1431_ = -(_e290);
                }
                let _e294 = phi_1431_;
                let _e299 = (mat2x2<f32>(vec2<f32>(_e286, _e294), vec2<f32>(-(_e294), _e286)) * _e265);
                let _e300 = (_e280 == 268435456u);
                phi_685_ = _e300;
                if !(_e300) {
                    phi_685_ = ((_e280 != 402653184u) && (_e286 < 0.25f));
                }
                let _e306 = phi_685_;
                let _e308 = ((_e250 & 2097152u) != 0u);
                if (_e280 == 402653184u) {
                    phi_1448_ = (_e265 + _e299);
                } else {
                    phi_1449_ = _e265;
                    if (_e308 || !(_e306)) {
                        if _e306 {
                            phi_1434_ = _e286;
                        } else {
                            phi_1434_ = (65535f / _e285);
                        }
                        let _e315 = phi_1434_;
                        phi_1449_ = (_e299 * _e315);
                    }
                    let _e318 = phi_1449_;
                    phi_1448_ = _e318;
                }
                let _e320 = phi_1448_;
                phi_1459_ = _e269;
                if (!(_e83) && _e306) {
                    phi_1459_ = (0.5f * _e279);
                }
                let _e328 = phi_1459_;
                phi_1467_ = select(_e265, _e299, vec2((_e306 || _e308)));
                phi_1456_ = _e328;
                phi_1443_ = _e320;
            }
            let _e330 = phi_1467_;
            let _e332 = phi_1456_;
            let _e334 = phi_1443_;
            phi_1466_ = _e330;
            phi_1455_ = _e332;
            phi_1452_ = _e279;
            phi_1442_ = _e334;
        }
        let _e336 = phi_1466_;
        let _e338 = phi_1455_;
        let _e340 = phi_1452_;
        let _e342 = phi_1442_;
        let _e347 = ((_e177 * (bitcast<vec2<f32>>(_e252.xy) + (_e342 * (_e340 * bitcast<f32>(_e185.z))))) + _e187);
        phi_1478_ = _e347;
        if (_e338 != 0f) {
            phi_1478_ = (_e347 + (sign((_e336 * _naga_inverse_2x2_f32(_e177))) * _e338));
        }
        let _e355 = phi_1478_;
        phi_1477_ = _e355;
    } else {
        if _e130 {
            phi_1403_ = _e158;
        } else {
            phi_1403_ = bitcast<vec2<f32>>(_e252.xy);
        }
        let _e359 = phi_1403_;
        phi_1477_ = ((_e177 * _e359) + _e187);
    }
    let _e363 = phi_1477_;
    if kj {
        let _e364 = (_e160 * 8u);
        let _e365 = (_e364 + 2u);
        let _e372 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e365 & 255u)), bitcast<i32>((_e365 >> bitcast<u32>(8i)))), 0i);
        let _e380 = (_e364 + 3u);
        let _e387 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e380 & 255u)), bitcast<i32>((_e380 >> bitcast<u32>(8i)))), 0i);
        if any((_e372 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e392 = ((mat2x2<f32>(vec2<f32>(_e372.x, _e372.y), vec2<f32>(_e372.z, _e372.w)) * _e363) + _e387.xy);
            unnamed.gl_ClipDistance[0i] = (_e392.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e392.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e392.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e392.y);
        } else {
            let _e408 = (_e387.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e408;
            unnamed.gl_ClipDistance[2i] = _e408;
            unnamed.gl_ClipDistance[1i] = _e408;
            unnamed.gl_ClipDistance[0i] = _e408;
        }
    }
    if ((_e78 & 268435456i) != 0i) {
        N0_ = vec4<f32>(0f, 0f, 0f, 0f);
        O0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e423 = textureLoad(XC, vec2<i32>(bitcast<i32>((_e156.z & 255u)), bitcast<i32>((_e160 >> bitcast<u32>(8i)))), 0i);
        let _e425 = (_e423.x & 15u);
        phi_1528_ = false;
        if lj {
            let _e428 = ((_e423.x >> bitcast<u32>(4i)) & 15u);
            S0_ = f32(_e428);
            phi_1528_ = (_e428 != 0u);
        }
        let _e432 = phi_1528_;
        if (_e425 == 1u) {
            N0_ = unpack4x8unorm(_e423.y);
            if _e432 {
                let _e437 = N0_[3u];
                N0_[3u] = (_e437 * _e126);
            } else {
                let _e439 = N0_;
                N0_ = (_e439 * _e126);
            }
        } else {
            let _e441 = (_e160 * 8u);
            let _e448 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e441 & 255u)), bitcast<i32>((_e441 >> bitcast<u32>(8i)))), 0i);
            let _e456 = (_e441 + 1u);
            let _e463 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e456 & 255u)), bitcast<i32>((_e456 >> bitcast<u32>(8i)))), 0i);
            let _e468 = ((mat2x2<f32>(vec2<f32>(_e448.x, _e448.y), vec2<f32>(_e448.z, _e448.w)) * _e363) + _e463.xy);
            let _e479 = ((_e463.w + (f32(_e425) * 0.125f)) + (max(_e463.z, 0f) * 0.00024414063f));
            if (_e463.z < 0f) {
                phi_1526_ = -(_e479);
            } else {
                phi_1526_ = _e479;
            }
            let _e482 = phi_1526_;
            N0_ = vec4<f32>(_e468.x, _e468.y, _e482, (((_e126 * -0.5f) - 0.25f) - round((bitcast<f32>(_e423.y) * 255f))));
        }
        phi_939_ = nj;
        if nj {
            phi_939_ = ((_e423.x & 2048u) != 0u);
        }
        let _e492 = phi_939_;
        if _e492 {
            let _e493 = (_e160 * 8u);
            let _e494 = (_e493 + 4u);
            let _e501 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e494 & 255u)), bitcast<i32>((_e494 >> bitcast<u32>(8i)))), 0i);
            let _e509 = (_e493 + 5u);
            let _e516 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e509 & 255u)), bitcast<i32>((_e509 >> bitcast<u32>(8i)))), 0i);
            let _e519 = ((mat2x2<f32>(vec2<f32>(_e501.x, _e501.y), vec2<f32>(_e501.z, _e501.w)) * _e363) + _e516.xy);
            phi_1529_ = (1f + _e516.z);
            if ((_e423.x & 4096u) != 0u) {
                phi_1529_ = (-1f - f32(((_e423.x & 24576u) >> bitcast<u32>(13u))));
            }
            let _e530 = phi_1529_;
            O0_ = vec3<f32>(_e519.x, _e519.y, _e530);
        } else {
            O0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e535 = j.Sg;
    let _e537 = j.Tg;
    let _e545 = vec4<f32>(((_e363.x * _e535) - 1f), ((_e363.y * _e537) - sign(_e537)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1540_ = true;
                break;
            }
            if qe {
                phi_1540_ = true;
                break;
            }
            phi_1540_ = false;
            break;
        }
    }
    let _e548 = phi_1540_;
    if _e548 {
        let _e550 = u32((_e126 * 254f));
        phi_1550_ = _e550;
        if ((_e78 & 1073741824i) == 0i) {
            phi_1550_ = (_e550 + bitcast<u32>(1i));
        }
        let _e555 = phi_1550_;
        phi_1549_ = _e555;
    } else {
        phi_1549_ = 255u;
    }
    let _e557 = phi_1549_;
    unnamed.gl_Position = vec4<f32>(_e545.x, _e545.y, ((f32(((_e197.x << bitcast<u32>(8u)) | _e557)) * 0.000000059604645f) + 0.000000029802322f), _e545.w);
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
