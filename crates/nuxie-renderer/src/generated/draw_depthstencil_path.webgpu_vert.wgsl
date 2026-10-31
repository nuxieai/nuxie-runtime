enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct xi {
    z2_: array<vec4<u32>>,
}

struct nd {
    z2_: array<vec4<u32>>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(16) override G6_: bool = false;
@id(17) override ue: bool = false;
@id(1) override vj: bool = true;
@id(2) override wj: bool = true;
@id(4) override yj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(6)
var WB: texture_2d<u32>;
@group(0) @binding(4)
var<storage> CD: xi;
@group(0) @binding(2)
var<storage> JB: nd;
@group(0) @binding(3)
var<storage> KB: Cg;
var<private> N0_: vec4<f32>;
var<private> O0_: vec3<f32>;
var<private> S0_: f32;
@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var ib: sampler;

fn main_1() {
    var phi_1282_: bool;
    var phi_1283_: i32;
    var phi_1345_: f32;
    var phi_1295_: f32;
    var phi_1291_: bool;
    var phi_1287_: i32;
    var phi_1286_: i32;
    var phi_1284_: i32;
    var phi_1388_: f32;
    var phi_1336_: f32;
    var phi_1335_: f32;
    var phi_1298_: bool;
    var phi_1334_: f32;
    var phi_1297_: bool;
    var phi_1304_: i32;
    var phi_1309_: bool;
    var phi_1311_: vec4<u32>;
    var phi_1310_: vec4<u32>;
    var phi_1353_: u32;
    var phi_1329_: vec4<u32>;
    var phi_1318_: bool;
    var phi_1355_: f32;
    var phi_1367_: f32;
    var phi_1359_: f32;
    var phi_663_: bool;
    var phi_1362_: f32;
    var phi_1377_: vec2<f32>;
    var phi_1376_: vec2<f32>;
    var phi_1408_: f32;
    var phi_1416_: vec2<f32>;
    var phi_1405_: f32;
    var phi_1371_: vec2<f32>;
    var phi_1415_: vec2<f32>;
    var phi_1404_: f32;
    var phi_1380_: f32;
    var phi_1370_: vec2<f32>;
    var phi_1427_: vec2<f32>;
    var phi_1330_: vec2<f32>;
    var phi_1426_: vec2<f32>;
    var phi_1478_: bool;
    var phi_1476_: f32;
    var phi_875_: bool;
    var phi_1479_: f32;
    var phi_1490_: bool;
    var phi_1500_: u32;
    var phi_1499_: u32;

    let _e78 = gl_VertexIndex_1;
    let _e83 = ((_e78 & 536870912i) != 0i);
    let _e84 = (_e78 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if G6_ {
                phi_1282_ = true;
                break;
            }
            if ue {
                phi_1282_ = true;
                break;
            }
            phi_1282_ = false;
            break;
        }
    }
    let _e87 = phi_1282_;
    if _e87 {
        let _e88 = select(5i, 6i, _e83);
        let _e94 = (_e84 & ((1i << bitcast<u32>(_e88)) - 1i));
        let _e95 = select(1i, 2i, _e83);
        let _e101 = (_e94 & ((1i << bitcast<u32>(_e95)) - 1i));
        phi_1283_ = _e101;
        if (G6_ && !(_e83)) {
            phi_1283_ = (_e101 + 1i);
        }
        let _e106 = phi_1283_;
        phi_1345_ = select(1f, 0f, ((_e106 == 0i) || (_e106 == 3i)));
        phi_1295_ = select(-1f, 1f, (_e106 < 2i));
        phi_1291_ = false;
        phi_1287_ = (_e84 >> bitcast<u32>(_e88));
        phi_1286_ = 8i;
        phi_1284_ = (_e94 >> bitcast<u32>(_e95));
    } else {
        let _e113 = select(4i, 5i, _e83);
        let _e119 = (_e84 & ((1i << bitcast<u32>(_e113)) - 1i));
        let _e123 = (!(_e83) && (_e119 == 9i));
        phi_1345_ = 1f;
        phi_1295_ = 0f;
        phi_1291_ = _e123;
        phi_1287_ = (_e84 >> bitcast<u32>(_e113));
        phi_1286_ = select(8i, 17i, _e83);
        phi_1284_ = select(_e119, 0i, _e123);
    }
    let _e126 = phi_1345_;
    let _e128 = phi_1295_;
    let _e130 = phi_1291_;
    let _e132 = phi_1287_;
    let _e134 = phi_1286_;
    let _e136 = phi_1284_;
    let _e138 = min(_e136, (_e134 - 1i));
    let _e140 = ((_e132 * _e134) + _e138);
    let _e145 = textureLoad(WB, vec2<i32>((_e140 & 2047i), (_e140 >> bitcast<u32>(11i))), 0i);
    let _e152 = CD.z2_[(max((_e145.w & 65535u), 1u) - 1u)];
    let _e154 = bitcast<vec2<f32>>(_e152.xy);
    let _e156 = (_e152.z & 65535u);
    let _e158 = (_e156 * 4u);
    let _e161 = JB.z2_[_e158];
    let _e162 = bitcast<vec4<f32>>(_e161);
    let _e169 = mat2x2<f32>(vec2<f32>(_e162.x, _e162.y), vec2<f32>(_e162.z, _e162.w));
    let _e173 = JB.z2_[(_e158 + 1u)];
    let _e175 = bitcast<vec2<f32>>(_e173.xy);
    phi_1388_ = 0f;
    if G6_ {
        let _e183 = JB.z2_[(_e158 + 2u)][0u];
        phi_1388_ = bitcast<f32>(_e183);
    }
    let _e186 = phi_1388_;
    let _e187 = (_e145.w & 8388608u);
    if G6_ {
        phi_1334_ = _e128;
        phi_1297_ = false;
    } else {
        if ue {
            let _e188 = (_e187 != 0u);
            phi_1336_ = _e128;
            if _e188 {
                phi_1336_ = -(_e128);
            }
            let _e191 = phi_1336_;
            phi_1335_ = _e191;
            phi_1298_ = _e188;
        } else {
            phi_1335_ = _e128;
            phi_1298_ = (((_e187 != 0u) && !(_e83)) && !(_e130));
        }
        let _e198 = phi_1335_;
        let _e200 = phi_1298_;
        phi_1334_ = _e198;
        phi_1297_ = _e200;
    }
    let _e202 = phi_1334_;
    let _e204 = phi_1297_;
    phi_1304_ = _e136;
    if _e204 {
        phi_1304_ = (_e136 - 1i);
    }
    let _e207 = phi_1304_;
    phi_1353_ = _e145.w;
    phi_1329_ = _e145;
    if (_e207 != _e138) {
        let _e210 = ((_e140 + _e207) - _e138);
        let _e215 = textureLoad(WB, vec2<i32>((_e210 & 2047i), (_e210 >> bitcast<u32>(11i))), 0i);
        if ((_e215.w & 8454143u) != (_e145.w & 8454143u)) {
            if G6_ {
                phi_1309_ = (_e154.x != 0f);
            } else {
                phi_1309_ = true;
            }
            let _e223 = phi_1309_;
            phi_1311_ = _e145;
            if _e223 {
                let _e224 = bitcast<i32>(_e152.w);
                let _e229 = textureLoad(WB, vec2<i32>((_e224 & 2047i), (_e224 >> bitcast<u32>(11i))), 0i);
                phi_1311_ = _e229;
            }
            let _e231 = phi_1311_;
            phi_1310_ = _e231;
        } else {
            phi_1310_ = _e215;
        }
        let _e233 = phi_1310_;
        phi_1353_ = ((_e233.w & 4286578687u) | _e187);
        phi_1329_ = _e233;
    }
    let _e238 = phi_1353_;
    let _e240 = phi_1329_;
    switch bitcast<i32>(0u) {
        default: {
            if G6_ {
                phi_1318_ = true;
                break;
            }
            if ue {
                phi_1318_ = true;
                break;
            }
            phi_1318_ = false;
            break;
        }
    }
    let _e243 = phi_1318_;
    if _e243 {
        let _e246 = (_e202 * sign(determinant(_e169)));
        let _e249 = (f32(_e240.z) * 0.0000000014629181f);
        let _e253 = vec2<f32>(sin(_e249), -(cos(_e249)));
        let _e257 = select(0f, _e246, (_e126 == 0f));
        phi_1415_ = _e253;
        phi_1404_ = _e257;
        phi_1380_ = _e246;
        phi_1370_ = _e253;
        if G6_ {
            let _e259 = ((_e238 & 1048576u) != 0u);
            phi_1355_ = _e246;
            if _e259 {
                phi_1355_ = min(_e246, 0f);
            }
            let _e262 = phi_1355_;
            phi_1367_ = _e262;
            if ((_e238 & 524288u) != 0u) {
                phi_1367_ = max(_e262, 0f);
            }
            let _e267 = phi_1367_;
            let _e268 = (_e238 & 469762048u);
            phi_1416_ = _e253;
            phi_1405_ = _e257;
            phi_1371_ = _e253;
            if (_e268 > 201326592u) {
                let _e273 = f32((_e240.z & 65535u));
                let _e274 = (_e273 * 0.000015259022f);
                let _e278 = sqrt(max((1f - (_e274 * _e274)), 0f));
                phi_1359_ = _e278;
                if (((_e238 & 4194304u) != 0u) == _e259) {
                    phi_1359_ = -(_e278);
                }
                let _e282 = phi_1359_;
                let _e287 = (mat2x2<f32>(vec2<f32>(_e274, _e282), vec2<f32>(-(_e282), _e274)) * _e253);
                let _e288 = (_e268 == 268435456u);
                phi_663_ = _e288;
                if !(_e288) {
                    phi_663_ = ((_e268 != 402653184u) && (_e274 < 0.25f));
                }
                let _e294 = phi_663_;
                let _e296 = ((_e238 & 2097152u) != 0u);
                if (_e268 == 402653184u) {
                    phi_1376_ = (_e253 + _e287);
                } else {
                    phi_1377_ = _e253;
                    if (_e296 || !(_e294)) {
                        if _e294 {
                            phi_1362_ = _e274;
                        } else {
                            phi_1362_ = (65535f / _e273);
                        }
                        let _e303 = phi_1362_;
                        phi_1377_ = (_e287 * _e303);
                    }
                    let _e306 = phi_1377_;
                    phi_1376_ = _e306;
                }
                let _e308 = phi_1376_;
                phi_1408_ = _e257;
                if (!(_e83) && _e294) {
                    phi_1408_ = (0.5f * _e267);
                }
                let _e316 = phi_1408_;
                phi_1416_ = select(_e253, _e287, vec2((_e294 || _e296)));
                phi_1405_ = _e316;
                phi_1371_ = _e308;
            }
            let _e318 = phi_1416_;
            let _e320 = phi_1405_;
            let _e322 = phi_1371_;
            phi_1415_ = _e318;
            phi_1404_ = _e320;
            phi_1380_ = _e267;
            phi_1370_ = _e322;
        }
        let _e324 = phi_1415_;
        let _e326 = phi_1404_;
        let _e328 = phi_1380_;
        let _e330 = phi_1370_;
        let _e335 = ((_e169 * (bitcast<vec2<f32>>(_e240.xy) + (_e330 * (_e328 * _e186)))) + _e175);
        phi_1427_ = _e335;
        if (_e326 != 0f) {
            phi_1427_ = (_e335 + (sign((_e324 * _naga_inverse_2x2_f32(_e169))) * _e326));
        }
        let _e343 = phi_1427_;
        phi_1426_ = _e343;
    } else {
        if _e130 {
            phi_1330_ = _e154;
        } else {
            phi_1330_ = bitcast<vec2<f32>>(_e240.xy);
        }
        let _e347 = phi_1330_;
        phi_1426_ = ((_e169 * _e347) + _e175);
    }
    let _e351 = phi_1426_;
    if vj {
        let _e352 = (_e156 * 8u);
        let _e356 = KB.z2_[(_e352 + 2u)];
        let _e367 = KB.z2_[(_e352 + 3u)];
        if any((_e356 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e372 = ((mat2x2<f32>(vec2<f32>(_e356.x, _e356.y), vec2<f32>(_e356.z, _e356.w)) * _e351) + _e367.xy);
            unnamed.gl_ClipDistance[0i] = (_e372.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e372.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e372.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e372.y);
        } else {
            let _e388 = (_e367.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e388;
            unnamed.gl_ClipDistance[2i] = _e388;
            unnamed.gl_ClipDistance[1i] = _e388;
            unnamed.gl_ClipDistance[0i] = _e388;
        }
    }
    if ((_e78 & 268435456i) != 0i) {
        N0_ = vec4<f32>(0f, 0f, 0f, 0f);
        O0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e397 = (_e173.z & 15u);
        phi_1478_ = false;
        if wj {
            let _e400 = ((_e173.z >> bitcast<u32>(4i)) & 15u);
            S0_ = f32(_e400);
            phi_1478_ = (_e400 != 0u);
        }
        let _e404 = phi_1478_;
        if (_e397 == 1u) {
            N0_ = unpack4x8unorm(_e173.w);
            if _e404 {
                let _e409 = N0_[3u];
                N0_[3u] = (_e409 * _e126);
            } else {
                let _e411 = N0_;
                N0_ = (_e411 * _e126);
            }
        } else {
            let _e413 = (_e156 * 8u);
            let _e416 = KB.z2_[_e413];
            let _e427 = KB.z2_[(_e413 + 1u)];
            let _e432 = ((mat2x2<f32>(vec2<f32>(_e416.x, _e416.y), vec2<f32>(_e416.z, _e416.w)) * _e351) + _e427.xy);
            let _e443 = ((_e427.w + (f32(_e397) * 0.125f)) + (max(_e427.z, 0f) * 0.00024414063f));
            if (_e427.z < 0f) {
                phi_1476_ = -(_e443);
            } else {
                phi_1476_ = _e443;
            }
            let _e446 = phi_1476_;
            N0_ = vec4<f32>(_e432.x, _e432.y, _e446, (((_e126 * -0.5f) - 0.25f) - round((bitcast<f32>(_e173.w) * 255f))));
        }
        phi_875_ = yj;
        if yj {
            phi_875_ = ((_e173.z & 2048u) != 0u);
        }
        let _e456 = phi_875_;
        if _e456 {
            let _e457 = (_e156 * 8u);
            let _e461 = KB.z2_[(_e457 + 4u)];
            let _e472 = KB.z2_[(_e457 + 5u)];
            let _e475 = ((mat2x2<f32>(vec2<f32>(_e461.x, _e461.y), vec2<f32>(_e461.z, _e461.w)) * _e351) + _e472.xy);
            phi_1479_ = (1f + _e472.z);
            if ((_e173.z & 4096u) != 0u) {
                phi_1479_ = (-1f - f32(((_e173.z & 24576u) >> bitcast<u32>(13u))));
            }
            let _e486 = phi_1479_;
            O0_ = vec3<f32>(_e475.x, _e475.y, _e486);
        } else {
            O0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e491 = j.ah;
    let _e493 = j.bh;
    let _e501 = vec4<f32>(((_e351.x * _e491) - 1f), ((_e351.y * _e493) - sign(_e493)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if G6_ {
                phi_1490_ = true;
                break;
            }
            if ue {
                phi_1490_ = true;
                break;
            }
            phi_1490_ = false;
            break;
        }
    }
    let _e504 = phi_1490_;
    if _e504 {
        let _e506 = u32((_e126 * 254f));
        phi_1500_ = _e506;
        if ((_e78 & 1073741824i) == 0i) {
            phi_1500_ = (_e506 + bitcast<u32>(1i));
        }
        let _e511 = phi_1500_;
        phi_1499_ = _e511;
    } else {
        phi_1499_ = 255u;
    }
    let _e513 = phi_1499_;
    unnamed.gl_Position = vec4<f32>(_e501.x, _e501.y, ((f32((((_e173.z >> bitcast<u32>(16u)) << bitcast<u32>(8u)) | _e513)) * 0.000000059604645f) + 0.000000029802322f), _e501.w);
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
