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
@id(17) override qe: bool = false;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var eb: sampler;

fn main_1() {
    var phi_1273_: bool;
    var phi_1274_: i32;
    var phi_1330_: f32;
    var phi_1283_: f32;
    var phi_1280_: bool;
    var phi_1278_: i32;
    var phi_1277_: i32;
    var phi_1275_: i32;
    var phi_1322_: f32;
    var phi_1321_: f32;
    var phi_1286_: bool;
    var phi_1320_: f32;
    var phi_1285_: bool;
    var phi_1291_: i32;
    var phi_1296_: bool;
    var phi_1298_: vec4<u32>;
    var phi_1297_: vec4<u32>;
    var phi_1338_: u32;
    var phi_1315_: vec4<u32>;
    var phi_1304_: bool;
    var phi_1340_: f32;
    var phi_1352_: f32;
    var phi_1344_: f32;
    var phi_636_: bool;
    var phi_1347_: f32;
    var phi_1362_: vec2<f32>;
    var phi_1361_: vec2<f32>;
    var phi_1372_: f32;
    var phi_1380_: vec2<f32>;
    var phi_1369_: f32;
    var phi_1356_: vec2<f32>;
    var phi_1379_: vec2<f32>;
    var phi_1368_: f32;
    var phi_1365_: f32;
    var phi_1355_: vec2<f32>;
    var phi_1391_: vec2<f32>;
    var phi_1316_: vec2<f32>;
    var phi_1390_: vec2<f32>;
    var phi_1437_: bool;
    var phi_1435_: f32;
    var phi_893_: bool;
    var phi_1438_: f32;
    var phi_1447_: bool;
    var phi_1457_: u32;
    var phi_1456_: u32;

    let _e77 = gl_VertexIndex_1;
    let _e82 = ((_e77 & 536870912i) != 0i);
    let _e83 = (_e77 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1273_ = true;
                break;
            }
            if qe {
                phi_1273_ = true;
                break;
            }
            phi_1273_ = false;
            break;
        }
    }
    let _e86 = phi_1273_;
    if _e86 {
        let _e87 = select(5i, 6i, _e82);
        let _e93 = (_e83 & ((1i << bitcast<u32>(_e87)) - 1i));
        let _e94 = select(1i, 2i, _e82);
        let _e100 = (_e93 & ((1i << bitcast<u32>(_e94)) - 1i));
        phi_1274_ = _e100;
        if (O7_ && !(_e82)) {
            phi_1274_ = (_e100 + 1i);
        }
        let _e105 = phi_1274_;
        phi_1330_ = select(1f, 0f, ((_e105 == 0i) || (_e105 == 3i)));
        phi_1283_ = select(-1f, 1f, (_e105 < 2i));
        phi_1280_ = false;
        phi_1278_ = (_e83 >> bitcast<u32>(_e87));
        phi_1277_ = 8i;
        phi_1275_ = (_e93 >> bitcast<u32>(_e94));
    } else {
        let _e112 = select(4i, 5i, _e82);
        let _e118 = (_e83 & ((1i << bitcast<u32>(_e112)) - 1i));
        let _e122 = (!(_e82) && (_e118 == 9i));
        phi_1330_ = 1f;
        phi_1283_ = 0f;
        phi_1280_ = _e122;
        phi_1278_ = (_e83 >> bitcast<u32>(_e112));
        phi_1277_ = select(8i, 17i, _e82);
        phi_1275_ = select(_e118, 0i, _e122);
    }
    let _e125 = phi_1330_;
    let _e127 = phi_1283_;
    let _e129 = phi_1280_;
    let _e131 = phi_1278_;
    let _e133 = phi_1277_;
    let _e135 = phi_1275_;
    let _e137 = min(_e135, (_e133 - 1i));
    let _e139 = ((_e131 * _e133) + _e137);
    let _e144 = textureLoad(VB, vec2<i32>((_e139 & 2047i), (_e139 >> bitcast<u32>(11i))), 0i);
    let _e148 = (max((_e144.w & 65535u), 1u) - 1u);
    let _e155 = textureLoad(DD, vec2<i32>(bitcast<i32>((_e148 & 255u)), bitcast<i32>((_e148 >> bitcast<u32>(8i)))), 0i);
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
    if O7_ {
        phi_1320_ = _e127;
        phi_1285_ = false;
    } else {
        if qe {
            let _e199 = (_e198 != 0u);
            phi_1322_ = _e127;
            if _e199 {
                phi_1322_ = -(_e127);
            }
            let _e202 = phi_1322_;
            phi_1321_ = _e202;
            phi_1286_ = _e199;
        } else {
            phi_1321_ = _e127;
            phi_1286_ = (((_e198 != 0u) && !(_e82)) && !(_e129));
        }
        let _e209 = phi_1321_;
        let _e211 = phi_1286_;
        phi_1320_ = _e209;
        phi_1285_ = _e211;
    }
    let _e213 = phi_1320_;
    let _e215 = phi_1285_;
    phi_1291_ = _e135;
    if _e215 {
        phi_1291_ = (_e135 - 1i);
    }
    let _e218 = phi_1291_;
    phi_1338_ = _e144.w;
    phi_1315_ = _e144;
    if (_e218 != _e137) {
        let _e221 = ((_e139 + _e218) - _e137);
        let _e226 = textureLoad(VB, vec2<i32>((_e221 & 2047i), (_e221 >> bitcast<u32>(11i))), 0i);
        if ((_e226.w & 8454143u) != (_e144.w & 8454143u)) {
            if O7_ {
                phi_1296_ = (_e157.x != 0f);
            } else {
                phi_1296_ = true;
            }
            let _e234 = phi_1296_;
            phi_1298_ = _e144;
            if _e234 {
                let _e235 = bitcast<i32>(_e155.w);
                let _e240 = textureLoad(VB, vec2<i32>((_e235 & 2047i), (_e235 >> bitcast<u32>(11i))), 0i);
                phi_1298_ = _e240;
            }
            let _e242 = phi_1298_;
            phi_1297_ = _e242;
        } else {
            phi_1297_ = _e226;
        }
        let _e244 = phi_1297_;
        phi_1338_ = ((_e244.w & 4286578687u) | _e198);
        phi_1315_ = _e244;
    }
    let _e249 = phi_1338_;
    let _e251 = phi_1315_;
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1304_ = true;
                break;
            }
            if qe {
                phi_1304_ = true;
                break;
            }
            phi_1304_ = false;
            break;
        }
    }
    let _e254 = phi_1304_;
    if _e254 {
        let _e257 = (_e213 * sign(determinant(_e176)));
        let _e260 = (f32(_e251.z) * 0.0000000014629181f);
        let _e264 = vec2<f32>(sin(_e260), -(cos(_e260)));
        let _e268 = select(0f, _e257, (_e125 == 0f));
        phi_1379_ = _e264;
        phi_1368_ = _e268;
        phi_1365_ = _e257;
        phi_1355_ = _e264;
        if O7_ {
            let _e270 = ((_e249 & 1048576u) != 0u);
            phi_1340_ = _e257;
            if _e270 {
                phi_1340_ = min(_e257, 0f);
            }
            let _e273 = phi_1340_;
            phi_1352_ = _e273;
            if ((_e249 & 524288u) != 0u) {
                phi_1352_ = max(_e273, 0f);
            }
            let _e278 = phi_1352_;
            let _e279 = (_e249 & 469762048u);
            phi_1380_ = _e264;
            phi_1369_ = _e268;
            phi_1356_ = _e264;
            if (_e279 > 201326592u) {
                let _e284 = f32((_e251.z & 65535u));
                let _e285 = (_e284 * 0.000015259022f);
                let _e289 = sqrt(max((1f - (_e285 * _e285)), 0f));
                phi_1344_ = _e289;
                if (((_e249 & 4194304u) != 0u) == _e270) {
                    phi_1344_ = -(_e289);
                }
                let _e293 = phi_1344_;
                let _e298 = (mat2x2<f32>(vec2<f32>(_e285, _e293), vec2<f32>(-(_e293), _e285)) * _e264);
                let _e299 = (_e279 == 268435456u);
                phi_636_ = _e299;
                if !(_e299) {
                    phi_636_ = ((_e279 != 402653184u) && (_e285 < 0.25f));
                }
                let _e305 = phi_636_;
                let _e307 = ((_e249 & 2097152u) != 0u);
                if (_e279 == 402653184u) {
                    phi_1361_ = (_e264 + _e298);
                } else {
                    phi_1362_ = _e264;
                    if (_e307 || !(_e305)) {
                        if _e305 {
                            phi_1347_ = _e285;
                        } else {
                            phi_1347_ = (65535f / _e284);
                        }
                        let _e314 = phi_1347_;
                        phi_1362_ = (_e298 * _e314);
                    }
                    let _e317 = phi_1362_;
                    phi_1361_ = _e317;
                }
                let _e319 = phi_1361_;
                phi_1372_ = _e268;
                if (!(_e82) && _e305) {
                    phi_1372_ = (0.5f * _e278);
                }
                let _e327 = phi_1372_;
                phi_1380_ = select(_e264, _e298, vec2((_e305 || _e307)));
                phi_1369_ = _e327;
                phi_1356_ = _e319;
            }
            let _e329 = phi_1380_;
            let _e331 = phi_1369_;
            let _e333 = phi_1356_;
            phi_1379_ = _e329;
            phi_1368_ = _e331;
            phi_1365_ = _e278;
            phi_1355_ = _e333;
        }
        let _e335 = phi_1379_;
        let _e337 = phi_1368_;
        let _e339 = phi_1365_;
        let _e341 = phi_1355_;
        let _e346 = ((_e176 * (bitcast<vec2<f32>>(_e251.xy) + (_e341 * (_e339 * bitcast<f32>(_e184.z))))) + _e186);
        phi_1391_ = _e346;
        if (_e337 != 0f) {
            phi_1391_ = (_e346 + (sign((_e335 * _naga_inverse_2x2_f32(_e176))) * _e337));
        }
        let _e354 = phi_1391_;
        phi_1390_ = _e354;
    } else {
        if _e129 {
            phi_1316_ = _e157;
        } else {
            phi_1316_ = bitcast<vec2<f32>>(_e251.xy);
        }
        let _e358 = phi_1316_;
        phi_1390_ = ((_e176 * _e358) + _e186);
    }
    let _e362 = phi_1390_;
    if ((_e77 & 268435456i) != 0i) {
        N0_ = vec4<f32>(0f, 0f, 0f, 0f);
        O0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e369 = textureLoad(XC, vec2<i32>(bitcast<i32>((_e155.z & 255u)), bitcast<i32>((_e159 >> bitcast<u32>(8i)))), 0i);
        let _e371 = (_e369.x & 15u);
        phi_1437_ = false;
        if lj {
            let _e374 = ((_e369.x >> bitcast<u32>(4i)) & 15u);
            S0_ = f32(_e374);
            phi_1437_ = (_e374 != 0u);
        }
        let _e378 = phi_1437_;
        if (_e371 == 1u) {
            N0_ = unpack4x8unorm(_e369.y);
            if _e378 {
                let _e383 = N0_[3u];
                N0_[3u] = (_e383 * _e125);
            } else {
                let _e385 = N0_;
                N0_ = (_e385 * _e125);
            }
        } else {
            let _e387 = (_e159 * 8u);
            let _e394 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e387 & 255u)), bitcast<i32>((_e387 >> bitcast<u32>(8i)))), 0i);
            let _e402 = (_e387 + 1u);
            let _e409 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e402 & 255u)), bitcast<i32>((_e402 >> bitcast<u32>(8i)))), 0i);
            let _e414 = ((mat2x2<f32>(vec2<f32>(_e394.x, _e394.y), vec2<f32>(_e394.z, _e394.w)) * _e362) + _e409.xy);
            let _e425 = ((_e409.w + (f32(_e371) * 0.125f)) + (max(_e409.z, 0f) * 0.00024414063f));
            if (_e409.z < 0f) {
                phi_1435_ = -(_e425);
            } else {
                phi_1435_ = _e425;
            }
            let _e428 = phi_1435_;
            N0_ = vec4<f32>(_e414.x, _e414.y, _e428, (((_e125 * -0.5f) - 0.25f) - round((bitcast<f32>(_e369.y) * 255f))));
        }
        phi_893_ = nj;
        if nj {
            phi_893_ = ((_e369.x & 2048u) != 0u);
        }
        let _e438 = phi_893_;
        if _e438 {
            let _e439 = (_e159 * 8u);
            let _e440 = (_e439 + 4u);
            let _e447 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e440 & 255u)), bitcast<i32>((_e440 >> bitcast<u32>(8i)))), 0i);
            let _e455 = (_e439 + 5u);
            let _e462 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e455 & 255u)), bitcast<i32>((_e455 >> bitcast<u32>(8i)))), 0i);
            let _e465 = ((mat2x2<f32>(vec2<f32>(_e447.x, _e447.y), vec2<f32>(_e447.z, _e447.w)) * _e362) + _e462.xy);
            phi_1438_ = (1f + _e462.z);
            if ((_e369.x & 4096u) != 0u) {
                phi_1438_ = (-1f - f32(((_e369.x & 24576u) >> bitcast<u32>(13u))));
            }
            let _e476 = phi_1438_;
            O0_ = vec3<f32>(_e465.x, _e465.y, _e476);
        } else {
            O0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e481 = j.Sg;
    let _e483 = j.Tg;
    let _e491 = vec4<f32>(((_e362.x * _e481) - 1f), ((_e362.y * _e483) - sign(_e483)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1447_ = true;
                break;
            }
            if qe {
                phi_1447_ = true;
                break;
            }
            phi_1447_ = false;
            break;
        }
    }
    let _e494 = phi_1447_;
    if _e494 {
        let _e496 = u32((_e125 * 254f));
        phi_1457_ = _e496;
        if ((_e77 & 1073741824i) == 0i) {
            phi_1457_ = (_e496 + bitcast<u32>(1i));
        }
        let _e501 = phi_1457_;
        phi_1456_ = _e501;
    } else {
        phi_1456_ = 255u;
    }
    let _e503 = phi_1456_;
    unnamed.gl_Position = vec4<f32>(_e491.x, _e491.y, ((f32(((_e196.x << bitcast<u32>(8u)) | _e503)) * 0.000000059604645f) + 0.000000029802322f), _e491.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e8 = N0_;
    let _e9 = O0_;
    let _e10 = S0_;
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
